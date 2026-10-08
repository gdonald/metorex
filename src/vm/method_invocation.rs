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
        // An object moved to another Ractor answers nothing the interpreter
        // asks of it either.
        if self.was_moved(&receiver) {
            return Err(crate::vm::errors::simple_exception(
                "NoMethodError",
                &format!("undefined method '{name}' for an instance of Ractor::MovedObject"),
                position,
            ));
        }
        // A refinement in force here stands ahead of what the receiver's own
        // class answers, whether the call was written out or reached through
        // `send`, a Symbol turned into a block, or text being built.
        if let Some(method) = crate::vm::method_lookup::find_refinement(&receiver, name, self) {
            let class = self.builtins().class_of(&receiver);
            return self.invoke_method(class, method, receiver, arguments, position);
        }
        // A module-level method lives either on the singleton class, put
        // there by `class << Mod`, or under the name-mangled key `def
        // self.name` uses. Neither is reachable by an instance-method lookup.
        if let Object::Class(class) | Object::Module(class) = &receiver {
            // What the module itself defines comes first, whether written as
            // `def self.name` or in a `class << self` body. A method it
            // answers to only because it extended a module sits behind those,
            // which is what lets `def Mod.name` override an extended one and
            // reach it again through `super`.
            let module_method = class
                .singleton_class_slot()
                .as_ref()
                .and_then(|singleton| singleton.find_own_method(name))
                .or_else(|| crate::vm::method_lookup::module_own_method(class, name))
                .or_else(|| {
                    class
                        .singleton_class_slot()
                        .as_ref()
                        .and_then(|singleton| singleton.find_method(name))
                })
                .or_else(|| crate::vm::method_lookup::module_extended_method(class, name));
            if let Some(method) = module_method
                && !method.is_undefined
            {
                let owner = Rc::clone(class);
                return self.invoke_method(owner, method, receiver, arguments, position);
            }
        }
        // A class's own native methods come ahead of the instance methods it
        // has as an object.
        if self.object_method_behind_native(&receiver, name) {
            let class = self.builtins().class_of(&receiver);
            if let Some(result) =
                self.call_native_method(class.as_ref(), &receiver, name, &arguments, position)?
            {
                return Ok(result);
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
        if let Some((object_class, method)) = self.object_table_method(name) {
            return self.invoke_method(object_class, method, receiver, arguments, position);
        }
        // A name the object carries no method for reaches `method_missing`,
        // which is where a program of its own decides what to do with it.
        if name != "method_missing"
            && let Some((owner, handler)) = self.lookup_method(&receiver, "method_missing")
            && !handler.is_undefined
        {
            let mut passed = vec![Object::symbol(name.to_string())];
            passed.extend(arguments);
            return self.invoke_method(owner, handler, receiver, passed, position);
        }
        let wording = self.receiver_wording_for(&receiver, position);
        Err(crate::vm::errors::undefined_method_error_worded(
            name, &receiver, &arguments, wording, position,
        ))
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
                let Some(number) = number_for_name(&given.as_str()) else {
                    return Err(invalid(format!("invalid signal name {}", given)));
                };
                (
                    number,
                    format!(
                        "SIG{}",
                        given
                            .as_str()
                            .strip_prefix("SIG")
                            .unwrap_or(&*given.as_str())
                    ),
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
    pub(crate) fn invoke_class(
        &mut self,
        class: Rc<Class>,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let made = self.build_from_class(class, arguments, position)?;
        self.record_allocation(&made, position);
        Ok(made)
    }

    fn build_from_class(
        &mut self,
        class: Rc<Class>,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // What the object space reports about a class is counted from what
        // the program has built, since there is no heap to walk.
        *self
            .allocation_counts
            .entry(Rc::as_ptr(&class) as usize)
            .or_insert(0) += 1;
        // `Hash.new`, `Hash.new(default)`, and `Hash.new { |hash, key| ... }`
        // all answer a native Dict, with the default kept beside the entries.
        if class.name() == "Hash" && arguments.len() <= 2 {
            // `capacity:` only sizes the storage, so it is accepted and
            // dropped. Any other keyword is refused the way Ruby refuses it.
            let mut arguments = arguments;
            if let Some(Object::Dict(entries)) = arguments.last() {
                let named: Vec<String> = entries
                    .borrow()
                    .keys()
                    .filter(|key| key.as_str() != crate::vm::param_binding::KWARGS_MARKER)
                    .cloned()
                    .collect();
                if entries
                    .borrow()
                    .contains_key(crate::vm::param_binding::KWARGS_MARKER)
                {
                    if let Some(unknown) = named.iter().find(|key| key.as_str() != ":capacity") {
                        let message = format!(
                            "unknown keyword: {}",
                            unknown.strip_prefix(':').unwrap_or(unknown)
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &message,
                            position,
                        ));
                    }
                    arguments.pop();
                }
            }
            if arguments.len() > 1 {
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "wrong number of arguments (given 2, expected 0..1)",
                    position,
                ));
            }
            let block = self.pending_block.take();
            // Ruby refuses a default value and a default block together,
            // since a hash answers with one or the other.
            if block.is_some() && !arguments.is_empty() {
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "wrong number of arguments (given 1, expected 0)",
                    position,
                ));
            }
            let mut map = indexmap::IndexMap::new();
            if let Some(block_obj) = block {
                map.insert("__MX_DEFAULT_PROC__".to_string(), block_obj);
            }
            if let Some(default) = arguments.first() {
                map.insert("__MX_DEFAULT__".to_string(), default.clone());
            }
            return Ok(Object::Dict(Rc::new(RefCell::new(map))));
        }

        // Regexp.new(source, flags) — the runtime form of a regex literal,
        // which is how an interpolated literal is assembled.
        if class.name() == "Regexp" && !arguments.is_empty() {
            let source = match &arguments[0] {
                Object::String(s) => s.as_str().to_string(),
                Object::Regex(pattern, _) => pattern.as_str().to_string(),
                other => format!("{}", other),
            };
            let flags = match arguments.get(1) {
                Some(Object::String(f)) => f.as_str().to_string(),
                _ => String::new(),
            };
            return Ok(Object::Regex(Rc::new(source), Rc::new(flags)));
        }

        // The Kernel conversion function Array()
        if arguments.len() == 1
            && class.name() == "Array"
            && let Some(converted) = self.call_kernel_conversion("Array", &arguments, position)?
        {
            return Ok(converted);
        }

        // Rational(numerator, denominator) and Complex(real, imaginary) kernel functions
        if class.name() == "Rational" && arguments.len() <= 2 {
            let num = arguments.first().cloned().unwrap_or(Object::Int(0));
            let den = arguments.get(1).cloned().unwrap_or(Object::Int(1));
            let inst = crate::object::Instance::new(Rc::clone(&class));
            inst.borrow_mut().set_var("numerator".to_string(), num);
            inst.borrow_mut().set_var("denominator".to_string(), den);
            return Ok(Object::Instance(inst));
        }
        if class.name() == "Complex"
            && let Some(converted) = self.call_kernel_conversion("Complex", &arguments, position)?
        {
            return Ok(converted);
        }

        if descends_from(&class, "String") {
            return self.build_string(class, arguments, position);
        }

        // `Range.new(first, last, exclusive)` builds the same value a literal
        // does. A subclass holds one in an instance variable, since a plain
        // Range is a primitive rather than something a subclass carries state
        // on, and unlike a plain one it is not frozen.
        if descends_from(&class, "Range") && (2..=3).contains(&arguments.len()) {
            self.pending_block.take();
            self.check_range_ends(&arguments[0], &arguments[1], position)?;
            let made = Object::Range {
                start: Box::new(arguments[0].clone()),
                end: Box::new(arguments[1].clone()),
                exclusive: arguments.get(2).is_some_and(|flag| flag.is_truthy()),
                mark: std::rc::Rc::new(()),
            };
            if class.name() == "Range" {
                return Ok(made);
            }
            let instance = crate::object::Instance::new(Rc::clone(&class));
            instance.borrow_mut().set_var(
                crate::vm::native_methods::RANGE_SUBCLASS_VAR.to_string(),
                made,
            );
            let object = Object::Instance(instance);
            if let Some(initialize) = class.find_method("initialize")
                && !initialize.is_undefined
                && !initialize.body.is_empty()
            {
                self.invoke_method(
                    Rc::clone(&class),
                    initialize,
                    object.clone(),
                    arguments,
                    position,
                )?;
            }
            return Ok(object);
        }

        // A subclass of Hash holds its entries in an instance variable, since
        // a plain Hash is a primitive rather than an instance. The storage is
        // in place before `initialize` runs, so `self[key] = value` inside it
        // reaches the hash the instance is backed by.
        if descends_from(&class, "Hash") && class.name() != "Hash" {
            let instance = crate::object::Instance::new(Rc::clone(&class));
            let mut entries = indexmap::IndexMap::new();
            // The default and the block are Hash's own `initialize`'s to take,
            // so a subclass that writes one of its own takes them only by
            // handing them on with `super`.
            let own_initialize = class
                .find_method("initialize")
                .is_some_and(|initialize| !initialize.is_undefined && !initialize.body.is_empty());
            if !own_initialize {
                if let Some(block) = self.pending_block.take() {
                    entries.insert("__MX_DEFAULT_PROC__".to_string(), block);
                }
                if let Some(default) = arguments.first() {
                    entries.insert("__MX_DEFAULT__".to_string(), default.clone());
                }
            }
            instance.borrow_mut().set_var(
                crate::vm::native_methods::HASH_SUBCLASS_VAR.to_string(),
                Object::Dict(Rc::new(RefCell::new(entries))),
            );
            let object = Object::Instance(instance);
            if let Some(initialize) = class.find_method("initialize")
                && !initialize.is_undefined
                && !initialize.body.is_empty()
            {
                self.invoke_method(
                    Rc::clone(&class),
                    initialize,
                    object.clone(),
                    arguments,
                    position,
                )?;
            }
            return Ok(object);
        }

        // A subclass of Array holds its elements in an instance variable,
        // since a plain Array is a primitive rather than an instance. The
        // storage is in place before `initialize` runs, so `self << x` inside
        // it appends to the array the instance is backed by.
        if descends_from(&class, "Array") {
            let instance = crate::object::Instance::new(Rc::clone(&class));
            instance.borrow_mut().set_var(
                crate::vm::native_methods::ARRAY_SUBCLASS_VAR.to_string(),
                Object::array(Vec::new()),
            );
            let object = Object::Instance(instance);
            match class.find_method("initialize") {
                Some(initialize) if !initialize.is_undefined && !initialize.body.is_empty() => {
                    self.invoke_method(
                        Rc::clone(&class),
                        initialize,
                        object.clone(),
                        arguments,
                        position,
                    )?;
                }
                _ => {
                    let elements = self.build_array_elements(&arguments, position)?;
                    if let Some(Object::Array(storage)) =
                        crate::vm::native_methods::array_subclass_value(&object)
                    {
                        *storage.borrow_mut() = elements;
                    }
                }
            }
            return Ok(object);
        }

        // A subclass of Proc holds the block it stands for in an instance
        // variable. The block is attached before `initialize` runs, which is
        // what lets a subclass whose `initialize` never calls `super` still
        // answer a call.
        if descends_from(&class, "Proc") {
            // `Proc.new(&callable)` answers that same callable rather than
            // wrapping it again.
            if arguments.is_empty()
                && let Some(source) = self.pending_block_source.take()
                && crate::vm::native_methods::proc_subclass_value(&source).is_some()
            {
                self.pending_block = None;
                return Ok(source);
            }
            let held = match self.pending_block.take() {
                Some(block @ Object::Block(_)) => block,
                other => {
                    self.pending_block = other;
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "tried to create Proc object without a block",
                        position,
                    ));
                }
            };
            let instance = crate::object::Instance::new(Rc::clone(&class));
            instance.borrow_mut().set_var(
                crate::vm::native_methods::PROC_SUBCLASS_VAR.to_string(),
                held,
            );
            let object = Object::Instance(instance);
            if let Some(initialize) = class.find_method("initialize")
                && !initialize.is_undefined
                && !initialize.body.is_empty()
            {
                self.invoke_method(
                    Rc::clone(&class),
                    initialize,
                    object.clone(),
                    arguments,
                    position,
                )?;
            }
            return Ok(object);
        }

        // A subclass of Set holds its elements in an instance variable, the
        // same way an Array subclass holds its own.
        if descends_from(&class, "Set") {
            let instance = crate::object::Instance::new(Rc::clone(&class));
            instance.borrow_mut().set_var(
                crate::vm::native_methods::SET_SUBCLASS_VAR.to_string(),
                Object::empty_set(),
            );
            let object = Object::Instance(instance);
            match class.find_method("initialize") {
                Some(initialize) if !initialize.is_undefined && !initialize.body.is_empty() => {
                    self.invoke_method(
                        Rc::clone(&class),
                        initialize,
                        object.clone(),
                        arguments,
                        position,
                    )?;
                }
                _ => {
                    let set_class = Rc::clone(&self.builtins().set_class);
                    let built = self.call_class_methods(&set_class, "new", &arguments, position)?;
                    if let (Some(Object::Set(storage)), Some(Object::Set(built))) = (
                        crate::vm::native_methods::set_subclass_value(&object),
                        built,
                    ) {
                        *storage.borrow_mut() = built.borrow().clone();
                    }
                }
            }
            return Ok(object);
        }

        // Check if this is an exception class
        if self.is_exception_class(&class) {
            // A SignalException is named by the signal it stands for, which
            // its own constructor works out. Interrupt is the exception to
            // that: its argument is an ordinary message.
            if descends_from(&class, "SignalException") && !descends_from(&class, "Interrupt") {
                return self.build_signal_exception(&class, &arguments, position);
            }
            // SystemExit is named by the status it leaves with, which comes
            // first and may be a boolean standing for success or failure. A
            // subclass writing its own `initialize` takes over.
            if descends_from(&class, "SystemExit")
                && class
                    .find_method("initialize")
                    .is_none_or(|method| method.is_undefined || method.body.is_empty())
            {
                return self.build_system_exit(&class, &arguments, position);
            }
            // SystemCallError and the Errno classes under it are named by a
            // number, which decides both the class that comes back and the
            // message it reports. A subclass writing its own `initialize`
            // takes over, so it is built like any other exception.
            if Self::is_system_call_error(&class)
                && class
                    .find_method("initialize")
                    .is_none_or(|method| method.is_undefined || method.body.is_empty())
            {
                return self.build_system_call_error(&class, &arguments, position);
            }
            // `FrozenError.new(message, receiver: obj)` records the object the
            // modification was attempted on, and `KeyError.new(receiver:, key:)`
            // records the lookup that missed.
            let mut named_receiver = None;
            let mut named_key = None;
            let mut named_name = None;
            let mut named_args = None;
            let mut named_private_call = None;
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
                    // A message of nil is no message at all, so the exception
                    // names itself the way one built with none does.
                    Object::Nil => String::new(),
                    Object::String(s) => s.as_str().to_string(),
                    // Ruby renders the message with `to_s`, which a message
                    // object is free to define.
                    other => match self.send_to_object(other.clone(), "to_s", vec![], position)? {
                        Object::String(text) => text.as_str().to_string(),
                        rendered => rendered.to_string(),
                    },
                }
            } else if ((2..=3).contains(&arguments.len()) && descends_from(&class, "NameError"))
                || (arguments.len() == 4 && descends_from(&class, "NoMethodError"))
            {
                // `NameError.new(message, name)` records the name the lookup
                // was for, which `#name` answers as the object given, and
                // `NoMethodError.new(message, name, args)` its arguments too.
                named_name = Some(arguments[1].clone());
                named_args = arguments.get(2).cloned();
                named_private_call = arguments.get(3).map(|held| Object::Bool(held.is_truthy()));
                match &arguments[0] {
                    Object::String(text) => text.as_str().to_string(),
                    other => self.coerce_name_argument(other, position)?,
                }
            } else if class
                .find_method("initialize")
                .is_some_and(|initialize| !initialize.is_undefined && !initialize.body.is_empty())
            {
                // A subclass writing its own `initialize` decides what the
                // arguments mean, and the message is whatever it hands to
                // `super`, which is nothing until it does.
                String::new()
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
            // A NameError keeps the local variables of the code that built
            // it, which `#local_variables` answers.
            let creating_frame_locals = self.name_error_locals(&class, position);
            // An anonymous class has no name to look up later, so the class
            // itself travels with the exception.
            if let Object::Exception(details) = &exception {
                let mut details = details.borrow_mut();
                details.class = Some(Rc::clone(&class));
                let writes_its_own_initialize = class
                    .find_method("initialize")
                    .is_some_and(|held| !held.is_undefined && !held.body.is_empty());
                details.message_given = !writes_its_own_initialize
                    && !arguments.is_empty()
                    && !matches!(arguments.first(), Some(Object::Nil));
                // The String given is kept as it was, encoding and all, for
                // Marshal to write.
                if details.message_given
                    && let Some(given @ Object::String(_)) = arguments.first()
                {
                    details
                        .instance_vars
                        .insert(crate::vm::MESSAGE_STRING_KEY.to_string(), given.clone());
                }
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
                if let Some(locals) = creating_frame_locals {
                    details
                        .instance_vars
                        .insert(crate::vm::LOCAL_VARIABLES_KEY.to_string(), locals);
                }
                if let Some(value) = named_private_call {
                    details
                        .instance_vars
                        .insert(crate::vm::PRIVATE_CALL_KEY.to_string(), value);
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

        // Create a new instance of the class, with the allocator a C
        // extension gave it when there is one.
        let instance_obj = match self.allocate_through_c(&class, position)? {
            Some(made) => made,
            None => Object::Instance(crate::object::Instance::new(Rc::clone(&class))),
        };

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

    /// Whether `class` is SystemCallError or one of its Errno subclasses.
    /// A String, or an instance of a subclass of String, which holds its
    /// characters in an instance variable since a plain String is a
    /// primitive rather than an instance. Kept out of `build_from_class`, whose
    /// frame is on the stack once for every object built in a nested call.
    #[inline(never)]
    fn build_string(
        &mut self,
        class: Rc<Class>,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if class.name() == "String" {
            let spelled = self.string_from_new_arguments(&arguments, position)?;
            self.pending_block.take();
            return Ok(spelled);
        }
        // A subclass that writes its own `initialize` decides what the
        // characters are, and what it was handed need not be a String,
        // so it starts with none. One that does not takes the characters
        // it was built with.
        let defines_initialize = class.find_method("initialize").is_some();
        // The characters it starts with are what `String.new` makes of
        // nothing, which are bytes rather than text.
        let starting = if defines_initialize {
            self.string_from_new_arguments(&[], position)?
        } else {
            self.string_from_new_arguments(&arguments, position)?
        };
        let instance = crate::object::Instance::new(Rc::clone(&class));
        instance.borrow_mut().set_var(
            crate::vm::native_methods::STRING_SUBCLASS_VAR.to_string(),
            starting,
        );
        let made = Object::Instance(instance);
        if !defines_initialize {
            self.pending_block.take();
            return Ok(made);
        }
        if let Some((owner, method)) = self.lookup_method(&made, "initialize") {
            self.invoke_method(owner, method, made.clone(), arguments.clone(), position)?;
        }
        Ok(made)
    }

    /// The local variables of the code building an instance of `class`, when
    /// it is a NameError, which keeps them for `#local_variables`.
    #[inline(never)]
    fn name_error_locals(&mut self, class: &Rc<Class>, position: Position) -> Option<Object> {
        if !descends_from(class, "NameError") {
            return None;
        }
        self.local_variable_names(Vec::new(), position).ok()
    }

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

    /// `SystemExit.new(status = 0, message = nil)`. A leading Integer, true,
    /// or false is the status Ruby leaves with, and anything else is the
    /// message, which an exception built without one reports as its class
    /// name.
    fn build_system_exit(
        &mut self,
        class: &Rc<Class>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (status, rest) = match arguments.first() {
            Some(Object::Int(status)) => (*status, &arguments[1..]),
            Some(Object::Bool(success)) => (i64::from(!*success), &arguments[1..]),
            _ => (0, arguments),
        };
        let message = match rest.first() {
            None | Some(Object::Nil) => None,
            Some(Object::String(text)) => Some(text.as_str().to_string()),
            Some(other) => Some(self.coerce_name_argument(other, position)?),
        };
        let exception = Object::exception(class.name(), message.clone().unwrap_or_default());
        if let Object::Exception(details) = &exception {
            let mut details = details.borrow_mut();
            details.class = Some(Rc::clone(class));
            details.status = Some(status);
            details.message_given = message.is_some();
            if matches!(arguments.first(), Some(Object::Nil)) {
                details.message_given = false;
            }
        }
        Ok(exception)
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

impl VirtualMachine {
    /// The string `String.new` was asked for: the characters of its argument,
    /// in the encoding the `encoding:` keyword names or the one the argument
    /// already carried. With no argument at all the string is empty and reads
    /// as bytes, which is what Ruby answers.
    pub(crate) fn string_from_new_arguments(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut named = None;
        let mut positional = arguments;
        if let Some(Object::Dict(options)) = arguments.last() {
            let held = options.borrow();
            if held.contains_key(":encoding") || held.contains_key(":capacity") {
                if let Some(wanted) = held.get(":encoding") {
                    named = Some(self.encoding_name_argument(wanted, position)?);
                }
                positional = &arguments[..arguments.len() - 1];
            }
        }
        let (text, held) = match positional.first() {
            None => (String::new(), "ASCII-8BIT".to_string()),
            Some(Object::String(given)) => (given.to_text(), given.encoding_name()),
            Some(other) if self.responds_to(other, "to_str") => {
                match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                    Object::String(given) => (given.to_text(), given.encoding_name()),
                    _ => {
                        return Err(self.string_conversion_error(other, position));
                    }
                }
            }
            Some(other) => match crate::vm::native_methods::string_subclass_value(other) {
                Some(Object::String(given)) => (given.to_text(), given.encoding_name()),
                _ => return Err(self.string_conversion_error(other, position)),
            },
        };
        let made = crate::object::StringValue::with_encoding(text, named.unwrap_or(held));
        Ok(Object::String(Rc::new(made)))
    }

    /// The TypeError Ruby raises for an object that does not read as a String.
    pub(crate) fn string_conversion_error(
        &mut self,
        given: &Object,
        position: Position,
    ) -> MetorexError {
        let message = format!(
            "no implicit conversion of {} into String",
            self.builtins().class_of(given).ruby_name()
        );
        crate::vm::errors::simple_exception("TypeError", &message, position)
    }
}
