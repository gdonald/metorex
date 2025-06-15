//! Native method implementations for Exception objects

use super::super::VirtualMachine;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::utils::*;
use std::rc::Rc;

impl VirtualMachine {
    /// Call a native method on an Exception object
    pub(crate) fn call_exception_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        _arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let exception = match receiver {
            Object::Exception(exc) => exc,
            _ => {
                return Err(MetorexError::runtime_error(
                    format!("Expected Exception, got {:?}", receiver),
                    position_to_location(position),
                ));
            }
        };

        match method_name {
            "message" => {
                // Return the exception message as a String
                let message = exception.borrow().message.clone();
                Ok(Some(Object::String(Rc::new(message))))
            }
            "type" | "exception_type" => {
                // Return the exception type as a String
                let exception_type = exception.borrow().exception_type.clone();
                Ok(Some(Object::String(Rc::new(exception_type))))
            }
            _ => Ok(None), // No native method found, let it fall through
        }
    }
}
