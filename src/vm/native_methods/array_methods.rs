//! Native method implementations for the Array class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

fn compare_for_sort(a: &Object, b: &Object) -> std::cmp::Ordering {
    match (a, b) {
        (Object::Int(x), Object::Int(y)) => x.cmp(y),
        (Object::Float(x), Object::Float(y)) => {
            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Object::Int(x), Object::Float(y)) => (*x as f64)
            .partial_cmp(y)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Object::Float(x), Object::Int(y)) => x
            .partial_cmp(&(*y as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (Object::String(x), Object::String(y)) => x.as_str().cmp(y.as_str()),
        _ => a.to_string().cmp(&b.to_string()),
    }
}

impl VirtualMachine {
    /// Execute native methods for the Array class.
    pub(crate) fn call_array_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Array(array_rc) = receiver else {
            return Ok(None);
        };
        match method_name {
            "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(array_rc.borrow().len() as i64)))
            }
            "push" | "append" | "<<" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                array_rc.borrow_mut().push(arguments[0].clone());
                Ok(Some(receiver.clone()))
            }
            "pop" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(array_rc.borrow_mut().pop().unwrap_or(Object::Nil)))
            }
            "[]" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(self.evaluate_index_operation(
                    receiver.clone(),
                    arguments[0].clone(),
                    position,
                )?))
            }
            "each" => {
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
                            "each requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                let array = array_rc.borrow();
                for element in array.iter() {
                    let args = vec![element.clone()];
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
                Ok(Some(receiver.clone()))
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
                            "map requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                let array = array_rc.borrow();
                let mut results = Vec::new();
                for element in array.iter() {
                    let args = vec![element.clone()];
                    let value = self.execute_block_body(&block, args)?;
                    results.push(value);
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            "select" | "filter" => {
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
                            "select requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                let array = array_rc.borrow();
                let mut results = Vec::new();
                for element in array.iter() {
                    let args = vec![element.clone()];
                    let value = self.execute_block_body(&block, args)?;
                    let is_truthy = !matches!(value, Object::Bool(false) | Object::Nil);
                    if is_truthy {
                        results.push(element.clone());
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            "reduce" => {
                if arguments.len() > 1 {
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
                            "reduce requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                let array = array_rc.borrow();

                let (initial_value, start_index) = if arguments.len() == 1 {
                    (Some(arguments[0].clone()), 0)
                } else {
                    (None, 1)
                };

                if array.is_empty() {
                    return Ok(Some(Object::Nil));
                }

                let mut accumulator = if let Some(init) = initial_value {
                    init
                } else {
                    array[0].clone()
                };

                for element in array.iter().skip(start_index) {
                    let args = vec![accumulator.clone(), element.clone()];
                    accumulator = self.execute_block_body(&block, args)?;
                }
                Ok(Some(accumulator))
            }
            "zip" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();

                let mut other_arrays = Vec::new();
                for arg in arguments {
                    match arg {
                        Object::Array(arr_rc) => {
                            other_arrays.push(arr_rc.borrow().clone());
                        }
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "Array",
                                arg,
                                position,
                            ));
                        }
                    }
                }

                let mut results = Vec::new();
                for (i, element) in array.iter().enumerate() {
                    let mut tuple = vec![element.clone()];
                    for other_array in &other_arrays {
                        if i < other_array.len() {
                            tuple.push(other_array[i].clone());
                        } else {
                            tuple.push(Object::Nil);
                        }
                    }
                    results.push(Object::Array(Rc::new(RefCell::new(tuple))));
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            "transpose" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();

                if array.is_empty() {
                    return Ok(Some(Object::Array(Rc::new(RefCell::new(Vec::new())))));
                }

                let mut row_arrays = Vec::new();
                for element in array.iter() {
                    match element {
                        Object::Array(arr_rc) => {
                            row_arrays.push(arr_rc.borrow().clone());
                        }
                        _ => {
                            return Err(MetorexError::runtime_error(
                                format!(
                                    "transpose requires all elements to be arrays, found {}",
                                    element.type_name()
                                ),
                                position_to_location(position),
                            ));
                        }
                    }
                }

                let max_cols = row_arrays.iter().map(|row| row.len()).max().unwrap_or(0);

                let mut transposed = Vec::new();
                for col_idx in 0..max_cols {
                    let mut new_row = Vec::new();
                    for row in &row_arrays {
                        if col_idx < row.len() {
                            new_row.push(row[col_idx].clone());
                        } else {
                            new_row.push(Object::Nil);
                        }
                    }
                    transposed.push(Object::Array(Rc::new(RefCell::new(new_row))));
                }

                Ok(Some(Object::Array(Rc::new(RefCell::new(transposed)))))
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
                Ok(Some(Object::Int(array_rc.borrow().len() as i64)))
            }
            "shift" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut array = array_rc.borrow_mut();
                if array.is_empty() {
                    Ok(Some(Object::Nil))
                } else {
                    Ok(Some(array.remove(0)))
                }
            }
            "unshift" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                array_rc.borrow_mut().insert(0, arguments[0].clone());
                Ok(Some(receiver.clone()))
            }
            "sort" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut sorted = array_rc.borrow().clone();
                sorted.sort_by(compare_for_sort);
                Ok(Some(Object::Array(Rc::new(RefCell::new(sorted)))))
            }
            "reverse" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut reversed = array_rc.borrow().clone();
                reversed.reverse();
                Ok(Some(Object::Array(Rc::new(RefCell::new(reversed)))))
            }
            "join" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let sep = if arguments.is_empty() {
                    String::new()
                } else {
                    match &arguments[0] {
                        Object::String(s) => s.as_ref().clone(),
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                &arguments[0],
                                position,
                            ));
                        }
                    }
                };
                let parts: Vec<String> = array_rc
                    .borrow()
                    .iter()
                    .map(|obj| format!("{obj}"))
                    .collect();
                Ok(Some(Object::string(parts.join(&sep))))
            }
            _ => Ok(None),
        }
    }
}
