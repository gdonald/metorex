use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::native_methods::is_valid_constant_name;
use crate::vm::utils::{is_truthy, position_to_location};
use std::rc::Rc;

/// The most elements an array Ruby builds may hold, which is what it refuses
/// a larger count against.
const WIDEST_ARRAY: i64 = 1152921504606846975;

/// Ruby's ArgumentError for an array asked to hold more than it can.
fn array_size_too_big(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("ArgumentError", "array size too big", position)
}

impl VirtualMachine {
    /// Where a bare `autoload` registers: the scope the call sits in. A class
    /// or module body is that scope, and inside a method body it is the
    /// module the method was written in.
    pub(crate) fn autoload_definee(&self) -> Option<Rc<crate::class::Class>> {
        if let Some(open) = self.def_scope_stack.last() {
            return Some(Rc::clone(open));
        }
        // A block carries the scope it was written in, which its own
        // `def_scope_stack` holds, so only a method body reads the nesting
        // the method captured.
        let running_a_method = self.call_stack.last().is_some_and(|frame| {
            frame.block_depth() == 0
                && matches!(
                    frame.kind(),
                    crate::vm::call_frame::FrameKind::Method { .. }
                )
        });
        if !running_a_method {
            return None;
        }
        self.method_nesting_stack
            .last()
            .and_then(|nesting| nesting.first())
            .map(Rc::clone)
    }

    /// The elements `Array.new` builds from its arguments and block, which is
    /// also what a subclass of Array starts out holding.
    pub(crate) fn build_array_elements(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut elements = Vec::new();
        self.collect_array_elements(arguments, position, &mut elements)?;
        Ok(elements)
    }

