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
        match method_name {
            "each" => {
                // each takes a block parameter
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Range {
                    start,
                    end,
                    exclusive,
                } = receiver
                {
                    let block = match &arguments[0] {
                        Object::Block(block) => block.clone(),
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "Block",
                                &arguments[0],
                                position,
                            ));
                        }
                    };

                    // Only support integer ranges for now
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
                } else {
                    Ok(None)
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
                if let Object::Range {
                    start,
                    end,
                    exclusive,
                } = receiver
                {
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
                } else {
                    Ok(None)
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
                if let Object::Range {
                    start,
                    end,
                    exclusive,
                } = receiver
                {
                    match (start.as_ref(), end.as_ref(), &arguments[0]) {
                        (Object::Int(start_val), Object::Int(end_val), Object::Int(test_val)) => {
                            let in_range = if *exclusive {
                                *test_val >= *start_val && *test_val < *end_val
                            } else {
                                *test_val >= *start_val && *test_val <= *end_val
                            };
                            Ok(Some(Object::Bool(in_range)))
                        }
                        _ => Err(MetorexError::runtime_error(
                            "Range.include? only supports integer ranges".to_string(),
                            position_to_location(position),
                        )),
                    }
                } else {
                    Ok(None)
                }
            }
            "map" => {
                // map takes a block parameter
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Range {
                    start,
                    end,
                    exclusive,
                } = receiver
                {
                    let block = match &arguments[0] {
                        Object::Block(block) => block.clone(),
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "Block",
                                &arguments[0],
                                position,
                            ));
                        }
                    };

                    // Only support integer ranges for now
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
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
