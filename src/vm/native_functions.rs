//! Native (built-in) function implementations for the virtual machine.
//!
//! This module contains implementations of global built-in functions like puts, print, etc.

use super::VirtualMachine;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

impl VirtualMachine {
    /// Call a native function by name.
    pub(crate) fn call_native_function(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match name {
            "puts" => {
                // puts prints each argument on a new line
                for arg in &arguments {
                    // Try to call to_s or inspect method if it exists on the object
                    let output = self.get_string_representation(arg, position)?;
                    println!("{}", output);
                }
                Ok(Object::Nil)
            }
            "method" => {
                // method(:name) returns a Method object for the given method name
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!("method() expects 1 argument, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }

                let method_name = match &arguments[0] {
                    Object::Symbol(name) => name.as_str(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "method() expects a Symbol argument, got {}",
                                arguments[0].type_name()
                            ),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };

                // Look up the method in the current environment
                if let Some(obj) = self.environment().get(method_name) {
                    match obj {
                        Object::Method(_) => Ok(obj),
                        _ => Err(MetorexError::runtime_error(
                            format!("'{}' is not a method", method_name),
                            crate::vm::utils::position_to_location(position),
                        )),
                    }
                } else {
                    Err(MetorexError::runtime_error(
                        format!("undefined method '{}'", method_name),
                        crate::vm::utils::position_to_location(position),
                    ))
                }
            }
            _ => Err(MetorexError::runtime_error(
                format!("Unknown native function: {}", name),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }

    /// Get the string representation of an object by calling to_s or inspect if available.
    fn get_string_representation(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        // First try to_s, then inspect, then fall back to Display
        match obj {
            Object::Instance(_) => {
                // Try to_s first
                if let Some((class, method)) = self.lookup_method(obj, "to_s") {
                    let result =
                        self.invoke_method(class, method, obj.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok(s.to_string());
                    }
                }
                // Try inspect as fallback
                if let Some((class, method)) = self.lookup_method(obj, "inspect") {
                    let result =
                        self.invoke_method(class, method, obj.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok(s.to_string());
                    }
                }
                // Fall back to default Display
                Ok(format!("{}", obj))
            }
            _ => Ok(format!("{}", obj)),
        }
    }
}
