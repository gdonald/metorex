//! Native method implementations for the Object class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Public method names the receiver's own singleton layer contributes:
    /// `def obj.name`, `class << obj`, `define_singleton_method`, and the
    /// modules `extend` attached. Private ones are left out, matching what
    /// `Object#methods` reports.
    fn singleton_layer_names(&self, receiver: &Object) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        // `def obj.name` records the method on the instance itself.
        if let Object::Instance(inst) = receiver {
            for name in inst.borrow().singleton_methods.borrow().keys() {
                names.push(name.clone());
            }
        }
        let singleton = match receiver {
            Object::Class(c) | Object::Module(c) => c.singleton_class_slot().clone(),
            Object::Instance(inst) => inst.borrow().singleton_class.borrow().clone(),
            _ => None,
        };
        let Some(singleton_class) = singleton else {
            return names;
        };
        for name in singleton_class.method_names() {
            if !name.starts_with("__")
                && !singleton_class.is_method_private(&name)
                && !names.contains(&name)
            {
                names.push(name);
            }
        }
        // A tombstone `undef_method` left on the singleton class removes the
        // name from the object, however it originally arrived.
        names.retain(|name| {
            singleton_class
                .find_own_method(name)
                .is_none_or(|method| !method.is_undefined)
        });
        // `obj.extend(Mod)` mixes Mod into the singleton class.
        for mixin in singleton_class.transitive_mixins() {
            for name in mixin.method_names() {
                if !name.starts_with("__")
                    && !mixin.is_method_private(&name)
                    && !names.contains(&name)
                {
                    names.push(name);
                }
            }
        }
        names
    }

    /// Whether `receiver` claims `name` through `respond_to_missing?`. Ruby
    /// passes `true` for the private flag here, so a name the object handles
    /// only privately still produces a Method.
    fn responds_via_missing(
        &mut self,
        receiver: &Object,
        name: &str,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let Some((class, method)) = self.lookup_method(receiver, "respond_to_missing?") else {
            return Ok(false);
        };
        if method.is_undefined {
            return Ok(false);
        }
        let arguments = vec![
            Object::Symbol(std::rc::Rc::new(name.to_string())),
            Object::Bool(true),
        ];
        let answer = self.invoke_method(class, method, receiver.clone(), arguments, position)?;
        Ok(!matches!(answer, Object::Nil | Object::Bool(false)))
    }

    /// Execute native methods for the Object class.
    pub(crate) fn call_object_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Kernel#autoload / #autoload? — a top-level (or any non-module)
        // receiver registers the autoload on Object, Ruby's home for
        // top-level constants.
        if matches!(method_name, "autoload" | "autoload?")
            && let Some(Object::Class(object_class)) = self.globals().get("Object")
        {
            return self.call_class_methods(&object_class, method_name, arguments, position);
        }

        // Operators reached by name rather than by syntax — `1.send(:+, 2)`,
        // or a method body built from `:+.to_proc`. Route them back through
        // the binary-operator evaluator.
        if arguments.len() == 1
            && let Some(op) = binary_op_for_method_name(method_name)
        {
            return self
                .evaluate_binary_operation(&op, receiver.clone(), arguments[0].clone(), position)
                .map(Some);
        }

        // Symbol#to_proc — `:foo.to_proc` is a two-parameter callable that
        // sends `foo` to its first argument.
        if method_name == "to_proc"
            && let Object::Symbol(name) = receiver
        {
            return Ok(Some(Object::Block(std::rc::Rc::new(symbol_to_proc_block(
                name, position,
            )))));
        }

        // Nil-specific conversions: in Ruby `nil.to_i == 0`, `nil.to_s == ""`,
        // `nil.to_a == []`, `nil.to_f == 0.0`. The dispatch above checks the
        // class of the receiver first, so we have to intercept here for Nil
        // before falling through to the generic Object methods.
        if matches!(receiver, Object::Nil) {
            match method_name {
                "to_i" => return Ok(Some(Object::Int(0))),
                "to_f" => return Ok(Some(Object::Float(0.0))),
                "to_a" => {
                    return Ok(Some(Object::Array(std::rc::Rc::new(
                        std::cell::RefCell::new(Vec::new()),
                    ))));
                }
                "to_h" => {
                    return Ok(Some(Object::Dict(std::rc::Rc::new(
                        std::cell::RefCell::new(indexmap::IndexMap::new()),
                    ))));
                }
                "to_s" => return Ok(Some(Object::string(""))),
                "inspect" => return Ok(Some(Object::string("nil"))),
                "to_r" | "rationalize" => {
                    if method_name == "rationalize" && arguments.len() > 1 {
                        let exc = Object::exception(
                            "ArgumentError",
                            format!(
                                "wrong number of arguments (given {}, expected 0..1)",
                                arguments.len()
                            ),
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: format!(
                                "wrong number of arguments (given {}, expected 0..1)",
                                arguments.len()
                            ),
                        });
                    }
                    // Return Rational(0, 1) — create an instance via global function
                    if let Some(Object::Class(rational_class)) = self.globals().get("Rational") {
                        let mut inst = crate::object::Instance::new(rational_class);
                        inst.set_var("numerator".to_string(), Object::Int(0));
                        inst.set_var("denominator".to_string(), Object::Int(1));
                        return Ok(Some(Object::Instance(std::rc::Rc::new(
                            std::cell::RefCell::new(inst),
                        ))));
                    }
                    return Ok(Some(Object::Int(0)));
                }
                "to_c" => {
                    if let Some(Object::Class(complex_class)) = self.globals().get("Complex") {
                        let mut inst = crate::object::Instance::new(complex_class);
                        inst.set_var("real".to_string(), Object::Int(0));
                        inst.set_var("imaginary".to_string(), Object::Int(0));
                        return Ok(Some(Object::Instance(std::rc::Rc::new(
                            std::cell::RefCell::new(inst),
                        ))));
                    }
                    return Ok(Some(Object::Int(0)));
                }
                _ => {}
            }
        }
        match method_name {
            "__send__" | "send" | "public_send" => {
                if arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        "wrong number of arguments (given 0, expected 1+)".to_string(),
                        position_to_location(position),
                    ));
                }
                let method = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            format!("{} requires a String or Symbol method name", method_name),
                            position_to_location(position),
                        ));
                    }
                };
                let rest_args: Vec<Object> = arguments[1..].to_vec();
                // Prefer full lookup (walks singleton class + mixins) so mocked
                // or per-instance overrides take precedence over the class's
                // own method table.
                if let Some((resolved_class, m)) = self.lookup_method(receiver, &method)
                    && !m.is_undefined
                {
                    return Ok(Some(self.invoke_method(
                        resolved_class,
                        m,
                        receiver.clone(),
                        rest_args,
                        position,
                    )?));
                }
                let class = self.builtins().class_of(receiver);
                if let Some(result) = self.call_native_method(
                    class.as_ref(),
                    receiver,
                    &method,
                    &rest_args,
                    position,
                )? {
                    return Ok(Some(result));
                }
                if let Some(result) =
                    self.call_object_method(receiver, &method, &rest_args, position)?
                {
                    return Ok(Some(result));
                }
                Err(undefined_method_error(&method, receiver, position))
            }
            // Kernel#lambda reached by dispatch (`send(:lambda) { }`) rather
            // than by a bare call. The block is already in `pending_block`.
            "lambda" => self
                .call_native_function("lambda", arguments.to_vec(), position)
                .map(Some),
            "itself" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(receiver.clone()))
            }
            "frozen?" => Ok(Some(Object::Bool(self.object_is_frozen(receiver)))),
            "freeze" => {
                match receiver {
                    Object::Class(c) | Object::Module(c) => {
                        c.freeze();
                    }
                    Object::Instance(inst) => {
                        inst.borrow_mut().frozen = true;
                        // Freezing an object freezes its singleton class, so
                        // no singleton method can be added afterwards.
                        let singleton = inst.borrow().singleton_class.borrow().clone();
                        if let Some(sc) = singleton {
                            sc.freeze();
                        }
                    }
                    _ => {}
                }
                Ok(Some(receiver.clone()))
            }
            "to_sym" => match receiver {
                Object::Symbol(_) => Ok(Some(receiver.clone())),
                Object::String(s) => Ok(Some(Object::Symbol(s.clone()))),
                _ => Ok(None),
            },
            // Kernel's conversion functions are private instance methods on
            // every object, which is how `obj.send(:Integer, "10")` reaches
            // them.
            name if super::kernel_conversion::is_kernel_conversion(name) => {
                self.call_kernel_conversion(name, arguments, position)
            }
            // `abort` is a private instance method on Kernel, so every
            // object reaches it — the fixture classes make it public.
            "abort" => self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some),
            // `send(:block_given?)` reports on the frame that sent it, the
            // same as the bare form.
            "block_given?" if arguments.is_empty() => Ok(Some(Object::Bool(matches!(
                self.environment().get("block_given?"),
                Some(Object::Bool(true))
            )))),
            // ARGF#gets reads a line from the input stream.
            "gets"
                if arguments.is_empty()
                    && matches!(receiver, Object::Instance(inst) if inst.borrow().class.name() == "ARGF.class") =>
            {
                self.read_line_from_stdin(position).map(Some)
            }
            // `initialize_copy(source)` is the hook `dup` and `clone` call on
            // the new object. Its default does nothing beyond checking that
            // the copy is allowed.
            "initialize_copy" if arguments.len() == 1 => {
                let source = &arguments[0];
                if same_object(receiver, source) {
                    return Ok(Some(receiver.clone()));
                }
                if self.object_is_frozen(receiver) {
                    let message = format!("can't modify frozen object: {}", receiver);
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("FrozenError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                let receiver_class = self.builtins().class_of(receiver);
                let source_class = self.builtins().class_of(source);
                if !std::rc::Rc::ptr_eq(&receiver_class, &source_class) {
                    let message = "initialize_copy should take same class object".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                Ok(Some(receiver.clone()))
            }
            // `clone` and `dup` reach `initialize_copy` through these, so a
            // class can hook either copy on its own.
            "initialize_clone" | "initialize_dup" if !arguments.is_empty() => {
                let source = arguments[0].clone();
                match self.lookup_method(receiver, "initialize_copy") {
                    Some((class, method)) if !method.is_undefined => {
                        self.invoke_method(
                            class,
                            method,
                            receiver.clone(),
                            vec![source],
                            position,
                        )?;
                    }
                    _ => {
                        self.call_object_method(receiver, "initialize_copy", &[source], position)?;
                    }
                }
                Ok(Some(receiver.clone()))
            }
            // `fail` is private on Kernel too, so `send(:fail, ...)` reaches it.
            "fail" if arguments.len() <= 2 => self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some),
            // `binding` is likewise private on Kernel. It captures the frame
            // that called it rather than anything about this receiver, so
            // `obj.send(:binding)` answers the sender's context.
            "binding" if arguments.is_empty() => self
                .call_native_function("binding_kernel", Vec::new(), position)
                .map(Some),
            // Kernel functions that report on the running method, so
            // `send(:__callee__)` reaches them like any other Kernel method.
            "__method__" | "__callee__" => self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some),
            // `hash` — equal values answer equal digests, and reference
            // types fall back to their identity.
            "hash" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(self.hash_digest(receiver, position)?)))
            }
            "object_id" | "__id__" => {
                // A reference type is identified by its address. An immediate
                // gets a value-derived id, so two literals that are the same
                // value share one, the way Ruby's do.
                let id = match receiver {
                    Object::Instance(inst) => std::rc::Rc::as_ptr(inst) as i64,
                    Object::Array(arr) => std::rc::Rc::as_ptr(arr) as i64,
                    Object::Dict(dict) => std::rc::Rc::as_ptr(dict) as i64,
                    Object::Set(set) => std::rc::Rc::as_ptr(set) as i64,
                    Object::Class(cls) => std::rc::Rc::as_ptr(cls) as i64,
                    Object::Module(m) => std::rc::Rc::as_ptr(m) as i64,
                    Object::Block(block) => std::rc::Rc::as_ptr(block) as i64,
                    Object::Exception(exc) => std::rc::Rc::as_ptr(exc) as i64,
                    // Ruby's fixnum object_id. Wrapping keeps the far ends of
                    // the range from overflowing.
                    Object::Int(n) => n.wrapping_mul(2).wrapping_add(1),
                    Object::Bool(true) => 2,
                    Object::Bool(false) => 0,
                    Object::Nil => 4,
                    Object::Symbol(name) => value_object_id("symbol", name),
                    Object::String(text) => value_object_id("string", text),
                    Object::Float(value) => value_object_id("float", &value.to_bits().to_string()),
                    other => value_object_id("other", &other.to_string()),
                };
                Ok(Some(Object::Int(id)))
            }
            "clamp" => {
                // Comparable#clamp — 1 or 2 args, Range, or beginless/endless ranges
                let (min, max) = if arguments.len() == 1 {
                    // Range argument
                    match &arguments[0] {
                        Object::Range {
                            start,
                            end,
                            exclusive,
                        } => {
                            // Exclusive range is an error — except when end is nil
                            // (endless exclusive range like `x...` is allowed).
                            if *exclusive && !matches!(**end, Object::Nil) {
                                let exc = Object::exception(
                                    "ArgumentError",
                                    "cannot clamp with an exclusive range".to_string(),
                                );
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: "cannot clamp with an exclusive range".to_string(),
                                });
                            }
                            ((**start).clone(), (**end).clone())
                        }
                        _ => {
                            return Err(method_argument_error(
                                method_name,
                                1,
                                arguments.len(),
                                position,
                            ));
                        }
                    }
                } else if arguments.len() == 2 {
                    (arguments[0].clone(), arguments[1].clone())
                } else {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                };

                // Verify min <= max when both are non-nil (also raise if incomparable)
                if !matches!(min, Object::Nil) && !matches!(max, Object::Nil) {
                    let cmp = self.dispatch_spaceship(&min, &max, position)?;
                    match cmp {
                        Some(n) if n > 0 => {
                            let exc = Object::exception(
                                "ArgumentError",
                                "min argument must be smaller than max argument".to_string(),
                            );
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: "min argument must be smaller than max argument"
                                    .to_string(),
                            });
                        }
                        None => {
                            let exc =
                                Object::exception("ArgumentError", "comparison failed".to_string());
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: "comparison failed".to_string(),
                            });
                        }
                        _ => {}
                    }
                }

                // Clamp: return min if self < min, max if self > max, else self
                if !matches!(min, Object::Nil) {
                    let cmp = self.dispatch_spaceship(receiver, &min, position)?;
                    if matches!(cmp, Some(n) if n < 0) {
                        return Ok(Some(min));
                    }
                }
                if !matches!(max, Object::Nil) {
                    let cmp = self.dispatch_spaceship(receiver, &max, position)?;
                    if matches!(cmp, Some(n) if n > 0) {
                        return Ok(Some(max));
                    }
                }
                Ok(Some(receiver.clone()))
            }
            "between?" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // between?(min, max) returns true if min <= self <= max
                // Try <=> dispatch
                let cmp_min = self.dispatch_spaceship(receiver, &arguments[0], position)?;
                let cmp_max = self.dispatch_spaceship(receiver, &arguments[1], position)?;
                let result = match (cmp_min, cmp_max) {
                    (Some(a), Some(b)) => a >= 0 && b <= 0,
                    _ => false,
                };
                Ok(Some(Object::Bool(result)))
            }
            "singleton_class" => {
                let sc = self.singleton_class_of(receiver);
                Ok(Some(Object::Class(sc)))
            }
            // Object#extend(mod) — add `mod` as a mixin on the receiver's
            // singleton class so the module's instance methods become callable
            // on this specific object. (Class/Module already have their own
            // dedicated `extend` in class_methods.rs that hits earlier.)
            "extend" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        "extend",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                for arg in arguments {
                    // Ruby takes a module here and rejects a class.
                    let module_rc = match arg {
                        Object::Module(m) => std::rc::Rc::clone(m),
                        other => {
                            return Err(method_argument_type_error(
                                "extend", "Module", other, position,
                            ));
                        }
                    };
                    self.apply_module_extend(receiver, &module_rc, position)?;
                }
                Ok(Some(receiver.clone()))
            }
            "singleton_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "no implicit conversion into String".to_string(),
                            position_to_location(position),
                        ));
                    }
                };
                let msg = format!("undefined singleton method '{}' for {}", method, receiver);
                let exc = Object::exception("NameError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            "to_s" | "inspect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Symbol(s) = receiver {
                    return Ok(Some(Object::string(if method_name == "to_s" {
                        (**s).clone()
                    } else {
                        format!(":{}", s)
                    })));
                }
                Ok(Some(Object::string(receiver.to_string())))
            }
            "class" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // For exceptions, return the specific exception class, not generic Exception
                if let Object::Exception(exc_ref) = receiver {
                    let exc_type = exc_ref.borrow().exception_type.clone();
                    if let Some(Object::Class(cls)) = self.globals().get(&exc_type) {
                        return Ok(Some(Object::Class(cls)));
                    }
                }
                // A class is an instance of Class, a module an instance of
                // Module. `class_of` answers Object for both because it drives
                // `is_a?` over the inheritance chain.
                match receiver {
                    Object::Class(_) => {
                        if let Some(Object::Class(cls)) = self.globals().get("Class") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    Object::Module(_) => {
                        if let Some(Object::Class(cls)) = self.globals().get("Module") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    _ => {}
                }
                // For booleans and nil, return the specific class
                match receiver {
                    Object::Bool(true) => {
                        if let Some(Object::Class(cls)) = self.globals().get("TrueClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    Object::Bool(false) => {
                        if let Some(Object::Class(cls)) = self.globals().get("FalseClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    Object::Nil => {
                        if let Some(Object::Class(cls)) = self.globals().get("NilClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    _ => {}
                }
                Ok(Some(Object::Class(self.builtins().class_of(receiver))))
            }
            "method" => {
                // `obj.method(:name)` returns a Method object bound to obj.
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = self.coerce_name_argument(&arguments[0], position)?;
                // Full lookup so singleton methods (`class << obj`,
                // `def self.foo`) and mixins resolve, not just the class's
                // own method table.
                if let Some((resolved_class, method)) = self.lookup_method(receiver, &name_str) {
                    let mut bound = method.as_ref().clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    if bound.owner_class.is_none() {
                        bound.owner_class = Some(resolved_class);
                    }
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
                // Module and Class instance methods are implemented natively
                // rather than living in a method table, so hand out a stub
                // carrying the right parameter list.
                if matches!(receiver, Object::Class(_) | Object::Module(_))
                    && let Some(mut stub) =
                        super::class_methods::native_module_method_stub(&name_str)
                {
                    stub.receiver = Some(Box::new(receiver.clone()));
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // The Kernel methods every object carries are native too.
                if let Some(mut stub) = super::class_methods::native_kernel_method_stub(&name_str) {
                    stub.receiver = Some(Box::new(receiver.clone()));
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // An object that answers `respond_to_missing?` for the name
                // has a method as far as Ruby is concerned, so hand out one
                // that routes through `method_missing`.
                if self.responds_via_missing(receiver, &name_str, position)? {
                    let stub = method_missing_dispatcher(&name_str, receiver, position);
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                let cls = self.builtins().class_of(receiver);
                let msg = format!("undefined method '{}' for class '{}'", name_str, cls.name());
                let exc = Object::exception("NameError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            "respond_to?" => {
                // Accept String or Symbol method name, plus the optional
                // `include_private` flag.
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method_query = match &arguments[0] {
                    Object::String(name) => name.as_str().to_string(),
                    Object::Symbol(name) => name.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let include_private = matches!(arguments.get(1), Some(value) if value.is_truthy());
                Ok(Some(Object::Bool(
                    self.responds_to(receiver, &method_query)
                        && (include_private || !self.method_is_restricted(receiver, &method_query)),
                )))
            }
            "nil?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(matches!(receiver, Object::Nil))))
            }
            "get_source" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let query = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                match self.lookup_method(receiver, &query) {
                    Some((_class, method)) => Ok(Some(Object::Method(method))),
                    None => Ok(Some(Object::Nil)),
                }
            }
            "is_a?" | "kind_of?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let target_class = match &arguments[0] {
                    Object::Class(c) => c,
                    Object::Module(m) => m,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Class",
                            other,
                            position,
                        ));
                    }
                };
                // A Class object is itself an instance of Class (and its
                // ancestors: Module, Object, BasicObject). Walk the global
                // Class's chain so anonymous Class.new instances answer
                // correctly without needing their own class pointer.
                if matches!(receiver, Object::Class(_) | Object::Module(_)) {
                    let target_name = target_class.name();
                    let meta_name = match receiver {
                        Object::Class(_) => "Class",
                        Object::Module(_) => "Module",
                        _ => unreachable!(),
                    };
                    if let Some(Object::Class(meta)) = self.globals().get(meta_name) {
                        if self.builtins().is_subclass_of(&meta, target_class) {
                            return Ok(Some(Object::Bool(true)));
                        }
                        let mut current = meta.superclass();
                        while let Some(anc) = current {
                            if anc.name() == target_name {
                                return Ok(Some(Object::Bool(true)));
                            }
                            current = anc.superclass();
                        }
                    }
                }
                Ok(Some(Object::Bool(
                    self.builtins().is_instance_of(receiver, target_class),
                )))
            }
            "instance_variables" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let vars = if let Object::Instance(inst_rc) = receiver {
                    let inst = inst_rc.borrow();
                    inst.instance_vars
                        .keys()
                        .map(|k| Object::Symbol(std::rc::Rc::new(format!("@{}", k))))
                        .collect()
                } else {
                    vec![]
                };
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(vars),
                ))))
            }
            // `instance_variable_defined?(name)` — whether the receiver holds
            // that variable. Nothing but an instance, class, or module can.
            "instance_variable_defined?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let var_name = match &arguments[0] {
                    Object::String(name) => name.as_str().to_string(),
                    Object::Symbol(name) => name.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let bare_name = var_name.strip_prefix('@').unwrap_or(&var_name);
                let defined = match receiver {
                    Object::Instance(inst_rc) => {
                        inst_rc.borrow().instance_vars.contains_key(bare_name)
                    }
                    Object::Class(class_rc) | Object::Module(class_rc) => {
                        class_rc.get_class_var(&format!("@{}", bare_name)).is_some()
                    }
                    _ => false,
                };
                Ok(Some(Object::Bool(defined)))
            }
            "instance_variable_get" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let clean_name = self.coerce_instance_variable_name(&arguments[0], position)?;
                let clean_name = clean_name.as_str();
                match receiver {
                    Object::Instance(inst_rc) => {
                        let inst = inst_rc.borrow();
                        Ok(Some(
                            inst.instance_vars
                                .get(clean_name)
                                .cloned()
                                .unwrap_or(Object::Nil),
                        ))
                    }
                    Object::Class(class_rc) => Ok(Some(
                        class_rc
                            .get_class_var(&format!("@{}", clean_name))
                            .unwrap_or(Object::Nil),
                    )),
                    Object::Module(module_rc) => Ok(Some(
                        module_rc
                            .get_class_var(&format!("@{}", clean_name))
                            .unwrap_or(Object::Nil),
                    )),
                    _ => Ok(Some(Object::Nil)),
                }
            }
            "instance_variable_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // Ruby validates the name before it checks whether the
                // receiver is frozen, so `"".instance_variable_set(:c, 1)`
                // raises NameError rather than FrozenError.
                let var_name = self.coerce_instance_variable_name(&arguments[0], position)?;
                let value = arguments[1].clone();
                match receiver {
                    Object::Instance(instance_rc) => {
                        let is_frozen = instance_rc.borrow().frozen;
                        if is_frozen {
                            let class_name = instance_rc.borrow().class.name().to_string();
                            let msg = format!("can't modify frozen {}", class_name);
                            let exc = Object::exception("FrozenError", msg.clone());
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: msg,
                            });
                        }
                        instance_rc.borrow_mut().set_var(var_name, value.clone());
                        Ok(Some(value))
                    }
                    Object::Class(class_rc) => {
                        class_rc.set_class_var(format!("@{}", var_name), value.clone());
                        Ok(Some(value))
                    }
                    Object::Module(module_rc) => {
                        module_rc.set_class_var(format!("@{}", var_name), value.clone());
                        Ok(Some(value))
                    }
                    // Immediates (true/false/nil/integers/symbols/floats) are
                    // always frozen — assigning an ivar raises FrozenError to
                    // match Ruby (FrozenError is a subclass of RuntimeError).
                    other => {
                        let class_name = match other {
                            Object::Bool(true) => "TrueClass".to_string(),
                            Object::Bool(false) => "FalseClass".to_string(),
                            Object::Nil => "NilClass".to_string(),
                            Object::Symbol(_) => "Symbol".to_string(),
                            _ => self.builtins().class_of(other).name().to_string(),
                        };
                        let msg = format!("can't modify frozen {}: {}", class_name, other);
                        let exc = Object::exception("FrozenError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                }
            }
            "instance_of?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let target_class = match &arguments[0] {
                    Object::Class(c) => c,
                    // An object is never an instance *of* a module, so a
                    // module argument is simply false rather than an error.
                    Object::Module(_) => return Ok(Some(Object::Bool(false))),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Class",
                            other,
                            position,
                        ));
                    }
                };
                // Exceptions are stored as a single Object::Exception variant
                // tagged with the specific exception type (e.g. "NameError").
                // `instance_of?` should match against that type rather than
                // the broad `Exception` class so `e.instance_of?(NameError)`
                // works for a rescued NameError.
                if let Object::Exception(exc) = receiver {
                    let actual = exc.borrow().exception_type.clone();
                    return Ok(Some(Object::Bool(actual == target_class.name())));
                }
                let obj_class = self.builtins().class_of(receiver);
                Ok(Some(Object::Bool(obj_class.name() == target_class.name())))
            }
            // `public_methods` is `methods` minus the restricted ones.
            "public_methods" => {
                let Some(Object::Array(names)) =
                    self.call_object_method(receiver, "methods", arguments, position)?
                else {
                    return Ok(None);
                };
                names.borrow_mut().retain(|name| match name {
                    Object::Symbol(n) => !self.method_is_restricted(receiver, n),
                    _ => true,
                });
                Ok(Some(Object::Array(names)))
            }
            "methods" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Optional `include_super` arg (default true). When false, the
                // walk over inherited methods is skipped — but the receiver's
                // own methods are still collected. Mirrors Ruby's
                // `obj.methods(false)`.
                let include_super = !matches!(arguments.first(), Some(Object::Bool(false)));
                // `obj.methods(false)` is the singleton methods alone: those
                // defined with `def obj.name` and any on the singleton class.
                if !include_super {
                    return Ok(Some(self.singleton_method_names(receiver)));
                }
                let class = self.builtins().class_of(receiver);
                let mut names = class.method_names();
                if include_super {
                    // Walk the superclass chain to collect inherited methods
                    let mut current = class.superclass();
                    while let Some(parent) = current {
                        for name in parent.method_names() {
                            if !names.contains(&name) {
                                names.push(name);
                            }
                        }
                        current = parent.superclass();
                    }
                }
                // For instances, also include methods from the instance's class
                if let Object::Instance(inst_rc) = receiver {
                    let inst = inst_rc.borrow();
                    for name in inst.class.method_names() {
                        if !names.contains(&name) {
                            names.push(name);
                        }
                    }
                    if include_super {
                        let mut parent = inst.class.superclass();
                        while let Some(p) = parent {
                            for name in p.method_names() {
                                if !names.contains(&name) {
                                    names.push(name);
                                }
                            }
                            parent = p.superclass();
                        }
                    }
                }
                // For Class/Module receivers, also include the receiver's own
                // instance methods. Ruby's `Object.methods` exposes instance
                // methods of Object too (because Object's singleton class
                // inherits from Class → Module → Object). This also lets
                // `define_method` at TOPLEVEL_BINDING be observable via
                // `Object.methods.include?(...)`.
                if let Object::Class(c) | Object::Module(c) = receiver {
                    for name in c.method_names() {
                        // `def self.name` lands in the method table under the
                        // `__class__` convention; report it under the name
                        // Ruby shows.
                        let name = match name.strip_prefix("__class__") {
                            Some(bare) => bare.to_string(),
                            None => name,
                        };
                        if !names.contains(&name) {
                            names.push(name);
                        }
                    }
                    if include_super {
                        let mut parent = c.superclass();
                        while let Some(p) = parent {
                            for name in p.method_names() {
                                if !names.contains(&name) {
                                    names.push(name);
                                }
                            }
                            parent = p.superclass();
                        }
                    }
                }
                for name in self.singleton_layer_names(receiver) {
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                // `def self.name` is stored under the `__class__` convention;
                // it belongs to the class object, not to its instances.
                if matches!(receiver, Object::Instance(_)) {
                    names.retain(|name| !name.starts_with("__class__"));
                }
                // A tombstone left by `undef_method` is not a method any more.
                let lookup_class = match receiver {
                    Object::Class(c) | Object::Module(c) => Some(std::rc::Rc::clone(c)),
                    Object::Instance(inst) => Some(std::rc::Rc::clone(&inst.borrow().class)),
                    _ => None,
                };
                if let Some(class) = lookup_class {
                    names.retain(|n| class.find_method(n).is_none_or(|m| !m.is_undefined));
                }
                let singleton = match receiver {
                    Object::Class(c) | Object::Module(c) => c.singleton_class_slot().clone(),
                    Object::Instance(inst) => inst.borrow().singleton_class.borrow().clone(),
                    _ => None,
                };
                if let Some(singleton_class) = singleton {
                    names.retain(|n| {
                        singleton_class
                            .find_own_method(n)
                            .is_none_or(|m| !m.is_undefined)
                    });
                }
                names.sort();
                names.dedup();
                let method_symbols: Vec<Object> = names
                    .into_iter()
                    .map(|n| Object::Symbol(std::rc::Rc::new(n)))
                    .collect();
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(method_symbols),
                ))))
            }
            "eql?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(receiver.equals(&arguments[0]))))
            }
            "equal?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = &arguments[0];
                let identity = match (receiver, other) {
                    (Object::Instance(a), Object::Instance(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Array(a), Object::Array(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Dict(a), Object::Dict(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Class(a), Object::Class(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Module(a), Object::Module(b)) => std::rc::Rc::ptr_eq(a, b),
                    // For value types, equal? is the same as ==
                    (a, b) => a == b,
                };
                Ok(Some(Object::Bool(identity)))
            }
            "dup" | "clone" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                match receiver {
                    // Complex and Rational are value objects: there is nothing
                    // to copy, so Ruby answers the receiver itself.
                    Object::Instance(inst_rc)
                        if matches!(inst_rc.borrow().class.name(), "Complex" | "Rational") =>
                    {
                        Ok(Some(receiver.clone()))
                    }
                    Object::Instance(inst_rc) => {
                        let copy = {
                            let inst = inst_rc.borrow();
                            let mut new_inst =
                                crate::object::Instance::new(std::rc::Rc::clone(&inst.class));
                            for (k, v) in &inst.instance_vars {
                                new_inst.set_var(k.clone(), v.clone());
                            }
                            Object::Instance(std::rc::Rc::new(std::cell::RefCell::new(new_inst)))
                        };
                        // The copy gets `initialize_copy` with the original, so
                        // a class can deep-copy what the shallow copy shared.
                        if let Some((class, method)) = self.lookup_method(&copy, "initialize_copy")
                            && !method.is_undefined
                        {
                            self.invoke_method(
                                class,
                                method,
                                copy.clone(),
                                vec![receiver.clone()],
                                position,
                            )?;
                        }
                        Ok(Some(copy))
                    }
                    Object::Array(arr_rc) => {
                        let arr = arr_rc.borrow().clone();
                        Ok(Some(Object::Array(std::rc::Rc::new(
                            std::cell::RefCell::new(arr),
                        ))))
                    }
                    Object::Dict(dict_rc) => {
                        let dict = dict_rc.borrow().clone();
                        Ok(Some(Object::Dict(std::rc::Rc::new(
                            std::cell::RefCell::new(dict),
                        ))))
                    }
                    Object::Class(class_rc) => {
                        if class_rc.name() == "BasicObject" {
                            let msg = "can't copy the root class".to_string();
                            let exc = Object::exception("TypeError", msg.clone());
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: msg,
                            });
                        }
                        let copy = crate::class::Class::duplicate(class_rc);
                        Ok(Some(Object::Class(std::rc::Rc::new(copy))))
                    }
                    Object::Module(mod_rc) => {
                        let copy = crate::class::Class::duplicate(mod_rc);
                        Ok(Some(Object::Module(std::rc::Rc::new(copy))))
                    }
                    // Immutable types return themselves
                    _ => Ok(Some(receiver.clone())),
                }
            }
            "=~" => {
                // Regex match: string =~ regex or regex =~ string
                if arguments.len() != 1 {
                    return Err(method_argument_error("=~", 1, arguments.len(), position));
                }
                match (receiver, &arguments[0]) {
                    (Object::Regex(pattern, flags), Object::String(s))
                    | (Object::String(s), Object::Regex(pattern, flags)) => {
                        let case_insensitive = flags.contains('i');
                        let re_pattern = if case_insensitive {
                            format!("(?i){}", pattern)
                        } else {
                            pattern.as_ref().clone()
                        };
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => {
                                if let Some(m) = re.find(s.as_ref()) {
                                    Ok(Some(Object::Int(m.start() as i64)))
                                } else {
                                    Ok(Some(Object::Nil))
                                }
                            }
                            Err(_) => Ok(Some(Object::Nil)),
                        }
                    }
                    _ => Ok(Some(Object::Nil)),
                }
            }
            // Ruby's `!~` is `!(self =~ other)`. An object with no `=~` of
            // its own has no `!~` either, so this raises rather than
            // answering true.
            "!~" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error("!~", 1, arguments.len(), position));
                }
                if let Some((class, method)) = self.lookup_method(receiver, "=~")
                    && !method.is_undefined
                {
                    let matched = self.invoke_method(
                        class,
                        method,
                        receiver.clone(),
                        vec![arguments[0].clone()],
                        position,
                    )?;
                    return Ok(Some(Object::Bool(!matched.is_truthy())));
                }
                if matches!(
                    (receiver, &arguments[0]),
                    (Object::Regex(_, _), Object::String(_))
                        | (Object::String(_), Object::Regex(_, _))
                ) {
                    let matched = self
                        .call_object_method(receiver, "=~", arguments, position)?
                        .unwrap_or(Object::Nil);
                    return Ok(Some(Object::Bool(!matched.is_truthy())));
                }
                let cls = self.builtins().class_of(receiver);
                let msg = format!("undefined method '=~' for an instance of {}", cls.name());
                let exc = Object::exception("NoMethodError", msg.clone());
                if let Object::Exception(cell) = &exc {
                    cell.borrow_mut().name = Some("=~".to_string());
                }
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            // An instance of a `Module` subclass (`class Sub < Module; end;
            // Sub.new`) is itself a module and answers the module-body methods.
            // Metorex models it as an `Instance`, so back it with a cached
            // anonymous class that hosts any methods the body defines.
            "class_exec" | "module_exec" | "class_eval" | "module_eval"
                if self.instance_acts_as_module(receiver) =>
            {
                let inst = match receiver {
                    Object::Instance(i) => std::rc::Rc::clone(i),
                    _ => unreachable!(),
                };
                let existing = match inst.borrow().get_var("__module_body_class__") {
                    Some(Object::Class(c)) => Some(std::rc::Rc::clone(c)),
                    _ => None,
                };
                let backing = match existing {
                    Some(c) => c,
                    None => {
                        let c = std::rc::Rc::new(crate::class::Class::new("", None));
                        inst.borrow_mut().set_var(
                            "__module_body_class__".to_string(),
                            Object::Class(std::rc::Rc::clone(&c)),
                        );
                        c
                    }
                };
                if method_name == "class_exec" || method_name == "module_exec" {
                    let block = match self.pending_block.take() {
                        Some(Object::Block(b)) => b,
                        _ => return Err(local_jump_error(method_name, position)),
                    };
                    let result = self.class_exec_block(
                        &backing,
                        receiver.clone(),
                        &block,
                        arguments.to_vec(),
                        position,
                    )?;
                    return Ok(Some(result));
                }
                let result =
                    self.class_eval_with_args(&backing, receiver.clone(), arguments, position)?;
                Ok(Some(result))
            }
            // `then` / `yield_self` pass the receiver to the block and answer
            // the block's value; `tap` answers the receiver instead.
            "then" | "yield_self" | "tap" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    _ => {
                        return Err(MetorexError::runtime_error(
                            format!("{} requires a block", method_name),
                            position_to_location(position),
                        ));
                    }
                };
                // A block that declares no parameter is called with none:
                // Ruby's non-lambda blocks drop what they did not ask for.
                let block_arguments = if block.binding_parameters().is_empty() {
                    Vec::new()
                } else {
                    vec![receiver.clone()]
                };
                let value = block.call(self, block_arguments, position)?;
                Ok(Some(if method_name == "tap" {
                    receiver.clone()
                } else {
                    value
                }))
            }
            "instance_exec" | "instance_eval" => {
                let block = self.pending_block.take().or_else(|| {
                    if !arguments.is_empty()
                        && let Object::Block(_) = &arguments[0]
                    {
                        Some(arguments[0].clone())
                    } else {
                        None
                    }
                });
                match block {
                    Some(Object::Block(b)) => {
                        let args: Vec<Object> = arguments
                            .iter()
                            .filter(|a| !matches!(a, Object::Block(_)))
                            .cloned()
                            .collect();
                        let result =
                            self.execute_block_with_receiver(&b, receiver.clone(), args, position)?;
                        Ok(Some(result))
                    }
                    _ => Err(MetorexError::runtime_error(
                        format!("{} requires a block", method_name),
                        position_to_location(position),
                    )),
                }
            }
            _ => Ok(None),
        }
    }

    /// Whether `receiver` is an `Instance` whose class descends from `Module`
    /// (e.g. `class Sub < Module; end; Sub.new`). Such instances are modules
    /// and answer the module-body methods (`class_eval`/`class_exec`/...).
    fn instance_acts_as_module(&self, receiver: &Object) -> bool {
        let Object::Instance(inst) = receiver else {
            return false;
        };
        let class = std::rc::Rc::clone(&inst.borrow().class);
        match self.globals().get("Module") {
            Some(Object::Class(module_class)) => {
                self.builtins().is_subclass_of(&class, &module_class)
            }
            _ => false,
        }
    }

    /// Dispatch <=> on receiver with other, returning Some(i64) or None.
    pub(crate) fn dispatch_spaceship(
        &mut self,
        receiver: &Object,
        other: &Object,
        position: Position,
    ) -> Result<Option<i64>, MetorexError> {
        if let Some((class, method)) = self.lookup_method(receiver, "<=>") {
            let result = self.invoke_method(
                class,
                method,
                receiver.clone(),
                vec![other.clone()],
                position,
            )?;
            match result {
                Object::Int(n) => Ok(Some(n)),
                Object::Nil => Ok(None),
                _ => Ok(None),
            }
        } else {
            // Fallback for built-in types
            match (receiver, other) {
                (Object::Int(a), Object::Int(b)) => Ok(Some((*a).cmp(b) as i64)),
                (Object::Float(a), Object::Float(b)) => Ok(a.partial_cmp(b).map(|o| o as i64)),
                (Object::String(a), Object::String(b)) => Ok(Some((**a).cmp(b) as i64)),
                _ => Ok(None),
            }
        }
    }

    /// The receiver's singleton method names as an array of symbols. A class
    /// or module keeps `def self.name` in its own table under the
    /// `__class__` convention; other objects keep them on a singleton class.
    fn singleton_method_names(&mut self, receiver: &Object) -> Object {
        let mut names: Vec<String> = Vec::new();
        if let Object::Class(c) | Object::Module(c) = receiver {
            for name in c.method_names() {
                if let Some(bare) = name.strip_prefix("__class__") {
                    names.push(bare.to_string());
                }
            }
        }
        for name in self.singleton_layer_names(receiver) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names.sort();
        names.dedup();
        let symbols: Vec<Object> = names
            .into_iter()
            .map(|n| Object::Symbol(std::rc::Rc::new(n)))
            .collect();
        Object::Array(std::rc::Rc::new(std::cell::RefCell::new(symbols)))
    }
}