    /// The same, filling `elements` as it goes so a `break` out of the block
    /// leaves behind what it had already built.
    pub(crate) fn collect_array_elements(
        &mut self,
        arguments: &[Object],
        position: Position,
        elements: &mut Vec<Object>,
    ) -> Result<(), MetorexError> {
        if arguments.len() > 2 {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(0, 2),
                arguments.len(),
                position,
            ));
        }
        // `Array.new { ... }` names no size, so the block has nothing to
        // fill and Ruby says it went unused.
        if arguments.is_empty() && matches!(self.pending_block, Some(Object::Block(_))) {
            self.pending_block = None;
            let file = self
                .current_source_file
                .clone()
                .unwrap_or_else(|| "-".to_string());
            let message = format!(
                "{}:{}: warning: given block not used\n",
                file, position.line
            );
            self.warn_through_warning_module(message, position)?;
        }
        // A lone array names the elements rather than a size, and anything
        // that reads as one through `to_ary` names them the same way. A
        // default alongside it has nothing to fill.
        let read_as_array = match arguments.first() {
            Some(held @ Object::Array(_)) => Some(held.clone()),
            Some(held) if crate::vm::native_methods::array_subclass_value(held).is_some() => {
                crate::vm::native_methods::array_subclass_value(held)
            }
            Some(held)
                if !matches!(held, Object::Int(_) | Object::Nil)
                    && self.responds_to(held, "to_ary") =>
            {
                Some(self.send_to_object(held.clone(), "to_ary", vec![], position)?)
            }
            _ => None,
        };
        if let Some(Object::Array(held)) = read_as_array {
            if arguments.len() > 1 {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "no implicit conversion of Array into Integer",
                    position,
                ));
            }
            self.pending_block.take();
            elements.extend(held.borrow().iter().cloned());
            return Ok(());
        }
        let size = match arguments.first() {
            None => 0_i64,
            Some(Object::Int(n)) => *n,
            // A count past the widest array Ruby builds is refused rather
            // than attempted.
            Some(Object::BigInt(_)) => return Err(array_size_too_big(position)),
            Some(Object::Nil) => {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "no implicit conversion from nil to integer",
                    position,
                ));
            }
            Some(other) if self.responds_to(other, "to_int") => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    Object::Int(held) => held,
                    _ => {
                        return Err(method_argument_type_error(
                            "Array.new",
                            "Integer",
                            other,
                            position,
                        ));
                    }
                }
            }
            Some(other) => {
                return Err(method_argument_type_error(
                    "Array.new",
                    "Integer",
                    other,
                    position,
                ));
            }
        };
        if size < 0 {
            let message = "negative array size".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        if size > WIDEST_ARRAY {
            return Err(array_size_too_big(position));
        }
        let block = self.pending_block.take();
        // A block fills every place, so a default named alongside it has
        // nothing left to fill and Ruby says so.
        if matches!(block, Some(Object::Block(_))) && arguments.len() > 1 {
            let file = self
                .current_source_file
                .clone()
                .unwrap_or_else(|| "-".to_string());
            let message = format!(
                "{}:{}: warning: block supersedes default value argument\n",
                file, position.line
            );
            self.warn_through_warning_module(message, position)?;
        }
        if let Some(Object::Block(body)) = block {
            // The block may take 0 args (`Array.new(10) { rand }`) or 1
            // (`Array.new(10) { |i| ... }`). Match metorex's strict arity
            // check by only passing the index when the block declares a
            // positional parameter for it.
            let pass_index = body
                .parameters
                .iter()
                .any(|name| !name.starts_with('&') && !name.starts_with('*'));
            for index in 0..size {
                let block_arguments = if pass_index {
                    vec![Object::Int(index)]
                } else {
                    vec![]
                };
                let value = self.execute_block_callable(&body, block_arguments, position)?;
                elements.push(value);
            }
        } else {
            let default = arguments.get(1).cloned().unwrap_or(Object::Nil);
            for _ in 0..size {
                elements.push(default.clone());
            }
        }
        Ok(())
    }

    pub(crate) fn call_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Some(result) =
            self.call_warning_methods(class_rc, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }
        // A refinement names the class it refines.
        if matches!(method_name, "target" | "refined_class")
            && let Some(found) =
                class_rc.get_class_var(super::module_methods::REFINEMENT_TARGET_KEY)
        {
            return Ok(Some(found));
        }
        // Every symbol the program has spelled, which the parser records as
        // it reads them and the constructor records as they are made.
        if class_rc.name() == "Symbol" && method_name == "all_symbols" {
            let named = crate::symbol_registry::all()
                .into_iter()
                .map(Object::symbol)
                .collect();
            return Ok(Some(Object::array(named)));
        }
        // The shape of a network address belongs to the operating system, so
        // the socket library reads and writes them here.
        if class_rc.name() == "Socket" && method_name == "__address__" {
            return self.socket_address(arguments, position).map(Some);
        }
        // The name the C library gives the encoding the locale calls for,
        // which differs between platforms for one and the same locale.
        if class_rc.name() == "Encoding" && method_name == "__charmap__" {
            return Ok(Some(Object::string(crate::vm::locale_charmap_name())));
        }
        if class_rc.name() == "Socket" && method_name == "__net__" {
            return self.socket_net(arguments, position).map(Some);
        }
        if class_rc.name() == "IO" && method_name == "__stream__" {
            return self.stream_action(arguments, position).map(Some);
        }
        // `IO.__write_standard__(name, text)` writes through the
        // interpreter's own writer, which is what keeps a program's output in
        // the order it wrote it whichever route each piece took.
        if class_rc.name() == "IO" && method_name == "__write_standard__" {
            let named = match arguments.first() {
                Some(Object::String(held)) => held.as_str().to_string(),
                _ => String::new(),
            };
            let text = match arguments.get(1) {
                Some(Object::String(held)) => held.as_str().to_string(),
                other => other.map(|held| held.to_string()).unwrap_or_default(),
            };
            crate::vm::native_functions::write_to_standard_stream(&named, &text);
            return Ok(Some(Object::Nil));
        }
        // A class that defines `new` of its own builds its instances that
        // way, rather than through the allocate-and-initialize every class
        // is given.
        if method_name == "new" && self.class_method_of(class_rc, "new").is_some() {
            return Ok(None);
        }
        // `Integer.sqrt` and `Integer.try_convert` belong to the class rather
        // than to a number.
        if class_rc.name() == "Integer" && matches!(method_name, "sqrt" | "try_convert") {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            return self
                .integer_class_method(method_name, &arguments[0], position)
                .map(Some);
        }
        // `String.try_convert` belongs to the class rather than to a string.
        // `Encoding.__source__` names the encoding the source running now is
        // written in, which is what `__ENCODING__` answers.
        if class_rc.name() == "Encoding"
            && method_name == "__running_source__"
            && arguments.is_empty()
        {
            let named = self
                .current_source_encoding
                .clone()
                .map(|held| {
                    crate::vm::native_methods::string_methods::canonical_encoding_name(&held)
                })
                .unwrap_or_else(|| crate::object::string_value::DEFAULT_ENCODING.to_string());
            let found = self.send_to_object(
                Object::Class(Rc::clone(class_rc)),
                "find",
                vec![Object::string(named)],
                position,
            )?;
            return Ok(Some(found));
        }
        if class_rc.name() == "String" && method_name == "try_convert" {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            return self.string_try_convert(&arguments[0], position).map(Some);
        }
        // The top-level `self` is Object, so a `using` sent to it is
        // `main.using`. Ruby permits that only at the top level, which a
        // class or module body is not.
        if method_name == "using" && class_rc.name() == "Object" {
            let inside_body = matches!(
                self.environment().get("self"),
                Some(Object::Class(current) | Object::Module(current)) if current.name() != "Object"
            );
            if inside_body {
                let message = "main.using is permitted only at toplevel".to_string();
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("RuntimeError", message.clone()),
                    location: position_to_location(position),
                    message,
                });
            }
            return self
                .call_native_function("using", arguments.to_vec(), position)
                .map(Some);
        }
        // A number is not built by hand: `Float.new`, `Rational.new` and
        // `Complex.new` do not exist, and `allocate` has nothing to allocate.
        if matches!(
            class_rc.name(),
            "Float" | "Rational" | "Complex" | "Integer"
        ) && matches!(method_name, "new" | "allocate")
        {
            let message = format!(
                "undefined method '{}' for class '{}'",
                method_name,
                class_rc.name()
            );
            let exception = if method_name == "allocate" {
                Object::exception(
                    "TypeError",
                    format!("allocator undefined for {}", class_rc.name()),
                )
            } else {
                crate::vm::errors::no_method_error(
                    &message,
                    method_name,
                    &Object::Class(Rc::clone(class_rc)),
                    arguments,
                )
            };
            return Err(MetorexError::UncaughtException {
                exception,
                location: position_to_location(position),
                message,
            });
        }
        // A Thread has nothing to be without the block that gives it something
        // to run, so Ruby refuses to hand back an uninitialized one.
        // An allocated String holds nothing and is read as bytes until it is
        // told otherwise.
        if method_name == "allocate" && class_rc.name() == "String" {
            return Ok(Some(Object::String(Rc::new(
                crate::object::StringValue::from_bytes(String::new()),
            ))));
        }
        if method_name == "allocate" && class_rc.name() == "Thread" {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                "allocator undefined for Thread",
                position,
            ));
        }
        // `allocate` on a class whose instances are primitives answers an
        // empty one of them, since there is no separate uninitialized form.
        if method_name == "allocate" && matches!(class_rc.name(), "Array" | "Hash" | "Set") {
            if !arguments.is_empty() {
                return Err(method_argument_error(
                    "allocate",
                    0,
                    arguments.len(),
                    position,
                ));
            }
            return Ok(Some(match class_rc.name() {
                "Array" => Object::empty_array(),
                "Hash" => Object::empty_dict(),
                _ => Object::empty_set(),
            }));
        }
        // A Proc has no allocator, and MatchData has no `allocate` at all.
        if method_name == "allocate" && class_rc.name() == "MatchData" {
            let message = "undefined method 'allocate' for class 'MatchData'".to_string();
            return Err(MetorexError::UncaughtException {
                exception: crate::vm::errors::no_method_error(
                    &message,
                    method_name,
                    &Object::Class(Rc::clone(class_rc)),
                    arguments,
                ),
                location: position_to_location(position),
                message,
            });
        }
        // `Proc.new` builds one from a block, but there is no uninitialized
        // Proc for `allocate` to hand back.
        let non_instantiable = matches!(
            class_rc.name(),
            "TrueClass" | "FalseClass" | "NilClass" | "Symbol"
        );
        if (non_instantiable || class_rc.name() == "Proc") && method_name == "allocate" {
            let exc = Object::exception(
                "TypeError",
                format!("allocator undefined for {}", class_rc.name()),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: format!("allocator undefined for {}", class_rc.name()),
            });
        }
        // Class.allocate and subclasses: uninitialized class instance. `new` and
        // `superclass` on it must raise TypeError (Ruby semantics).
        if class_rc.get_class_var("__uninitialized__").is_some()
            && matches!(method_name, "new" | "superclass")
        {
            let message = "uninitialized class".to_string();
            let exc = Object::exception("TypeError", message.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message,
            });
        }
        // A class that defines its own `allocate` answers with that, so the
        // uninitialized instance below is not what it hands back.
        if method_name == "allocate" && self.class_method_of(class_rc, "allocate").is_some() {
            return Ok(None);
        }
        if method_name == "allocate" {
            if class_rc.name() == "Class" {
                let anon = Rc::new(Class::new("", None));
                anon.set_class_var("__uninitialized__", Object::Bool(true));
                return Ok(Some(Object::Class(anon)));
            }
            let inst = crate::object::Instance::new(Rc::clone(class_rc));
            return Ok(Some(Object::Instance(Rc::new(std::cell::RefCell::new(
                inst,
            )))));
        }
        // Class#initialize: private; already-initialized classes raise
        // TypeError, and passing `Class` itself as the superclass argument also
        // raises TypeError (MRI rejects `Class` as a superclass regardless of
        // whether the receiver was freshly allocated).
        if method_name == "initialize" {
            if let Some(Object::Class(c)) = arguments.first()
                && c.name() == "Class"
            {
                let msg = "already initialized class".to_string();
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            if class_rc.get_class_var("__uninitialized__").is_none() {
                let msg = "already initialized class".to_string();
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            class_rc.remove_class_var("__uninitialized__");
            return Ok(Some(Object::Nil));
        }
        if method_name == "constants" {
            // Collect one class/module's visible constants: its constant
            // table (public, uppercase-leading), plus registered autoloads
            // and names whose autoload fired without defining the constant
            // (MRI keeps those in `#constants` even though `const_defined?`
            // and `autoload?` both report nothing).
            let collect_from = |cls: &Rc<Class>, names: &mut Vec<String>| {
                for n in cls.class_var_names() {
                    if n.starts_with("__")
                        || !n.chars().next().is_some_and(|c| c.is_uppercase())
                        || cls.is_private_constant(&n)
                    {
                        continue;
                    }
                    if !names.contains(&n) {
                        names.push(n);
                    }
                }
                for n in cls
                    .autoload_names()
                    .into_iter()
                    .chain(cls.unrealized_autoload_names())
                {
                    if !names.contains(&n) {
                        names.push(n);
                    }
                }
            };
            // `Module.constants` with no argument is special-cased by Ruby
            // to the constants reachable at the call site — for our model,
            // the top-level constants (globals plus Object's table).
            if class_rc.name() == "Module" && arguments.is_empty() {
                let mut names: Vec<String> = self
                    .globals()
                    .iter()
                    .map(|(n, _)| n.clone())
                    .filter(|n| {
                        !n.starts_with("__")
                            && !n.contains("::")
                            && n.chars().next().is_some_and(|c| c.is_uppercase())
                    })
                    .collect();
                if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                    collect_from(&object_class, &mut names);
                }
                let names: Vec<Object> = names.into_iter().map(Object::symbol).collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(names)))));
            }
            // constants(inherit = true): with inherit, include constants
            // from mixins (transitively) and the superclass chain — but not
            // Object's, which are top-level constants.
            let inherit = match arguments.first() {
                None => true,
                Some(v) => crate::vm::utils::is_truthy(v),
            };
            let mut names: Vec<String> = Vec::new();
            collect_from(class_rc, &mut names);
            // Every top-level constant is a constant of Object, including the
            // built-in class names, which live in the global registry rather
            // than in Object's own table.
            if class_rc.name() == "Object" {
                for (name, _) in self.globals().iter() {
                    if name.starts_with("__")
                        || name.contains("::")
                        || !name.chars().next().is_some_and(|c| c.is_uppercase())
                        || names.contains(name)
                    {
                        continue;
                    }
                    names.push(name.clone());
                }
            }
            if inherit {
                let mut queue: Vec<Rc<Class>> = class_rc.prepend_chain();
                queue.extend(class_rc.mixin_chain());
                let mut cursor = class_rc.superclass();
                while let Some(sc) = cursor {
                    if matches!(sc.name(), "Object" | "BasicObject") {
                        break;
                    }
                    queue.push(Rc::clone(&sc));
                    cursor = sc.superclass();
                }
                let mut seen: Vec<*const Class> = vec![Rc::as_ptr(class_rc)];
                let mut idx = 0;
                while idx < queue.len() {
                    let current = Rc::clone(&queue[idx]);
                    idx += 1;
                    let ptr = Rc::as_ptr(&current);
                    if seen.contains(&ptr) {
                        continue;
                    }
                    seen.push(ptr);
                    collect_from(&current, &mut names);
                    for prepended in current.prepend_chain() {
                        queue.push(prepended);
                    }
                    for mixin in current.mixin_chain() {
                        queue.push(mixin);
                    }
                }
            }
            let names: Vec<Object> = names.into_iter().map(Object::symbol).collect();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(names)))));
        }
        if method_name == "attached_object" {
            let is_singleton = class_rc.get_class_var("__singleton__").is_some();
            if !is_singleton {
                let msg = format!("'{}' is not a singleton class", class_rc.name());
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            let attached = class_rc
                .get_class_var("__attached__")
                .unwrap_or(Object::Nil);
            // Singleton classes of nil / true / false exist but their attached
            // object can't be obtained directly — MRI raises TypeError here.
            let tag = match &attached {
                Object::Nil => Some("NilClass"),
                Object::Bool(true) => Some("TrueClass"),
                Object::Bool(false) => Some("FalseClass"),
                _ => None,
            };
            if let Some(name) = tag {
                let msg = format!("'{}' is not a singleton class", name);
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            return Ok(Some(attached));
        }
        if non_instantiable && method_name == "new" {
            let exc = Object::exception(
                "NoMethodError",
                format!("undefined method 'new' for {}:Class", class_rc.name()),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: format!("undefined method 'new' for {}:Class", class_rc.name()),
            });
        }
        if class_rc.name() == "Kernel"
            && let Some(result) = self.call_kernel_conversion(method_name, arguments, position)?
        {
            return Ok(Some(result));
        }
        if class_rc.name() == "Kernel" && method_name == "abort" {
            return self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some);
        }
        // `Kernel.block_given?` reports on the frame that called it, the same
        // as the bare form.
        if class_rc.name() == "Kernel" && method_name == "block_given?" {
            return Ok(Some(Object::Bool(matches!(
                self.environment().get("block_given?"),
                Some(Object::Bool(true))
            ))));
        }
        if class_rc.name() == "Kernel" && method_name == "binding" {
            return self
                .call_native_function("binding_kernel", arguments.to_vec(), position)
                .map(Some);
        }
        if method_name == "new" && class_rc.name() == "Class" {
            let superclass = match arguments.first() {
                Some(Object::Class(c)) => {
                    // Singleton (meta) classes can't be used as a superclass —
                    // MRI raises TypeError("can't make subclass of singleton
                    // class") in this case.
                    if c.get_class_var("__singleton__").is_some() {
                        return Err(MetorexError::type_error(
                            "can't make subclass of singleton class",
                            position_to_location(position),
                        ));
                    }
                    Some(Rc::clone(c))
                }
                Some(other) => {
                    return Err(MetorexError::type_error(
                        format!("superclass must be a Class (given {})", other.type_name()),
                        position_to_location(position),
                    ));
                }
                None => self.globals().get("Object").and_then(|o| {
                    if let Object::Class(c) = o {
                        Some(c)
                    } else {
                        None
                    }
                }),
            };
            let anon = Rc::new(Class::new("", superclass.clone()));
            if let Some(sc) = &superclass {
                sc.add_subclass(&anon);
            }
            // Extract the pending block before triggering the `inherited`
            // hook — the hook's own invoke_method would otherwise consume it.
            let pending = self.pending_block.take();
            // Ruby: `inherited` hook fires before the block runs, so a hook
            // that records `self` sees the parent first, then the block's
            // `self` push appends the subclass.
            if let Some(sc) = &superclass {
                self.trigger_inherited_hook(sc, Rc::clone(&anon), position)?;
            }
            if let Some(Object::Block(block)) = pending {
                self.apply_block_as_class_body(&anon, &block, position)?;
            }
            return Ok(Some(Object::Class(anon)));
        }
        if method_name == "new" && class_rc.name() == "Module" {
            let anon = Rc::new(Class::new_module(""));
            if let Some(Object::Block(block)) = self.pending_block.take() {
                self.apply_block_as_class_body_with_self(
                    &anon,
                    &block,
                    position,
                    Object::Module(Rc::clone(&anon)),
                )?;
            }
            return Ok(Some(Object::Module(anon)));
        }
        // `autoload :CONST, "path"` — register the constant→path mapping.
        // `autoload?(:CONST, [inherit=true])` returns the registered path.
        if method_name == "autoload" {
            // `Kernel.autoload` registers where the caller sits, the same way
            // the bare form does, rather than on Kernel itself.
            if class_rc.name() == "Kernel"
                && let Some(definee) = self.autoload_definee()
                && !Rc::ptr_eq(&definee, class_rc)
            {
                return self.call_class_methods(&definee, method_name, arguments, position);
            }
            let const_name = match arguments.first() {
                Some(Object::Symbol(s)) => s.as_str().to_string(),
                Some(Object::String(s)) => s.as_str().to_string(),
                _ => return Ok(Some(Object::Nil)),
            };
            if !is_valid_constant_name(&const_name) {
                let msg = format!("autoload must be constant name: {}", const_name);
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            if class_rc.is_frozen() {
                let msg = format!(
                    "can't modify frozen {}: {}",
                    class_rc.kind_name(),
                    class_rc.name()
                );
                let exc = Object::exception("FrozenError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            let path = match arguments.get(1) {
                Some(Object::String(s)) => s.as_str().to_string(),
                Some(Object::Symbol(s)) => s.as_str().to_string(),
                Some(other) => {
                    let other_obj = other.clone();
                    if let Some((cls, method)) = self.lookup_method(&other_obj, "to_path") {
                        let result =
                            self.invoke_method(cls, method, other_obj, Vec::new(), position)?;
                        match result {
                            Object::String(s) => s.as_str().to_string(),
                            _ => {
                                let msg = "to_path must return a String".to_string();
                                let exc = Object::exception("TypeError", msg.clone());
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: msg,
                                });
                            }
                        }
                    } else {
                        let msg = format!(
                            "no implicit conversion of {} into String",
                            other.type_name()
                        );
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
                None => return Ok(Some(Object::Nil)),
            };
            if path.is_empty() {
                let msg = "empty file name".to_string();
                let exc = Object::exception("ArgumentError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            let caller_file = self
                .get_current_file()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            class_rc.set_autoload_location(const_name.clone(), caller_file, position.line as i64);
            class_rc.set_autoload(const_name.clone(), path);
            self.trigger_const_added_hook(
                Object::Class(Rc::clone(class_rc)),
                &const_name,
                position,
            )?;
            return Ok(Some(Object::Nil));
        }
        // `Encoding.default_external` names the encoding metorex reads and
        // writes with. Every string is UTF-8, so the setting is remembered and
        // reported without changing how one is stored.
        if class_rc.name() == "Encoding"
            && matches!(method_name, "default_external" | "default_external=")
        {
            if method_name == "default_external=" {
                let Some(value) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                // The external encoding always names one, so nothing stands
                // for "no encoding" there.
                if matches!(value, Object::Nil) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "default external cannot be nil",
                        position,
                    ));
                }
                let settled = self.encoding_setting(class_rc, value, position)?;
                self.globals_mut()
                    .set("__Encoding_default_external", settled);
                return Ok(Some(value.clone()));
            }
            return Ok(Some(
                self.globals()
                    .get("__Encoding_default_external")
                    .or_else(|| self.globals().get("Encoding::UTF_8"))
                    .unwrap_or(Object::Nil),
            ));
        }
        // `Encoding.default_internal` names the encoding a string is
        // converted to on the way in. Metorex converts nothing, so it stays
        // nil unless a program sets it.
        if class_rc.name() == "Encoding"
            && matches!(method_name, "default_internal" | "default_internal=")
        {
            if method_name == "default_internal=" {
                let Some(value) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                let settled = self.encoding_setting(class_rc, value, position)?;
                self.globals_mut()
                    .set("__Encoding_default_internal", settled);
                return Ok(Some(value.clone()));
            }
            return Ok(Some(
                self.globals()
                    .get("__Encoding_default_internal")
                    .unwrap_or(Object::Nil),
            ));
        }
        // `Encoding.find` answers the encoding a name stands for. The names
        // that stand for a setting rather than an encoding answer what that
        // setting holds.
        // An encoding is a class standing under Encoding, so the readings an
        // encoding answers are handled here rather than from Encoding's own
        // instance methods.
        if class_rc
            .superclass()
            .is_some_and(|parent| parent.name() == "Encoding")
        {
            match method_name {
                "dummy?" => {
                    return Ok(Some(Object::Bool(
                        crate::vm::init::ENCODING_NAMES
                            .iter()
                            .any(|(_, display, dummy)| *dummy && *display == class_rc.name()),
                    )));
                }
                // A dummy encoding converts nothing, so nothing it holds
                // stands for ASCII either. The wide UTF forms are not ASCII
                // compatible for the plainer reason that their code units are
                // more than a byte.
                "ascii_compatible?" => {
                    let dummy = crate::vm::init::ENCODING_NAMES
                        .iter()
                        .any(|(_, display, dummy)| *dummy && *display == class_rc.name());
                    return Ok(Some(Object::Bool(
                        !dummy
                            && !class_rc.name().starts_with("UTF-16")
                            && !class_rc.name().starts_with("UTF-32"),
                    )));
                }
                "inspect" => {
                    // Ruby shows ASCII-8BIT under the name BINARY, with the
                    // name it reports alongside.
                    let dummy = crate::vm::init::ENCODING_NAMES
                        .iter()
                        .any(|(_, display, dummy)| *dummy && *display == class_rc.name());
                    let shown = if class_rc.name() == "ASCII-8BIT" {
                        "BINARY (ASCII-8BIT)".to_string()
                    } else {
                        class_rc.name().to_string()
                    };
                    let tail = if dummy { " (dummy)" } else { "" };
                    return Ok(Some(Object::string(format!(
                        "#<Encoding:{}{}>",
                        shown, tail
                    ))));
                }
                _ => {}
            }
        }

        // `Encoding.compatible?` answers the encoding two objects could be
        // read in together, or nil when there is none.
        if class_rc.name() == "Encoding" && method_name == "compatible?" {
            if arguments.len() != 2 {
                return Err(method_argument_error(
                    method_name,
                    2,
                    arguments.len(),
                    position,
                ));
            }
            let answer = self.compatible_encoding(&arguments[0], &arguments[1]);
            return Ok(Some(match answer {
                Some(named) => self.encoding_object(&named),
                None => Object::Nil,
            }));
        }
        // Every encoding metorex names, each one listed once however many
        // constants reach it.
        if class_rc.name() == "Encoding" && method_name == "list" {
            return Ok(Some(
                self.globals()
                    .get("__Encoding_list")
                    .unwrap_or_else(|| Object::array(Vec::new())),
            ));
        }
        // The names that stand for an encoding already listed under another
        // name, each paired with the name it stands for.
        if class_rc.name() == "Encoding" && method_name == "aliases" {
            let mut seen: Vec<&str> = Vec::new();
            let mut pairs: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
            for (constant, display, _) in crate::vm::init::ENCODING_NAMES {
                if seen.contains(&display) {
                    pairs.insert(constant.to_string(), Object::string(display.to_string()));
                } else {
                    seen.push(display);
                }
            }
            // Ruby lists the settings among the aliases, so "external" and
            // "locale" name the encodings they stand for.
            for named in ["external", "locale"] {
                let found =
                    self.call_class_methods(class_rc, "find", &[Object::string(named)], position)?;
                if let Some(encoding) = found {
                    let name = self.send_to_object(encoding, "name", Vec::new(), position)?;
                    pairs.insert(named.to_string(), name);
                }
            }
            return Ok(Some(Object::Dict(Rc::new(std::cell::RefCell::new(pairs)))));
        }
        if class_rc.name() == "Encoding" && matches!(method_name, "find" | "[]") {
            let wanted = match arguments.first() {
                Some(Object::String(held)) => held.as_str().to_string(),
                Some(Object::Class(held)) if held.name().contains('-') => {
                    return Ok(Some(arguments[0].clone()));
                }
                // An encoding is named by a String, and a Symbol is not one.
                // Anything that spells itself out names one all the same.
                Some(other @ Object::Symbol(_)) => {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        other,
                        position,
                    ));
                }
                Some(other) if self.responds_to(other, "to_str") => {
                    let spelled =
                        self.send_to_object(other.clone(), "to_str", Vec::new(), position)?;
                    match spelled {
                        Object::String(held) => held.as_str().to_string(),
                        held => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                &held,
                                position,
                            ));
                        }
                    }
                }
                _ => return Err(method_argument_error(method_name, 1, 0, position)),
            };
            let settled =
                match wanted.to_ascii_lowercase().as_str() {
                    // The locale names an encoding of its own, which is
                    // what the default external starts as but need not stay.
                    "locale" => {
                        let named = Object::Class(Rc::clone(class_rc));
                        let charmap =
                            self.send_to_object(named, "locale_charmap", Vec::new(), position)?;
                        if let charmap @ Object::String(_) = charmap {
                            return self.call_class_methods(
                                class_rc,
                                "find",
                                std::slice::from_ref(&charmap),
                                position,
                            );
                        }
                        return Ok(Some(
                            self.globals()
                                .get("__Encoding_default_external")
                                .or_else(|| self.globals().get("Encoding::UTF_8"))
                                .unwrap_or(Object::Nil),
                        ));
                    }
                    "external" | "filesystem" => {
                        return Ok(Some(
                            self.globals()
                                .get("__Encoding_default_external")
                                .or_else(|| self.globals().get("Encoding::UTF_8"))
                                .unwrap_or(Object::Nil),
                        ));
                    }
                    "internal" => {
                        return Ok(Some(
                            self.globals()
                                .get("__Encoding_default_internal")
                                .unwrap_or(Object::Nil),
                        ));
                    }
                    "utf8" => "UTF_8",
                    "ascii" | "ansi_x3.4-1968" => "US_ASCII",
                    "binary" => "BINARY",
                    "sjis" => "SHIFT_JIS",
                    // Every other name is matched against the table the constants
                    // were built from, so a constant and the name it reports find
                    // the same encoding.
                    _ => {
                        let Some((settled, _, _)) = crate::vm::init::ENCODING_NAMES.iter().find(
                            |(constant, display, _)| {
                                constant.eq_ignore_ascii_case(&wanted)
                                    || display.eq_ignore_ascii_case(&wanted)
                            },
                        ) else {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &format!("unknown encoding name - {wanted}"),
                                position,
                            ));
                        };
                        settled
                    }
                };
            return Ok(Some(
                self.globals()
                    .get(&format!("Encoding::{settled}"))
                    .unwrap_or(Object::Nil),
            ));
        }
        if method_name == "autoload?" {
            let const_name = match arguments.first() {
                Some(Object::Symbol(s)) => s.as_str().to_string(),
                Some(Object::String(s)) => s.as_str().to_string(),
                _ => return Ok(Some(Object::Nil)),
            };
            let inherit = !matches!(arguments.get(1), Some(Object::Bool(false)));
            let class_for_autoload = Rc::clone(class_rc);
            let local_only_blocked = !inherit && class_rc.get_autoload(&const_name).is_none();
            let path = if local_only_blocked {
                None
            } else {
                self.effective_autoload(&class_for_autoload, &const_name)
            };
            return Ok(Some(match path {
                Some(p) => Object::string(p),
                None => Object::Nil,
            }));
        }
        // `Klass.include(Mod)` / `Klass.prepend(Mod)`: mix the module into
        // the class through the `append_features` dispatch path so user
        // overrides on the module's singleton class fire and the cyclic /
        // frozen checks run. `prepend` ordering is still approximated as a
        // regular include (sufficient for current fixture setup).
        // The hooks whose default implementation does nothing and returns
        // nil. `included` and friends with real behavior are handled above.
        if matches!(
            method_name,
            "method_added"
                | "method_removed"
                | "method_undefined"
                | "included"
                | "extended"
                | "prepended"
        ) && arguments.len() == 1
            && !has_user_defined_method(class_rc, method_name)
        {
            return Ok(Some(Object::Nil));
        }
        // `Klass.include?(Mod)` — whether Mod appears in the ancestors,
        // excluding the receiver itself. A class argument is a TypeError.
        if method_name == "include?" && arguments.len() == 1 {
            let Object::Module(queried) = &arguments[0] else {
                return Err(method_argument_type_error(
                    method_name,
                    "Module",
                    &arguments[0],
                    position,
                ));
            };
            let mut chain: Vec<Object> = Vec::new();
            let mut seen: Vec<*const Class> = Vec::new();
            push_class_ancestors(class_rc, &mut chain, &mut seen);
            let found = chain.iter().any(|ancestor| match ancestor {
                Object::Class(c) | Object::Module(c) => {
                    Rc::ptr_eq(c, queried) && !Rc::ptr_eq(c, class_rc)
                }
                _ => false,
            });
            return Ok(Some(Object::Bool(found)));
        }
        // Bare `include` / `prepend` inside a class or module body: Ruby
        // reports the missing argument rather than a missing method.
        if matches!(method_name, "include" | "prepend")
            && arguments.is_empty()
            && !has_user_defined_method(class_rc, method_name)
        {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                0,
                position,
            ));
        }
        // A refinement holds methods for one class only, and Ruby removed both
        // of these from it rather than leave a way to mix into it.
        if matches!(method_name, "include" | "prepend")
            && class_rc
                .get_class_var(crate::vm::native_methods::REFINEMENT_LABEL_KEY)
                .is_some()
        {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("Refinement#{} has been removed", method_name),
                position,
            ));
        }
        if matches!(method_name, "include" | "prepend") && !arguments.is_empty() {
            // Ruby applies the arguments in reverse, so the first module
            // listed ends up nearest the receiver in the ancestor chain.
            for arg in arguments.iter().rev() {
                if let Some(module_rc) =
                    self.resolve_include_argument(arg, method_name, position)?
                {
                    if method_name == "prepend" {
                        self.apply_module_prepend(class_rc, &module_rc, position)?;
                    } else {
                        self.apply_module_include(class_rc, &module_rc, position)?;
                    }
                }
            }
            return Ok(Some(Object::Class(Rc::clone(class_rc))));
        }
        // `mod.append_features(target)` / `mod.prepend_features(target)`:
        // default behavior — add `mod` as a mixin on `target`, with the
        // standard cyclic/frozen checks. Defer to a user-defined override
        // (singleton method on the receiver) when one is present.
        if matches!(method_name, "append_features" | "prepend_features") && !arguments.is_empty() {
            let class_method_key = format!("__class__{}", method_name);
            if class_rc.find_method(&class_method_key).is_some() {
                return Ok(None);
            }
            if let Some(sc) = class_rc.singleton_class_slot().clone()
                && sc.find_method(method_name).is_some()
            {
                return Ok(None);
            }
            for arg in arguments {
                match arg {
                    Object::Module(t) | Object::Class(t) => {
                        if method_name == "prepend_features" {
                            self.default_prepend_features(t, class_rc, position)?;
                        } else {
                            self.default_append_features(t, class_rc, position)?;
                        }
                    }
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Module",
                            other,
                            position,
                        ));
                    }
                }
            }
            return Ok(Some(Object::Class(Rc::clone(class_rc))));
        }
        // `Klass.subclasses` returns the direct subclasses (Class objects).
        if method_name == "subclasses" {
            let subs: Vec<Object> = class_rc
                .subclasses()
                .into_iter()
                .map(Object::Class)
                .collect();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(subs)))));
        }
        // `Module.nesting` returns the modules/classes currently being defined
        // (innermost first). We approximate with the def_scope_stack snapshot.
        // Regexp.escape / Regexp.quote: the string with every regex
        // metacharacter escaped, so it matches itself literally.
        if class_rc.name() == "Regexp"
            && matches!(method_name, "escape" | "quote")
            && arguments.len() == 1
        {
            let source = self.coerce_name_argument(&arguments[0], position)?;
            let named = match &arguments[0] {
                Object::String(held) => held.encoding_name(),
                _ => "US-ASCII".to_string(),
            };
            let escaped = quoted_for_pattern(&source);
            // Ruby tags the answer US-ASCII when nothing but ASCII went in,
            // and keeps the source's own encoding otherwise.
            let tagged = if source.is_ascii() {
                "US-ASCII".to_string()
            } else {
                named
            };
            let made = crate::object::StringValue::with_encoding(escaped, tagged);
            // Characters standing for bytes go on standing for them in the
            // pattern built out of them.
            if matches!(&arguments[0], Object::String(held) if held.holds_bytes()) {
                made.mark_bytes();
            }
            return Ok(Some(Object::String(Rc::new(made))));
        }
        // `Regexp.last_match` answers the whole MatchData, and with a number
        // the capture that number names.
        if class_rc.name() == "Regexp" && method_name == "last_match" {
            let last = self
                .globals()
                .get(crate::vm::native_methods::LAST_MATCH)
                .unwrap_or(Object::Nil);
            return match arguments.first() {
                None => Ok(Some(last)),
                Some(_) if matches!(last, Object::Nil) => Ok(Some(Object::Nil)),
                Some(index) => self
                    .send_to_object(last, "[]", vec![index.clone()], position)
                    .map(Some),
            };
        }
        // `Regexp.new` / `Regexp.compile` build a pattern from a source
        // string, with the second argument turning case folding on.
        // `__literal__` is how a pattern written out with `#{}` in it is
        // built once its parts are known. A literal names its encoding with
        // `n` and `u`, which `new` refuses.
        if class_named_in_chain(class_rc, "Regexp")
            && matches!(method_name, "new" | "compile" | "__literal__")
        {
            let written_out = method_name == "__literal__";
            // A pattern written with `o` is built once and stands for every
            // time the line it is written on is reached again.
            let built_once = match arguments.get(2) {
                Some(Object::String(site)) => Some(site.as_str().to_string()),
                _ => None,
            };
            if let Some(site) = &built_once
                && let Some(held) = self.patterns_built_once.get(site)
            {
                return Ok(Some(held.clone()));
            }
            let source_encoding = match arguments.first() {
                Some(Object::String(text)) => Some(text.encoding_name()),
                _ => None,
            };
            let source = match arguments.first() {
                Some(Object::Regex(pattern, flags)) => {
                    // A pattern built from another one keeps that one's flags,
                    // and says so about any written alongside it.
                    if arguments.len() > 1 && !matches!(arguments[1], Object::Nil) {
                        self.emit_warning_to_stderr("warning: flags ignored", position);
                    }
                    return Ok(Some(Object::Regex(Rc::clone(pattern), Rc::clone(flags))));
                }
                // Only a String, or something answering `to_str`, spells a
                // pattern: a Symbol names no source.
                Some(Object::String(text)) => text.as_str().to_string(),
                Some(argument) => {
                    let named = self.builtins().class_of(argument).name().to_string();
                    if !self.answers_to(argument, "to_str", position)? {
                        let message = format!("no implicit conversion of {} into String", named);
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    }
                    match self.send_to_object(argument.clone(), "to_str", vec![], position)? {
                        Object::String(text) => text.as_str().to_string(),
                        _ => {
                            let message = format!("can\'t convert {} into String", named);
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                &message,
                                position,
                            ));
                        }
                    }
                }
                None => {
                    return Err(method_argument_error("new", 1, 0, position));
                }
            };
            refuse_bad_pattern(&source, position)?;
            let mut flags = String::new();
            match arguments.get(1) {
                Some(Object::Int(options)) => {
                    if options & 1 != 0 {
                        flags.push('i');
                    }
                    if options & 2 != 0 {
                        flags.push('x');
                    }
                    if options & 4 != 0 {
                        flags.push('m');
                    }
                    // A pattern told to match in one encoding whatever the
                    // text is tagged with carries that as the `u` a literal
                    // would have been written with, and one told to match
                    // bytes carries the `n`.
                    if options & 16 != 0 {
                        flags.push('u');
                    }
                    if options & 32 != 0 {
                        flags.push('n');
                    }
                }
                // A String names the flags by the letters a literal is
                // written with, which are the three that change how the
                // pattern reads and the two that name the encoding it
                // matches in.
                Some(Object::String(written)) => {
                    let spelling = written.as_str().to_string();
                    if spelling.chars().any(|held| {
                        !(matches!(held, 'i' | 'm' | 'x')
                            || (written_out && matches!(held, 'n' | 'u' | 'o')))
                    }) {
                        let message = format!("unknown regexp option: {}", spelling);
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &message,
                            position,
                        ));
                    }
                    // `o` says when the pattern is built rather than what it
                    // matches, so the built one does not carry it.
                    for held in ['i', 'm', 'x', 'n', 'u'] {
                        if spelling.contains(held) {
                            flags.push(held);
                        }
                    }
                }
                Some(Object::Bool(true)) => flags.push('i'),
                Some(Object::Bool(false)) | Some(Object::Nil) | None => {}
                // A keyword hash in that place names `timeout:`, which is
                // read below rather than as an ignorecase argument.
                Some(Object::Dict(_)) => {}
                Some(other) => {
                    // Anything else is read as a plain truth, which Ruby says
                    // so about rather than asking it for a number.
                    let shown = self.send_to_object(other.clone(), "inspect", vec![], position)?;
                    let message =
                        format!("warning: expected true or false as ignorecase: {}", shown);
                    self.emit_warning_to_stderr(&message, position);
                    flags.push('i');
                }
            }
            // `timeout:` says how long a match of this pattern may take,
            // which stands in place of what the class names.
            if let Some(Object::Dict(options)) = arguments.get(1).or(arguments.get(2))
                && let Some(named) = options.borrow().get(":timeout")
            {
                let held = match named {
                    Object::Nil => None,
                    other => {
                        let seconds = self.float_value_of(other, position)?;
                        if seconds <= 0.0 {
                            let shown =
                                self.send_to_object(other.clone(), "to_s", vec![], position)?;
                            let message = match &shown {
                                Object::String(held) => {
                                    format!("invalid timeout: {}", held.as_str())
                                }
                                held => format!("invalid timeout: {held}"),
                            };
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &message,
                                position,
                            ));
                        }
                        Some(std::time::Duration::from_secs_f64(seconds))
                    }
                };
                self.pattern_timeouts.insert(source.clone(), held);
            }
            let built = Rc::new(source);
            // A pattern built here is not frozen, which is what tells it
            // apart from one written as a literal.
            self.built_patterns.insert(Rc::as_ptr(&built) as usize);
            if let Some(named) = source_encoding {
                self.pattern_encodings
                    .insert(Rc::as_ptr(&built) as usize, named);
            }
            let made = Object::Regex(built, Rc::new(flags));
            if let Some(site) = built_once {
                self.patterns_built_once.insert(site, made.clone());
            }
            if class_rc.name() == "Regexp" {
                return Ok(Some(made));
            }
            // A subclass answers Regexp's methods through the pattern it
            // keeps, which is what lets it be a Regexp and its own class at
            // once.
            let mut instance = crate::object::Instance::new(Rc::clone(class_rc));
            instance.set_var(
                crate::vm::native_methods::REGEXP_SUBCLASS_VAR.to_string(),
                made,
            );
            let built = Object::Instance(Rc::new(std::cell::RefCell::new(instance)));
            // A subclass writing its own `initialize` sees the arguments the
            // pattern was built from.
            if let Some((owner, method)) = self.lookup_method(&built, "initialize")
                && !method.is_undefined
                && !method.body.is_empty()
            {
                if let Object::Instance(held) = &built {
                    held.borrow_mut()
                        .set_var("__building_regexp__".to_string(), Object::Bool(true));
                }
                let outcome =
                    self.invoke_method(owner, method, built.clone(), arguments.to_vec(), position);
                if let Object::Instance(held) = &built {
                    held.borrow_mut()
                        .instance_vars
                        .shift_remove("__building_regexp__");
                }
                outcome?;
            }
            return Ok(Some(built));
        }
        // `Regexp.union` matches any of what it was given.
        if class_rc.name() == "Regexp" && method_name == "union" {
            let parts: Vec<Object> = match arguments.first() {
                Some(Object::Array(array)) if arguments.len() == 1 => array.borrow().clone(),
                _ => arguments.to_vec(),
            };
            let mut sources = Vec::with_capacity(parts.len());
            for part in &parts {
                sources.push(match part {
                    Object::Regex(pattern, _) => pattern.as_str().to_string(),
                    other => crate::regexp::escape(&self.coerce_name_argument(other, position)?),
                });
            }
            return Ok(Some(Object::Regex(
                Rc::new(sources.join("|")),
                Rc::new(String::new()),
            )));
        }
        // Module.used_refinements: the refinements `using` has brought into
        // the current scope.
        if method_name == "used_refinements" && class_rc.name() == "Module" {
            let refinements = self.active_refinements();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                refinements,
            )))));
        }
        if method_name == "nesting" && class_rc.name() == "Module" {
            // Inside a method body the answer is the nesting captured where
            // the method was defined; elsewhere it is the scopes open here.
            let scopes = match self.method_nesting_stack.last() {
                Some(captured) => captured.clone(),
                None => self.snapshot_lexical_nesting(),
            };
            let nesting: Vec<Object> = scopes
                .into_iter()
                .map(|scope| {
                    if scope.is_module() {
                        Object::Module(scope)
                    } else {
                        Object::Class(scope)
                    }
                })
                .collect();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                nesting,
            )))));
        }
        // Minimal File class methods used by mspec's `fixture` helper.
        if class_rc.name() == "File" {
            match method_name {
                // Every part of a path but the last, as many times over as
                // the level says.
                "dirname" => {
                    if arguments.is_empty() || arguments.len() > 2 {
                        return Err(crate::vm::errors::argument_count_error(
                            crate::vm::errors::Arity::Range(1, 2),
                            arguments.len(),
                            position,
                        ));
                    }
                    let named = self.path_name_argument("dirname", &arguments[0], position)?;
                    let encoding = match &arguments[0] {
                        Object::String(held) => held.encoding_name(),
                        _ => "UTF-8".to_string(),
                    };
                    if let Object::String(held) = &arguments[0]
                        && !crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(
                            &held.encoding_name(),
                        )
                    {
                        let message = format!(
                            "path name must be ASCII-compatible ({}): {:?}",
                            held.encoding_name(),
                            held.as_str()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "Encoding::CompatibilityError",
                            &message,
                            position,
                        ));
                    }
                    let mut level = match arguments.get(1) {
                        None => 1_i64,
                        Some(held) => self.integer_argument("dirname", held, position)?,
                    };
                    if level < 0 {
                        let message = format!("negative level: {}", level);
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &message,
                            position,
                        ));
                    }
                    let mut held = named;
                    while level > 0 {
                        let stepped = parent_of_path(&held);
                        if stepped == held {
                            break;
                        }
                        held = stepped;
                        level -= 1;
                    }
                    return Ok(Some(Object::String(Rc::new(
                        crate::object::StringValue::with_encoding(held, encoding),
                    ))));
                }
                // The last part of a path, with a suffix taken off when one
                // is named. `".*"` means whatever extension the name carries.
                "basename" => {
                    if arguments.is_empty() || arguments.len() > 2 {
                        return Err(crate::vm::errors::argument_count_error(
                            crate::vm::errors::Arity::Range(1, 2),
                            arguments.len(),
                            position,
                        ));
                    }
                    // A name written in an encoding that spells the ASCII
                    // letters in more than one byte cannot be read as a path.
                    if let Object::String(held) = &arguments[0]
                        && !crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(
                            &held.encoding_name(),
                        )
                    {
                        let message = format!(
                            "path name must be ASCII-compatible ({}): {:?}",
                            held.encoding_name(),
                            held.as_str()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "Encoding::CompatibilityError",
                            &message,
                            position,
                        ));
                    }
                    // A suffix is named by text, and nothing else stands for
                    // one.
                    if let Some(held) = arguments.get(1)
                        && !matches!(held, Object::String(_))
                        && !self.answers_to(held, "to_str", position)?
                    {
                        return Err(method_argument_type_error(
                            "basename", "String", held, position,
                        ));
                    }
                    let path = self.path_name_argument("basename", &arguments[0], position)?;
                    let trimmed = path.trim_end_matches('/');
                    let held = if trimmed.is_empty() {
                        if path.is_empty() { "" } else { "/" }
                    } else {
                        trimmed.rsplit('/').next().unwrap_or(trimmed)
                    };
                    let mut name = held.to_string();
                    if let Some(Object::String(suffix)) = arguments.get(1) {
                        let suffix = suffix.to_text();
                        if suffix == ".*" {
                            if let Some(dot) = name.rfind('.')
                                && dot > 0
                            {
                                name.truncate(dot);
                            }
                        } else if name.len() > suffix.len() && name.ends_with(&*suffix) {
                            name.truncate(name.len() - suffix.len());
                        }
                    }
                    // The piece is written in the same encoding the whole
                    // name was.
                    let made = Object::string(name);
                    if let (Object::String(held), Object::String(piece)) = (&arguments[0], &made) {
                        piece.set_encoding(held.encoding_name());
                        if held.holds_bytes() {
                            piece.mark_bytes();
                        }
                    }
                    return Ok(Some(made));
                }
                // The extension a name carries, and the empty string for a
                // name that carries none.
                "extname" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let path = self.path_name_argument("extname", &arguments[0], position)?;
                    let held = path.trim_end_matches('/').rsplit('/').next().unwrap_or("");
                    // A dot at the front of a name is part of the name rather
                    // than the start of an extension, so a name of nothing but
                    // dots has no extension at all. A name that ends in a dot
                    // carries that dot as its extension.
                    let stripped = held.trim_start_matches('.');
                    let leading = held.len() - stripped.len();
                    let found = match stripped.rfind('.') {
                        Some(dot) if dot + 1 == stripped.len() => ".",
                        Some(dot) => &held[leading + dot..],
                        None => "",
                    };
                    return Ok(Some(Object::string(found.to_string())));
                }
                // The directory and the name a path is made of, as a pair.
                "split" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let path = self.path_name_argument("split", &arguments[0], position)?;
                    let arguments = &[Object::string(path)];
                    let directory = self
                        .call_class_methods(class_rc, "dirname", arguments, position)?
                        .unwrap_or(Object::Nil);
                    let name = self
                        .call_class_methods(class_rc, "basename", arguments, position)?
                        .unwrap_or(Object::Nil);
                    return Ok(Some(Object::array(vec![directory, name])));
                }
                "realpath" => {
                    if let Some(Object::String(s)) = arguments.first() {
                        let expanded = std::fs::canonicalize(&*s.as_str())
                            .ok()
                            .and_then(|p| p.to_str().map(String::from))
                            .unwrap_or_else(|| s.as_str().to_string());
                        return Ok(Some(Object::string(expanded)));
                    }
                }
                "respond_to?" => {
                    if let Some(name_arg) = arguments.first() {
                        let name_str = match name_arg {
                            Object::String(s) => s.as_str().to_string(),
                            Object::Symbol(s) => s.as_str().to_string(),
                            _ => return Ok(Some(Object::Bool(false))),
                        };
                        let known = matches!(
                            name_str.as_str(),
                            "dirname"
                                | "expand_path"
                                | "realpath"
                                | "absolute_path"
                                | "join"
                                | "respond_to?"
                        );
                        return Ok(Some(Object::Bool(known)));
                    }
                }
                _ => {}
            }
        }
        // Time is written in Ruby against a few calendar helpers the C
        // library carries out, so a local time follows the operating
        // system's zone rules.
        if class_rc.name() == "Time"
            && let Some(answered) =
                self.call_time_class_methods(method_name, arguments, position)?
        {
            return Ok(Some(answered));
        }
        // Queue.new / SizedQueue.new — synchronous FIFO stub. The instance
        // carries an Array under `__queue_items`; SizedQueue ignores its
        // capacity argument (we never block).
        // Mutex.new / ConditionVariable.new — single-threaded stubs (no shared
        // state needed; the synchronize/wait/broadcast methods are no-ops).
        if method_name == "new"
            && (class_rc.name() == "Mutex" || class_rc.name() == "ConditionVariable")
        {
            use crate::object::Instance;
            let instance = Instance::new(Rc::clone(class_rc));
            let inst_rc = Rc::new(std::cell::RefCell::new(instance));
            return Ok(Some(Object::Instance(inst_rc)));
        }
        if method_name == "new" && (class_rc.name() == "Queue" || class_rc.name() == "SizedQueue") {
            use crate::object::Instance;
            let instance = Instance::new(Rc::clone(class_rc));
            let inst_rc = Rc::new(std::cell::RefCell::new(instance));
            inst_rc.borrow_mut().set_var(
                "__queue_items".to_string(),
                Object::Array(Rc::new(std::cell::RefCell::new(Vec::new()))),
            );
            // `Queue.new(enumerable)` starts the queue off with what the
            // enumerable holds, in the order it holds them.
            if class_rc.name() == "Queue"
                && let Some(held) = arguments.first()
            {
                let seeded = self.queue_seed_argument(held, position)?;
                inst_rc
                    .borrow_mut()
                    .set_var("__queue_items".to_string(), Object::array(seeded));
            }
            // `SizedQueue.new(n)` says how many the queue holds, which it
            // reports whether or not anything ever waits on it.
            if class_rc.name() == "SizedQueue" {
                let counted = match arguments.first() {
                    Some(held) => self.queue_capacity_argument(held, position)?,
                    None => {
                        return Err(method_argument_error("new", 1, 0, position));
                    }
                };
                inst_rc
                    .borrow_mut()
                    .set_var("__queue_max".to_string(), Object::Int(counted));
            }
            return Ok(Some(Object::Instance(inst_rc)));
        }
        // Thread.new captures the block; we run it lazily on `value` so that
        // serialized "concurrent" specs (which set a shared flag between
        // construction and value-collection) still observe the flag change.
        // Newly-constructed threads land on `pending_threads` so an empty
        // `Queue#pop` (which would block in real Ruby) can drain them and
        // make forward progress.
        if matches!(method_name, "new" | "start" | "fork")
            && class_named_in_chain(class_rc, "Thread")
        {
            use crate::object::Instance;
            let block = self.pending_block.take().unwrap_or(Object::Nil);
            let instance = Instance::new(Rc::clone(class_rc));
            let inst_rc = Rc::new(std::cell::RefCell::new(instance));
            let obj = Object::Instance(Rc::clone(&inst_rc));
            // A subclass may write its own `initialize`, and what it hands to
            // `super` is what the thread runs. `start` and `fork` never go
            // through it, which is what tells them apart from `new`.
            let own_initialize = match self.lookup_method(&obj, "initialize") {
                Some((defined_in, method)) if method_name == "new" => match defined_in.name() {
                    "Thread" | "Object" | "BasicObject" => None,
                    _ => Some((defined_in, method)),
                },
                _ => None,
            };
            match own_initialize {
                Some((defined_in, method)) => {
                    self.pending_block = match block {
                        Object::Nil => None,
                        held => Some(held),
                    };
                    self.invoke_method(
                        defined_in,
                        method,
                        obj.clone(),
                        arguments.to_vec(),
                        position,
                    )?;
                }
                None => {
                    // A thread has to be given something to run. `new`
                    // reports that as a ThreadError, where `start` and `fork`
                    // report it the way any method missing its block does.
                    if matches!(block, Object::Nil) {
                        return Err(crate::vm::errors::simple_exception(
                            if method_name == "new" {
                                "ThreadError"
                            } else {
                                "ArgumentError"
                            },
                            "must be called with a block",
                            position,
                        ));
                    }
                    self.give_thread_a_body(&inst_rc, block, arguments);
                }
            }
            if inst_rc.borrow().get_var("__thread_block").is_none() {
                return Err(crate::vm::errors::simple_exception(
                    "ThreadError",
                    "uninitialized thread - check `Thread#initialize'",
                    position,
                ));
            }
            self.pending_threads.push(obj.clone());
            return Ok(Some(obj));
        }
        // `Fiber.new { ... }` makes a fiber the block runs on when it is
        // first resumed.
        if method_name == "new" && class_named_in_chain(class_rc, "Fiber") {
            use crate::object::Instance;
            let Some(Object::Block(block)) = self.pending_block.take() else {
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "tried to create a Fiber without a block",
                    position,
                ));
            };
            // `Fiber.new(blocking: true)` asks for a fiber that blocks when
            // it waits rather than handing control to a scheduler.
            let blocking = match arguments.last() {
                Some(Object::Dict(options)) => options
                    .borrow()
                    .get(":blocking")
                    .is_some_and(|held| held.is_truthy()),
                _ => false,
            };
            // `storage:` names what the fiber keeps for itself. Without it
            // the fiber inherits what the one making it held.
            let named = match arguments.last() {
                Some(Object::Dict(options)) => options.borrow().get(":storage").cloned(),
                _ => None,
            };
            let storage = match named {
                // A fiber inherits a copy of what the one making it keeps, so
                // writing a name in the new fiber leaves the old one alone.
                None => {
                    let holder = self.fiber_current_handle();
                    match self.fiber_storage_if_held(holder) {
                        Some(Object::Dict(held)) => {
                            let copied = held.borrow().clone();
                            Some(Object::Dict(Rc::new(std::cell::RefCell::new(copied))))
                        }
                        other => other,
                    }
                }
                Some(Object::Nil) => None,
                Some(held) => {
                    self.check_fiber_storage(&held, position)?;
                    Some(held)
                }
            };
            let handle = self.fiber_create(block, blocking, storage);
            let instance = Instance::new(Rc::clone(class_rc));
            let inst_rc = Rc::new(std::cell::RefCell::new(instance));
            inst_rc
                .borrow_mut()
                .set_var("__fiber__".to_string(), Object::Int(handle as i64));
            return Ok(Some(Object::Instance(inst_rc)));
        }
        if class_named_in_chain(class_rc, "Fiber") {
            match method_name {
                // `Fiber.yield` suspends the fiber holding the interpreter,
                // handing its arguments to whoever resumed it.
                "yield" => {
                    let handed = match arguments.len() {
                        0 => Object::Nil,
                        1 => arguments[0].clone(),
                        _ => Object::array(arguments.to_vec()),
                    };
                    let given = self.fiber_suspend(handed, position)?;
                    return Ok(Some(match given.len() {
                        0 => Object::Nil,
                        1 => given[0].clone(),
                        _ => Object::array(given),
                    }));
                }
                "current" => {
                    return Ok(Some(self.fiber_current()));
                }
                // `Fiber.blocking { |f| ... }` runs the block with the
                // running fiber blocking, and puts back what it was after.
                "blocking" if self.pending_block.is_some() => {
                    let Some(Object::Block(block)) = self.pending_block.take() else {
                        return Ok(Some(Object::Nil));
                    };
                    let held = self.fiber_current_handle();
                    let was = self.fiber_set_blocking(held, true);
                    let current = self.fiber_current();
                    let answered = self.execute_block_body(&block, vec![current]);
                    self.fiber_set_blocking(held, was);
                    return answered.map(Some);
                }
                // `Fiber[:name]` reads what the running fiber keeps under
                // that name, and `Fiber[:name] = held` writes it.
                "[]" if arguments.len() == 1 => {
                    let named = self.fiber_storage_name(&arguments[0], position)?;
                    let running = self.fiber_current_handle();
                    let Some(Object::Dict(held)) = self.fiber_storage_if_held(running) else {
                        return Ok(Some(Object::Nil));
                    };
                    let found = held.borrow().get(&named).cloned();
                    return Ok(Some(found.unwrap_or(Object::Nil)));
                }
                "[]=" if arguments.len() == 2 => {
                    let named = self.fiber_storage_name(&arguments[0], position)?;
                    let running = self.fiber_current_handle();
                    let Object::Dict(held) = self.fiber_storage(running) else {
                        return Ok(Some(arguments[1].clone()));
                    };
                    // A name given nil is dropped rather than kept as nil.
                    if matches!(arguments[1], Object::Nil) {
                        held.borrow_mut().shift_remove(&named);
                    } else {
                        held.borrow_mut().insert(named, arguments[1].clone());
                    }
                    return Ok(Some(arguments[1].clone()));
                }
                // Ruby answers the number of the blocking level here rather
                // than a flag, and false where nothing is blocking.
                "blocking?" => {
                    let held = self.fiber_current_handle();
                    return Ok(Some(if self.fiber_is_blocking(held) {
                        Object::Int(1)
                    } else {
                        Object::Bool(false)
                    }));
                }
                _ => {}
            }
        }
        // Thread.pass / Thread.current / Thread.report_on_exception= — minimal
        // stubs sufficient for fixture and spec helpers.
        if class_rc.name() == "Thread" {
            match method_name {
                // `Thread.pass` hands the turn over, which is what lets a
                // thread waiting on another make its own progress.
                "pass" => {
                    self.pass_to_other_threads(position)?;
                    return Ok(Some(Object::Nil));
                }
                // Hand control over, answering whether there was anything to
                // hand it to. A wait that nothing else can end stops here
                // rather than turning forever.
                "__hand_over__" => {
                    if !self.other_threads_are_waiting() {
                        return Ok(Some(Object::Bool(false)));
                    }
                    self.wait_for_other_threads(position);
                    // A thread stopped or handed an exception while it waited
                    // takes it here, which is where the wait ends.
                    self.deliver_thread_interrupts(true, position)?;
                    return Ok(Some(Object::Bool(true)));
                }
                // `Thread.kill` stops the thread it is handed, the way that
                // thread's own `kill` does.
                "kill" | "exit" => {
                    let target = match arguments.first() {
                        Some(named) => named.clone(),
                        None => self.running_thread(),
                    };
                    return self.call_thread_method(&target, "kill", &[], position);
                }
                // The threads that have not finished, which is what
                // `Thread.list` reports.
                "list" => {
                    // Asking which threads there are gives each of them a
                    // turn, so a loop watching the list is what lets them run.
                    if !self.running_a_thread_body() && !self.stepping_threads {
                        self.step_pending_threads(position);
                    }
                    let mut living = vec![self.running_thread()];
                    let main = self.globals().get("__Thread_main").unwrap_or(Object::Nil);
                    if !matches!(main, Object::Nil)
                        && !living.iter().any(|held| same_object(held, &main))
                    {
                        living.push(main);
                    }
                    for thread in self.pending_threads.clone() {
                        let over = matches!(&thread, Object::Instance(held)
                            if held.borrow().get_var("__thread_value").is_some());
                        if !over && !living.iter().any(|held| same_object(held, &thread)) {
                            living.push(thread);
                        }
                    }
                    return Ok(Some(Object::array(living)));
                }
                // `Thread.stop` puts the thread running now to sleep until
                // something wakes it.
                "stop" => {
                    self.sleep_until_woken(position)?;
                    return Ok(Some(Object::Nil));
                }
                // Thread.current returns the innermost Thread instance whose
                // block is being executed, or Nil at the top level. Used by
                // spec fixtures that thread-local-store via
                // `Thread.current[:k] = v` and read it back via the thread
                // instance after `.value`/`.join`. `Thread.main` is the same
                // shape but conceptually the program's root thread; we don't
                // distinguish, so it returns the same value.
                // Outside any thread block the running thread is the main
                // one, which is a Thread like any other.
                "current" | "main" => {
                    // `current` is the thread whose block is running, where
                    // `main` is always the one the program started on.
                    if method_name == "current"
                        && let Some(current) = self.thread_current_stack.last()
                    {
                        return Ok(Some(current.clone()));
                    }
                    if let Some(main) = self.globals().get("__Thread_main") {
                        return Ok(Some(main));
                    }
                    let instance = crate::object::Instance::new(Rc::clone(class_rc));
                    let main = Object::Instance(Rc::new(std::cell::RefCell::new(instance)));
                    self.globals_mut().set("__Thread_main", main.clone());
                    return Ok(Some(main));
                }
                "report_on_exception" | "report_on_exception=" => {
                    return Ok(Some(Object::Bool(true)));
                }
                "respond_to?" => {
                    if let Some(arg) = arguments.first() {
                        let n = match arg {
                            Object::String(s) => s.as_str().to_string(),
                            Object::Symbol(s) => s.as_str().to_string(),
                            _ => return Ok(Some(Object::Bool(false))),
                        };
                        let known = matches!(
                            n.as_str(),
                            "report_on_exception=" | "pass" | "current" | "new"
                        );
                        return Ok(Some(Object::Bool(known)));
                    }
                }
                _ => {}
            }
        }
        // Array.new — `new(size)`, `new(size, default)`, `new(size) { |i| ... }`.
        // Without arguments, returns an empty array.
        // `Set[1, 2, 3]` builds a set from the arguments directly.
        if method_name == "[]" && class_rc.name() == "Set" {
            let elements = Object::array(arguments.to_vec());
            return self
                .send_to_object(
                    Object::Class(Rc::clone(class_rc)),
                    "new",
                    vec![elements],
                    position,
                )
                .map(Some);
        }
        // `Hash[...]` builds a Hash from a single Hash, from an array of
        // pairs, or from an even number of key and value arguments.
        // `Hash.ruby2_keywords_hash` marks a copy of a hash as one that was
        // gathered from keyword arguments, and the predicate reports it.
        if matches!(method_name, "ruby2_keywords_hash" | "ruby2_keywords_hash?")
            && crate::vm::method_invocation::descends_from(class_rc, "Hash")
        {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let Some(Object::Dict(pairs)) = crate::vm::native_methods::as_dict(&arguments[0])
            else {
                let message = format!(
                    "wrong argument type {} (expected Hash)",
                    self.builtins().class_of(&arguments[0]).name()
                );
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ));
            };
            if method_name == "ruby2_keywords_hash?" {
                return Ok(Some(Object::Bool(pairs.borrow().contains_key(
                    crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY,
                ))));
            }
            let copy = self.call_object_method(&arguments[0], "dup", &[], position)?;
            let made = copy.unwrap_or_else(|| arguments[0].clone());
            if let Some(Object::Dict(copied)) = crate::vm::native_methods::as_dict(&made) {
                copied.borrow_mut().insert(
                    crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY.to_string(),
                    Object::Bool(true),
                );
            }
            return Ok(Some(made));
        }
        if method_name == "[]" && crate::vm::method_invocation::descends_from(class_rc, "Hash") {
            let mut entries: Vec<(Object, Object)> = Vec::new();
            match arguments {
                [] => {}
                [Object::Dict(source)] => {
                    for (key, value) in source.borrow().iter() {
                        if key.starts_with("__MX_") {
                            continue;
                        }
                        entries.push((Object::string(key.clone()), value.clone()));
                    }
                    let mut built = indexmap::IndexMap::new();
                    for (key, value) in entries {
                        let Object::String(rendered) = key else {
                            continue;
                        };
                        built.insert(rendered.as_str().to_string(), value);
                    }
                    return Ok(Some(hash_of_class(class_rc, built)));
                }
                [Object::Array(rows)] => {
                    let held = rows.borrow().clone();
                    for (at, row) in held.iter().enumerate() {
                        // Every element names a pair, so anything that is not
                        // one, and any pair with the wrong number of parts,
                        // is refused where it stands.
                        let Object::Array(pair) = row else {
                            let message = format!(
                                "wrong element type {} at {} (expected array)",
                                match row {
                                    Object::Nil => "nil".to_string(),
                                    other => self.builtins().class_of(other).ruby_name(),
                                },
                                at
                            );
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &message,
                                position,
                            ));
                        };
                        let pair = pair.borrow();
                        if pair.is_empty() || pair.len() > 2 {
                            let message =
                                format!("invalid number of elements ({} for 1..2)", pair.len());
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &message,
                                position,
                            ));
                        }
                        entries.push((
                            pair.first().cloned().unwrap_or(Object::Nil),
                            pair.get(1).cloned().unwrap_or(Object::Nil),
                        ));
                    }
                }
                // An instance of a Hash subclass is read by the entries it
                // holds, whatever `to_hash` it was given.
                [single] if crate::vm::native_methods::hash_subclass_value(single).is_some() => {
                    let held = crate::vm::native_methods::hash_subclass_value(single)
                        .expect("a hash subclass carries its entries");
                    return self.call_class_methods(class_rc, method_name, &[held], position);
                }
                // A single argument that reads as a hash, or as an array of
                // pairs, is read as one before anything else is tried.
                [single]
                    if !matches!(single, Object::Dict(_) | Object::Array(_))
                        && (self.responds_to(single, "to_hash")
                            || self.responds_to(single, "to_ary")) =>
                {
                    let named = if self.responds_to(single, "to_hash") {
                        "to_hash"
                    } else {
                        "to_ary"
                    };
                    let read = self.send_to_object(single.clone(), named, vec![], position)?;
                    return self.call_class_methods(class_rc, method_name, &[read], position);
                }
                values if values.len().is_multiple_of(2) => {
                    for pair in values.chunks(2) {
                        entries.push((pair[0].clone(), pair[1].clone()));
                    }
                }
                _ => {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "odd number of arguments for Hash",
                        position,
                    ));
                }
            }
            let mut built = indexmap::IndexMap::new();
            for (key, value) in entries {
                let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                built.insert(rendered, value);
            }
            return Ok(Some(hash_of_class(class_rc, built)));
        }
        // `Array[1, 2, 3]` and the same form on a subclass build a value from
        // the arguments directly, without running `initialize`.
        if method_name == "[]" && crate::vm::method_invocation::descends_from(class_rc, "Array") {
            let elements = arguments.to_vec();
            if class_rc.name() == "Array" {
                return Ok(Some(Object::array(elements)));
            }
            let mut instance = crate::object::Instance::new(Rc::clone(class_rc));
            instance.set_var(
                crate::vm::native_methods::ARRAY_SUBCLASS_VAR.to_string(),
                Object::array(elements),
            );
            return Ok(Some(Object::Instance(Rc::new(std::cell::RefCell::new(
                instance,
            )))));
        }
        if method_name == "new" && class_rc.name() == "Array" {
            let elements = self.build_array_elements(arguments, position)?;
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                elements,
            )))));
        }
        if method_name == "new" && class_rc.name() == "Set" {
            use crate::object::ObjectHash;
            let mut set: indexmap::IndexSet<ObjectHash> = indexmap::IndexSet::new();
            if arguments.len() == 1 {
                // An Array is taken as it is, and anything else that walks is
                // asked for one.
                let items = match &arguments[0] {
                    Object::Array(elements) => elements.borrow().clone(),
                    Object::Nil => Vec::new(),
                    // Ruby walks the seed with `each_entry`, falling back to
                    // `each`, and refuses anything that answers neither.
                    other => {
                        let walked = if self.responds_to(other, "each_entry") {
                            "each_entry"
                        } else if self.responds_to(other, "each") {
                            "each"
                        } else {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "value must be enumerable",
                                position,
                            ));
                        };
                        let walk = self.send_to_object(
                            other.clone(),
                            "to_enum",
                            vec![Object::symbol(walked.to_string())],
                            position,
                        )?;
                        match self.send_to_object(walk, "to_a", vec![], position)? {
                            Object::Array(elements) => elements.borrow().clone(),
                            _ => Vec::new(),
                        }
                    }
                };
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                for item in items {
                    // `Set.new(list) { |item| ... }` stores what the block
                    // answers rather than the item itself.
                    let item = match &block {
                        Some(block) => self.execute_block_callable(block, vec![item], position)?,
                        None => item,
                    };
                    if let Some(hash) = ObjectHash::from_object(&item) {
                        set.insert(hash);
                    } else {
                        return Err(MetorexError::runtime_error(
                            format!("Cannot add {} to set (not hashable)", item.type_name()),
                            position_to_location(position),
                        ));
                    }
                }
            } else if arguments.len() > 1 {
                return Err(MetorexError::runtime_error(
                    format!("Set.new expects 0-1 arguments, got {}", arguments.len()),
                    position_to_location(position),
                ));
            }
            return Ok(Some(Object::Set(Rc::new(std::cell::RefCell::new(set)))));
        }
        match method_name {
            // `Proc.new { ... }` yields the block itself — there is no
            // separate instance to build.
            "new" if Rc::ptr_eq(class_rc, &self.builtins().proc_class) => {
                if let Some(block) = self.pending_block.take() {
                    return Ok(Some(block));
                }
                let msg = "tried to create Proc object without a block";
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", msg.to_string()),
                    location: position_to_location(position),
                    message: msg.to_string(),
                });
            }
            // `Exception.exception` is another name for `new`.
            "new" | "exception" if method_name == "new" || self.is_exception_class(class_rc) => {
                return self
                    .invoke_callable(
                        Object::Class(Rc::clone(class_rc)),
                        arguments.to_vec(),
                        position,
                    )
                    .map(Some);
            }
            "name" => {
                // A singleton class has no name of its own, even though it
                // displays as `#<Class:Something>`.
                let name = class_rc.ruby_name();
                if name.is_empty() || class_rc.is_singleton_class() {
                    return Ok(Some(Object::Nil));
                }
                // A class answers one name object, not a fresh string each
                // time it is asked.
                let slot = format!("__class_name_{:p}_{}", Rc::as_ptr(class_rc), name);
                return Ok(Some(self.memoized_text(&slot, &name)));
            }
            // Module#instance_method / Class#instance_method: returns the
            // bound `Method` object so `parameters` and friends work on it.
            "instance_method" | "public_instance_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = self.coerce_method_name(&arguments[0], method_name, position)?;
                if let Some((owner, method)) = class_rc.find_method_with_owner(&name_str) {
                    // `public_instance_method` only hands out public methods.
                    if method.is_undefined
                        || (method_name == "public_instance_method"
                            && (owner.is_method_restricted(&name_str)
                                || self.method_is_restricted(
                                    &Object::Class(Rc::clone(class_rc)),
                                    &name_str,
                                )))
                    {
                        return Err(undefined_instance_method_error(
                            &name_str, class_rc, position,
                        ));
                    }
                    let mut unbound = (*method).clone();
                    // A `def self.name` method records the class it was
                    // written in, while the singleton class is what owns it.
                    if unbound.owner_class.is_none() || owner.is_singleton_class() {
                        unbound.owner = Some(owner.name().to_string());
                        unbound.owner_class = Some(owner);
                    }
                    unbound.origin_class = Some(Rc::clone(class_rc));
                    return Ok(Some(Object::Method(Rc::new(unbound))));
                }
                // Synthesize a stub for well-known Module-private mixin
                // hooks so `Module.instance_method(:append_features)` works
                // for spec patterns that bind/call them.
                if class_rc.name() == "Module" && MODULE_PRIVATE_HOOKS.contains(&name_str.as_str())
                {
                    let stub = Method::with_owner(
                        name_str.clone(),
                        vec!["target".to_string()],
                        vec![],
                        "Module".to_string(),
                    );
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                // The rest of Module's own methods are implemented natively,
                // so hand out a stub carrying the right parameter list.
                if matches!(class_rc.name(), "Module" | "Class")
                    && let Some(stub) = native_module_method_stub(&name_str)
                {
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                // `Class#new` is answered natively too, and belongs to Class
                // rather than to Module.
                if class_rc.name() == "Class" && name_str == "new" {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        "Class".to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    stub.native_alias = Some(name_str.clone());
                    stub.original_name = Some(name_str.clone());
                    stub.owner_class = Some(Rc::clone(class_rc));
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                // Kernel methods are implemented natively rather than living
                // in Object's method table. A body-less stub reaches the same
                // native implementation when invoked, so `Object` can hand out
                // an UnboundMethod for them.
                if matches!(class_rc.name(), "Object" | "Kernel")
                    && is_native_kernel_method(&name_str)
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        "Kernel".to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                // BasicObject answers its own handful natively rather than
                // holding them in a method table, so a body-less stub stands
                // for each of them.
                if class_rc.name() == "BasicObject"
                    && (NATIVE_BASIC_OBJECT_METHODS.contains(&name_str.as_str())
                        || BASIC_OBJECT_PRIVATE_METHODS.contains(&name_str.as_str()))
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        "BasicObject".to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    stub.native_alias = Some(name_str.clone());
                    stub.original_name = Some(name_str.clone());
                    stub.owner_class = Some(Rc::clone(class_rc));
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                // A builtin class answers many of its methods natively. A
                // body-less stub carrying the name reaches the same one.
                if let Some(probe) = sample_of_class(class_rc.name())
                    && self.responds_to(&probe, &name_str)
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        class_rc.name().to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    // Two names for one native method stand for that one
                    // method, which is what makes them equal and alike.
                    let named = super::object_methods::native_alias_target(
                        class_rc.name(),
                        name_str.as_str(),
                    )
                    .unwrap_or(name_str.as_str())
                    .to_string();
                    stub.native_alias = Some(named.clone());
                    stub.original_name = Some(named);
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                // An exception class takes its message, and whatever else a
                // subclass reads, through a native `initialize`. A body-less
                // stub carries the variadic arity Ruby reports for it.
                if name_str == "initialize"
                    && crate::vm::method_invocation::descends_from(class_rc, "Exception")
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        class_rc.name().to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                return Err(undefined_instance_method_error(
                    &name_str, class_rc, position,
                ));
            }
            // Module#undefined_instance_methods: the names this class itself
            // has undefined with `undef_method`, not those its ancestors did.
            "undefined_instance_methods" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let undefined: Vec<Object> = class_rc
                    .method_names()
                    .into_iter()
                    .filter(|name| {
                        class_rc
                            .find_own_method(name)
                            .is_some_and(|method| method.is_undefined)
                    })
                    .map(Object::symbol)
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    undefined,
                )))));
            }
            // Module#instance_methods / public_/private_/protected_ variants.
            // The `false` argument restricts to methods defined directly on
            // this class (excluding inherited and mixin methods).
            "instance_methods"
            | "public_instance_methods"
            | "private_instance_methods"
            | "protected_instance_methods" => {
                let include_super = match arguments.first() {
                    Some(Object::Bool(b)) => *b,
                    _ => true,
                };
                let mut method_list: Vec<String> = class_rc.method_names();
                // A `private`/`public` naming an inherited method marks the
                // visibility here without defining anything, and Ruby counts
                // that name among this class's own methods.
                let object_class = match self.globals().get("Object") {
                    Some(Object::Class(object_class)) => Some(object_class),
                    _ => None,
                };
                for name in class_rc.visibility_marked_names() {
                    let resolves = class_rc.find_method(&name).is_some()
                        || object_class
                            .as_ref()
                            .is_some_and(|oc| oc.find_method(&name).is_some());
                    if !method_list.contains(&name) && resolves {
                        method_list.push(name);
                    }
                }
                if include_super {
                    // The ancestor walk already covers mixins of mixins and
                    // each superclass's mixins.
                    let mut chain: Vec<Object> = Vec::new();
                    let mut seen: Vec<*const Class> = Vec::new();
                    push_class_ancestors(class_rc, &mut chain, &mut seen);
                    for ancestor in &chain {
                        let (Object::Class(c) | Object::Module(c)) = ancestor else {
                            continue;
                        };
                        for n in c.method_names() {
                            if !method_list.contains(&n) {
                                method_list.push(n);
                            }
                        }
                    }
                }
                // For the `Module` / `Class` receiver, advertise the native
                // mutation methods we actually implement so mspec matchers
                // (e.g. `have_public_instance_method(:alias_method, false)`)
                // recognize them as public instance methods.
                if matches!(class_rc.name(), "Module" | "Class") {
                    for (n, _, _) in NATIVE_MODULE_METHODS {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                    }
                }
                // A method removed with `undef_method` stays in the table as
                // a tombstone so lookups stop at it; it is not an instance
                // method any more.
                method_list.retain(|n| {
                    class_rc
                        .find_method(n)
                        .is_none_or(|method| !method.is_undefined)
                });
                // A name's visibility comes from the nearest ancestor that
                // defines it: an ancestor further along may mark its own copy
                // private without that reaching the one in front.
                let mut priv_set: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                let mut protected_set: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                // Module-private mixin hooks: append_features and friends
                // are private instance methods on Module. Class is *also* a
                // Module subclass — but `append_features` is undefined on
                // Class (Ruby sets it to undef), so only surface them when
                // the receiver is Module itself.
                if class_rc.name() == "Module" {
                    for n in MODULE_PRIVATE_HOOKS
                        .iter()
                        .chain(MODULE_PRIVATE_DECLARATIONS.iter())
                    {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                        priv_set.insert((*n).to_string());
                    }
                }
                // BasicObject's own methods are native too, and its table is
                // empty, so they are listed here the way Kernel's are.
                if class_rc.name() == "BasicObject" {
                    for n in NATIVE_BASIC_OBJECT_METHODS.iter() {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                    }
                    for n in BASIC_OBJECT_PRIVATE_METHODS.iter() {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                        priv_set.insert((*n).to_string());
                    }
                }
                // Kernel's methods are implemented natively rather than in
                // its table, so they are listed here. The public ones come
                // first; the pass below marks the private ones.
                if class_rc.name() == "Kernel" {
                    for (n, _, _) in NATIVE_KERNEL_METHODS.iter() {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                    }
                    for n in
                        crate::vm::native_methods::kernel_conversion::KERNEL_CONVERSION_FUNCTIONS
                            .iter()
                            .chain(KERNEL_PRIVATE_FUNCTIONS.iter())
                    {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                        priv_set.insert((*n).to_string());
                    }
                }
                let visibility_chain: Vec<Rc<Class>> = if include_super {
                    let mut chain: Vec<Object> = Vec::new();
                    let mut seen: Vec<*const Class> = Vec::new();
                    push_class_ancestors(class_rc, &mut chain, &mut seen);
                    chain
                        .iter()
                        .filter_map(|ancestor| match ancestor {
                            Object::Class(c) | Object::Module(c) => Some(Rc::clone(c)),
                            _ => None,
                        })
                        .collect()
                } else {
                    vec![Rc::clone(class_rc)]
                };
                for name in &method_list {
                    for ancestor in &visibility_chain {
                        if ancestor.has_public_override(name) {
                            break;
                        }
                        if ancestor.is_method_private(name) {
                            priv_set.insert(name.clone());
                            break;
                        }
                        if ancestor.is_method_protected(name) {
                            protected_set.insert(name.clone());
                            break;
                        }
                        if ancestor.find_own_method(name).is_some() {
                            break;
                        }
                    }
                }
                let filtered: Vec<Object> = method_list
                    .into_iter()
                    .filter(|n| !is_internal_method_key(n))
                    .filter(|n| match method_name {
                        "private_instance_methods" => priv_set.contains(n),
                        "protected_instance_methods" => protected_set.contains(n),
                        "public_instance_methods" => {
                            !priv_set.contains(n) && !protected_set.contains(n)
                        }
                        // Ruby's `instance_methods` covers public and
                        // protected alike.
                        "instance_methods" => !priv_set.contains(n),
                        _ => true,
                    })
                    .map(Object::symbol)
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    filtered,
                )))));
            }
            // attr_reader/attr_writer/attr_accessor as runtime instance
            // methods on Module/Class. Define the accessors on the receiver,
            // apply the receiver's current visibility, and return the array
            // of newly-defined method names as symbols (Ruby 3.0+).
            "attr_reader" | "attr_writer" | "attr_accessor" | "attr" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                // `attr name, true|false` is the deprecated 2-arg boolean form:
                // the second arg controls writer creation, and only the first
                // arg is a name. Ruby warns under `$VERBOSE = true`.
                let want_reader = matches!(method_name, "attr_reader" | "attr_accessor" | "attr");
                let mut want_writer = matches!(method_name, "attr_writer" | "attr_accessor");
                let names_slice: &[Object] = if method_name == "attr"
                    && arguments.len() == 2
                    && matches!(&arguments[1], Object::Bool(_))
                {
                    if matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
                        self.emit_warning_to_stderr(
                            "warning: optional boolean argument is obsoleted",
                            position,
                        );
                    }
                    want_writer = matches!(&arguments[1], Object::Bool(true));
                    &arguments[..1]
                } else {
                    arguments
                };
                let mut names: Vec<String> = Vec::with_capacity(names_slice.len());
                for arg in names_slice {
                    let n = self.coerce_method_name(arg, method_name, position)?;
                    names.push(n);
                }
                let visibility = class_rc.current_visibility();
                let mut defined: Vec<Object> = Vec::new();
                let mut newly_defined_names: Vec<String> = Vec::new();
                for attr_name in &names {
                    if want_reader {
                        let getter_body = vec![crate::ast::Statement::Return {
                            value: Some(crate::ast::Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            }),
                            position,
                        }];
                        let getter =
                            crate::object::Method::new(attr_name.clone(), vec![], getter_body);
                        class_rc.define_method(attr_name, Rc::new(getter));
                        if visibility != "public" {
                            class_rc.set_method_private(attr_name.clone());
                        }
                        class_rc.declare_instance_var(attr_name);
                        defined.push(Object::symbol(attr_name.clone()));
                        newly_defined_names.push(attr_name.clone());
                    }
                    if want_writer {
                        let setter_body = vec![crate::ast::Statement::Assignment {
                            target: crate::ast::Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: crate::ast::Expression::Identifier {
                                name: crate::object::UNNAMED_PARAMETER.to_string(),
                                position,
                            },
                            position,
                        }];
                        let setter_name = format!("{}=", attr_name);
                        let setter = crate::object::Method::new(
                            setter_name.clone(),
                            vec![crate::object::UNNAMED_PARAMETER.to_string()],
                            setter_body,
                        );
                        class_rc.define_method(&setter_name, Rc::new(setter));
                        if visibility != "public" {
                            class_rc.set_method_private(setter_name.clone());
                        }
                        class_rc.declare_instance_var(attr_name);
                        defined.push(Object::symbol(setter_name.clone()));
                        newly_defined_names.push(setter_name);
                    }
                }
                // Fire `method_added` (or `singleton_method_added` when the
                // receiver is a singleton class) for each method we just
                // installed, so user-defined hooks observe attr_* the same
                // way they observe `def`.
                let is_singleton = class_rc.get_class_var("__singleton__").is_some();
                let hook_name = if is_singleton {
                    "singleton_method_added"
                } else {
                    "method_added"
                };
                for added in &newly_defined_names {
                    self.invoke_class_hook(class_rc, hook_name, added, position)?;
                }
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    defined,
                )))));
            }
            // Module#method_defined?(name) — true when `name` resolves to a
            // public or protected instance method on the receiver, including
            // inherited methods. The optional second arg (default true) limits
            // the search to the receiver itself when false.
            "method_defined?"
            | "public_method_defined?"
            | "private_method_defined?"
            | "protected_method_defined?" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name = self.coerce_method_name(&arguments[0], method_name, position)?;
                let include_super = match arguments.get(1) {
                    Some(Object::Bool(b)) => *b,
                    _ => true,
                };
                let found = if include_super {
                    class_rc.find_method_with_owner(&name)
                } else {
                    class_rc
                        .find_own_method(&name)
                        .map(|method| (Rc::clone(class_rc), method))
                };
                // Kernel's own methods live in the native dispatch tables
                // rather than in its method map, so the private ones are
                // listed rather than looked up.
                if class_rc.name() == "Kernel"
                    && found.is_none()
                    && KERNEL_PRIVATE_FUNCTIONS.contains(&name.as_str())
                {
                    return Ok(Some(Object::Bool(method_name == "private_method_defined?")));
                }
                // The public ones live there too, so Kernel reports them the
                // same way rather than answering that it has none.
                if matches!(class_rc.name(), "Kernel" | "Object")
                    && found.is_none()
                    && crate::vm::native_methods::is_native_kernel_method(&name)
                {
                    return Ok(Some(Object::Bool(matches!(
                        method_name,
                        "method_defined?" | "public_method_defined?"
                    ))));
                }
                let answer = match found {
                    // A tombstone left by `undef_method` is not a definition.
                    Some((_, method)) if method.is_undefined => false,
                    None => false,
                    Some((owner, _)) => {
                        let is_private = owner.is_method_private(&name);
                        let is_protected = owner.is_method_protected(&name);
                        match method_name {
                            "method_defined?" => !is_private,
                            "public_method_defined?" => !is_private && !is_protected,
                            "private_method_defined?" => is_private,
                            "protected_method_defined?" => is_protected,
                            _ => unreachable!(),
                        }
                    }
                };
                return Ok(Some(Object::Bool(answer)));
            }
            // Module#extend: mix the given module's instance methods into the
            // receiver's singleton class, so `klass.some_module_method` works.
            "extend" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "extend",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Ruby takes a module here and rejects a class.
                let module_rc = match &arguments[0] {
                    Object::Module(m) => Rc::clone(m),
                    other => {
                        return Err(method_argument_type_error(
                            "extend", "Module", other, position,
                        ));
                    }
                };
                let target = Object::Class(Rc::clone(class_rc));
                self.apply_module_extend(&target, &module_rc, position)?;
                return Ok(Some(target));
            }
            // `private_class_method :name` / `public_class_method :name` —
            // flip the class-method visibility on the receiver's singleton
            // class. Visibility is otherwise only honoured for private calls;
            // the inherited hook (line inherited_spec.rb:43) ensures a
            // marked-private method still fires via `super`/hook invocation.
            "private_class_method" | "public_class_method" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                // A lone array argument names several methods at once.
                let named: Vec<Object> = match arguments {
                    [Object::Array(names)] => names.borrow().clone(),
                    other => other.to_vec(),
                };
                let target_class = Object::Class(Rc::clone(class_rc));
                let singleton = self.singleton_class_of(&target_class);
                for argument in &named {
                    let name = self.coerce_method_name(argument, method_name, position)?;
                    // Mirror the method onto the singleton class so
                    // `lookup_method` finds it there (where visibility lives).
                    // The `inherited` hook is inherited from Class's singleton
                    // table via the `__class__` convention, so copy it across
                    // and we have something to toggle visibility on.
                    if singleton.find_method(&name).is_none() {
                        match self.class_method_of(class_rc, &name) {
                            Some(method) => singleton.define_method(&name, method),
                            // `new` and `allocate` are answered by the runtime
                            // rather than a method table, so there is nothing
                            // to mirror. The marking on the singleton is what
                            // dispatch consults.
                            None if matches!(name.as_str(), "new" | "allocate") => {}
                            None => {
                                let msg = format!(
                                    "undefined method '{}' for {} '{}'",
                                    name,
                                    class_rc.kind_name().to_lowercase(),
                                    class_rc.ruby_name()
                                );
                                let exc = Object::exception("NameError", msg.clone());
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: msg,
                                });
                            }
                        }
                    }
                    if method_name == "private_class_method" {
                        singleton.set_method_private(&name);
                    } else {
                        singleton.set_method_public(&name);
                    }
                }
                return Ok(Some(Object::Class(Rc::clone(class_rc))));
            }
            // Module#set_temporary_name: a display name for an anonymous
            // module, cleared by passing nil. A permanent name cannot be
            // replaced, and the name may not look like a constant path.
            "set_temporary_name" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if class_rc.has_permanent_name() {
                    let msg = "can't change permanent name".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("RuntimeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let receiver = if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                };
                if matches!(arguments[0], Object::Nil) {
                    class_rc.set_temporary_name(None);
                    return Ok(Some(receiver));
                }
                let name = self.coerce_name_argument(&arguments[0], position)?;
                let complaint = if name.is_empty() {
                    Some("empty class/module name")
                } else if looks_like_constant_path(&name) {
                    Some("the temporary name must not be a constant path to avoid confusion")
                } else {
                    None
                };
                if let Some(msg) = complaint {
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", msg.to_string()),
                        location: position_to_location(position),
                        message: msg.to_string(),
                    });
                }
                class_rc.set_temporary_name(Some(name));
                return Ok(Some(receiver));
            }
            // Module#remove_const: remove a constant from this module's table.
            "remove_const" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "remove_const",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_name = self.coerce_name_argument(&arguments[0], position)?;
                if !is_valid_constant_name(&const_name) {
                    let msg = format!("wrong constant name {}", const_name);
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Only a constant of the receiver itself can be removed, and
                // a pending autoload counts as one.
                if class_rc.get_class_var(&const_name).is_none()
                    && class_rc.get_autoload(&const_name).is_none()
                    && !class_rc.unrealized_autoload_names().contains(&const_name)
                    && !(class_rc.name() == "Object" && self.globals().contains(&const_name))
                {
                    let msg = format!(
                        "constant {}::{} not defined",
                        class_rc.ruby_name(),
                        const_name
                    );
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Drop the resolved constant (if any), any pending autoload
                // registration, and any "loaded but unrealized" bookkeeping.
                // Without removing all three, the name would still surface
                // in `#constants` because the constants list aggregates
                // class_vars + autoloads + unrealized autoloads.
                self.warn_deprecated_constant(class_rc, &const_name, position);
                let mut removed = class_rc.remove_class_var(&const_name);
                // Object's constants are top-level constants — drop the
                // globals binding too so bare references stop resolving.
                if class_rc.name() == "Object" {
                    let from_globals = self.globals_mut().remove(&const_name);
                    if removed.is_none() {
                        removed = from_globals;
                    }
                }
                let removed_autoload = class_rc.remove_autoload(&const_name);
                class_rc.clear_unrealized_autoload(&const_name);
                // Drop the recorded source location so a subsequent
                // `autoload` for the same name surfaces *its* location via
                // `const_source_location` instead of the stale class-def
                // location from before the removal.
                class_rc.remove_const_location(&const_name);
                // Removing a constant that was only registered for autoload
                // answers with nil: it never held a value.
                let _ = removed_autoload;
                return Ok(Some(removed.unwrap_or(Object::Nil)));
            }
            "private" | "public" | "protected" => {
                return self
                    .apply_class_visibility_modifier(class_rc, method_name, arguments, position)
                    .map(Some);
            }
            "private_methods" => {
                let include_super = !matches!(
                    arguments.first(),
                    Some(Object::Bool(false)) | Some(Object::Nil)
                );
                // What is private *on the class object* lives in its
                // singleton chain. Its own private instance methods are its
                // instances' business, so they only surface once ancestors do.
                let mut names: Vec<String> = self
                    .private_method_names_for(&Object::Class(Rc::clone(class_rc)), include_super);
                if include_super {
                    names.extend(class_rc.private_method_names());
                }
                // Classes and modules inherit Module's private instance
                // methods. `extend_object` and the `*_features` pair are
                // undefined on Class, so the module dispatch path adds those.
                names.extend(
                    [
                        "extended",
                        "included",
                        "prepended",
                        "const_added",
                        "method_added",
                    ]
                    .map(String::from),
                );
                names.extend(MODULE_PRIVATE_DECLARATIONS.iter().map(|n| (*n).to_string()));
                if include_super {
                    let mut current = class_rc.superclass();
                    while let Some(parent) = current {
                        names.extend(parent.private_method_names());
                        current = parent.superclass();
                    }
                }
                names.sort();
                names.dedup();
                let syms: Vec<Object> = names.into_iter().map(Object::symbol).collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(syms)))));
            }
            // Module#singleton_class?: whether this is the class of exactly
            // one object, as `class << obj` opens.
            "singleton_class?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                return Ok(Some(Object::Bool(class_rc.is_singleton_class())));
            }
            "superclass" => {
                return match class_rc.superclass() {
                    Some(parent) => Ok(Some(Object::Class(parent))),
                    None => Ok(Some(Object::Nil)),
                };
            }
            "ancestors" => {
                let mut chain: Vec<Object> = Vec::new();
                let mut seen: Vec<*const Class> = Vec::new();
                push_class_ancestors(class_rc, &mut chain, &mut seen);
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(chain)))));
            }
            // The ancestors that are modules rather than classes, with the
            // receiver itself left out.
            "included_modules" => {
                let mut chain: Vec<Object> = Vec::new();
                let mut seen: Vec<*const Class> = Vec::new();
                push_class_ancestors(class_rc, &mut chain, &mut seen);
                let modules: Vec<Object> = chain
                    .into_iter()
                    .filter(|ancestor| match ancestor {
                        Object::Module(m) => !Rc::ptr_eq(m, class_rc),
                        _ => false,
                    })
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    modules,
                )))));
            }
            "const_defined?" => {
                // const_defined?(name [, inherit=true]) — when inherit is
                // false, only check the receiver itself; otherwise also
                // search mixins and the superclass chain. The autoload
                // registry counts as defined (Ruby treats a registered
                // autoload as a constant entry). Scoped names
                // (`A::B`, `::Top`) resolve segment by segment; invalid
                // segments raise NameError. Never calls const_missing.
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        "const_defined?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_path =
                    self.coerce_method_name(&arguments[0], "const_defined?", position)?;
                let inherit = match arguments.get(1) {
                    None => true,
                    Some(v) => crate::vm::utils::is_truthy(v),
                };
                let mut rest: &str = &const_path;
                let mut current = Rc::clone(class_rc);
                if let Some(stripped) = rest.strip_prefix("::") {
                    rest = stripped;
                    current = match self.globals().get("Object") {
                        Some(Object::Class(c)) => c,
                        _ => return Ok(Some(Object::Bool(false))),
                    };
                }
                let segments: Vec<&str> = rest.split("::").collect();
                for seg in &segments {
                    if !is_valid_constant_name(seg) {
                        let msg = format!("wrong constant name {}", const_path);
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
                for (i, seg) in segments.iter().enumerate() {
                    let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                    if i + 1 == segments.len() {
                        return Ok(Some(Object::Bool(entry.is_some())));
                    }
                    // Intermediate segments must resolve to a class/module
                    // value; a registered-but-unloaded autoload can't be
                    // traversed without triggering the load.
                    match entry {
                        Some((_, Some(Object::Class(c)))) | Some((_, Some(Object::Module(c)))) => {
                            current = c;
                        }
                        _ => return Ok(Some(Object::Bool(false))),
                    }
                }
                return Ok(Some(Object::Bool(false)));
            }
            "const_source_location" => {
                // const_source_location(name [, inherit=true]) — same search
                // as const_get, but returns the recorded [file, line] of the
                // constant's definition, [] for constants without a Ruby
                // source (builtins), and nil when not found (never calls
                // const_missing).
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        "const_source_location",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let was_symbol = matches!(&arguments[0], Object::Symbol(_));
                let const_path =
                    self.coerce_method_name(&arguments[0], "const_source_location", position)?;
                let inherit = match arguments.get(1) {
                    None => true,
                    Some(v) => crate::vm::utils::is_truthy(v),
                };
                let wrong_name = |path: &str| {
                    let msg = format!("wrong constant name {}", path);
                    let exc = Object::exception("NameError", msg.clone());
                    MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    }
                };
                // A Symbol must be a simple name — scope separators raise.
                if was_symbol && const_path.contains("::") {
                    return Err(wrong_name(&const_path));
                }
                let mut rest: &str = &const_path;
                let mut current = Rc::clone(class_rc);
                if let Some(stripped) = rest.strip_prefix("::") {
                    rest = stripped;
                    current = match self.globals().get("Object") {
                        Some(Object::Class(c)) => c,
                        _ => return Err(wrong_name(&const_path)),
                    };
                }
                let segments: Vec<&str> = rest.split("::").collect();
                for seg in &segments {
                    if !is_valid_constant_name(seg) {
                        return Err(wrong_name(&const_path));
                    }
                }
                let loc_array = |loc: Option<(String, i64)>| {
                    let items = match loc {
                        // A constant the interpreter defines stands in no
                        // file of the program's, which Ruby reports as no
                        // location at all. One written in the core library's
                        // own Ruby source is the same to a reader.
                        Some((file, _))
                            if file.is_empty()
                                || file.starts_with(crate::vm::INTERNAL_FILE_PREFIX) =>
                        {
                            Vec::new()
                        }
                        Some((file, line)) => {
                            vec![Object::string(file), Object::Int(line)]
                        }
                        None => Vec::new(),
                    };
                    Object::Array(Rc::new(std::cell::RefCell::new(items)))
                };
                for (i, seg) in segments.iter().enumerate() {
                    if i + 1 == segments.len() {
                        // Thread-aware: if this autoload is currently loading
                        // on a different thread, report the autoload
                        // registration's location — the constant isn't
                        // really defined from that thread's view yet.
                        let thread = self
                            .thread_current_stack
                            .last()
                            .cloned()
                            .unwrap_or(Object::Nil);
                        let other_thread_loading =
                            self.autoload_loading.iter().any(|(cls, n, loader)| {
                                if !Rc::ptr_eq(cls, &current) || n != *seg {
                                    return false;
                                }
                                let same = match (loader, &thread) {
                                    (Object::Nil, Object::Nil) => true,
                                    (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
                                    _ => false,
                                };
                                !same
                            });
                        if other_thread_loading {
                            return Ok(Some(loc_array(current.get_autoload_location(seg))));
                        }
                        let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                        return Ok(Some(match entry {
                            Some((owner, Some(_))) => loc_array(
                                owner
                                    .get_const_location(seg)
                                    .or_else(|| owner.get_autoload_location(seg)),
                            ),
                            Some((owner, None)) => loc_array(owner.get_autoload_location(seg)),
                            // A still-registered autoload that the lookup
                            // treats as cleared (e.g. this thread is the one
                            // loading it) keeps reporting its registration
                            // location until the constant is defined.
                            None if current.get_autoload(seg).is_some() => {
                                loc_array(current.get_autoload_location(seg))
                            }
                            None => Object::Nil,
                        }));
                    }
                    // Intermediate segments resolve like const_get, firing
                    // registered autoloads along the way.
                    let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                    let resolved = match entry {
                        Some((_, Some(v))) => Some(v),
                        Some((owner, None)) => self.try_autoload_constant(&owner, seg)?,
                        None => self.try_autoload_constant(&current, seg)?,
                    };
                    match resolved {
                        Some(Object::Class(c)) | Some(Object::Module(c)) => current = c,
                        _ => return Ok(Some(Object::Nil)),
                    }
                }
                return Ok(Some(Object::Nil));
            }
            "const_get" => {
                // const_get(name [, inherit=true]) — search order mirrors
                // const_defined?: the receiver, then (when inherit) mixins
                // and the superclass chain, with Object exposing top-level
                // constants and modules falling back to Object for a
                // directly-named constant. Scoped names (`A::B`, `::Top`)
                // resolve segment by segment; a Symbol must be a simple
                // name. Registered autoloads fire; unresolvable names
                // dispatch const_missing (default: NameError with `name`).
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        "const_get",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let was_symbol = matches!(&arguments[0], Object::Symbol(_));
                let const_path = self.coerce_method_name(&arguments[0], "const_get", position)?;
                let inherit = match arguments.get(1) {
                    None => true,
                    Some(v) => crate::vm::utils::is_truthy(v),
                };
                let wrong_name = |path: &str| {
                    let msg = format!("wrong constant name {}", path);
                    let exc = Object::exception("NameError", msg.clone());
                    MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    }
                };
                if was_symbol && const_path.contains("::") {
                    return Err(wrong_name(&const_path));
                }
                let mut rest: &str = &const_path;
                let mut current = Rc::clone(class_rc);
                if let Some(stripped) = rest.strip_prefix("::") {
                    rest = stripped;
                    current = match self.globals().get("Object") {
                        Some(Object::Class(c)) => c,
                        _ => return Err(wrong_name(&const_path)),
                    };
                }
                let segments: Vec<&str> = rest.split("::").collect();
                for seg in &segments {
                    if !is_valid_constant_name(seg) {
                        return Err(wrong_name(&const_path));
                    }
                }
                let mut value = Object::Nil;
                for (i, seg) in segments.iter().enumerate() {
                    let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                    let resolved = match entry {
                        Some((_, Some(v))) => Some(v),
                        // Registered autoload — fire the load on the owner.
                        Some((owner, None)) => self.try_autoload_constant(&owner, seg)?,
                        // No entry — a registered autoload whose file was
                        // already loaded without defining the constant can
                        // still be satisfied by a re-load (several autoloads
                        // may point at one path); `try_autoload_constant`
                        // owns that logic.
                        None => self.try_autoload_constant(&current, seg)?,
                    };
                    let resolved = match resolved {
                        Some(v) => v,
                        None => {
                            let missing = self.dispatch_const_missing(&current, seg, position)?;
                            if i + 1 == segments.len() {
                                return Ok(Some(missing));
                            }
                            missing
                        }
                    };
                    if i + 1 == segments.len() {
                        self.warn_deprecated_constant(&current, seg, position);
                        value = resolved;
                    } else {
                        match resolved {
                            Object::Class(c) | Object::Module(c) => current = c,
                            other => {
                                let msg =
                                    format!("{} does not refer to class/module", other.type_name());
                                let exc = Object::exception("TypeError", msg.clone());
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: msg,
                                });
                            }
                        }
                    }
                }
                return Ok(Some(value));
            }
            // Default `Module#const_added` — a no-op returning nil. User
            // hooks (`def self.const_added`) are dispatched before native
            // fallback, so this only fires for the base implementation.
            "const_added" => {
                // Native dispatch runs before the user method body in
                // `invoke_method`, so step aside when a user hook exists.
                if class_rc.find_method("__class__const_added").is_some() {
                    return Ok(None);
                }
                let mut cursor = Some(Rc::clone(class_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && sc.find_method("const_added").is_some()
                    {
                        return Ok(None);
                    }
                    cursor = current.superclass();
                }
                // A reopened `Module` or `Class` gives every class the hook as
                // an instance method, and that user body wins over this one.
                if self.user_const_added_hook_defined() {
                    return Ok(None);
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "const_added",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                return Ok(Some(Object::Nil));
            }
            // Default `Module#const_missing` — raise NameError with the
            // qualified constant path and the `name` attribute set. User
            // hooks step aside the same way const_added's do.
            "const_missing" => {
                if class_rc.find_method("__class__const_missing").is_some() {
                    return Ok(None);
                }
                let mut cursor = Some(Rc::clone(class_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && sc.find_method("const_missing").is_some()
                    {
                        return Ok(None);
                    }
                    cursor = current.superclass();
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "const_missing",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_name = match &arguments[0] {
                    Object::Symbol(s) => s.as_str().to_string(),
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "const_missing",
                            "Symbol or String",
                            other,
                            position,
                        ));
                    }
                };
                return self
                    .dispatch_const_missing(class_rc, &const_name, position)
                    .map(Some);
            }
            "const_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "const_set",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // FrozenError fires before name validation or coercion.
                if class_rc.is_frozen() {
                    let kind = if class_rc.superclass().is_some() {
                        "Class"
                    } else {
                        "Module"
                    };
                    let msg = format!("can't modify frozen {}: {}", kind, class_rc.inspect_name());
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let const_name = self.coerce_method_name(&arguments[0], "const_set", position)?;
                if !is_valid_constant_name(&const_name) {
                    let msg = format!("wrong constant name {}", const_name);
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Overwriting a bound value warns; replacing a pending
                // autoload registration does not.
                if class_rc.get_class_var(&const_name).is_some() {
                    let msg = format!(
                        "warning: already initialized constant {}::{}",
                        class_rc.inspect_name(),
                        const_name
                    );
                    self.emit_warning_to_stderr(&msg, position);
                }
                // Setting the constant cancels any pending autoload for it
                // and clears any "loaded but unrealized" bookkeeping.
                class_rc.remove_autoload(&const_name);
                class_rc.clear_unrealized_autoload(&const_name);
                class_rc.set_class_var(&const_name, arguments[1].clone());
                let assign_file = self
                    .reported_current_file()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                class_rc.set_const_location(&const_name, assign_file, position.line as i64);
                // Object's constants are top-level constants — publish to
                // globals so bare references resolve.
                if class_rc.name() == "Object" {
                    self.globals_mut()
                        .set(const_name.clone(), arguments[1].clone());
                }
                // An anonymous module/class value takes the constant path as
                // its name, cascading into anonymous modules nested under it.
                if let Object::Class(v) | Object::Module(v) = &arguments[1] {
                    let qualified = if class_rc.name() == "Object" {
                        const_name.clone()
                    } else {
                        format!("{}::{}", class_rc.inspect_name(), const_name)
                    };
                    v.assign_name_recursive(&qualified);
                }
                self.trigger_const_added_hook(
                    Object::Class(Rc::clone(class_rc)),
                    &const_name,
                    position,
                )?;
                return Ok(Some(arguments[1].clone()));
            }
            "class_eval" | "module_eval" => {
                let result = self.class_eval_with_args(
                    class_rc,
                    Object::Class(Rc::clone(class_rc)),
                    arguments,
                    position,
                )?;
                return Ok(Some(result));
            }
            "class_exec" | "module_exec" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    _ => return Err(local_jump_error(method_name, position)),
                };
                let result = self.class_exec_block(
                    class_rc,
                    Object::Class(Rc::clone(class_rc)),
                    &block,
                    arguments.to_vec(),
                    position,
                )?;
                return Ok(Some(result));
            }
            "define_method" => {
                return self
                    .module_define_method(class_rc, arguments, position)
                    .map(Some);
            }
            "remove_method" => {
                // Ruby accepts any number of names, including none, and
                // answers with the receiver.
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    names.push(self.coerce_method_name(argument, method_name, position)?);
                }
                if class_rc.is_frozen() && !names.is_empty() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.ruby_name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                for name in names {
                    // A method put on a class by `def Klass.name` lives under
                    // the `__class__` name on the class itself, so a
                    // `class << Klass` body has to reach it there.
                    let removed = class_rc.remove_method(&name)
                        || self.attached_class_of(class_rc).is_some_and(|attached| {
                            attached.remove_method(&format!("__class__{}", name))
                        });
                    // A name the interpreter answers natively has no entry
                    // in any method table, so removing it leaves a tombstone
                    // that the lookup reads as undefined.
                    let removed = removed
                        || (crate::vm::native_methods::is_native_kernel_method(&name)
                            || name == "method_missing")
                            && {
                                let sentinel = Method::undefined(name.clone());
                                class_rc.define_method(&name, Rc::new(sentinel));
                                true
                            };
                    if !removed {
                        let msg =
                            format!("method '{}' not defined in {}", name, class_rc.ruby_name());
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    let hook = if class_rc.get_class_var("__singleton__").is_some() {
                        "singleton_method_removed"
                    } else {
                        "method_removed"
                    };
                    self.invoke_class_hook(class_rc, hook, &name, position)?;
                }
                return Ok(Some(if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                }));
            }
            "undef_method" => {
                // Like `remove_method`, this takes any number of names and
                // answers with the receiver. The method must exist somewhere
                // in the ancestry, though it need not be the receiver's own.
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    names.push(self.coerce_method_name(argument, method_name, position)?);
                }
                if class_rc.is_frozen() && !names.is_empty() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.ruby_name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                for name in names {
                    // A method put on a class by `def Klass.name` lives under
                    // the `__class__` name on the class itself, so a
                    // `class << Klass` body has to reach it there.
                    let class_method_owner = self.attached_class_of(class_rc).filter(|attached| {
                        attached
                            .find_method(&format!("__class__{}", name))
                            .is_some_and(|method| !method.is_undefined)
                    });
                    // Kernel methods live in the native dispatch tables rather
                    // than in a class's method map, so they count as present.
                    if class_method_owner.is_none()
                        && class_rc
                            .find_method(&name)
                            .is_none_or(|method| method.is_undefined)
                        && !is_native_kernel_method(&name)
                        // The default hooks are native no-ops rather than
                        // table entries, so they count as present too.
                        && !MODULE_PRIVATE_HOOKS.contains(&name.as_str())
                        && !BASIC_OBJECT_PRIVATE_METHODS.contains(&name.as_str())
                        // An exception answers its own methods natively, so a
                        // class holding exceptions counts them as present.
                        && !answers_exception_method(self, class_rc, &name)
                    {
                        let msg = format!(
                            "undefined method '{}' for {} '{}'",
                            name,
                            class_rc.kind_name().to_lowercase(),
                            undef_target_name(class_rc)
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        if let Object::Exception(cell) = &exc {
                            cell.borrow_mut().name = Some(name.clone());
                        }
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    let sentinel = Method::undefined(name.clone());
                    match &class_method_owner {
                        // The sentinel has to sit where the lookup will find
                        // it, which for a class method is the `__class__` name
                        // on the attached class.
                        Some(attached) => {
                            attached.define_method(format!("__class__{}", name), Rc::new(sentinel))
                        }
                        None => class_rc.define_method(&name, Rc::new(sentinel)),
                    }
                    let hook = if class_rc.get_class_var("__singleton__").is_some() {
                        "singleton_method_undefined"
                    } else {
                        "method_undefined"
                    };
                    self.invoke_class_hook(class_rc, hook, &name, position)?;
                }
                return Ok(Some(if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                }));
            }
            "alias_method" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "alias_method",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let new_name = self.coerce_method_name(&arguments[0], "alias_method", position)?;
                let old_name = self.coerce_method_name(&arguments[1], "alias_method", position)?;
                if class_rc.is_frozen() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                self.install_alias(class_rc, &new_name, &old_name, position)?;
                if matches!(
                    new_name.as_str(),
                    "initialize"
                        | "initialize_copy"
                        | "initialize_clone"
                        | "initialize_dup"
                        | "respond_to_missing?"
                ) {
                    class_rc.set_method_private(new_name.clone());
                }
                self.invoke_class_hook(class_rc, "method_added", &new_name, position)?;
                return Ok(Some(Object::symbol(new_name)));
            }
            "module_function" => {
                // Ruby undefines `module_function` on Class, so a rebound
                // call with a class receiver is a TypeError.
                if !class_rc.is_module() {
                    let msg = "module_function must be called for modules".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // With no arguments it is a toggle: every method defined
                // afterwards in the body becomes a module function.
                if arguments.is_empty() {
                    class_rc.set_current_visibility(MODULE_FUNCTION_VISIBILITY);
                    return Ok(Some(Object::Nil));
                }
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    let name = self.coerce_method_name(argument, method_name, position)?;
                    self.copy_to_module_function(class_rc, &name, position)?;
                    names.push(Object::symbol(name));
                }
                return Ok(Some(match names.len() {
                    1 => names.remove(0),
                    _ => Object::Array(Rc::new(std::cell::RefCell::new(names))),
                }));
            }
            "class_variable_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "class_variable_set",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                if class_rc.is_frozen() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let key = self.coerce_class_variable_name(
                    &arguments[0],
                    &Object::Class(Rc::clone(class_rc)),
                    position,
                )?;
                // A write reaches the ancestor furthest up the chain that
                // already holds the name, which is the one every class below
                // it reads.
                crate::vm::core::VirtualMachine::class_var_owner(class_rc, &key)
                    .set_class_var(key, arguments[1].clone());
                return Ok(Some(arguments[1].clone()));
            }
            "class_variable_get" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "class_variable_get",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(
                    &arguments[0],
                    &Object::Class(Rc::clone(class_rc)),
                    position,
                )?;
                // A class variable an ancestor defines after a class below
                // it already had one is ambiguous, so reading it is refused.
                if let Some(overtaken) = overtaking_ancestor(class_rc, &key) {
                    let msg = format!(
                        "class variable @@{} of {} is overtaken by {}",
                        key,
                        class_rc.inspect_name(),
                        overtaken.inspect_name()
                    );
                    return Err(MetorexError::runtime_error(
                        msg,
                        position_to_location(position),
                    ));
                }
                match class_rc.lookup_class_var(&key) {
                    Some(value) => return Ok(Some(value)),
                    None => {
                        let msg = format!(
                            "uninitialized class variable @@{} in {}",
                            key,
                            class_rc.name()
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
            }
            // Module#remove_class_variable: only a variable defined directly
            // on the receiver can be removed, and its value comes back.
            "remove_class_variable" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(
                    &arguments[0],
                    &Object::Class(Rc::clone(class_rc)),
                    position,
                )?;
                match class_rc.remove_class_var(&key) {
                    Some(value) => return Ok(Some(value)),
                    None => {
                        let msg = format!(
                            "class variable @@{} not defined for {}",
                            key,
                            class_rc.ruby_name()
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
            }
            "class_variable_defined?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "class_variable_defined?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(
                    &arguments[0],
                    &Object::Class(Rc::clone(class_rc)),
                    position,
                )?;
                return Ok(Some(Object::Bool(
                    class_rc.lookup_class_var(&key).is_some(),
                )));
            }
            "class_variables" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        "class_variables",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // `class_variables(inherit = true)` — when inherit is false,
                // only this class/module's own class variables are reported.
                let inherit = arguments.first().map(is_truthy).unwrap_or(true);
                let names = if inherit {
                    class_rc.inherited_class_variable_names()
                } else {
                    class_rc.own_class_variable_names()
                };
                let symbols: Vec<Object> = names
                    .into_iter()
                    .map(|n| Object::symbol(format!("@@{}", n)))
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    symbols,
                )))));
            }
            // Module methods we treat as no-ops (Metorex doesn't track these
            // concepts, but class bodies that use them still need to load).
            "deprecate_constant" => {
                return Ok(Some(Object::Nil));
            }
            // `ruby2_keywords :name` only applies to a method whose last
            // parameter is a bare `*args` splat. Anything else keeps its
            // signature and gets a warning; a name with no method raises.
            "ruby2_keywords" => {
                for argument in arguments {
                    if !matches!(argument, Object::Symbol(_) | Object::String(_)) {
                        let shown =
                            self.send_to_object(argument.clone(), "inspect", vec![], position)?;
                        let message = format!("{} is not a symbol nor a string", shown);
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    }
                    let name = self.coerce_name_argument(argument, position)?;
                    let Some(method) = class_rc.find_method(&name) else {
                        let message = format!(
                            "undefined method '{}' for class '{}'",
                            name,
                            class_rc.ruby_name()
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("NameError", message.clone()),
                            location: position_to_location(position),
                            message,
                        });
                    };
                    let takes_bare_splat = method
                        .variadic_param
                        .as_ref()
                        .is_some_and(|(index, _)| *index + 1 == method.parameters.len());
                    let takes_keywords = !method.keyword_parameters.is_empty()
                        || method.keyword_rest_parameter.is_some();
                    if !takes_bare_splat || takes_keywords {
                        self.emit_warning_to_stderr(
                            &format!(
                                "Skipping set of ruby2_keywords flag for {} (method accepts keywords or method does not accept argument splat)",
                                name
                            ),
                            position,
                        );
                        continue;
                    }
                    // The flag is shared with every copy of the method, so an
                    // alias made before or after this call carries it too.
                    method.ruby2_keywords.set(true);
                }
                return Ok(Some(Object::Nil));
            }
            _ => {}
        }
        Ok(None)
    }

    /// Coerce a name argument to a String: Strings and Symbols are used
    /// directly, anything else goes through `to_str`. Raises TypeError when
    /// that conversion is missing or returns a non-String.
    /// The encoding a setting was given. A name is looked up, and anything
    /// else already stands for an encoding.
    fn encoding_setting(
        &mut self,
        class_rc: &Rc<Class>,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // An Encoding stands for itself, and so does nil. Anything else is a
        // name, which an object of the program's own spells through `to_str`.
        let named = match value {
            Object::String(_) => value.clone(),
            Object::Nil | Object::Class(_) | Object::Module(_) => return Ok(value.clone()),
            other if self.responds_to(other, "to_str") => {
                Object::string(self.coerce_name_argument(other, position)?)
            }
            other => {
                let message = format!(
                    "no implicit conversion of {} into String",
                    self.builtins().class_of(other).name()
                );
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ));
            }
        };
        let found =
            self.call_class_methods(class_rc, "find", std::slice::from_ref(&named), position)?;
        Ok(found.unwrap_or(named))
    }

    pub(crate) fn coerce_name_argument(
        &mut self,
        arg: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match arg {
            Object::Symbol(s) => Ok(s.as_str().to_string()),
            Object::String(s) => Ok(crate::vm::native_methods::name_text(s)),
            other => {
                let other_obj = other.clone();
                let source_class = self.builtins().class_of(other).name().to_string();
                let converted =
                    if let Some((cls, method)) = self.lookup_method(&other_obj, "to_str") {
                        self.invoke_method(cls, method, other_obj, Vec::new(), position)?
                    } else {
                        let msg = format!("no implicit conversion of {} into String", source_class);
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    };
                match converted {
                    Object::String(s) => Ok(s.as_str().to_string()),
                    other => {
                        let converted_class = self.builtins().class_of(&other).name().to_string();
                        let msg = format!("can't convert {} to String", converted_class);
                        let exc = Object::exception("TypeError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                }
            }
        }
    }

    /// Coerce a class-variable name argument to its storage key (the name with
    /// the leading `@@` removed). Strings and Symbols are used directly; any
    /// other object is converted via `to_str`. Raises TypeError when that
    /// conversion is missing or returns a non-String, and NameError when the
    /// resulting name is not a valid class variable name.
    pub(crate) fn coerce_class_variable_name(
        &mut self,
        arg: &Object,
        receiver: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let name = self.coerce_name_argument(arg, position)?;
        if let Some(rest) = name.strip_prefix("@@")
            && is_valid_class_variable_ident(rest)
        {
            return Ok(rest.to_string());
        }
        let msg = format!("`{}' is not allowed as a class variable name", name);
        Err(crate::vm::errors::invalid_name_error(
            msg, arg, receiver, position,
        ))
    }

    /// Coerce an instance-variable name argument to its storage key (the name
    /// with the leading `@` removed). Strings and Symbols are used directly;
    /// any other object is converted via `to_str`. Raises TypeError when that
    /// conversion is missing or returns a non-String, and NameError when the
    /// resulting name is not a valid instance variable name.
    pub(crate) fn coerce_instance_variable_name(
        &mut self,
        arg: &Object,
        receiver: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let name = self.coerce_name_argument(arg, position)?;
        if let Some(rest) = name.strip_prefix('@')
            && is_valid_class_variable_ident(rest)
        {
            return Ok(rest.to_string());
        }
        let msg = format!("`{}' is not allowed as an instance variable name", name);
        Err(crate::vm::errors::invalid_name_error(
            msg, arg, receiver, position,
        ))
    }

    /// What a queue starts off holding: the elements an enumerable answers
    /// for `to_a`, in the order it answers them.
    pub(crate) fn queue_seed_argument(
        &mut self,
        held: &Object,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if let Object::Array(elements) = held {
            return Ok(elements.borrow().clone());
        }
        let named = self.builtins().class_of(held).ruby_name().to_string();
        if !self.responds_to(held, "to_a") {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("can't convert {named} into Array"),
                position,
            ));
        }
        let answered = self.send_to_object(held.clone(), "to_a", vec![], position)?;
        match answered {
            Object::Array(elements) => Ok(elements.borrow().clone()),
            other => {
                let gives = self.builtins().class_of(&other).ruby_name().to_string();
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!("can't convert {named} into Array ({named}#to_a gives {gives})"),
                    position,
                ))
            }
        }
    }

    /// How many a SizedQueue holds, which is a whole number above zero.
    pub(crate) fn queue_capacity_argument(
        &mut self,
        held: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let counted: i64 = match held {
            Object::Int(counted) => *counted,
            other => {
                let named = self.coerce_integer_argument(other, position)?;
                named.try_into().unwrap_or(i64::MAX)
            }
        };
        if counted <= 0 {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "queue size must be positive",
                position,
            ));
        }
        Ok(counted)
    }

    /// Search `class_rc` for constant `name` the way `const_defined?` does:
    /// its own constant table and autoload registry, plus — when `inherit` —
    /// its mixins (transitively) and superclass chain with their mixins.
    /// `Object` additionally sees top-level constants (globals), and a
    /// module receiver falls back to `Object` as a last resort (Ruby scopes
    /// module constant lookup through Object). `object_fallback` controls
    /// that top-level visibility — it is on for a directly-named constant
    /// and off for the trailing segments of a scoped name (`A::B` must not
    /// find `B` at the top level). Returns `Some((owner, Some(value)))` for
    /// a bound constant, `Some((owner, None))` for a registered-but-unloaded
    /// autoload, `None` when absent. Never triggers autoload loads or
    /// const_missing.
    pub(crate) fn const_entry_on(
        &mut self,
        class_rc: &Rc<Class>,
        name: &str,
        inherit: bool,
        object_fallback: bool,
    ) -> Option<(Rc<Class>, Option<Object>)> {
        let mut queue: Vec<Rc<Class>> = vec![Rc::clone(class_rc)];
        let mut seen: Vec<*const Class> = Vec::new();
        let mut idx = 0;
        while idx < queue.len() {
            let current = Rc::clone(&queue[idx]);
            idx += 1;
            let ptr = Rc::as_ptr(&current);
            if seen.contains(&ptr) {
                continue;
            }
            seen.push(ptr);
            if let Some(v) = current.get_class_var(name) {
                return Some((current, Some(v)));
            }
            // A bound top-level constant beats a still-registered autoload
            // (an autoloaded file may have defined the constant in globals
            // without clearing Object's registration).
            if object_fallback
                && current.name() == "Object"
                && let Some(v) = self.globals().get(name)
            {
                return Some((current, Some(v)));
            }
            // Thread-aware, read-only autoload check: the loading thread
            // sees its own in-progress autoload as cleared, other threads
            // still see it as registered.
            {
                let cls = Rc::clone(&current);
                if self.autoload_pending(&cls, name) {
                    return Some((current, None));
                }
            }
            if inherit {
                for mixin in current.mixin_chain() {
                    queue.push(mixin);
                }
                if let Some(sc) = current.superclass() {
                    queue.push(sc);
                }
            }
        }
        // Module receivers (no superclass chain) see Object's constants.
        if inherit
            && object_fallback
            && class_rc.superclass().is_none()
            && class_rc.name() != "Object"
            && class_rc.name() != "BasicObject"
            && let Some(Object::Class(object_class)) = self.globals().get("Object")
            && !seen.contains(&Rc::as_ptr(&object_class))
        {
            return self.const_entry_on(&object_class, name, inherit, object_fallback);
        }
        None
    }

    /// Dispatch `const_missing(name)` on `module_rc` — the user-defined hook
    /// (a `def self.const_missing` anywhere on the superclass chain, or a
    /// singleton-class method, e.g. an mspec mock) when present, otherwise
    /// the default behavior: raise NameError with the `name` attribute set.
    pub(crate) fn dispatch_const_missing(
        &mut self,
        module_rc: &Rc<Class>,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut found: Option<(Rc<Class>, Rc<crate::object::Method>)> = None;
        let mut cursor = Some(Rc::clone(module_rc));
        while let Some(current) = cursor {
            if let Some(m) = current.find_method("__class__const_missing") {
                found = Some((current, m));
                break;
            }
            if let Some(sc) = current.singleton_class_slot().clone()
                && let Some(m) = sc.find_method("const_missing")
            {
                found = Some((sc, m));
                break;
            }
            cursor = current.superclass();
        }
        if let Some((holder, method)) = found
            && !method.is_undefined
        {
            return self.invoke_method(
                holder,
                method,
                Object::Class(Rc::clone(module_rc)),
                vec![Object::symbol(name.to_string())],
                position,
            );
        }
        let owner = module_rc.ruby_name();
        let qualified = if owner.is_empty() || owner == "Object" {
            name.to_string()
        } else {
            format!("{}::{}", owner, name)
        };
        let msg = format!("uninitialized constant {}", qualified);
        let exc = Object::exception("NameError", msg.clone());
        if let Object::Exception(e) = &exc {
            let mut details = e.borrow_mut();
            details.name = Some(name.to_string());
            details.receiver = Some(Box::new(if module_rc.is_module() {
                Object::Module(Rc::clone(module_rc))
            } else {
                Object::Class(Rc::clone(module_rc))
            }));
        }
        Err(MetorexError::UncaughtException {
            exception: exc,
            location: position_to_location(position),
            message: msg,
        })
    }

    /// Invoke a `method_added` / `singleton_method_added` hook on `class_rc` if
    /// the user defined one. The method receives the new method's name as a
    /// symbol; errors raised by the hook propagate.
    ///
    /// For `singleton_method_added`, Ruby fires the hook on the *attached
    /// object* (the object whose singleton class gained the method), not on
    /// the singleton class itself — so when `class_rc` is a singleton class we
    /// pivot to the attached object before lookup.
    /// The class-level method `name` on `class_rc`: one stored under the
    /// `__class__` convention, one on a singleton class along the superclass
    /// chain, or one copied in by `extend`.
    pub(crate) fn class_method_of(
        &mut self,
        class_rc: &Rc<Class>,
        name: &str,
    ) -> Option<Rc<Method>> {
        if let Some(method) = class_rc.find_method(&format!("__class__{}", name)) {
            return Some(method);
        }
        if let Some(Object::Method(method)) = class_rc.get_class_var(&format!("__ext__{}", name)) {
            return Some(method);
        }
        let mut cursor = Some(Rc::clone(class_rc));
        while let Some(current) = cursor {
            if let Some(sc) = current.singleton_class_slot().clone()
                && let Some(method) = sc.find_method(name)
            {
                return Some(method);
            }
            cursor = current.superclass();
        }
        None
    }

    /// Copy an instance method to the module object as a module function:
    /// the copy is a public module-level method and the original becomes a
    /// private instance method.
    pub(crate) fn copy_to_module_function(
        &mut self,
        module_rc: &Rc<Class>,
        name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let method = module_rc.find_method(name).or_else(|| {
            // Kernel methods live on Object, which a module does not
            // inherit from; `module_function :require` copies from there.
            match self.globals().get("Object") {
                Some(Object::Class(object_class)) => object_class.find_method(name),
                _ => None,
            }
        });
        let method = match method {
            Some(method) => method,
            // Kernel's own methods are native rather than table entries; a
            // stub reaches the same implementation when invoked.
            None if is_native_kernel_method(name) => {
                let mut stub = Method::with_owner(
                    name.to_string(),
                    vec!["args".to_string()],
                    vec![],
                    "Kernel".to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                Rc::new(stub)
            }
            None => {
                return Err(MetorexError::runtime_error(
                    format!(
                        "undefined method '{}' for module '{}'",
                        name,
                        module_rc.name()
                    ),
                    position_to_location(position),
                ));
            }
        };
        module_rc.define_method(format!("__class__{}", name), Rc::clone(&method));
        if module_rc.find_own_method(name).is_some() {
            module_rc.set_method_private(name.to_string());
        }
        self.invoke_class_hook(module_rc, "singleton_method_added", name, position)?;
        Ok(())
    }

    /// A hook that was undefined still reaches a user-defined
    /// `method_missing`. Answers None when nothing defines one, so the caller
    /// raises the NoMethodError itself.
    fn undefined_hook_via_method_missing(
        &mut self,
        receiver: &Object,
        hook: &str,
        argument: &Object,
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        let (owner, handler) = self.lookup_method(receiver, "method_missing")?;
        if handler.is_undefined {
            return None;
        }
        let arguments = vec![Object::symbol(hook.to_string()), argument.clone()];
        Some(self.invoke_method(owner, handler, receiver.clone(), arguments, position))
    }

    /// The class or module a singleton class is attached to, when it is one.
    fn attached_class_of(&self, class_rc: &Rc<Class>) -> Option<Rc<Class>> {
        class_rc.get_class_var("__singleton__")?;
        match class_rc.get_class_var("__attached__") {
            Some(Object::Class(attached) | Object::Module(attached)) => Some(attached),
            _ => None,
        }
    }

    pub(crate) fn invoke_class_hook(
        &mut self,
        class_rc: &Rc<Class>,
        hook: &str,
        added_name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let arg = Object::symbol(added_name.to_string());

        // A hook that was undefined raises when it would have been called,
        // the same as any other call to a method that is not there. A
        // singleton hook lives on the singleton class, which is where
        // `class << obj; undef_method ...; end` puts the marker.
        let undefined_marker = class_rc
            .find_method(hook)
            .filter(|existing| existing.is_undefined)
            .or_else(|| {
                class_rc
                    .singleton_class_slot()
                    .clone()
                    .and_then(|singleton| singleton.find_method(hook))
                    .filter(|existing| existing.is_undefined)
            });
        if undefined_marker.is_some() {
            let attached = class_rc
                .get_class_var("__attached__")
                .unwrap_or(Object::Class(Rc::clone(class_rc)));
            // An undefined method still reaches `method_missing`, which is
            // where the default implementation raises.
            if let Some(handled) =
                self.undefined_hook_via_method_missing(&attached, hook, &arg, position)
            {
                handled?;
                return Ok(());
            }
            let message = format!("undefined method '{}' for {}", hook, attached);
            return Err(MetorexError::UncaughtException {
                exception: crate::vm::errors::no_method_error(
                    &message,
                    hook,
                    &attached,
                    std::slice::from_ref(&arg),
                ),
                location: position_to_location(position),
                message,
            });
        }

        if hook.starts_with("singleton_method_")
            && class_rc.get_class_var("__singleton__").is_some()
            && let Some(attached) = class_rc.get_class_var("__attached__")
        {
            let attached_class = match &attached {
                Object::Class(c) | Object::Module(c) => Some(Rc::clone(c)),
                _ => None,
            };
            if let Some(target_class) = attached_class {
                // The hook may sit on the attached class's singleton class or
                // under the `__class__` name `def self.hook` stores it as.
                if let Some(sc) = target_class.singleton_class_slot().clone()
                    && let Some(method) = sc.find_method(hook)
                {
                    self.invoke_method(sc, method, attached.clone(), vec![arg], position)?;
                    return Ok(());
                }
                if let Some(method) = self.class_method_of(&target_class, hook) {
                    self.invoke_method(
                        target_class,
                        method,
                        attached.clone(),
                        vec![arg],
                        position,
                    )?;
                }
                return Ok(());
            }
            // The attached object is an ordinary one, so the hook is looked
            // up on it the way any other method call on it would be.
            if let Some((owner, method)) = self.lookup_method(&attached, hook)
                && !method.is_undefined
            {
                self.invoke_method(owner, method, attached.clone(), vec![arg], position)?;
            }
            return Ok(());
        }

        let class_method_name = format!("__class__{}", hook);
        let receiver = Object::Class(Rc::clone(class_rc));

        if let Some(method) = class_rc.find_method(&class_method_name) {
            self.invoke_method(Rc::clone(class_rc), method, receiver, vec![arg], position)?;
            return Ok(());
        }
        if let Some(sc) = class_rc.singleton_class_slot().clone()
            && let Some(method) = sc.find_method(hook)
        {
            self.invoke_method(sc, method, receiver, vec![arg], position)?;
        }
        Ok(())
    }
}

