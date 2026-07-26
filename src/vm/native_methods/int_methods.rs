//! Native method implementations for the Integer class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
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
        let Object::Int(n) = receiver else {
            return Ok(None);
        };
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
                Ok(Some(Object::Int(n.abs())))
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
                Ok(Some(Object::Float(*n as f64)))
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
            "size" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(8)))
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
                Ok(Some(Object::String(Rc::new(n.to_string()))))
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
                    // Without a block, Ruby returns an Enumerator. We
                    // approximate by returning the integer range as an Array
                    // so chained calls like `n.times.map { ... }` work.
                    None => {
                        let nums: Vec<Object> = (0..*n).map(Object::Int).collect();
                        return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(nums)))));
                    }
                };
                for i in 0..*n {
                    let args = vec![Object::Int(i)];
                    match self.execute_block_with_control_flow(&block, args)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        super::super::ControlFlow::Break { .. } => break,
                        super::super::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: super::super::utils::position_to_location(position),
                            });
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
            }
            _ => Ok(None),
        }
    }
}
