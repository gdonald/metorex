//! Top-level callable invocation for the virtual machine.
//!
//! This module provides `invoke_callable` which dispatches to the appropriate
//! execution path based on the object type (Block, Method, Class, NativeFunction).
//! The actual execution logic lives in sibling modules:
//!   - `block_execution` — block/lambda/proc execution
//!   - `method_execution` — method and function body execution
//!   - `begin_rescue` — begin/rescue/else/ensure evaluation
//!   - `param_binding` — parameter binding utilities

use super::VirtualMachine;
use super::errors::*;
use super::utils::*;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::cell::RefCell;
use std::rc::Rc;

use super::param_binding::positional_arg_count;

impl VirtualMachine {
    /// Invoke a resolved method with evaluated arguments.
    /// Send `name` to `receiver` with already-evaluated arguments, the way
    /// `Object#send` does: a method the receiver defines wins over a native.
    pub(crate) fn send_to_object(
        &mut self,
        receiver: Object,
        name: &str,
        arguments: Vec<Object>,
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        // A module-level method lives either on the singleton class, put
        // there by `class << Mod`, or under the name-mangled key `def
        // self.name` uses. Neither is reachable by an instance-method lookup.
        if let Object::Class(class) | Object::Module(class) = &receiver {
            let singleton_method = class
                .singleton_class_slot()
                .as_ref()
                .and_then(|singleton| singleton.find_method(name));
            if let Some(method) = singleton_method
                .or_else(|| crate::vm::method_lookup::module_level_method(class, name))
                && !method.is_undefined
            {
                let owner = Rc::clone(class);
                return self.invoke_method(owner, method, receiver, arguments, position);
            }
        }
        if let Some((owner, method)) = self.lookup_method(&receiver, name)
            && !method.is_undefined
        {
            return self.invoke_method(owner, method, receiver, arguments, position);
        }
        let class = self.builtins().class_of(&receiver);
        if let Some(result) =
            self.call_native_method(class.as_ref(), &receiver, name, &arguments, position)?
        {
            return Ok(result);
        }
        if let Some(result) = self.call_object_method(&receiver, name, &arguments, position)? {
            return Ok(result);
        }
        let message = format!("undefined method '{}' for {}", name, receiver.type_name());
        Err(MetorexError::UncaughtException {
            exception: Object::exception("NoMethodError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }

    /// Give a freshly built `Interrupt` the signal it stands for. Its
    /// argument is a message, and the signal is always SIGINT.
    fn record_signal_state(class: &Rc<crate::class::Class>, exception: &Object) {
        if class.name() != "Interrupt" {
            return;
        }
        let Object::Exception(details) = exception else {
            return;
        };
        details.borrow_mut().instance_vars.insert(
            crate::vm::signals::SIGNO_KEY.to_string(),
            Object::Int(libc::SIGINT as i64),
        );
    }

    /// `SignalException.new(signal)` is named by its argument rather than
    /// carrying a message of its own, and `SignalException.new(number, text)`
    /// takes that text as the name. Anything that does not name a signal is
    /// an ArgumentError.
    fn build_signal_exception(
        &mut self,
        class: &Rc<crate::class::Class>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        use crate::vm::signals::{name_for_number, number_for_name};
        let invalid = |detail: String| {
            crate::vm::errors::simple_exception("ArgumentError", &detail, position)
        };
        let (number, name) = match arguments.first() {
            Some(Object::Int(given)) => {
                let number = i32::try_from(*given).ok().unwrap_or(-1);
                let Some(name) = name_for_number(number) else {
                    return Err(invalid(format!("invalid signal number {}", given)));
                };
                (number, format!("SIG{}", name))
            }
            Some(Object::String(given) | Object::Symbol(given)) => {
                // A name and a message together is one argument too many:
                // the name is already the message.
                if arguments.len() > 1 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Exact(1),
                        arguments.len(),
                        position,
                    ));
                }
                let Some(number) = number_for_name(given) else {
                    return Err(invalid(format!("invalid signal name {}", given)));
                };
                (
                    number,
                    format!("SIG{}", given.strip_prefix("SIG").unwrap_or(given)),
                )
            }
            other => {
                let type_name = match other {
                    Some(value) => self.builtins().class_of(value).name().to_string(),
                    None => {
                        return Err(crate::vm::errors::argument_count_error(
                            crate::vm::errors::Arity::Range(1, 2),
                            0,
                            position,
                        ));
                    }
                };
                return Err(invalid(format!("bad signal type {}", type_name)));
            }
        };
        // A second argument replaces the name the signal would have gone by.
        let message = match arguments.get(1) {
            Some(text) => self.coerce_name_argument(text, position)?,
            None => name,
        };
        let exception = Object::exception(class.name(), message);
        if let Object::Exception(details) = &exception {
            let mut details = details.borrow_mut();
            details.class = Some(Rc::clone(class));
            details.message_given = true;
            details.instance_vars.insert(
                crate::vm::signals::SIGNO_KEY.to_string(),
                Object::Int(number as i64),
            );
        }
        Ok(exception)
    }

    pub(crate) fn invoke_callable(
        &mut self,
        callable: Object,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match callable {
            Object::Block(block) => block.call(self, arguments, position),
            Object::Method(method) => {
                // Call standalone function (represented as Method object)
                // Validate positional argument count, accounting for defaults
                let expected = method.parameters.len();
                let positional_count = positional_arg_count(&arguments);
                let has_variadic = method.variadic_param.is_some();
                let required =
                    expected - method.default_parameters.len() - if has_variadic { 1 } else { 0 };
                if !has_variadic && (positional_count < required || positional_count > expected) {
                    let accepted = if required == expected {
                        crate::vm::errors::Arity::Exact(expected)
                    } else {
                        crate::vm::errors::Arity::Range(required, expected)
                    };
                    return Err(crate::vm::errors::argument_count_error(
                        accepted,
                        positional_count,
                        position,
                    ));
                }
                if has_variadic && positional_count < required {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::AtLeast(required),
                        positional_count,
                        position,
                    ));
                }
                // Execute function body without self.
                self.user_def_nesting += 1;
                let has_captured = !method.captured_refinements.is_empty();
                if has_captured {
                    self.refinement_scopes.push(
                        method
                            .captured_refinements
                            .iter()
                            .map(|(module, classes)| crate::vm::core::RefinementEntry {
                                module: Rc::clone(module),
                                classes: classes.iter().cloned().collect(),
                            })
                            .collect(),
                    );
                }
                // A top-level function is still a method activation, so
                // `__method__` and `__callee__` inside it name it.
                let defined_name = method
                    .original_name
                    .clone()
                    .unwrap_or_else(|| method.name.clone());
                let result = self.with_call_frame(
                    crate::vm::CallFrame::method(
                        method.name.clone(),
                        None,
                        method.name.clone(),
                        defined_name,
                    ),
                    |vm| vm.execute_function_body(&method, arguments),
                );
                if has_captured {
                    self.refinement_scopes.pop();
                }
                self.user_def_nesting = self.user_def_nesting.saturating_sub(1);
                result
            }
            Object::Class(class) => self.invoke_class(class, arguments, position),
            Object::NativeFunction(name) => self.call_native_function(&name, arguments, position),
            other => Err(not_callable_error(&other, position)),
        }
    }

    /// Handle class invocation: instantiation, kernel conversion functions,
    /// and exception class construction.
    fn invoke_class(
        &mut self,
        class: Rc<Class>,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Hash.new (with or without default block) returns a native Dict.
        if class.name() == "Hash" && arguments.is_empty() {
            let block = self.pending_block.take();
            let mut map = indexmap::IndexMap::new();
            if let Some(block_obj) = block {
                map.insert("__MX_DEFAULT_PROC__".to_string(), block_obj);
            }
            return Ok(Object::Dict(Rc::new(RefCell::new(map))));
        }

        // Regexp.new(source, flags) — the runtime form of a regex literal,
        // which is how an interpolated literal is assembled.
        if class.name() == "Regexp" && !arguments.is_empty() {
            let source = match &arguments[0] {
                Object::String(s) => (**s).clone(),
                Object::Regex(pattern, _) => (**pattern).clone(),
                other => format!("{}", other),
            };
            let flags = match arguments.get(1) {
                Some(Object::String(f)) => (**f).clone(),
                _ => String::new(),
            };
            return Ok(Object::Regex(Rc::new(source), Rc::new(flags)));
        }

        // Kernel conversion functions: Integer(), String(), Array()
        if arguments.len() == 1 {
            match class.name() {
                "String" => {
                    return Ok(Object::String(Rc::new(format!("{}", arguments[0]))));
                }
                "Array" => {
                    if let Some(converted) =
                        self.call_kernel_conversion("Array", &arguments, position)?
                    {
                        return Ok(converted);
                    }
                }
                _ => {}
            }
        }

        // Rational(numerator, denominator) and Complex(real, imaginary) kernel functions
        if class.name() == "Rational" && arguments.len() <= 2 {
            let num = arguments.first().cloned().unwrap_or(Object::Int(0));
            let den = arguments.get(1).cloned().unwrap_or(Object::Int(1));
            let mut inst = crate::object::Instance::new(Rc::clone(&class));
            inst.set_var("numerator".to_string(), num);
            inst.set_var("denominator".to_string(), den);
            return Ok(Object::Instance(Rc::new(RefCell::new(inst))));
        }
        if class.name() == "Complex"
            && let Some(converted) = self.call_kernel_conversion("Complex", &arguments, position)?
        {
            return Ok(converted);
        }

        // A subclass of String holds its characters in an instance variable,
        // since a plain String is a primitive rather than an instance.
        if descends_from(&class, "String") && class.find_method("initialize").is_none() {
            self.pending_block.take();
            let text = match arguments.first() {
                Some(Object::String(text)) => (**text).clone(),
                Some(other) => format!("{}", other),
                None => String::new(),
            };
            let mut instance = crate::object::Instance::new(Rc::clone(&class));
            instance.set_var(
                crate::vm::native_methods::STRING_SUBCLASS_VAR.to_string(),
                Object::string(text),
            );
            return Ok(Object::Instance(Rc::new(RefCell::new(instance))));
        }

        // Check if this is an exception class
        if self.is_exception_class(&class) {
            // A SignalException is named by the signal it stands for, which
            // its own constructor works out. Interrupt is the exception to
            // that: its argument is an ordinary message.
            if descends_from(&class, "SignalException") && !descends_from(&class, "Interrupt") {
                return self.build_signal_exception(&class, &arguments, position);
            }
            // An Errno class reports the message its number stands for, with
            // any custom message and location appended the way Ruby does.
            if let Some(default) = Self::errno_default_message(&class) {
                let custom = match arguments.first() {
                    None | Some(Object::Nil) => None,
                    Some(value) => Some(self.coerce_name_argument(value, position)?),
                };
                let location = match arguments.get(1) {
                    None | Some(Object::Nil) => None,
                    Some(value) => Some(self.coerce_name_argument(value, position)?),
                };
                let message = match (custom, location) {
                    (None, _) => default,
                    (Some(custom), None) => format!("{} - {}", default, custom),
                    (Some(custom), Some(location)) => {
                        format!("{} @ {} - {}", default, location, custom)
                    }
                };
                let exception = Object::exception(class.name(), message);
                if let Object::Exception(details) = &exception {
                    details.borrow_mut().class = Some(Rc::clone(&class));
                }
                return Ok(exception);
            }
            // `FrozenError.new(message, receiver: obj)` records the object the
            // modification was attempted on, and `KeyError.new(receiver:, key:)`
            // records the lookup that missed.
            let mut named_receiver = None;
            let mut named_key = None;
            let mut named_name = None;
            let mut named_args = None;
            let mut arguments = arguments;
            if let Some(Object::Dict(entries)) = arguments.last() {
                let entries = entries.borrow();
                named_receiver = entries.get(":receiver").cloned();
                named_key = entries.get(":key").cloned();
                let recognized = named_receiver.is_some() as usize + named_key.is_some() as usize;
                let named = entries
                    .keys()
                    .filter(|key| key.as_str() != crate::vm::param_binding::KWARGS_MARKER)
                    .count();
                let consumed = recognized > 0 && recognized == named;
                drop(entries);
                if consumed {
                    arguments.pop();
                }
            }
            let message = if arguments.is_empty() {
                String::new()
            } else if arguments.len() == 1 {
                match &arguments[0] {
                    Object::String(s) => (**s).clone(),
                    // Ruby renders the message with `to_s`, which a message
                    // object is free to define.
                    other => match self.send_to_object(other.clone(), "to_s", vec![], position)? {
                        Object::String(text) => (*text).clone(),
                        rendered => rendered.to_string(),
                    },
                }
            } else if (2..=3).contains(&arguments.len()) && descends_from(&class, "NameError") {
                // `NameError.new(message, name)` records the name the lookup
                // was for, which `#name` answers as the object given, and
                // `NoMethodError.new(message, name, args)` its arguments too.
                named_name = Some(arguments[1].clone());
                named_args = arguments.get(2).cloned();
                match &arguments[0] {
                    Object::String(text) => (**text).clone(),
                    other => self.coerce_name_argument(other, position)?,
                }
            } else if arguments.len() == 2 && Self::is_system_call_error(&class) {
                // `SystemCallError.new(message, errno)` answers an instance of
                // the Errno class that number names.
                let text = match &arguments[0] {
                    Object::String(text) => (**text).clone(),
                    other => other.to_string(),
                };
                let Object::Int(number) = arguments[1] else {
                    return Err(MetorexError::runtime_error(
                        "SystemCallError.new expects an Integer errno".to_string(),
                        position_to_location(position),
                    ));
                };
                let named = self
                    .errno_class_for(number)
                    .unwrap_or_else(|| class.name().to_string());
                return Ok(Object::exception(named, text));
            } else {
                return Err(MetorexError::runtime_error(
                    format!(
                        "Exception.new takes 0 or 1 argument, got {}",
                        arguments.len()
                    ),
                    position_to_location(position),
                ));
            };
            let exception = Object::exception(class.name(), message);
            // An anonymous class has no name to look up later, so the class
            // itself travels with the exception.
            if let Object::Exception(details) = &exception {
                let mut details = details.borrow_mut();
                details.class = Some(Rc::clone(&class));
                details.message_given = !arguments.is_empty();
                if let Some(value) = named_receiver {
                    details.receiver = Some(Box::new(value));
                }
                if let Some(value) = named_key {
                    details
                        .instance_vars
                        .insert(crate::vm::KEY_ERROR_KEY.to_string(), value);
                }
                if let Some(value) = named_name {
                    details
                        .instance_vars
                        .insert(crate::vm::NAME_ERROR_NAME_KEY.to_string(), value);
                }
                if let Some(value) = named_args {
                    details
                        .instance_vars
                        .insert(crate::vm::NO_METHOD_ARGS_KEY.to_string(), value);
                }
            }
            Self::record_signal_state(&class, &exception);
            // A subclass that writes its own `initialize` runs it, so state it
            // sets on itself is there. The built-in behaviour already put the
            // message in place, which is what its `super` would have done.
            if let Some(initialize) = class.find_method("initialize")
                && !initialize.is_undefined
                && !initialize.body.is_empty()
            {
                self.invoke_method(
                    Rc::clone(&class),
                    initialize,
                    exception.clone(),
                    arguments,
                    position,
                )?;
            }
            return Ok(exception);
        }

        // Create a new instance of the class
        let instance = Rc::new(RefCell::new(crate::object::Instance::new(Rc::clone(
            &class,
        ))));
        let instance_obj = Object::Instance(Rc::clone(&instance));

        // Look for an 'initialize' method and call it if present. A class
        // without one still consumes the block `new` was given, the way
        // Ruby's default `initialize` does, so it cannot leak to the next
        // call.
        if class.find_method("initialize").is_none() {
            self.pending_block.take();
        }
        if let Some(init_method) = class.find_method("initialize") {
            self.invoke_method(
                class,
                init_method,
                instance_obj.clone(),
                arguments,
                position,
            )?;
        } else if !arguments.is_empty() {
            // The default `initialize` takes none, so extra arguments are an
            // ArgumentError, the way any other arity mismatch is.
            let message = format!(
                "wrong number of arguments (given {}, expected 0)",
                arguments.len()
            );
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }

        Ok(instance_obj)
    }

    /// Check if a class is an exception class (Exception or its subclasses)
    /// The message an Errno class stands for, inherited by a subclass of one.
    fn errno_default_message(class: &Rc<Class>) -> Option<String> {
        let mut cursor = Some(Rc::clone(class));
        while let Some(current) = cursor {
            if let Some(Object::String(message)) =
                current.get_class_var(crate::vm::init::ERRNO_MESSAGE_KEY)
            {
                return Some((*message).clone());
            }
            cursor = current.superclass();
        }
        None
    }

    /// Whether `class` is SystemCallError or one of its Errno subclasses.
    fn is_system_call_error(class: &Rc<Class>) -> bool {
        let mut cursor = Some(Rc::clone(class));
        while let Some(current) = cursor {
            if current.name() == "SystemCallError" {
                return true;
            }
            cursor = current.superclass();
        }
        false
    }

    /// The name of the `Errno` class carrying `number`, when one does.
    fn errno_class_for(&self, number: i64) -> Option<String> {
        let Some(Object::Module(errno) | Object::Class(errno)) = self.globals().get("Errno") else {
            return None;
        };
        errno
            .class_var_names()
            .into_iter()
            .find(|name| {
                matches!(
                    errno.get_class_var(name),
                    Some(Object::Class(class))
                        if class.get_class_var("Errno") == Some(Object::Int(number))
                )
            })
            .map(|name| format!("Errno::{}", name))
    }

    pub(crate) fn is_exception_class(&self, class: &Class) -> bool {
        Self::is_exception_class_static(class)
    }

    /// Static helper to check if a class is an exception class
    fn is_exception_class_static(class: &Class) -> bool {
        let exception_classes = [
            "Exception",
            "StandardError",
            "RuntimeError",
            "TypeError",
            "ValueError",
            "LoadError",
            "ArgumentError",
            "NameError",
            "NoMethodError",
            "NotImplementedError",
            "ScriptError",
            "ZeroDivisionError",
            "FloatDomainError",
            "IndexError",
            "KeyError",
            "RangeError",
            "StopIteration",
            "IOError",
            "FrozenError",
            "Errno::ENOENT",
            "Errno::ENOTDIR",
            "Errno::EACCES",
        ];

        if exception_classes.contains(&class.name()) {
            return true;
        }

        if let Some(superclass) = class.superclass() {
            return Self::is_exception_class_static(&superclass);
        }

        false
    }
}

/// Whether `class` is `name` or descends from it.
pub(crate) fn descends_from(class: &Rc<Class>, name: &str) -> bool {
    let mut cursor = Some(Rc::clone(class));
    while let Some(current) = cursor {
        if current.name() == name {
            return true;
        }
        cursor = current.superclass();
    }
    false
}