/// The visibility state a bare `module_function` sets in a module body. Every
/// method defined afterwards is copied to the module object and made private
/// as an instance method.
pub(crate) const MODULE_FUNCTION_VISIBILITY: &str = "module_function";

/// Kernel's process-control functions, which Ruby exposes as private instance
/// methods on Kernel and as public singleton methods on the module.
/// Whether a name is one of the mangled keys metorex stores alongside real
/// methods. Ruby's own `__`-prefixed methods, such as `__send__`, are not
/// among them and stay visible.
fn is_internal_method_key(name: &str) -> bool {
    const INTERNAL_PREFIXES: &[&str] = &[
        "__class__",
        "__ext__",
        "__refine__",
        "__singleton__",
        "__attached__",
        "__module_body_class__",
        "__struct_",
        "__refinement_label__",
    ];
    INTERNAL_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// BasicObject's public instance methods, which are native rather than
/// entries in its method table.
pub(super) const NATIVE_BASIC_OBJECT_METHODS: &[&str] = &[
    "!",
    "!=",
    "==",
    "__id__",
    "__send__",
    "equal?",
    "instance_eval",
    "instance_exec",
];

/// BasicObject's private instance methods.
pub(super) const BASIC_OBJECT_PRIVATE_METHODS: &[&str] = &[
    "initialize",
    "method_missing",
    "singleton_method_added",
    "singleton_method_removed",
    "singleton_method_undefined",
];

pub(crate) const KERNEL_PRIVATE_FUNCTIONS: &[&str] = &[
    "`",
    "abort",
    "caller",
    "caller_locations",
    "chomp",
    "chop",
    "exec",
    "exit",
    "exit!",
    "fork",
    "format",
    "load",
    "open",
    "spawn",
    "sprintf",
    "at_exit",
    "autoload",
    "autoload?",
    "binding",
    "block_given?",
    "catch",
    "fail",
    "gets",
    "global_variables",
    "initialize_clone",
    "initialize_copy",
    "initialize_dup",
    "lambda",
    "local_variables",
    "loop",
    "p",
    "pp",
    "print",
    "printf",
    "proc",
    "raise",
    "rand",
    "readline",
    "readlines",
    "require",
    "require_relative",
    "respond_to_missing?",
    "sleep",
    "srand",
    "system",
    "putc",
    "puts",
    "throw",
    "trace_var",
    "trap",
    "untrace_var",
    "warn",
];

/// The hooks Module defines as private instance methods with a no-op default
/// implementation. Each takes one argument and returns nil unless the module
/// overrides it.
pub(super) const MODULE_PRIVATE_HOOKS: &[&str] = &[
    "append_features",
    "prepend_features",
    "extend_object",
    "extended",
    "included",
    "prepended",
    "const_added",
    "method_added",
    "method_removed",
    "method_undefined",
];

/// Module's private instance methods beyond the hooks: the declarations a
/// class or module body calls without a receiver. `alias_method` and
/// `define_method` are deliberately absent, being public in Ruby.
pub(super) const MODULE_PRIVATE_DECLARATIONS: &[&str] = &[
    MODULE_FUNCTION_VISIBILITY,
    "private",
    "public",
    "protected",
    "remove_const",
];

/// The native Module and Class instance methods metorex implements, with the
/// parameters each takes and whether the last one is variadic.
/// `Module#instance_methods` advertises the names, and `Object#method` builds
/// a callable stub from the parameter list.
pub(crate) const NATIVE_MODULE_METHODS: &[(&str, &[&str], bool)] = &[
    ("alias_method", &["new_name", "old_name"], false),
    ("attr", &["names"], true),
    ("attr_accessor", &["names"], true),
    ("attr_reader", &["names"], true),
    ("attr_writer", &["names"], true),
    ("constants", &["inherit"], true),
    ("define_method", &["name", "body"], true),
    ("include", &["modules"], true),
    ("method_defined?", &["name", "inherit"], true),
    ("prepend", &["modules"], true),
    ("instance_method", &["name"], false),
    ("remove_method", &["names"], true),
    ("undef_method", &["names"], true),
    ("public_instance_method", &["name"], false),
    ("protected_instance_methods", &["include_super"], true),
    ("instance_methods", &["include_super"], true),
    ("public_instance_methods", &["include_super"], true),
    ("private_instance_methods", &["include_super"], true),
    ("module_function", &["names"], true),
    ("name", &[], false),
];

/// The Kernel methods `call_object_method` implements natively, with the
/// parameter list each one takes, so `obj.method(:name)` can hand out a stub
/// whose `arity` matches Ruby's. A trailing `true` marks the last parameter
/// variadic.
pub(super) const NATIVE_KERNEL_METHODS: &[(&str, &[&str], bool)] = &[
    ("class", &[], false),
    ("clone", &["options"], true),
    ("dup", &[], false),
    ("eql?", &["other"], false),
    ("equal?", &["other"], false),
    ("extend", &["modules"], true),
    ("freeze", &[], false),
    ("frozen?", &[], false),
    ("hash", &[], false),
    ("inspect", &[], false),
    ("instance_of?", &["klass"], false),
    ("instance_variable_get", &["name"], false),
    ("instance_variable_set", &["name", "value"], false),
    ("instance_variables", &[], false),
    ("is_a?", &["klass"], false),
    ("itself", &[], false),
    ("kind_of?", &["klass"], false),
    ("lambda", &[], false),
    ("proc", &[], false),
    ("raise", &["arguments"], true),
    ("method", &["name"], false),
    ("public_method", &["name"], false),
    ("remove_instance_variable", &["name"], false),
    ("singleton_methods", &["all"], true),
    ("methods", &["regular"], true),
    ("nil?", &[], false),
    ("object_id", &[], false),
    ("public_send", &["arguments"], true),
    ("require", &["path"], false),
    ("require_relative", &["path"], false),
    ("respond_to?", &["arguments"], true),
    ("respond_to_missing?", &["name", "include_private"], false),
    ("send", &["arguments"], true),
    ("tap", &[], false),
    ("to_s", &[], false),
    ("__id__", &[], false),
    ("__send__", &["arguments"], true),
    ("instance_exec", &["arguments"], true),
    ("instance_eval", &["arguments"], true),
    ("warn", &["messages"], true),
];

/// A body-less stub for one of the natively implemented Kernel methods.
pub(super) fn native_kernel_method_stub(name: &str) -> Option<Method> {
    let (_, parameters, variadic) = NATIVE_KERNEL_METHODS
        .iter()
        .find(|(entry, _, _)| *entry == name)?;
    let mut stub = Method::with_owner(
        name.to_string(),
        parameters.iter().map(|p| (*p).to_string()).collect(),
        vec![],
        "Kernel".to_string(),
    );
    if *variadic {
        let last = parameters.len().saturating_sub(1);
        stub.variadic_param = Some((last, parameters[last].to_string()));
    }
    Some(stub)
}

/// The NameError `Module#instance_method` raises for a name that is not
/// defined, or has been removed with `undef_method`. Ruby exposes the missing
/// name through `NameError#name`.
fn undefined_instance_method_error(
    name: &str,
    class_rc: &Rc<Class>,
    position: Position,
) -> MetorexError {
    let msg = format!("undefined method '{}' for {}", name, class_rc.name());
    let exc = Object::exception("NameError", msg.clone());
    if let Object::Exception(cell) = &exc {
        cell.borrow_mut().name = Some(name.to_string());
    }
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message: msg,
    }
}

