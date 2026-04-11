//! Native method implementations for the Range class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the Range class.
    pub(crate) fn call_range_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Range {
            start,
            end,
            exclusive,
        } = receiver
        else {
            return Ok(None);
        };
        match method_name {
            "each" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let pending = self.pending_block.take();
                let block = match pending {
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
                            "each requires a block",
                            position_to_location(position),
                        ));
                    }
                };

                match (start.as_ref(), end.as_ref()) {
                    (Object::Int(start_val), Object::Int(end_val)) => {
                        let end_inclusive = if *exclusive { *end_val - 1 } else { *end_val };

                        for i in *start_val..=end_inclusive {
                            let args = vec![Object::Int(i)];
                            match self.execute_block_with_control_flow(&block, args)? {
                                super::super::ControlFlow::Next
                                | super::super::ControlFlow::Continue { .. } => continue,
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
                        Ok(Some(receiver.clone()))
                    }
                    _ => Err(MetorexError::runtime_error(
                        "Range.each only supports integer ranges".to_string(),
                        position_to_location(position),
                    )),
                }
            }
            "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                match (start.as_ref(), end.as_ref()) {
                    (Object::Int(start_val), Object::Int(end_val)) => {
                        let end_inclusive = if *exclusive { *end_val - 1 } else { *end_val };

                        let elements: Vec<Object> =
                            (*start_val..=end_inclusive).map(Object::Int).collect();
                        Ok(Some(Object::Array(Rc::new(RefCell::new(elements)))))
                    }
                    _ => Err(MetorexError::runtime_error(
                        "Range.to_a only supports integer ranges".to_string(),
                        position_to_location(position),
                    )),
                }
            }
            "include?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match (start.as_ref(), end.as_ref(), &arguments[0]) {
                    (Object::Int(start_val), Object::Int(end_val), Object::Int(test_val)) => {
                        let in_range = if *exclusive {
                            *test_val >= *start_val && *test_val < *end_val
                        } else {
                            *test_val >= *start_val && *test_val <= *end_val
                        };
                        Ok(Some(Object::Bool(in_range)))
                    }
                    (Object::String(s), Object::String(e), Object::String(v)) => {
                        let in_range = if *exclusive {
                            v.as_str() >= s.as_str() && v.as_str() < e.as_str()
                        } else {
                            v.as_str() >= s.as_str() && v.as_str() <= e.as_str()
                        };
                        Ok(Some(Object::Bool(in_range)))
                    }
                    _ => {
                        // Fallback: compare using Display representation
                        let s = format!("{}", start);
                        let e = format!("{}", end);
                        let v = format!("{}", arguments[0]);
                        let in_range = if *exclusive {
                            v >= s && v < e
                        } else {
                            v >= s && v <= e
                        };
                        Ok(Some(Object::Bool(in_range)))
                    }
                }
            }
            "map" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let pending = self.pending_block.take();
                let block = match pending {
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
                            "map requires a block",
                            position_to_location(position),
                        ));
                    }
                };

                match (start.as_ref(), end.as_ref()) {
                    (Object::Int(start_val), Object::Int(end_val)) => {
                        let end_inclusive = if *exclusive { *end_val - 1 } else { *end_val };

                        let mut results = Vec::new();
                        for i in *start_val..=end_inclusive {
                            let args = vec![Object::Int(i)];
                            let value = self.execute_block_body(&block, args)?;
                            results.push(value);
                        }
                        Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
                    }
                    _ => Err(MetorexError::runtime_error(
                        "Range.map only supports integer ranges".to_string(),
                        position_to_location(position),
                    )),
                }
            }
            "begin" | "first" => Ok(Some(start.as_ref().clone())),
            "end" | "last" => Ok(Some(end.as_ref().clone())),
            "exclude_end?" => Ok(Some(Object::Bool(*exclusive))),
            _ => Ok(None),
        }
    }
}
