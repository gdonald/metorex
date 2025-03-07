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
                if let Object::Float(float_value) = receiver {
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

                    // Round to the specified number of decimal places
                    let multiplier = 10_f64.powi(precision as i32);
                    let rounded = (float_value * multiplier).round() / multiplier;
                    Ok(Some(Object::Float(rounded)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