/// A body-less stub for one of the natively implemented Module methods,
/// carrying its parameter list so `arity` and `bind` behave. Invoking it
/// reaches the same native implementation.
pub(crate) fn native_module_method_stub(name: &str) -> Option<Method> {
    let (_, parameters, variadic) = NATIVE_MODULE_METHODS
        .iter()
        .find(|(entry, _, _)| *entry == name)?;
    let unnamed: Vec<String> = parameters
        .iter()
        .map(|_| crate::object::UNNAMED_PARAMETER.to_string())
        .collect();
    let mut stub = Method::with_owner(
        name.to_string(),
        unnamed.clone(),
        vec![],
        "Module".to_string(),
    );
    if *variadic {
        let last = unnamed.len().saturating_sub(1);
        stub.variadic_param = Some((last, unnamed[last].clone()));
    }
    Some(stub)
}

/// Whether a name is one of the methods an exception answers natively, on a
/// class that holds exceptions.
fn answers_exception_method(vm: &VirtualMachine, class_rc: &Rc<Class>, name: &str) -> bool {
    holds_exceptions(vm, class_rc)
        && super::exception_methods::NATIVE_EXCEPTION_METHODS.contains(&name)
}

/// Whether a class stands for exceptions, either because it is one of their
/// classes or because it is the singleton class of an exception.
fn holds_exceptions(vm: &VirtualMachine, class_rc: &Rc<Class>) -> bool {
    if vm.is_exception_class(class_rc) {
        return true;
    }
    class_rc.is_singleton_class()
        && matches!(
            class_rc.get_class_var("__attached__"),
            Some(Object::Exception(_))
        )
}

