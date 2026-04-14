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
            "clear" => {
                array_rc.borrow_mut().clear();
                Ok(Some(receiver.clone()))
            }
            "push" | "append" | "<<" => {
                if arguments.is_empty() {
                    return Ok(Some(receiver.clone()));
                }
                let mut arr = array_rc.borrow_mut();
                for arg in arguments {
                    arr.push(arg.clone());
                }
                drop(arr);
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
                if arguments.len() == 2 {
                    let (start, len) = match (&arguments[0], &arguments[1]) {
                        (Object::Int(s), Object::Int(l)) => (*s, *l),
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "Integer",
                                &arguments[0],
                                position,
                            ));
                        }
                    };
                    let array = array_rc.borrow();
                    let total = array.len() as i64;
                    let start_idx = if start < 0 { total + start } else { start };
                    if start_idx < 0 || start_idx > total || len < 0 {
                        return Ok(Some(Object::Nil));
                    }
                    let end_idx = (start_idx + len).min(total);
                    let slice: Vec<Object> = array[start_idx as usize..end_idx as usize].to_vec();
                    return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(slice)))));
                }
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
                        | super::super::ControlFlow::Value(_)
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
            "partition" => {
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
                            "partition requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                let array = array_rc.borrow();
                let mut truthy = Vec::new();
                let mut falsy = Vec::new();
                for element in array.iter() {
                    let args = vec![element.clone()];
                    let value = self.execute_block_body(&block, args)?;
                    if !matches!(value, Object::Bool(false) | Object::Nil) {
                        truthy.push(element.clone());
                    } else {
                        falsy.push(element.clone());
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(vec![
                    Object::Array(Rc::new(RefCell::new(truthy))),
                    Object::Array(Rc::new(RefCell::new(falsy))),
                ])))))
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
            "unshift" | "prepend" => {
                // Ruby: unshift(*items) prepends all items in order, accepts 0+ args
                let mut arr = array_rc.borrow_mut();
                for (i, item) in arguments.iter().enumerate() {
                    arr.insert(i, item.clone());
                }
                drop(arr);
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
            "inject" => {
                // inject is an alias for reduce
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
                            "inject requires a block",
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
                    return Ok(Some(initial_value.unwrap_or(Object::Nil)));
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
            "dup" | "clone" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();
                Ok(Some(Object::Array(Rc::new(RefCell::new(array.clone())))))
            }
            "flatten" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();
                let mut flat = Vec::new();
                for item in array.iter() {
                    if let Object::Array(inner) = item {
                        flat.extend(inner.borrow().iter().cloned());
                    } else {
                        flat.push(item.clone());
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(flat)))))
            }
            "compact" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();
                let compacted: Vec<Object> = array
                    .iter()
                    .filter(|obj| !matches!(obj, Object::Nil))
                    .cloned()
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(compacted)))))
            }
            "empty?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(array_rc.borrow().is_empty())))
            }
            "first" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(
                    array_rc.borrow().first().cloned().unwrap_or(Object::Nil),
                ))
            }
            "last" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(
                    array_rc.borrow().last().cloned().unwrap_or(Object::Nil),
                ))
            }
            "index" | "find_index" => {
                let array = array_rc.borrow();
                if arguments.len() == 1 {
                    let result = array
                        .iter()
                        .position(|obj| obj.equals(&arguments[0]))
                        .map(|i| Object::Int(i as i64))
                        .unwrap_or(Object::Nil);
                    Ok(Some(result))
                } else if let Some(Object::Block(block)) = self.pending_block.take() {
                    let mut result = Object::Nil;
                    for (i, element) in array.iter().enumerate() {
                        let val = self.execute_block_body(&block, vec![element.clone()])?;
                        if !matches!(val, Object::Bool(false) | Object::Nil) {
                            result = Object::Int(i as i64);
                            break;
                        }
                    }
                    Ok(Some(result))
                } else {
                    Ok(Some(Object::Nil))
                }
            }
            "include?" | "contains?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();
                let found = array.iter().any(|obj| obj.equals(&arguments[0]));
                Ok(Some(Object::Bool(found)))
            }
            "min" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();
                let min = array
                    .iter()
                    .fold(None, |acc: Option<&Object>, item| match acc {
                        None => Some(item),
                        Some(current) => match (current, item) {
                            (Object::Int(a), Object::Int(b)) => {
                                if b < a {
                                    Some(item)
                                } else {
                                    Some(current)
                                }
                            }
                            (Object::Float(a), Object::Float(b)) => {
                                if b < a {
                                    Some(item)
                                } else {
                                    Some(current)
                                }
                            }
                            _ => Some(current),
                        },
                    });
                Ok(Some(min.cloned().unwrap_or(Object::Nil)))
            }
            "max" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();
                let max = array
                    .iter()
                    .fold(None, |acc: Option<&Object>, item| match acc {
                        None => Some(item),
                        Some(current) => match (current, item) {
                            (Object::Int(a), Object::Int(b)) => {
                                if b > a {
                                    Some(item)
                                } else {
                                    Some(current)
                                }
                            }
                            (Object::Float(a), Object::Float(b)) => {
                                if b > a {
                                    Some(item)
                                } else {
                                    Some(current)
                                }
                            }
                            _ => Some(current),
                        },
                    });
                Ok(Some(max.cloned().unwrap_or(Object::Nil)))
            }
            "uniq" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();
                let mut seen = Vec::new();
                let mut unique = Vec::new();
                for item in array.iter() {
                    let repr = format!("{}", item);
                    if !seen.contains(&repr) {
                        seen.push(repr);
                        unique.push(item.clone());
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(unique)))))
            }
            "any?" | "all?" | "none?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = self.pending_block.take();
                let array = array_rc.borrow();
                let truthy = |v: &Object| !matches!(v, Object::Bool(false) | Object::Nil);
                let mut any_true = false;
                let mut all_true = true;
                for element in array.iter() {
                    let result = match &block {
                        Some(Object::Block(b)) => {
                            let args = vec![element.clone()];
                            self.execute_block_body(b, args)?
                        }
                        _ => element.clone(),
                    };
                    if truthy(&result) {
                        any_true = true;
                    } else {
                        all_true = false;
                    }
                }
                let value = match method_name {
                    "any?" => any_true,
                    "all?" => array.is_empty() || all_true,
                    "none?" => !any_true,
                    _ => unreachable!(),
                };
                Ok(Some(Object::Bool(value)))
            }
            "pack" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::String(format) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                let array = array_rc.borrow();
                let mut out: Vec<u8> = Vec::new();
                let mut idx = 0usize;
                let chars: Vec<char> = format.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let ch = chars[i];
                    let native = i + 1 < chars.len() && chars[i + 1] == '!';
                    if native {
                        i += 1;
                    }
                    i += 1;
                    let val = array.get(idx).cloned().unwrap_or(Object::Int(0));
                    let effective = if native && (ch == 'l' || ch == 'L' || ch == 'i' || ch == 'I')
                    {
                        'j'
                    } else {
                        ch
                    };
                    match effective {
                        'j' | 'J' | 'q' | 'Q' => {
                            let n = match val {
                                Object::Int(i) => i,
                                _ => 0,
                            };
                            out.extend_from_slice(&n.to_le_bytes());
                            idx += 1;
                        }
                        'l' | 'L' | 'i' | 'I' | 'V' => {
                            let n = match val {
                                Object::Int(i) => i as i32,
                                _ => 0,
                            };
                            out.extend_from_slice(&n.to_le_bytes());
                            idx += 1;
                        }
                        's' | 'S' | 'v' => {
                            let n = match val {
                                Object::Int(i) => i as i16,
                                _ => 0,
                            };
                            out.extend_from_slice(&n.to_le_bytes());
                            idx += 1;
                        }
                        'c' | 'C' => {
                            let n = match val {
                                Object::Int(i) => i as u8,
                                _ => 0,
                            };
                            out.push(n);
                            idx += 1;
                        }
                        _ => {
                            return Err(MetorexError::runtime_error(
                                format!("Array#pack: unsupported directive '{}'", ch),
                                position_to_location(position),
                            ));
                        }
                    }
                }
                let s: String = out.iter().map(|&b| b as char).collect();
                let _ = idx;
                Ok(Some(Object::String(Rc::new(s))))
            }
            _ => Ok(None),
        }
    }
}
