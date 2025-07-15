//! Native method implementations for Exception objects

use super::super::VirtualMachine;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::utils::*;
use std::cell::RefCell;
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
            "backtrace" => {
                // Return the backtrace as an Array of Strings
                let backtrace = exception.borrow().backtrace.clone();
                match backtrace {
                    Some(trace) => {
                        let trace_objects: Vec<Object> = trace
                            .iter()
                            .map(|line| Object::String(Rc::new(line.clone())))
                            .collect();
                        Ok(Some(Object::Array(Rc::new(RefCell::new(trace_objects)))))
                    }
                    None => Ok(Some(Object::Array(Rc::new(RefCell::new(Vec::new()))))),
                }
            }
            "to_s" => {
                // Return a formatted error message with stack trace
                let exc = exception.borrow();
                let mut result = format!("{}: {}", exc.exception_type, exc.message);

                // Add location if available
                if let Some(ref location) = exc.location {
                    result = format!("{} (at {}:{})", result, location.file, location.line);
                }

                // Add backtrace if available
                if let Some(ref backtrace) = exc.backtrace
                    && !backtrace.is_empty()
                {
                    result.push_str("\nBacktrace:\n");
                    for line in backtrace {
                        result.push_str(line);
                        result.push('\n');
                    }
                }

                Ok(Some(Object::String(Rc::new(result))))
            }
            _ => Ok(None), // No native method found, let it fall through
        }
    }
}