/// How `undef_method` names its receiver. The metaclass of a class or module
/// is reported as that class, while any other singleton class is reported by
/// its own display.
fn undef_target_name(class_rc: &Rc<Class>) -> String {
    if class_rc.is_singleton_class()
        && let Some(Object::Class(attached) | Object::Module(attached)) =
            class_rc.get_class_var("__attached__")
    {
        return attached.inspect_name();
    }
    class_rc.inspect_name()
}

/// Whether `name` reads as a constant path, which `set_temporary_name`
/// rejects: a `::`-separated chain whose every segment is a constant name,
/// with an optional leading `::`.
fn looks_like_constant_path(name: &str) -> bool {
    let path = name.strip_prefix("::").unwrap_or(name);
    !path.is_empty() && path.split("::").all(is_valid_constant_name)
}

/// Whether the receiver answers `name` through a method of its own: an
/// instance method, a `def self.name` class method, or a singleton method.
/// Used to let a user-defined accessor win over a native handler.
fn has_user_defined_method(class_rc: &Rc<Class>, name: &str) -> bool {
    class_rc.find_method(name).is_some()
        || class_rc
            .find_method(&format!("__class__{}", name))
            .is_some()
        || class_rc
            .singleton_class_slot()
            .as_ref()
            .is_some_and(|sc| sc.find_method(name).is_some())
}

