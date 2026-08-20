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
                    return Err(method_argument_error(
                        &method.name,
                        expected,
                        positional_count,
                        position,
                    ));
                }
                if has_variadic && positional_count < required {
                    return Err(method_argument_error(
                        &method.name,
                        required,
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
                            .map(|(m, cs)| crate::vm::core::RefinementEntry {
                                module: Rc::clone(m),
                                classes: cs.iter().cloned().collect(),
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
            let mut map = std::collections::HashMap::new();
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
                    return match &arguments[0] {
                        Object::Array(_) => Ok(arguments[0].clone()),
                        Object::Nil => Ok(Object::Array(Rc::new(RefCell::new(vec![])))),
                        other => Ok(Object::Array(Rc::new(RefCell::new(vec![other.clone()])))),
                    };
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
        if class.name() == "Complex" && arguments.len() <= 2 {
            let real = arguments.first().cloned().unwrap_or(Object::Int(0));
            let imag = arguments.get(1).cloned().unwrap_or(Object::Int(0));
            let mut inst = crate::object::Instance::new(Rc::clone(&class));
            inst.set_var("real".to_string(), real);
            inst.set_var("imaginary".to_string(), imag);
            return Ok(Object::Instance(Rc::new(RefCell::new(inst))));
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
            let message = if arguments.is_empty() {
                String::new()
            } else if arguments.len() == 1 {
                match &arguments[0] {
                    Object::String(s) => (**s).clone(),
                    other => format!("{:?}", other),
                }
            } else {
                return Err(MetorexError::runtime_error(
                    format!(
                        "Exception.new takes 0 or 1 argument, got {}",
                        arguments.len()
                    ),
                    position_to_location(position),
                ));
            };
            return Ok(Object::exception(class.name(), message));
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
            return Err(MetorexError::runtime_error(
                format!(
                    "No initialize method defined, but {} argument(s) provided",
                    arguments.len()
                ),
                position_to_location(position),
            ));
        }

        Ok(instance_obj)
    }

    /// Check if a class is an exception class (Exception or its subclasses)
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
fn descends_from(class: &Rc<Class>, name: &str) -> bool {
    let mut cursor = Some(Rc::clone(class));
    while let Some(current) = cursor {
        if current.name() == name {
            return true;
        }
        cursor = current.superclass();
    }
    false
}
