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
    // Two exact integers order exactly, whatever their magnitude.
    if let (Some(x), Some(y)) = (a.as_big_integer(), b.as_big_integer()) {
        return x.cmp(&y);
    }
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
        // Every method that changes the array in place refuses a frozen one.
        const MUTATORS: &[&str] = &[
            "<<",
            "append",
            "push",
            "pop",
            "shift",
            "unshift",
            "prepend",
            "insert",
            "delete_at",
            "clear",
            "concat",
            "replace",
            "fill",
            "compact!",
            "flatten!",
            "map!",
            "collect!",
            "reject!",
            "select!",
            "filter!",
            "reverse!",
            "rotate!",
            "shuffle!",
            "slice!",
            "sort!",
            "sort_by!",
            "uniq!",
            "[]=",
        ];
        if MUTATORS.contains(&method_name) && self.object_is_frozen(receiver) {
            return Err(self.frozen_modification_error(receiver, position));
        }
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
            "inspect" | "to_s" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = array_rc.borrow().clone();
                // Each element renders through its own `inspect`, so an
                // object that defines one is shown the way it asks to be.
                let mut parts = Vec::with_capacity(elements.len());
                for element in &elements {
                    parts.push(self.get_inspect_representation(element, position)?);
                }
                Ok(Some(Object::string(format!("[{}]", parts.join(", ")))))
            }
            "clear" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                array_rc.borrow_mut().clear();
                Ok(Some(receiver.clone()))
            }
            "replace" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = match &arguments[0] {
                    Object::Array(a) => a.borrow().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Array",
                            other,
                            position,
                        ));
                    }
                };
                let mut arr = array_rc.borrow_mut();
                arr.clear();
                arr.extend(other);
                drop(arr);
                Ok(Some(receiver.clone()))
            }
            // Ruby asks each element whether it is `==` to the object, and
            // runs the block when nothing matched.
            "delete" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let elements = array_rc.borrow().clone();
                let mut kept = Vec::new();
                let mut found = false;
                for element in elements {
                    if self.elements_equal(&element, &arguments[0], position)? {
                        found = true;
                        continue;
                    }
                    kept.push(element);
                }
                // A frozen array refuses the removal, but only when there is
                // one to make.
                if found {
                    if self.object_is_frozen(receiver) {
                        return Err(self.frozen_modification_error(receiver, position));
                    }
                    *array_rc.borrow_mut() = kept;
                    return Ok(Some(arguments[0].clone()));
                }
                match block {
                    Some(block) => self
                        .execute_block_body(&block, vec![arguments[0].clone()])
                        .map(Some),
                    None => Ok(Some(Object::Nil)),
                }
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
            // `dig(index, *rest)` — index, then keep digging into the result.
            "dig" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let Object::Int(index) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                let array = array_rc.borrow();
                let length = array.len() as i64;
                let resolved = if *index < 0 { index + length } else { *index };
                if resolved < 0 || resolved >= length {
                    return Ok(Some(Object::Nil));
                }
                let value = array[resolved as usize].clone();
                drop(array);
                if arguments.len() == 1 {
                    return Ok(Some(value));
                }
                if matches!(value, Object::Nil) {
                    return Ok(Some(Object::Nil));
                }
                self.dig_into(&value, &arguments[1..], position).map(Some)
            }
            // `each_with_index` yields the element and its position. Without
            // a block it answers an Enumerator, the way `each` does.
            "each_with_index" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .build_enumerator(receiver.clone(), method_name, vec![], None, position)
                        .map(Some);
                };
                let elements = array_rc.borrow().clone();
                for (index, element) in elements.iter().enumerate() {
                    let args = vec![element.clone(), Object::Int(index as i64)];
                    match self.execute_block_with_control_flow(&block, args)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Continue { .. } => continue,
                        super::super::ControlFlow::Break { value, .. } => {
                            return Ok(Some(value));
                        }
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
                            return Err(MetorexError::UncaughtException {
                                exception: exception.clone(),
                                location: super::super::utils::position_to_location(position),
                                message: super::super::utils::format_exception(&exception),
                            });
                        }
                    }
                }
                Ok(Some(receiver.clone()))
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
                let mut break_value: Option<Object> = None;
                for element in array.iter() {
                    let args = vec![element.clone()];
                    match self.execute_block_with_control_flow(&block, args)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        // `break <value>` from inside the block is what
                        // Ruby returns from `each` — capture the value
                        // and stop iterating. (Bare `break` carries Nil.)
                        super::super::ControlFlow::Break { value, .. } => {
                            break_value = Some(value);
                            break;
                        }
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
                            return Err(MetorexError::UncaughtException {
                                exception: exception.clone(),
                                location: super::super::utils::position_to_location(position),
                                message: super::super::utils::format_exception(&exception),
                            });
                        }
                    }
                }
                Ok(Some(break_value.unwrap_or_else(|| receiver.clone())))
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
            "select" | "filter" | "reject" => {
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
                            format!("{} requires a block", method_name),
                            position_to_location(position),
                        ));
                    }
                };
                let reject = method_name == "reject";
                let array = array_rc.borrow();
                let mut results = Vec::new();
                for element in array.iter() {
                    let args = vec![element.clone()];
                    let value = self.execute_block_body(&block, args)?;
                    let is_truthy = !matches!(value, Object::Bool(false) | Object::Nil);
                    if is_truthy != reject {
                        results.push(element.clone());
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            // `grep(pattern)` keeps the elements the pattern matches under
            // `===`, passing each through the block when one is given.
            "grep" | "grep_v" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => Some(b),
                    _ => None,
                };
                let inverted = method_name == "grep_v";
                let pattern = arguments[0].clone();
                let elements = array_rc.borrow().clone();
                let mut results = Vec::new();
                for element in elements {
                    let matched = self.evaluate_binary_operation(
                        &crate::ast::BinaryOp::CaseEqual,
                        pattern.clone(),
                        element.clone(),
                        position,
                    )?;
                    if matched.is_truthy() == inverted {
                        continue;
                    }
                    results.push(match &block {
                        Some(block) => self.execute_block_body(block, vec![element])?,
                        None => element,
                    });
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            // The first element the block accepts, or nil.
            "find" | "detect" => {
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
                            format!("{} requires a block", method_name),
                            position_to_location(position),
                        ));
                    }
                };
                let elements = array_rc.borrow().clone();
                for element in elements {
                    let value = self.execute_block_body(&block, vec![element.clone()])?;
                    if !matches!(value, Object::Bool(false) | Object::Nil) {
                        return Ok(Some(element));
                    }
                }
                Ok(Some(Object::Nil))
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
                // A block decides the order, answering negative, zero, or
                // positive the way `<=>` does.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut sorted = array_rc.borrow().clone();
                match block {
                    None => sorted.sort_by(compare_for_sort),
                    Some(block) => {
                        sorted = self.sort_with_block(sorted, &block, position)?;
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(sorted)))))
            }
            "sort_by" => {
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
                    None => {
                        return Err(MetorexError::runtime_error(
                            "sort_by requires a block",
                            position_to_location(position),
                        ));
                    }
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                };
                let mut keyed: Vec<(Object, Object)> = Vec::new();
                for element in array_rc.borrow().iter() {
                    let key = self.execute_block_body(&block, vec![element.clone()])?;
                    keyed.push((key, element.clone()));
                }
                keyed.sort_by(|(a, _), (b, _)| compare_for_sort(a, b));
                let sorted: Vec<Object> = keyed.into_iter().map(|(_, v)| v).collect();
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
            // `first` and `last` answer one element, or the first or last
            // `count` of them when given a count.
            "first" | "last" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = array_rc.borrow().clone();
                let Some(argument) = arguments.first() else {
                    let element = match method_name {
                        "first" => elements.first(),
                        _ => elements.last(),
                    };
                    return Ok(Some(element.cloned().unwrap_or(Object::Nil)));
                };
                let count = self.coerce_integer_argument(argument, position)?;
                let count = i64::try_from(&count).unwrap_or(i64::MAX);
                if count < 0 {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative array size",
                        position,
                    ));
                }
                let count = (count as usize).min(elements.len());
                let taken = match method_name {
                    "first" => elements[..count].to_vec(),
                    _ => elements[elements.len() - count..].to_vec(),
                };
                Ok(Some(Object::array(taken)))
            }
            // `take(n)` answers the first n elements, and `drop(n)` the rest.
            // A negative count raises, as Ruby does.
            "take" | "drop" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // The count is read the way `Integer()` reads one, so an
                // object carrying `to_int` counts.
                let count = self.coerce_integer_argument(&arguments[0], position)?;
                let count = &i64::try_from(&count).unwrap_or(i64::MAX);
                if *count < 0 {
                    let message = format!("attempt to {} negative size", method_name);
                    let exception = Object::exception("ArgumentError", message.clone());
                    return Err(MetorexError::UncaughtException {
                        exception,
                        location: position_to_location(position),
                        message,
                    });
                }
                let borrowed = array_rc.borrow();
                let count = (*count as usize).min(borrowed.len());
                let taken = if method_name == "take" {
                    borrowed[..count].to_vec()
                } else {
                    borrowed[count..].to_vec()
                };
                Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(taken)))))
            }
            // `rindex` scans from the end, answering the last position that
            // matches rather than the first.
            "rindex" => {
                if arguments.len() == 1 {
                    self.warn_unused_block(position)?;
                    let elements = array_rc.borrow().clone();
                    for index in (0..elements.len()).rev() {
                        if self.elements_equal(&elements[index], &arguments[0], position)? {
                            return Ok(Some(Object::Int(index as i64)));
                        }
                    }
                    return Ok(Some(Object::Nil));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                // Walking backwards reads the position each step, so an array
                // the block shortens is still walked safely.
                let mut index = array_rc.borrow().len();
                while index > 0 {
                    index -= 1;
                    let Some(element) = array_rc.borrow().get(index).cloned() else {
                        continue;
                    };
                    let answer = self.execute_block_body(&block, vec![element])?;
                    if answer.is_truthy() {
                        return Ok(Some(Object::Int(index as i64)));
                    }
                }
                Ok(Some(Object::Nil))
            }
            // `concat` appends every element of each array it is given.
            "concat" => {
                let mut added = Vec::new();
                for argument in arguments {
                    let other = self.coerce_to_array(argument, position)?;
                    added.extend(other);
                }
                array_rc.borrow_mut().extend(added);
                Ok(Some(receiver.clone()))
            }
            // `delete_at` removes the element at one index and answers it.
            "delete_at" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let index = self.coerce_integer_argument(&arguments[0], position)?;
                let index = Object::integer(index);
                let length = array_rc.borrow().len() as i64;
                let Some(index) = normalize_index(&index, length) else {
                    return Ok(Some(Object::Nil));
                };
                Ok(Some(array_rc.borrow_mut().remove(index)))
            }
            // `delete_if` and `keep_if` filter in place, and answer the array
            // itself so a chain carries on from it.
            "delete_if" | "keep_if" => {
                // Without a block there is nothing to filter by, so the array
                // is not touched and an Enumerator comes back even from a
                // frozen one.
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                // The survivors are moved forward as the walk goes and the
                // array is cut to length at the end, so it keeps its size
                // while the block runs and holds what was already decided if
                // the block raises.
                let mut kept = 0;
                let mut index = 0;
                let mut outcome = Ok(());
                while index < array_rc.borrow().len() {
                    let element = array_rc.borrow()[index].clone();
                    match self.execute_block_body(&block, vec![element.clone()]) {
                        Ok(answer) => {
                            let remove = if method_name == "delete_if" {
                                answer.is_truthy()
                            } else {
                                !answer.is_truthy()
                            };
                            if !remove {
                                array_rc.borrow_mut()[kept] = element;
                                kept += 1;
                            }
                        }
                        Err(error) => {
                            // The element that raised stays, which is where the
                            // walk stopped.
                            let remaining: Vec<Object> = array_rc.borrow()[index..].to_vec();
                            let mut array = array_rc.borrow_mut();
                            array.truncate(kept);
                            array.extend(remaining);
                            outcome = Err(error);
                            break;
                        }
                    }
                    index += 1;
                }
                outcome?;
                array_rc.borrow_mut().truncate(kept);
                Ok(Some(receiver.clone()))
            }
            // `each_index` walks the positions rather than the elements.
            "each_index" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                // The length is read again each step, so an array that grows
                // while it is walked is walked to its new end.
                let mut index = 0;
                while index < array_rc.borrow().len() {
                    self.execute_block_body(&block, vec![Object::Int(index as i64)])?;
                    index += 1;
                }
                Ok(Some(receiver.clone()))
            }
            "reverse_each" => {
                let elements = array_rc.borrow().clone();
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                // Walking backwards reads the position each step, so an array
                // that grows while it is walked keeps its remaining elements
                // lined up with the positions still to come.
                let _ = elements;
                let mut index = array_rc.borrow().len();
                while index > 0 {
                    index -= 1;
                    let element = match array_rc.borrow().get(index) {
                        Some(element) => element.clone(),
                        None => continue,
                    };
                    self.execute_block_body(&block, vec![element])?;
                }
                Ok(Some(receiver.clone()))
            }
            // `values_at` reads several positions at once, answering nil for
            // one the array does not reach.
            "values_at" => {
                let array = array_rc.borrow().clone();
                let length = array.len() as i64;
                let mut picked = Vec::new();
                for argument in arguments {
                    match normalize_index(argument, length) {
                        Some(index) => picked.push(array[index].clone()),
                        None => picked.push(Object::Nil),
                    }
                }
                Ok(Some(Object::array(picked)))
            }
            // `intersect?` asks whether the two arrays share an element.
            "intersect?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = self.coerce_to_array(&arguments[0], position)?;
                let shared = array_rc
                    .borrow()
                    .iter()
                    .any(|element| other.iter().any(|candidate| candidate.equals(element)));
                Ok(Some(Object::Bool(shared)))
            }
            // `assoc` and `rassoc` look through an array of arrays, matching
            // the first element or the second.
            "assoc" | "rassoc" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let slot = usize::from(method_name == "rassoc");
                let found = array_rc.borrow().iter().find_map(|element| {
                    let Object::Array(pair) = element else {
                        return None;
                    };
                    let pair = pair.borrow();
                    match pair.get(slot) {
                        Some(candidate) if candidate.equals(&arguments[0]) => Some(Object::Array(
                            std::rc::Rc::new(std::cell::RefCell::new(pair.clone())),
                        )),
                        _ => None,
                    }
                });
                Ok(Some(found.unwrap_or(Object::Nil)))
            }
            // The array is read a position at a time rather than held open,
            // since the block or the `==` it runs may change it.
            "index" | "find_index" => {
                if arguments.len() == 1 {
                    self.warn_unused_block(position)?;
                    let elements = array_rc.borrow().clone();
                    for (index, element) in elements.iter().enumerate() {
                        if self.elements_equal(element, &arguments[0], position)? {
                            return Ok(Some(Object::Int(index as i64)));
                        }
                    }
                    return Ok(Some(Object::Nil));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                let mut index = 0;
                while index < array_rc.borrow().len() {
                    let element = array_rc.borrow()[index].clone();
                    let answer = self.execute_block_body(&block, vec![element])?;
                    if answer.is_truthy() {
                        return Ok(Some(Object::Int(index as i64)));
                    }
                    index += 1;
                }
                Ok(Some(Object::Nil))
            }
            // Ruby asks each element whether it is `==` to what it was given,
            // in order, so an object that defines `==` decides for itself.
            // An Array is already the array these ask for.
            "to_ary" | "deconstruct" => {
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
            "include?" | "contains?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = array_rc.borrow().clone();
                for element in elements {
                    if self.elements_equal(&element, &arguments[0], position)? {
                        return Ok(Some(Object::Bool(true)));
                    }
                }
                Ok(Some(Object::Bool(false)))
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
            "any?" | "all?" | "none?" | "one?" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // With a pattern argument each element is tested with
                // `pattern === element` and any block is ignored.
                let pattern = arguments.first().cloned();
                let block = self.pending_block.take();
                let array = array_rc.borrow();
                let truthy = |v: &Object| !matches!(v, Object::Bool(false) | Object::Nil);
                let mut any_true = false;
                let mut all_true = true;
                let mut true_count = 0usize;
                for element in array.iter() {
                    let result = match (&pattern, &block) {
                        (Some(pattern), _) => self.evaluate_binary_operation(
                            &crate::ast::BinaryOp::CaseEqual,
                            pattern.clone(),
                            element.clone(),
                            position,
                        )?,
                        (None, Some(Object::Block(b))) => {
                            let args = vec![element.clone()];
                            self.execute_block_body(b, args)?
                        }
                        _ => element.clone(),
                    };
                    if truthy(&result) {
                        any_true = true;
                        true_count += 1;
                    } else {
                        all_true = false;
                    }
                }
                let value = match method_name {
                    "any?" => any_true,
                    "all?" => array.is_empty() || all_true,
                    "none?" => !any_true,
                    "one?" => true_count == 1,
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

/// Render array elements the way `Array#inspect` does, recursing so a nested
/// array's own strings and symbols keep their quoting.
fn inspect_elements(elements: &[Object]) -> String {
    let parts: Vec<String> = elements.iter().map(inspect_element).collect();
    format!("[{}]", parts.join(", "))
}

pub(crate) fn inspect_element(element: &Object) -> String {
    match element {
        Object::String(s) => format!("{:?}", s.as_str()),
        Object::Symbol(s) => format!(":{}", s.as_str()),
        Object::Nil => "nil".to_string(),
        Object::Array(nested) => inspect_elements(&nested.borrow()),
        other => other.to_string(),
    }
}

/// One index into an array, counted from the end when negative, and None when
/// it names no element at all.
fn normalize_index(value: &Object, length: i64) -> Option<usize> {
    let index = match value {
        Object::Int(index) => *index,
        Object::Float(index) => *index as i64,
        _ => return None,
    };
    let index = if index < 0 { index + length } else { index };
    if index < 0 || index >= length {
        return None;
    }
    Some(index as usize)
}

impl VirtualMachine {
    /// Whether an element equals what a search was given. Ruby sends `==` to
    /// the element, so an object of the program's own making answers.
    pub(crate) fn elements_equal(
        &mut self,
        element: &Object,
        candidate: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if matches!(element, Object::Instance(_)) || matches!(candidate, Object::Instance(_)) {
            let answer = self.evaluate_binary_operation(
                &crate::ast::BinaryOp::Equal,
                element.clone(),
                candidate.clone(),
                position,
            )?;
            return Ok(answer.is_truthy());
        }
        Ok(element.equals(candidate))
    }
}

impl VirtualMachine {
    /// Ruby warns when a method was handed both an argument and a block and
    /// the argument is the one it uses.
    pub(crate) fn warn_unused_block(&mut self, position: Position) -> Result<(), MetorexError> {
        if self.pending_block.take().is_none() {
            return Ok(());
        }
        let file = self
            .current_source_file
            .clone()
            .unwrap_or_else(|| "-".to_string());
        let message = format!(
            "{}:{}: warning: given block not used\n",
            file, position.line
        );
        self.warn_through_warning_module(message, position)
    }
}

impl VirtualMachine {
    /// Sort by what a block answers for each pair, which is an insertion sort
    /// so the comparisons run in the order Ruby makes them.
    pub(crate) fn sort_with_block(
        &mut self,
        elements: Vec<Object>,
        block: &Rc<crate::object::BlockStatement>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut sorted: Vec<Object> = Vec::with_capacity(elements.len());
        for element in elements {
            let mut place = sorted.len();
            for (index, other) in sorted.clone().into_iter().enumerate() {
                let answer =
                    self.execute_block_body(block, vec![element.clone(), other.clone()])?;
                let order = match answer {
                    Object::Int(order) => order,
                    Object::Float(order) => order as i64,
                    Object::Nil => {
                        let message = format!(
                            "comparison of {} with {} failed",
                            self.builtins().class_of(&element).name(),
                            self.builtins().class_of(&other).name()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &message,
                            position,
                        ));
                    }
                    _ => 0,
                };
                if order < 0 {
                    place = index;
                    break;
                }
            }
            sorted.insert(place, element);
        }
        Ok(sorted)
    }
}

impl VirtualMachine {
    /// The elements of an argument that stands for an array: one as it is, and
    /// anything else through `to_ary`.
    pub(crate) fn coerce_to_array(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if let Object::Array(elements) = value {
            return Ok(elements.borrow().clone());
        }
        let refuse = |vm: &mut Self| {
            let message = format!(
                "no implicit conversion of {} into Array",
                vm.builtins().class_of(value).name()
            );
            crate::vm::errors::simple_exception("TypeError", &message, position)
        };
        if !self.responds_to(value, "to_ary") {
            return Err(refuse(self));
        }
        match self.send_to_object(value.clone(), "to_ary", vec![], position)? {
            Object::Array(elements) => Ok(elements.borrow().clone()),
            _ => Err(refuse(self)),
        }
    }
}