/// Append the transitive ancestor chain of a module (including itself and all
/// modules it mixes in, recursively) onto `chain`. Uses pointer identity in
/// `seen` to skip modules that have already been added, matching Ruby's
/// dedup-on-first-sighting semantics.
/// Append the modules prepended to `owner`, ahead of `owner` itself. Each one
/// gets a fresh visited set: Ruby lists a module once per place it was mixed
/// in, so a module prepended here still appears again where a superclass or an
/// include already carried it.
fn push_prepend_ancestors(owner: &Rc<Class>, chain: &mut Vec<Object>) {
    for prepended in owner.prepend_chain() {
        let mut prepend_seen: Vec<*const Class> = Vec::new();
        push_module_ancestors(&prepended, chain, &mut prepend_seen);
    }
}

pub(super) fn push_module_ancestors(
    module: &Rc<Class>,
    chain: &mut Vec<Object>,
    seen: &mut Vec<*const Class>,
) {
    let ptr = Rc::as_ptr(module);
    if seen.contains(&ptr) {
        return;
    }
    seen.push(ptr);
    push_prepend_ancestors(module, chain);
    chain.push(Object::Module(Rc::clone(module)));
    for mixin in module.mixin_chain() {
        push_module_ancestors(&mixin, chain, seen);
    }
}

