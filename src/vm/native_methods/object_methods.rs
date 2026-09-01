//! Native method implementations for the Object class.

use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Private method names reachable on `receiver`.
    pub(crate) fn private_method_names_for(
        &self,
        receiver: &Object,
        include_super: bool,
    ) -> Vec<String> {
        self.restricted_method_names_for(receiver, include_super, Class::private_method_names)
    }

    /// Public method names reachable on `receiver`.
    pub(crate) fn public_method_names_for(
        &self,
        receiver: &Object,
        include_super: bool,
    ) -> Vec<String> {
        self.restricted_method_names_for(receiver, include_super, public_method_names)
    }

    /// Protected method names reachable on `receiver`.
    pub(crate) fn protected_method_names_for(
        &self,
        receiver: &Object,
        include_super: bool,
    ) -> Vec<String> {
        self.restricted_method_names_for(receiver, include_super, Class::protected_method_names)
    }

    /// Names of the methods `select` reports along the receiver's lookup path.
    /// The receiver's own singleton layer and its class always count;
    /// `include_super` adds the ancestors and the modules they mix in.
    fn restricted_method_names_for(
        &self,
        receiver: &Object,
        include_super: bool,
        select: fn(&Class) -> Vec<String>,
    ) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        let push = |name: String, names: &mut Vec<String>| {
            if !names.contains(&name) {
                names.push(name);
            }
        };

        // `class << obj` with `private`, and the modules `extend` attached.
        let singleton = match receiver {
            Object::Class(c) | Object::Module(c) => c.singleton_class_slot().clone(),
            Object::Instance(inst) => inst.borrow().singleton_class.borrow().clone(),
            _ => None,
        };
        if let Some(singleton_class) = singleton {
            // A class object's singleton chain stands in for its own class,
            // so it is walked even without `include_super`.
            let walk_chain = matches!(receiver, Object::Class(_) | Object::Module(_));
            let mut current = Some(singleton_class);
            while let Some(class) = current {
                for name in select(&class) {
                    push(name, &mut names);
                }
                for mixin in class.transitive_mixins() {
                    for name in select(&mixin) {
                        push(name, &mut names);
                    }
                }
                current = if walk_chain { class.superclass() } else { None };
            }
        }

        // `def self.name` lands in the class's own table under the
        // `__class__` convention. Those belong to the class object, so they
        // count here and are reported under the name Ruby shows.
        if let Object::Class(class_rc) | Object::Module(class_rc) = receiver {
            let mut current = Some(std::rc::Rc::clone(class_rc));
            while let Some(class) = current {
                for name in select(&class) {
                    if let Some(bare) = name.strip_prefix("__class__") {
                        push(bare.to_string(), &mut names);
                    }
                }
                // A class object's own methods live along this chain the way
                // an instance's live in its class, so it is walked even
                // without `include_super`, matching the singleton chain.
                current = class.superclass();
            }
        }

        let own_class = match receiver {
            Object::Instance(inst) => Some(std::rc::Rc::clone(&inst.borrow().class)),
            Object::Class(_) | Object::Module(_) => None,
            other => Some(self.builtins().class_of(other)),
        };
        if let Some(class) = own_class {
            // A class's `__class__` entries describe the class object, not
            // its instances, so they are left out here.
            let instance_level = |class: &Class| -> Vec<String> {
                select(class)
                    .into_iter()
                    .filter(|name| !name.starts_with("__class__"))
                    .collect()
            };
            for name in instance_level(&class) {
                push(name, &mut names);
            }
            if include_super {
                // A mixin is an ancestor, so its methods only count when
                // ancestors do.
                for mixin in class.transitive_mixins() {
                    for name in instance_level(&mixin) {
                        push(name, &mut names);
                    }
                }
                let mut current = class.superclass();
                while let Some(parent) = current {
                    for name in instance_level(&parent) {
                        push(name, &mut names);
                    }
                    for mixin in parent.transitive_mixins() {
                        for name in instance_level(&mixin) {
                            push(name, &mut names);
                        }
                    }
                    current = parent.superclass();
                }
            }
        }
        names
    }

    /// The method `name` names in the receiver's singleton layer: the
    /// singleton class's own table, then the modules attached to it by
    /// `include`, `prepend`, or `extend`. The object's class is not part of
    /// that layer, so a method it defines is not found here.
    fn singleton_layer_method(
        &self,
        receiver: &Object,
        name: &str,
    ) -> Option<(std::rc::Rc<Class>, std::rc::Rc<crate::object::Method>)> {
        // `def self.name` records the method in the class's own table under
        // the `__class__` convention rather than on the singleton class.
        if let Object::Class(class_rc) | Object::Module(class_rc) = receiver
            && let Some(found) = class_rc.find_method(&format!("__class__{}", name))
            && !found.is_undefined
        {
            return Some((std::rc::Rc::clone(class_rc), found));
        }
        let singleton = match receiver {
            Object::Class(c) | Object::Module(c) => c.singleton_class_slot().clone(),
            Object::Instance(inst) => inst.borrow().singleton_class.borrow().clone(),
            _ => None,
        }?;
        if let Some(found) = singleton.find_own_method(name)
            && !found.is_undefined
        {
            return Some((std::rc::Rc::clone(&singleton), found));
        }
        for mixin in singleton.transitive_mixins() {
            if let Some(found) = mixin.find_own_method(name)
                && !found.is_undefined
            {
                return Some((mixin, found));
            }
        }
        None
    }

    /// Public method names the receiver's own singleton layer contributes:
    /// `def obj.name`, `class << obj`, `define_singleton_method`, and the
    /// modules `extend` attached. Private ones are left out, matching what
    /// `Object#methods` reports.
    fn singleton_layer_names(&self, receiver: &Object) -> Vec<String> {
        self.singleton_layer_names_with_mixins(receiver, true)
    }

    /// The same walk, with `include_mixins` deciding whether the modules
    /// `extend` attached count. `singleton_methods(false)` leaves them out.
    fn singleton_layer_names_with_mixins(
        &self,
        receiver: &Object,
        include_mixins: bool,
    ) -> Vec<String> {
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
        if !include_mixins {
            return names;
        }
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

    /// Whether `receiver` claims `name` through `respond_to_missing?`.
    /// `include_private` is the flag Ruby passes as the second argument:
    /// `method` sends true, `public_method` sends false.
    fn responds_via_missing(
        &mut self,
        receiver: &Object,
        name: &str,
        include_private: bool,
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
            Object::Bool(include_private),
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
                    let message = "no method name given".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                let method = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => {
                        let message = format!(
                            "{} is not a symbol nor a string",
                            self.get_inspect_representation(other, position)?
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("TypeError", message.clone()),
                            location: position_to_location(position),
                            message,
                        });
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
                Err(undefined_method_error(
                    &method, receiver, &rest_args, position,
                ))
            }
            // Kernel#lambda reached by dispatch (`send(:lambda) { }`) rather
            // than by a bare call. The block is already in `pending_block`.
            "lambda" | "proc" | "raise" | "warn" => self
                .call_native_function(method_name, arguments.to_vec(), position)
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
                    // A collection has nowhere of its own to keep the flag,
                    // so the VM records the one it lives at.
                    Object::Array(_) | Object::Dict(_) | Object::Set(_) => {
                        if let Some(address) = Self::collection_address(receiver) {
                            self.frozen_collections.insert(address, receiver.clone());
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
                    // An integer past the i64 range is a heap object in Ruby
                    // too, so two of the same value have different ids.
                    Object::BigInt(value) => std::rc::Rc::as_ptr(value) as i64,
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
            // Kernel#singleton_class. nil, true, and false each have exactly
            // one instance, so their singleton class is the class itself.
            // Immediates that are not objects at all cannot have one.
            // Kernel#singleton_methods(all = true) — the names in the
            // object's singleton layer. `all` adds what its ancestors'
            // singleton classes supply.
            "singleton_methods" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let include_ancestors = !matches!(
                    arguments.first(),
                    Some(Object::Bool(false)) | Some(Object::Nil)
                );
                Ok(Some(self.singleton_method_names_with_ancestors(
                    receiver,
                    include_ancestors,
                )))
            }
            "singleton_class" => {
                if let Some(sole) = match receiver {
                    Object::Nil => Some("NilClass"),
                    Object::Bool(true) => Some("TrueClass"),
                    Object::Bool(false) => Some("FalseClass"),
                    _ => None,
                } && let Some(class @ Object::Class(_)) = self.globals().get(sole)
                {
                    return Ok(Some(class));
                }
                // Ruby also refuses a frozen deduplicated String, but metorex
                // has no per-string frozen flag to tell one from a mutable
                // string, which does have a singleton class.
                if matches!(
                    receiver,
                    Object::Int(_) | Object::Float(_) | Object::Symbol(_)
                ) {
                    let msg = "can't define singleton".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let singleton = self.singleton_class_of(receiver);
                // A frozen object's singleton class is frozen too, so no
                // method can be added to it afterwards. Only an instance
                // carries per-object frozen state; the immediates share one
                // singleton class per type, which must stay writable.
                if matches!(receiver, Object::Instance(_)) && self.object_is_frozen(receiver) {
                    singleton.freeze();
                }
                Ok(Some(Object::Class(singleton)))
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
                // Only the singleton layer counts: a method the object's
                // class defines is not a singleton method of the object.
                if let Some((owner, found)) = self.singleton_layer_method(receiver, &method) {
                    let mut bound = found.as_ref().clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    if bound.owner_class.is_none() {
                        bound.owner_class = Some(owner);
                    }
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
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
                    if let Some(class) = exc_ref.borrow().class.clone() {
                        return Ok(Some(Object::Class(class)));
                    }
                    let exc_type = exc_ref.borrow().exception_type.clone();
                    if let Some(Object::Class(cls)) = self.globals().get(&exc_type) {
                        return Ok(Some(Object::Class(cls)));
                    }
                    // A namespaced type such as `Errno::EINVAL` lives as a
                    // constant on its module rather than at the top level.
                    if let Some(class @ Object::Class(_)) =
                        self.resolve_qualified_constant(&exc_type)
                    {
                        return Ok(Some(class));
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
            // `obj.method(:name)` returns a Method object bound to obj.
            // `public_method` is the same lookup with private and protected
            // names refused.
            "method" | "public_method" => {
                // A class that defines `def self.method` or `def self.
                // public_method` of its own owns the name; the class-method
                // table records it under the `__class__` convention, which
                // the caller checks after this returns None.
                if let Object::Class(class_rc) | Object::Module(class_rc) = receiver
                    && class_rc
                        .find_method(&format!("__class__{}", method_name))
                        .is_some()
                {
                    return Ok(None);
                }
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
                let public_only = method_name == "public_method";
                if let Some((resolved_class, method)) = self.lookup_method(receiver, &name_str) {
                    if public_only && self.method_is_restricted(receiver, &name_str) {
                        let msg = format!(
                            "undefined method '{}' for class '{}'",
                            name_str,
                            self.builtins().class_of(receiver).name()
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
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
                if self.responds_via_missing(receiver, &name_str, !public_only, position)? {
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
            // Kernel#respond_to_missing? — the default answer is false. A
            // class overrides it to claim names it handles through
            // `method_missing`.
            "respond_to_missing?" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(false)))
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
                let method_query = self.coerce_method_name(&arguments[0], method_name, position)?;
                let include_private = matches!(arguments.get(1), Some(value) if value.is_truthy());
                if self.responds_to(receiver, &method_query)
                    && (include_private || !self.method_is_restricted(receiver, &method_query))
                {
                    return Ok(Some(Object::Bool(true)));
                }
                // Ruby asks `respond_to_missing?` for anything the lookup
                // missed, passing the same private flag it was given.
                Ok(Some(Object::Bool(self.responds_via_missing(
                    receiver,
                    &method_query,
                    include_private,
                    position,
                )?)))
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
                // An exception is tagged with its type name rather than
                // carrying a class pointer, so its ancestry is walked from the
                // class that name resolves to.
                if let Object::Exception(details) = receiver {
                    let carried = details.borrow().class.clone().map(Object::Class);
                    let exception_type = details.borrow().exception_type.clone();
                    let resolved = match carried {
                        Some(class) => Some(class),
                        None => match self.globals().get(&exception_type) {
                            Some(class @ Object::Class(_)) => Some(class),
                            _ => self.resolve_qualified_constant(&exception_type),
                        },
                    };
                    if let Some(Object::Class(exception_class)) = resolved {
                        return Ok(Some(Object::Bool(
                            self.builtins()
                                .is_subclass_of(&exception_class, target_class),
                        )));
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
                let clean_name =
                    self.coerce_instance_variable_name(&arguments[0], receiver, position)?;
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
            // Kernel#remove_instance_variable — take the variable off the
            // object and answer what it held. Unlike its siblings this one is
            // public. The name is validated before the frozen check, so
            // `o.remove_instance_variable(:foo)` raises NameError even on a
            // frozen receiver.
            "remove_instance_variable" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let var_name =
                    self.coerce_instance_variable_name(&arguments[0], receiver, position)?;
                if self.object_is_frozen(receiver) || !matches!(receiver, Object::Instance(_)) {
                    let class_name = self.builtins().class_of(receiver).name().to_string();
                    let msg = format!("can't modify frozen {}: {}", class_name, receiver);
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let Object::Instance(instance_rc) = receiver else {
                    unreachable!("only an instance reaches here")
                };
                let removed = instance_rc
                    .borrow_mut()
                    .instance_vars
                    .shift_remove(&var_name);
                match removed {
                    Some(value) => Ok(Some(value)),
                    None => {
                        let msg = format!("instance variable @{} not defined", var_name);
                        let exc = Object::exception("NameError", msg.clone());
                        if let Object::Exception(cell) = &exc {
                            cell.borrow_mut().name = Some(format!("@{}", var_name));
                        }
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
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
                let var_name =
                    self.coerce_instance_variable_name(&arguments[0], receiver, position)?;
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
                // A Class is an instance of Class and a Module of Module,
                // which `class_of` does not report: it answers Object for
                // both so `is_a?` can walk the inheritance chain.
                let actual_name = match receiver {
                    Object::Class(_) => "Class".to_string(),
                    Object::Module(_) => "Module".to_string(),
                    other => self.builtins().class_of(other).name().to_string(),
                };
                Ok(Some(Object::Bool(actual_name == target_class.name())))
            }
            // `public_methods` is `methods` minus the restricted ones.
            // Kernel#public_methods / #private_methods / #protected_methods —
            // the names of that visibility reachable on the object, including
            // those a `class << obj` or `extend` supplied.
            "public_methods" | "private_methods" | "protected_methods" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let include_super = !matches!(
                    arguments.first(),
                    Some(Object::Bool(false)) | Some(Object::Nil)
                );
                let mut names = match method_name {
                    "public_methods" => self.public_method_names_for(receiver, include_super),
                    "private_methods" => self.private_method_names_for(receiver, include_super),
                    _ => self.protected_method_names_for(receiver, include_super),
                };
                names.sort();
                names.dedup();
                let symbols: Vec<Object> = names
                    .into_iter()
                    .map(|name| Object::Symbol(std::rc::Rc::new(name)))
                    .collect();
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(symbols),
                ))))
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
                    (Object::Set(a), Object::Set(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Exception(a), Object::Exception(b)) => std::rc::Rc::ptr_eq(a, b),
                    // An integer past the i64 range is its own object, so two
                    // of the same value are not identical.
                    (Object::BigInt(a), Object::BigInt(b)) => std::rc::Rc::ptr_eq(a, b),
                    // A block can close over itself, so comparing two of them
                    // structurally would never terminate. Ruby's identity
                    // check is the address anyway.
                    (Object::Block(a), Object::Block(b)) => std::rc::Rc::ptr_eq(a, b),
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
                    // An exception copies its message, backtrace, cause, and
                    // instance variables. `dup` leaves the singleton class
                    // behind, so a method defined on the original is not on
                    // the copy.
                    Object::Exception(details) => {
                        let copy = {
                            // The backtrace Array is shared with the original,
                            // the way Ruby's copy shares it.
                            let copied = details.borrow().clone();
                            Object::Exception(std::rc::Rc::new(std::cell::RefCell::new(copied)))
                        };
                        if let Some((class, method)) = self.lookup_method(&copy, "initialize_copy")
                            && !method.is_undefined
                            && !method.body.is_empty()
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
                // A Symbol matches on the characters it is named with, so
                // `/_pri\z/ =~ :ds_pri` finds a match the way Ruby's does.
                match (matchable_text(receiver), matchable_text(&arguments[0])) {
                    (Some(MatchSide::Pattern(pattern, flags)), Some(MatchSide::Text(text)))
                    | (Some(MatchSide::Text(text)), Some(MatchSide::Pattern(pattern, flags))) => {
                        let re_pattern = if flags.contains('i') {
                            format!("(?i){}", pattern)
                        } else {
                            pattern
                        };
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => match re.find(&text) {
                                Some(found) => Ok(Some(Object::Int(found.start() as i64))),
                                None => Ok(Some(Object::Nil)),
                            },
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
                    (matchable_text(receiver), matchable_text(&arguments[0])),
                    (Some(MatchSide::Pattern(_, _)), Some(MatchSide::Text(_)))
                        | (Some(MatchSide::Text(_)), Some(MatchSide::Pattern(_, _)))
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
            // `to_enum(:method, *args)` wraps a method that yields, so the
            // caller can step through its values instead of taking a block.
            "to_enum" | "enum_for" => {
                let mut arguments = arguments.to_vec();
                let enumerated = if arguments.is_empty() {
                    "each".to_string()
                } else {
                    self.coerce_method_name(&arguments.remove(0), method_name, position)?
                };
                self.build_enumerator(receiver.clone(), &enumerated, arguments, None, position)
                    .map(Some)
            }
            "then" | "yield_self" | "tap" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    // `tap` yields, so without a block it raises. `then` and
                    // `yield_self` hand back an Enumerator of size one.
                    _ if method_name == "tap" => {
                        let message = "no block given (yield)".to_string();
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("LocalJumpError", message.clone()),
                            location: position_to_location(position),
                            message,
                        });
                    }
                    _ => {
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                vec![],
                                Some(1),
                                position,
                            )
                            .map(Some);
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
                let positional: Vec<Object> = arguments
                    .iter()
                    .filter(|argument| !matches!(argument, Object::Block(_)))
                    .cloned()
                    .collect();
                if let Some(Object::Block(body)) = block {
                    // Ruby refuses a singleton method on an immediate, and a
                    // `def` in the body would be exactly that.
                    if matches!(
                        receiver,
                        Object::Int(_) | Object::BigInt(_) | Object::Float(_) | Object::Symbol(_)
                    ) && body_defines_a_method(&body.body)
                    {
                        let message = "can't define singleton".to_string();
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("TypeError", message.clone()),
                            location: position_to_location(position),
                            message,
                        });
                    }
                    // `instance_eval` yields the receiver and takes no other
                    // arguments; `instance_exec` passes its own along.
                    if method_name == "instance_eval" {
                        if !positional.is_empty() {
                            return Err(crate::vm::errors::argument_count_error(
                                crate::vm::errors::Arity::Exact(0),
                                positional.len(),
                                position,
                            ));
                        }
                        let result = self.execute_block_with_receiver(
                            &body,
                            receiver.clone(),
                            vec![receiver.clone()],
                            position,
                        )?;
                        return Ok(Some(result));
                    }
                    let result = self.execute_block_with_receiver(
                        &body,
                        receiver.clone(),
                        positional,
                        position,
                    )?;
                    return Ok(Some(result));
                }
                // The String form runs source in the receiver's context. The
                // trailing file and line arguments only shape the positions
                // recorded for the code, which metorex takes from the source
                // it parses.
                if method_name == "instance_eval" {
                    if positional.is_empty() || positional.len() > 3 {
                        return Err(crate::vm::errors::argument_count_error(
                            crate::vm::errors::Arity::Range(1, 3),
                            positional.len(),
                            position,
                        ));
                    }
                    let source = self.coerce_name_argument(&positional[0], position)?;
                    return self
                        .evaluate_source_with_receiver(&source, receiver.clone(), position)
                        .map(Some);
                }
                // `instance_exec` yields, so without a block Ruby reports the
                // same LocalJumpError any bare `yield` would.
                let message = "no block given (yield)".to_string();
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("LocalJumpError", message.clone()),
                    location: position_to_location(position),
                    message,
                })
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
                (Object::Int(_) | Object::BigInt(_), Object::Int(_) | Object::BigInt(_)) => {
                    let a = receiver.as_big_integer().expect("integer-kinded");
                    let b = other.as_big_integer().expect("integer-kinded");
                    Ok(Some(a.cmp(&b) as i64))
                }
                (Object::String(a), Object::String(b)) => Ok(Some((**a).cmp(b) as i64)),
                // Integers and Floats compare against each other, so a Range
                // with one of each answers `include?` for either.
                (Object::Float(_) | Object::Int(_), Object::Float(_) | Object::Int(_)) => {
                    let to_f = |value: &Object| match value {
                        Object::Int(n) => *n as f64,
                        Object::Float(n) => *n,
                        _ => unreachable!(),
                    };
                    Ok(to_f(receiver).partial_cmp(&to_f(other)).map(|o| o as i64))
                }
                _ => Ok(None),
            }
        }
    }

    /// The receiver's singleton method names as an array of symbols. A class
    /// or module keeps `def self.name` in its own table under the
    /// `__class__` convention; other objects keep them on a singleton class.
    fn singleton_method_names(&mut self, receiver: &Object) -> Object {
        self.singleton_method_names_with_ancestors(receiver, false)
    }

    /// The receiver's singleton method names. `include_ancestors` adds the
    /// class methods and singleton-class methods its superclasses supply,
    /// which is what `singleton_methods` reports by default.
    fn singleton_method_names_with_ancestors(
        &mut self,
        receiver: &Object,
        include_ancestors: bool,
    ) -> Object {
        let mut names: Vec<String> = Vec::new();
        if let Object::Class(class_rc) | Object::Module(class_rc) = receiver {
            let mut current = Some(std::rc::Rc::clone(class_rc));
            while let Some(class) = current {
                // `def self.name` is stored under the `__class__` convention.
                for name in class.method_names() {
                    if let Some(bare) = name.strip_prefix("__class__")
                        && !class.is_method_private(&name)
                        && !names.contains(&bare.to_string())
                    {
                        names.push(bare.to_string());
                    }
                }
                if include_ancestors {
                    // `class << self` puts a method on the singleton class,
                    // and a subclass inherits its superclass's.
                    if let Some(singleton) = class.singleton_class_slot().clone() {
                        for name in singleton.method_names() {
                            if !name.starts_with("__")
                                && !singleton.is_method_private(&name)
                                && !names.contains(&name)
                            {
                                names.push(name);
                            }
                        }
                    }
                }
                current = if include_ancestors {
                    class.superclass()
                } else {
                    None
                };
            }
        }
        for name in self.singleton_layer_names_with_mixins(receiver, include_ancestors) {
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

/// One side of a `=~` comparison: the pattern, or the characters to search.
enum MatchSide {
    Pattern(String, String),
    Text(String),
}

/// Classify an operand of `=~`. A Symbol matches on its name, as Ruby's does.
fn matchable_text(object: &Object) -> Option<MatchSide> {
    match object {
        Object::Regex(pattern, flags) => {
            Some(MatchSide::Pattern((**pattern).clone(), (**flags).clone()))
        }
        Object::String(text) | Object::Symbol(text) => Some(MatchSide::Text((**text).clone())),
        _ => None,
    }
}

/// The public method names a class defines: everything in its table that it
/// has not marked private or protected. `def self.name` is stored under the
/// `__class__` convention and belongs to the class object, not its instances.
fn public_method_names(class: &Class) -> Vec<String> {
    class
        .method_names()
        .into_iter()
        .filter(|name| !class.is_method_private(name) && !class.is_method_protected(name))
        .collect()
}

/// Whether a block body defines a method at its top level, which inside
/// `instance_eval` or `instance_exec` means a singleton method on the
/// receiver.
fn body_defines_a_method(body: &[crate::ast::Statement]) -> bool {
    body.iter().any(|statement| {
        matches!(
            statement,
            crate::ast::Statement::MethodDef { .. } | crate::ast::Statement::FunctionDef { .. }
        )
    })
}
