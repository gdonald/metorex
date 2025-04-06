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