/// Build the block `Symbol#to_proc` returns: `{ |receiver, *args| receiver.send(name, *args) }`.
fn symbol_to_proc_block(name: &str, position: Position) -> crate::object::BlockStatement {
    use crate::ast::{Expression, Statement};

    let call = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "__symbol_proc_receiver".to_string(),
            position,
        }),
        method: name.to_string(),
        arguments: vec![Expression::Splat {
            expression: Box::new(Expression::Identifier {
                name: "__symbol_proc_args".to_string(),
                position,
            }),
            position,
        }],
        trailing_block: None,
        position,
    };

    crate::object::BlockStatement::new(
        vec![
            "__symbol_proc_receiver".to_string(),
            "*__symbol_proc_args".to_string(),
        ],
        vec![Statement::Expression {
            expression: call,
            position,
        }],
        std::collections::HashMap::new(),
    )
}

/// Map an operator method name back to its `BinaryOp`, for calls that arrive
/// by name (`send(:+, 2)`) instead of through operator syntax.
fn binary_op_for_method_name(name: &str) -> Option<crate::ast::BinaryOp> {
    use crate::ast::BinaryOp;
    Some(match name {
        "+" => BinaryOp::Add,
        "-" => BinaryOp::Subtract,
        "*" => BinaryOp::Multiply,
        "/" => BinaryOp::Divide,
        "%" => BinaryOp::Modulo,
        "**" => BinaryOp::Power,
        "==" => BinaryOp::Equal,
        "===" => BinaryOp::CaseEqual,
        "!=" => BinaryOp::NotEqual,
        "<" => BinaryOp::Less,
        ">" => BinaryOp::Greater,
        "<=" => BinaryOp::LessEqual,
        ">=" => BinaryOp::GreaterEqual,
        "<=>" => BinaryOp::Spaceship,
        "&" => BinaryOp::BitwiseAnd,
        "|" => BinaryOp::BitwiseOr,
        "^" => BinaryOp::Xor,
        _ => return None,
    })
}

