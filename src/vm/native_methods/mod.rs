//! Native (built-in) method implementations for the virtual machine.
//!
//! This module contains the implementations of all built-in methods for
//! standard classes like Object, String, and Array.

mod array_methods;
mod exception_methods;
mod float_methods;
mod hash_methods;
mod object_methods;
mod range_methods;
mod string_methods;

use super::VirtualMachine;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// Attempt to execute a native (built-in) method implementation.
    ///
    /// Returns `Ok(Some(result))` if a native method was found and executed successfully,
    /// `Ok(None)` if no native method exists (allowing fallback to user-defined methods),
    /// or `Err` if the method call failed.
    pub(crate) fn call_native_method(
        &mut self,
        class: &Class,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Special handling for Block/Lambda objects
        if let Object::Block(block) = receiver {
            match method_name {
                "call" => {
                    return Ok(Some(block.call(self, arguments.to_vec(), position)?));
                }
                "binding" => {
                    use crate::object::Binding;
                    // Create a Binding object from the block's captured variables
                    let binding = Binding::new(block.captured_vars().clone());
                    return Ok(Some(Object::Binding(Rc::new(binding))));
                }
                _ => {}
            }
        }

        // Special handling for Class objects
        if let Object::Class(class_rc) = receiver {
            match method_name {
                "new" => {
                    // Delegate to invoke_callable which handles instance creation and initialize
                    return self
                        .invoke_callable(
                            Object::Class(Rc::clone(class_rc)),
                            arguments.to_vec(),
                            position,
                        )
                        .map(Some);
                }
                "name" => {
                    return Ok(Some(Object::String(Rc::new(class_rc.name().to_string()))));
                }
                _ => {}
            }
        }

        // Special handling for Method objects
        if let Object::Method(method_obj) = receiver {
            match method_name {
                "name" => {
                    return Ok(Some(Object::String(Rc::new(method_obj.name.clone()))));
                }
                "owner" => {
                    let owner_name = method_obj.owner.as_deref().unwrap_or("main");
                    return Ok(Some(Object::String(Rc::new(owner_name.to_string()))));
                }
                "source_location" => {
                    if let Some(loc) = &method_obj.source_location {
                        return Ok(Some(Object::String(Rc::new(loc.to_string()))));
                    } else {
                        return Ok(Some(Object::String(Rc::new("unknown".to_string()))));
                    }
                }
                "parameters" => {
                    // Return an array of parameter names
                    let params: Vec<Object> = method_obj
                        .parameters
                        .iter()
                        .map(|p| Object::String(Rc::new(p.clone())))
                        .collect();
                    return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                        params,
                    )))));
                }
                _ => {}
            }
        }

        // Dispatch to the appropriate class-specific method implementation
        match class.name() {
            "Object" => self.call_object_method(receiver, method_name, arguments, position),
            "String" => self.call_string_method(receiver, method_name, arguments, position),
            "Array" => self.call_array_method(receiver, method_name, arguments, position),
            "Hash" => self.call_hash_method(receiver, method_name, arguments, position),
            "Float" => self.call_float_method(receiver, method_name, arguments, position),
            "Range" => self.call_range_method(receiver, method_name, arguments, position),
            "Exception" => self.call_exception_method(receiver, method_name, arguments, position),
            _ => Ok(None),
        }
    }
}
