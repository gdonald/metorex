//! Native method implementations for the Array class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the Array class.
    pub(crate) fn call_array_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
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
                if let Object::Array(array_rc) = receiver {
                    Ok(Some(Object::Int(array_rc.borrow().len() as i64)))
                } else {
                    Ok(None)
                }
            }
            "push" | "append" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Array(array_rc) = receiver {
                    array_rc.borrow_mut().push(arguments[0].clone());
                    Ok(Some(receiver.clone()))
                } else {
                    Ok(None)
                }
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
                if let Object::Array(array_rc) = receiver {
                    Ok(Some(array_rc.borrow_mut().pop().unwrap_or(Object::Nil)))
                } else {
                    Ok(None)
                }
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
                // each takes a block parameter
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Array(array_rc) = receiver {
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
                if let Object::Array(array_rc) = receiver {
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

                    let array = array_rc.borrow();
                    let mut results = Vec::new();
                    for element in array.iter() {
                        let args = vec![element.clone()];
                        let value = self.execute_block_body(&block, args)?;
                        results.push(value);
                    }
                    Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
                } else {
                    Ok(None)
                }
            }
            "select" | "filter" => {
                // select/filter takes a block parameter that returns a boolean
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Array(array_rc) = receiver {
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

                    let array = array_rc.borrow();
                    let mut results = Vec::new();
                    for element in array.iter() {
                        let args = vec![element.clone()];
                        let value = self.execute_block_body(&block, args)?;
                        // Check if the result is truthy
                        let is_truthy = !matches!(value, Object::Bool(false) | Object::Nil);
                        if is_truthy {
                            results.push(element.clone());
                        }
                    }
                    Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
                } else {
                    Ok(None)
                }
            }
            "reduce" => {
                // reduce takes a block parameter with 2 arguments (accumulator, element)
                // and optionally an initial value as the first argument
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Array(array_rc) = receiver {
                    let array = array_rc.borrow();

                    // Check if we have an initial value
                    let (block, initial_value, start_index) = if arguments.len() == 2 {
                        let block = match &arguments[1] {
                            Object::Block(block) => block.clone(),
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "Block",
                                    &arguments[1],
                                    position,
                                ));
                            }
                        };
                        (block, Some(arguments[0].clone()), 0)
                    } else {
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
                        (block, None, 1)
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
                } else {
                    Ok(None)
                }
            }
            "zip" => {
                // zip takes one or more arrays and returns an array of arrays
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Array(array_rc) = receiver {
                    let array = array_rc.borrow();

                    // Convert all arguments to arrays
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

                    // Create the zipped result
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
                } else {
                    Ok(None)
                }
            }
            "transpose" => {
                // transpose converts rows to columns and vice versa
                // expects an array of arrays (matrix)
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Array(array_rc) = receiver {
                    let array = array_rc.borrow();

                    // Handle empty array
                    if array.is_empty() {
                        return Ok(Some(Object::Array(Rc::new(RefCell::new(Vec::new())))));
                    }

                    // Verify all elements are arrays
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

                    // Find the maximum row length
                    let max_cols = row_arrays.iter().map(|row| row.len()).max().unwrap_or(0);

                    // Build the transposed matrix
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
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