impl VirtualMachine {
    /// The integer `Object#hash` answers. Value types digest their canonical
    /// string form so equal values agree; everything else uses its identity.
    pub(crate) fn hash_digest(
        &mut self,
        receiver: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        if let Some(hashable) = crate::object::ObjectHash::from_object(receiver) {
            let mut digest: i64 = 0;
            for byte in hashable.hash_value.bytes() {
                digest = digest.wrapping_mul(31).wrapping_add(byte as i64);
            }
            return Ok(digest);
        }
        match self.call_object_method(receiver, "object_id", &[], position)? {
            Some(Object::Int(id)) => Ok(id),
            _ => Ok(0),
        }
    }
}

/// Whether two values are the same object. Reference types compare by
/// identity, immediates by value.
fn same_object(left: &Object, right: &Object) -> bool {
    use std::rc::Rc;
    match (left, right) {
        (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
        (Object::String(a), Object::String(b)) => Rc::ptr_eq(a, b),
        (Object::Array(a), Object::Array(b)) => Rc::ptr_eq(a, b),
        (Object::Dict(a), Object::Dict(b)) => Rc::ptr_eq(a, b),
        (Object::Class(a), Object::Class(b)) => Rc::ptr_eq(a, b),
        (Object::Module(a), Object::Module(b)) => Rc::ptr_eq(a, b),
        (Object::Symbol(a), Object::Symbol(b)) => a == b,
        (Object::Int(a), Object::Int(b)) => a == b,
        (Object::Float(a), Object::Float(b)) => a == b,
        (Object::Bool(a), Object::Bool(b)) => a == b,
        (Object::Nil, Object::Nil) => true,
        _ => false,
    }
}

/// A `Method` that forwards to the receiver's `method_missing`, for a name the
/// object claims through `respond_to_missing?`. The name goes in front of the
/// call's own arguments, so `method_missing`'s arity applies as written.
fn method_missing_dispatcher(
    name: &str,
    receiver: &Object,
    position: Position,
) -> crate::object::Method {
    use crate::ast::{Expression, Statement};

    let call = Expression::MethodCall {
        receiver: Box::new(Expression::SelfExpr { position }),
        method: "method_missing".to_string(),
        arguments: vec![
            Expression::Symbol {
                value: name.to_string(),
                position,
            },
            Expression::Splat {
                expression: Box::new(Expression::Identifier {
                    name: "__method_missing_args".to_string(),
                    position,
                }),
                position,
            },
        ],
        trailing_block: None,
        position,
    };

    let mut method = crate::object::Method::new(
        name.to_string(),
        vec!["__method_missing_args".to_string()],
        vec![Statement::Expression {
            expression: call,
            position,
        }],
    );
    method.variadic_param = Some((0, "__method_missing_args".to_string()));
    method.receiver = Some(Box::new(receiver.clone()));
    method
}

/// A stable, non-negative id derived from a value rather than an address, for
/// the objects metorex stores by value. Two equal values share an id.
fn value_object_id(tag: &str, value: &str) -> i64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tag.hash(&mut hasher);
    value.hash(&mut hasher);
    (hasher.finish() >> 1) as i64
}
