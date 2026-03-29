//! Native method implementations for the Float class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Execute native methods for the Float class.
    pub(crate) fn call_float_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Float(f) = receiver else {
            return Ok(None);
        };
        match method_name {
            "round" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let precision = match &arguments[0] {
                    Object::Int(p) => *p,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            &arguments[0],
                            position,
                        ));
                    }
                };

                if precision < 0 {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "Float.round precision must be non-negative, got {}",
                            precision
                        ),
                        position_to_location(position),
                    ));
                }

                let multiplier = 10_f64.powi(precision as i32);
                let rounded = (f * multiplier).round() / multiplier;
                Ok(Some(Object::Float(rounded)))
            }
            "abs" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Float(f.abs())))
            }
            "ceil" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(f.ceil() as i64)))
            }
            "floor" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(f.floor() as i64)))
            }
            "to_i" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(*f as i64)))
            }
            "to_f" => {
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
            "to_s" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::String(std::rc::Rc::new(f.to_string()))))
            }
            _ => Ok(None),
        }
    }
}