/// Append the full ancestor chain of a class (class itself, its mixins
/// recursively, then each superclass with its own mixins) onto `chain`.
pub(crate) fn push_class_ancestors(
    class: &Rc<Class>,
    chain: &mut Vec<Object>,
    seen: &mut Vec<*const Class>,
) {
    let ptr = Rc::as_ptr(class);
    if !seen.contains(&ptr) {
        seen.push(ptr);
        push_prepend_ancestors(class, chain);
        chain.push(Object::Class(Rc::clone(class)));
    }
    for mixin in class.mixin_chain() {
        push_module_ancestors(&mixin, chain, seen);
    }
    let mut current = class.superclass();
    while let Some(parent) = current {
        let pptr = Rc::as_ptr(&parent);
        if !seen.contains(&pptr) {
            seen.push(pptr);
            push_prepend_ancestors(&parent, chain);
            chain.push(Object::Class(Rc::clone(&parent)));
        }
        for mixin in parent.mixin_chain() {
            push_module_ancestors(&mixin, chain, seen);
        }
        current = parent.superclass();
    }
}

/// Validate the identifier portion of a class variable name (the part after
/// `@@`): it must start with a letter or underscore and contain only
/// alphanumerics and underscores.
fn is_valid_class_variable_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || !c.is_ascii() => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
}

/// Kernel methods that `call_object_method` implements natively, so a
/// body-less stub can stand in for them in `Object.instance_method`.
/// The names an open IO handle answers to. Their bodies live in the native
/// dispatch tables rather than in the handle class's method map, so this is
/// what `respond_to?` has to consult for one.
pub(crate) fn is_native_io_method(name: &str) -> bool {
    matches!(
        name,
        "<<" | "each"
            | "each_line"
            | "eof"
            | "eof?"
            | "fileno"
            | "flush"
            | "getc"
            | "gets"
            | "lineno"
            | "lineno="
            | "path"
            | "pos"
            | "pos="
            | "print"
            | "putc"
            | "puts"
            | "read"
            | "readbyte"
            | "readchar"
            | "readline"
            | "readlines"
            | "rewind"
            | "seek"
            | "sync"
            | "sync="
            | "tell"
            | "to_io"
            | "write"
    )
}

