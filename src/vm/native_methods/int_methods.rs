//! Native method implementations for the Integer class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the Integer class.
    pub(crate) fn call_int_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "abs" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Int(n) = receiver {
                    Ok(Some(Object::Int(n.abs())))
                } else {
                    Ok(None)
                }
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
                if let Object::Int(n) = receiver {
                    Ok(Some(Object::Float(*n as f64)))
                } else {
                    Ok(None)
                }
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
                if let Object::Int(n) = receiver {
                    Ok(Some(Object::String(Rc::new(n.to_string()))))
                } else {
                    Ok(None)
                }
            }
            "times" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            "times requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                if let Object::Int(n) = receiver {
                    for i in 0..*n {
                        let args = vec![Object::Int(i)];
                        match self.execute_block_with_control_flow(&block, args)? {
                            super::super::ControlFlow::Next
                            | super::super::ControlFlow::Continue { .. } => {
                                continue;
                            }
                            super::super::ControlFlow::Break { .. } => break,
                            super::super::ControlFlow::Return { value: _, position } => {
                                return Err(super::super::errors::loop_control_error(
                                    "return", position,
                                ));
                            }
                            super::super::ControlFlow::Exception {
                                exception,
                                position,
                            } => {
                                return Err(MetorexError::runtime_error(
                                    format!(
                                        "Uncaught exception: {}",
                                        super::super::utils::format_exception(&exception)
                                    ),
                                    super::super::utils::position_to_location(position),
                                ));
                            }
                        }
                    }
                    Ok(Some(Object::Int(*n)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