pub(crate) fn is_native_kernel_method(name: &str) -> bool {
    // Kernel's private functions are native too, so an UnboundMethod for one
    // is available the same way.
    if KERNEL_PRIVATE_FUNCTIONS.contains(&name) {
        return true;
    }
    matches!(
        name,
        "class"
            | "clone"
            | "dup"
            | "eql?"
            | "equal?"
            | "extend"
            | "freeze"
            | "frozen?"
            | "hash"
            | "inspect"
            | "instance_of?"
            | "instance_variable_get"
            | "instance_variable_set"
            | "instance_variables"
            | "is_a?"
            | "itself"
            | "kind_of?"
            | "lambda"
            | "proc"
            | "method"
            | "methods"
            | "nil?"
            | "object_id"
            | "public_send"
            | "remove_instance_variable"
            | "singleton_methods"
            | "require"
            | "require_relative"
            | "respond_to?"
            | "respond_to_missing?"
            | "send"
            | "tap"
            | "to_s"
            | "to_enum"
            | "enum_for"
            | "display"
            | "then"
            | "yield_self"
            | "instance_variable_defined?"
            | "define_singleton_method"
            | "singleton_class"
            | "singleton_method"
            | "public_method"
            | "__id__"
            | "__send__"
    )
}

/// The hash `Hash[...]` answers: a plain one from Hash itself, and an
/// instance of the subclass when the call was made on one.
fn hash_of_class(
    class_rc: &Rc<crate::class::Class>,
    entries: indexmap::IndexMap<String, Object>,
) -> Object {
    let held = Object::Dict(Rc::new(std::cell::RefCell::new(entries)));
    if class_rc.name() == "Hash" {
        return held;
    }
    let mut instance = crate::object::Instance::new(Rc::clone(class_rc));
    instance.set_var(
        crate::vm::native_methods::HASH_SUBCLASS_VAR.to_string(),
        held,
    );
    Object::Instance(Rc::new(std::cell::RefCell::new(instance)))
}

impl VirtualMachine {
    /// The path an argument names, taking `to_path` from an object that
    /// answers one and refusing anything else.
    fn path_name_argument(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match argument {
            Object::String(path) => Ok(path.as_str().to_string()),
            // A path may be named by an object that answers `to_path`, and
            // by one that answers `to_str` the way any String argument may.
            other if self.responds_to(other, "to_path") || self.responds_to(other, "to_str") => {
                let asked = if self.responds_to(other, "to_path") {
                    "to_path"
                } else {
                    "to_str"
                };
                match self.send_to_object(other.clone(), asked, vec![], position)? {
                    Object::String(path) => Ok(path.as_str().to_string()),
                    converted => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &converted,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                method_name,
                "String",
                other,
                position,
            )),
        }
    }
}

/// One string written so a pattern matches it and nothing else. Ruby escapes
/// every character a pattern reads as punctuation, the space among them, and
/// spells out the whitespace that has no printable form.
fn quoted_for_pattern(source: &str) -> String {
    let mut written = String::with_capacity(source.len());
    for character in source.chars() {
        match character {
            '[' | ']' | '{' | '}' | '(' | ')' | '|' | '-' | '*' | '.' | '\\' | '?' | '+' | '^'
            | '$' | '#' | ' ' => {
                written.push('\\');
                written.push(character);
            }
            '\n' => written.push_str("\\n"),
            '\r' => written.push_str("\\r"),
            '\t' => written.push_str("\\t"),
            '\u{b}' => written.push_str("\\v"),
            '\u{c}' => written.push_str("\\f"),
            other => written.push(other),
        }
    }
    written
}

/// A value of the class named, for asking whether that class answers a
/// method natively.
fn sample_of_class(named: &str) -> Option<Object> {
    match named {
        "Integer" => Some(Object::Int(0)),
        "Float" => Some(Object::Float(0.0)),
        "String" => Some(Object::string("")),
        "Symbol" => Some(Object::symbol("held")),
        "Array" => Some(Object::array(Vec::new())),
        "Hash" => Some(Object::Dict(Rc::new(std::cell::RefCell::new(
            indexmap::IndexMap::new(),
        )))),
        "NilClass" => Some(Object::Nil),
        "TrueClass" => Some(Object::Bool(true)),
        "FalseClass" => Some(Object::Bool(false)),
        _ => None,
    }
}

impl VirtualMachine {
    /// The name a fiber keeps a value under. Ruby takes a Symbol, reads a
    /// String as one, and refuses anything else.
    pub(crate) fn fiber_storage_name(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match given {
            Object::Symbol(name) => Ok(format!(":{name}")),
            Object::String(name) => Ok(format!(":{}", name.as_str())),
            other if self.responds_to(other, "to_str") => {
                match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                    Object::String(name) => Ok(format!(":{}", name.as_str())),
                    _ => Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "wrong argument type (expected a Symbol)",
                        position,
                    )),
                }
            }
            _ => Err(crate::vm::errors::simple_exception(
                "TypeError",
                "wrong argument type (expected a Symbol)",
                position,
            )),
        }
    }

    /// Refuse anything a fiber cannot keep its names in: it has to be a Hash
    /// that may still be written to, and every name in it has to be a Symbol.
    pub(crate) fn check_fiber_storage(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let Object::Dict(entries) = given else {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!(
                    "no implicit conversion of {} into Hash",
                    self.builtins().class_of(given).name()
                ),
                position,
            ));
        };
        if self.object_is_frozen(given) {
            return Err(self.frozen_modification_error(given, position));
        }
        let named: Vec<Object> = {
            let held = entries.borrow();
            held.keys()
                .filter(|slot| !slot.starts_with("__MX_"))
                .map(|slot| crate::vm::utils::dict_key_to_object(slot))
                .collect()
        };
        for key in named {
            if !matches!(key, Object::Symbol(_)) {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "wrong argument type (expected a Symbol)",
                    position,
                ));
            }
        }
        Ok(())
    }
}

/// Whether a class is the named one or descends from it, which is what makes
/// a subclass answer the same native methods.
/// Whether two objects are the same instance.
fn same_object(one: &Object, other: &Object) -> bool {
    matches!((one, other), (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b))
}

fn class_named_in_chain(class_rc: &Rc<Class>, wanted: &str) -> bool {
    let mut cursor = Some(Rc::clone(class_rc));
    while let Some(held) = cursor {
        if held.name() == wanted {
            return true;
        }
        cursor = held.superclass();
    }
    false
}

impl VirtualMachine {
    /// Point `new_name` at whatever `old_name` already names on `class_rc`.
    /// A method written in Ruby is copied; one answered natively has no entry
    /// to copy, so a stub records the name to dispatch under instead.
    pub(crate) fn install_alias(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        new_name: &str,
        old_name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let new_name = new_name.to_string();
        let old_name = old_name.to_string();
        if !class_rc.alias_method(&new_name, &old_name) {
            let mut found = false;
            if let Some(Object::Class(object_class)) = self.globals().get("Object")
                && let Some(method) = object_class.find_method(&old_name)
            {
                class_rc.define_method(&new_name, method);
                found = true;
            }
            // Kernel methods live in the native dispatch tables, so
            // there is no entry to copy. A stub carrying the name
            // keeps the alias present for later removal.
            if !found && is_native_kernel_method(&old_name) {
                let mut stub = Method::with_owner(
                    new_name.clone(),
                    vec!["args".to_string()],
                    vec![],
                    "Kernel".to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                stub.native_alias = Some(old_name.clone());
                class_rc.define_method(&new_name, Rc::new(stub));
                // A Kernel function is a private method, and a name given to
                // one is private the same way.
                if crate::vm::native_methods::is_kernel_private_function(&old_name) {
                    class_rc.set_method_private(new_name.clone());
                }
                found = true;
            }
            // A singleton class aliasing one of the attached object's
            // native methods has no entry to copy either, so the stub
            // records the name it was cut from and the call reaches
            // the native implementation through that.
            if !found
                && let Some(attached) = class_rc.get_class_var("__attached__")
                && self.responds_to(&attached, &old_name)
            {
                let mut stub = Method::with_owner(
                    new_name.clone(),
                    vec!["args".to_string()],
                    vec![],
                    class_rc.name().to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                stub.original_name = Some(old_name.clone());
                stub.native_alias = Some(old_name.clone());
                class_rc.define_method(&new_name, Rc::new(stub));
                found = true;
            }
            // A builtin class answers many of its methods natively,
            // with no entry to copy. A stub carrying the name keeps
            // the alias reaching the native one.
            if !found
                && let Some(probe) = sample_of_class(class_rc.name())
                && self.responds_to(&probe, &old_name)
            {
                let mut stub = Method::with_owner(
                    new_name.clone(),
                    vec!["args".to_string()],
                    vec![],
                    class_rc.name().to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                stub.original_name = Some(old_name.clone());
                stub.native_alias = Some(old_name.clone());
                class_rc.define_method(&new_name, Rc::new(stub));
                found = true;
            }
            if !found {
                let msg = format!(
                    "undefined method '{}' for {} '{}'",
                    old_name,
                    class_rc.kind_name().to_lowercase(),
                    class_rc.name()
                );
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
        }

        Ok(())
    }
}

/// The ancestor above `class_rc` that holds `name` when `class_rc` holds it
/// too. Ruby refuses to read a class variable in that state: which of the two
/// the name stands for is no longer settled.
fn overtaking_ancestor(class_rc: &Rc<Class>, name: &str) -> Option<Rc<Class>> {
    class_rc.get_class_var(name)?;
    let mut cursor = class_rc.superclass();
    while let Some(current) = cursor {
        if current.get_class_var(name).is_some() {
            return Some(current);
        }
        cursor = current.superclass();
    }
    None
}

/// Every part of a path but the last. Trailing separators do not count as a
/// part, a path with no separator at all stands in the working directory, and
/// a run of separators at the front reads as the one root.
fn parent_of_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return if path.is_empty() {
            ".".to_string()
        } else {
            "/".to_string()
        };
    }
    let Some(cut) = trimmed.rfind('/') else {
        return ".".to_string();
    };
    let front = trimmed[..cut].trim_end_matches('/');
    if front.is_empty() {
        return "/".to_string();
    }
    if let Some(rest) = front.strip_prefix("//") {
        return format!("/{}", rest.trim_start_matches('/'));
    }
    front.to_string()
}

impl VirtualMachine {
    /// The encoding `Encoding.compatible?` answers for two objects, following
    /// Ruby's negotiation: an encoding both can be read in, or None.
    pub(crate) fn compatible_encoding(
        &mut self,
        first: &Object,
        second: &Object,
    ) -> Option<String> {
        let first_encoding = self.encoding_name_of(first)?;
        let second_encoding = self.encoding_name_of(second)?;
        if first_encoding == second_encoding {
            return Some(first_encoding);
        }
        let first_text = as_encoded_text(first);
        let second_text = as_encoded_text(second);
        let first_is_string = matches!(first, Object::String(_));
        let second_is_string = matches!(second, Object::String(_));
        if let Some(text) = &second_text
            && second_is_string
            && text.as_str().is_empty()
        {
            return Some(first_encoding);
        }
        if first_is_string
            && second_is_string
            && first_text
                .as_ref()
                .is_some_and(|text| text.as_str().is_empty())
        {
            if encoding_reads_alongside_ascii(&first_encoding) && holds_only_ascii(second) {
                return Some(first_encoding);
            }
            return Some(second_encoding);
        }
        if !encoding_reads_alongside_ascii(&first_encoding)
            || !encoding_reads_alongside_ascii(&second_encoding)
        {
            return None;
        }
        // An object whose encoding follows what it holds, rather than a
        // string, settles the answer as soon as it is plain ASCII.
        if !second_is_string && second_encoding == "US-ASCII" {
            return Some(first_encoding);
        }
        if !first_is_string && first_encoding == "US-ASCII" {
            return Some(second_encoding);
        }
        let (left, right, left_encoding, right_encoding, right_is_string) = if first_is_string {
            (
                first,
                second,
                first_encoding,
                second_encoding,
                second_is_string,
            )
        } else {
            (second, first, second_encoding, first_encoding, false)
        };
        if !matches!(left, Object::String(_)) {
            return None;
        }
        let left_ascii = holds_only_ascii(left);
        if right_is_string {
            let right_ascii = holds_only_ascii(right);
            if left_ascii != right_ascii {
                if left_ascii {
                    return Some(right_encoding);
                }
                return Some(left_encoding);
            }
            if right_ascii {
                return Some(left_encoding);
            }
        }
        if left_ascii {
            return Some(right_encoding);
        }
        None
    }

    /// The encoding an object reports, or None for one that carries none.
    fn encoding_name_of(&mut self, value: &Object) -> Option<String> {
        match value {
            Object::String(text) => Some(text.encoding_name()),
            // A symbol named in ASCII is written in ASCII, whatever the
            // source naming it was written in.
            Object::Symbol(text) => Some(if text.as_str().is_ascii() {
                "US-ASCII".to_string()
            } else {
                text.encoding_name()
            }),
            Object::Regex(pattern, flags) => Some(self.pattern_encoding_name(pattern, flags)),
            Object::Class(class_rc)
                if class_rc
                    .superclass()
                    .is_some_and(|parent| parent.name() == "Encoding") =>
            {
                Some(class_rc.name().to_string())
            }
            _ => None,
        }
    }

    /// The encoding a pattern is read in: the modifier it was written with,
    /// else the encoding of the string it was built from, else the encoding a
    /// literal is read in.
    pub(crate) fn pattern_encoding_name(&self, pattern: &Rc<String>, flags: &str) -> String {
        if flags.contains('u') {
            return "UTF-8".to_string();
        }
        if flags.contains('s') {
            return "Windows-31J".to_string();
        }
        if flags.contains('e') {
            return "EUC-JP".to_string();
        }
        let beyond_ascii = pattern_reaches_beyond_ascii(pattern);
        if flags.contains('n') {
            return if beyond_ascii {
                "ASCII-8BIT"
            } else {
                "US-ASCII"
            }
            .to_string();
        }
        if !beyond_ascii {
            return "US-ASCII".to_string();
        }
        self.pattern_encodings
            .get(&(Rc::as_ptr(pattern) as usize))
            .cloned()
            .unwrap_or_else(|| "UTF-8".to_string())
    }
}

/// The text behind a String or a Symbol, which is what an encoding
/// negotiation reads.
fn as_encoded_text(value: &Object) -> Option<Rc<crate::object::StringValue>> {
    match value {
        Object::String(text) | Object::Symbol(text) => Some(Rc::clone(text)),
        _ => None,
    }
}

/// Whether an encoding lays ASCII out one byte to a character, which is what
/// lets text in it be read alongside text in another such encoding.
pub(crate) fn encoding_reads_alongside_ascii(named: &str) -> bool {
    let dummy = crate::vm::init::ENCODING_NAMES
        .iter()
        .any(|(_, display, dummy)| *dummy && *display == named);
    !dummy && !named.starts_with("UTF-16") && !named.starts_with("UTF-32")
}

/// Whether everything an object holds is plain ASCII, which is what makes it
/// readable in any encoding that lays ASCII out the same way.
fn holds_only_ascii(value: &Object) -> bool {
    let Some(text) = as_encoded_text(value) else {
        return false;
    };
    if !encoding_reads_alongside_ascii(&text.encoding_name()) {
        return false;
    }
    crate::vm::native_methods::string_methods::binary_bytes(&text)
        .iter()
        .all(|byte| byte.is_ascii())
}

/// Whether a pattern's source names anything outside ASCII, counting the
/// escapes that stand for a byte or a codepoint as well as the characters
/// written directly.
fn pattern_reaches_beyond_ascii(source: &str) -> bool {
    let letters: Vec<char> = source.chars().collect();
    let mut at = 0;
    while at < letters.len() {
        let letter = letters[at];
        if !letter.is_ascii() {
            return true;
        }
        if letter == '\\' && at + 1 < letters.len() {
            match letters[at + 1] {
                // A `\u` escape names a codepoint, which reaches past ASCII
                // only when the codepoint itself does.
                'u' => {
                    let (points, next) = unicode_escape_points(&letters, at + 2);
                    if points.iter().any(|held| *held >= 0x80) {
                        return true;
                    }
                    at = next;
                    continue;
                }
                'x' => {
                    let digits: String = letters[at + 2..]
                        .iter()
                        .take(2)
                        .take_while(|held| held.is_ascii_hexdigit())
                        .collect();
                    if let Ok(value) = u32::from_str_radix(&digits, 16)
                        && value >= 0x80
                    {
                        return true;
                    }
                    at += 2 + digits.len();
                    continue;
                }
                _ => {}
            }
            at += 2;
            continue;
        }
        at += 1;
    }
    false
}

/// The RegexpError a pattern Ruby refuses raises, checked before the engine
/// underneath is asked to compile it.
fn refuse_bad_pattern(source: &str, position: Position) -> Result<(), MetorexError> {
    let letters: Vec<char> = source.chars().collect();
    let refuse = |named: &str| {
        let message = format!("{}: /{}/", named, source);
        Err(crate::vm::errors::simple_exception(
            "RegexpError",
            &message,
            position,
        ))
    };
    let mut at = 0;
    let mut class_opened = false;
    let mut depth = 0usize;
    while at < letters.len() {
        match letters[at] {
            '\\' => {
                let Some(escaped) = letters.get(at + 1) else {
                    return refuse("too short escape sequence");
                };
                match escaped {
                    'x' => {
                        let digits = letters[at + 2..]
                            .iter()
                            .take(2)
                            .take_while(|held| held.is_ascii_hexdigit())
                            .count();
                        if digits == 0 {
                            return refuse("invalid hex escape");
                        }
                        at += 2 + digits;
                        continue;
                    }
                    'u' if letters.get(at + 2) == Some(&'{') => {
                        let Some(closing) = letters[at + 3..].iter().position(|held| *held == '}')
                        else {
                            return refuse("invalid Unicode list");
                        };
                        let inside = &letters[at + 3..at + 3 + closing];
                        if inside.is_empty()
                            || inside
                                .iter()
                                .any(|held| !held.is_ascii_hexdigit() && !held.is_whitespace())
                        {
                            return refuse("invalid Unicode list");
                        }
                        if inside
                            .iter()
                            .filter(|held| held.is_ascii_hexdigit())
                            .count()
                            > 6
                        {
                            return refuse("invalid Unicode range");
                        }
                        at += 4 + closing;
                        continue;
                    }
                    'u' => {
                        let digits = letters[at + 2..]
                            .iter()
                            .take(4)
                            .take_while(|held| held.is_ascii_hexdigit())
                            .count();
                        if digits < 4 {
                            return refuse("invalid Unicode escape");
                        }
                        at += 2 + digits;
                        continue;
                    }
                    _ => {
                        at += 2;
                        continue;
                    }
                }
            }
            '[' if !class_opened => class_opened = true,
            ']' if class_opened => class_opened = false,
            '(' if !class_opened => {
                // `(?#...)` is a comment, which runs to the first `)` and
                // carries no group of its own.
                if letters.get(at + 1) == Some(&'?') && letters.get(at + 2) == Some(&'#') {
                    let Some(closing) = letters[at + 3..].iter().position(|held| *held == ')')
                    else {
                        return refuse("end pattern with unmatched parenthesis");
                    };
                    at += 4 + closing;
                    continue;
                }
                if let Some(opener) = group_name_opener(&letters, at) {
                    let named: String = letters[at + 3..]
                        .iter()
                        .take_while(|held| **held != opener)
                        .collect();
                    if named.is_empty() {
                        return refuse("group name is empty");
                    }
                    if named.starts_with(|held: char| held.is_ascii_digit() || held == '-') {
                        return refuse(&format!("invalid group name <{named}>"));
                    }
                }
                depth += 1;
            }
            ')' if !class_opened => {
                if depth == 0 {
                    return refuse("unmatched close parenthesis");
                }
                depth -= 1;
            }
            _ => {}
        }
        at += 1;
    }
    if class_opened {
        return refuse("premature end of char-class");
    }
    if depth > 0 {
        return refuse("end pattern with unmatched parenthesis");
    }
    // Whatever the checks above let through, the engine still has to be able
    // to read the pattern before there is anything to match with.
    if let Err(trouble) = crate::vm::native_methods::regexp_methods::read_pattern(source, "") {
        return refuse(&trouble);
    }
    Ok(())
}

/// The character that closes the name of a group opening at `at`, for
/// `(?<name>` and `(?'name'`. A lookbehind is written `(?<=` and `(?<!`, so
/// those carry no name.
fn group_name_opener(letters: &[char], at: usize) -> Option<char> {
    if letters.get(at + 1) != Some(&'?') {
        return None;
    }
    match letters.get(at + 2) {
        Some('<') if !matches!(letters.get(at + 3), Some('=') | Some('!')) => Some('>'),
        Some('\'') => Some('\''),
        _ => None,
    }
}

/// The codepoints a `\u` escape names, and where the pattern carries on.
fn unicode_escape_points(letters: &[char], at: usize) -> (Vec<u32>, usize) {
    if letters.get(at) == Some(&'{') {
        let Some(closing) = letters[at + 1..].iter().position(|held| *held == '}') else {
            return (Vec::new(), at + 1);
        };
        let inside: String = letters[at + 1..at + 1 + closing].iter().collect();
        let points = inside
            .split_whitespace()
            .filter_map(|held| u32::from_str_radix(held, 16).ok())
            .collect();
        return (points, at + closing + 2);
    }
    let digits: String = letters[at..]
        .iter()
        .take(4)
        .take_while(|held| held.is_ascii_hexdigit())
        .collect();
    let counted = digits.len();
    (
        u32::from_str_radix(&digits, 16).ok().into_iter().collect(),
        at + counted,
    )
}
