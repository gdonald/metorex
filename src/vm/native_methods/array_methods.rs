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
        // A method that would modify the array refuses on a frozen one. The
        // block-taking ones answer an Enumerator first when no block is given,
        // which Ruby allows even on a frozen array.
        let answers_enumerator = matches!(
            method_name,
            "map!"
                | "collect!"
                | "reject!"
                | "select!"
                | "filter!"
                | "keep_if"
                | "delete_if"
                | "sort_by!"
        ) && !matches!(self.pending_block, Some(Object::Block(_)));
        if MUTATORS.contains(&method_name) && !answers_enumerator && self.object_is_frozen(receiver)
        {
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
                let address = Rc::as_ptr(array_rc) as usize;
                // An array that reaches itself prints `[...]` rather than
                // recursing forever, which is what Ruby shows.
                if crate::object::rendering_in_progress(address) {
                    return Ok(Some(Object::string("[...]")));
                }
                crate::object::begin_rendering(address);
                // Each element renders through its own `inspect`, so an
                // object that defines one is shown the way it asks to be.
                let mut parts = Vec::with_capacity(elements.len());
                for element in &elements {
                    let rendered = self.get_inspect_representation(element, position);
                    match rendered {
                        Ok(rendered) => parts.push(rendered),
                        Err(error) => {
                            crate::object::end_rendering();
                            return Err(error);
                        }
                    }
                }
                crate::object::end_rendering();
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
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                };
                let mut break_value: Option<Object> = None;
                // Read one element at a time rather than holding a borrow, so
                // the block may append to the array it is walking, which Ruby
                // allows and the walk then visits.
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let args = vec![element];
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
            "map" | "collect" => {
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
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                };
                let mut results = Vec::new();
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let value = self.execute_block_body(&block, vec![element])?;
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
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                };
                let reject = method_name == "reject";
                let mut results = Vec::new();
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let value = self.execute_block_body(&block, vec![element.clone()])?;
                    let is_truthy = !matches!(value, Object::Bool(false) | Object::Nil);
                    if is_truthy != reject {
                        results.push(element);
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
            // `flatten` walks all the way down by default, or as many levels
            // as the argument names. `flatten!` writes the result back and
            // answers nil when there was nothing nested to flatten.
            "flatten" | "flatten!" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let depth = match arguments.first() {
                    None | Some(Object::Nil) => -1,
                    Some(Object::Int(level)) => *level,
                    Some(other) => self
                        .coerce_integer_argument(other, position)?
                        .try_into()
                        .unwrap_or(-1),
                };
                let elements = array_rc.borrow().clone();
                let mut flat = Vec::new();
                let mut in_flight = vec![Rc::as_ptr(array_rc) as usize];
                flatten_into(&elements, depth, &mut in_flight, &mut flat, position)?;
                if method_name == "flatten" {
                    return Ok(Some(Object::array(flat)));
                }
                let changed = flat.len() != elements.len()
                    || elements
                        .iter()
                        .any(|element| matches!(element, Object::Array(_)));
                *array_rc.borrow_mut() = flat;
                Ok(Some(if changed {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
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
                // Ruby matches on `hash` and `eql?`, so an object that
                // answers `eql?` decides for itself.
                let elements = array_rc.borrow().clone();
                let mut shared = false;
                'outer: for element in &elements {
                    for candidate in &other {
                        // The same object is shared whatever its `eql?` says,
                        // which is what Ruby's hash lookup amounts to.
                        if identical(element, candidate)
                            || self.values_eql(element, candidate, position)?
                        {
                            shared = true;
                            break 'outer;
                        }
                    }
                }
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
            "uniq" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let elements = array_rc.borrow().clone();
                let unique = self.unique_elements(&elements, block, position)?;
                Ok(Some(Object::array(unique)))
            }
            // `min`, `max`, and `minmax` order with `<=>`, or with the block
            // when one is given.
            "min" | "max" | "minmax" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let elements = array_rc.borrow().clone();
                if elements.is_empty() {
                    return Ok(Some(match method_name {
                        "minmax" => Object::array(vec![Object::Nil, Object::Nil]),
                        _ => Object::Nil,
                    }));
                }
                let mut smallest = elements[0].clone();
                let mut largest = elements[0].clone();
                for element in &elements[1..] {
                    if self.compare_elements(element, &smallest, &block, position)? < 0 {
                        smallest = element.clone();
                    }
                    if self.compare_elements(element, &largest, &block, position)? > 0 {
                        largest = element.clone();
                    }
                }
                Ok(Some(match method_name {
                    "min" => smallest,
                    "max" => largest,
                    _ => Object::array(vec![smallest, largest]),
                }))
            }
            // `count` answers how many elements there are, how many equal the
            // argument, or how many the block answers for.
            "count" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Some(wanted) = arguments.first() {
                    self.warn_unused_block(position)?;
                    let elements = array_rc.borrow().clone();
                    let mut counted = 0;
                    for element in &elements {
                        if self.elements_equal(element, wanted, position)? {
                            counted += 1;
                        }
                    }
                    return Ok(Some(Object::Int(counted)));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return Ok(Some(Object::Int(array_rc.borrow().len() as i64)));
                };
                let mut counted = 0;
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let verdict = self.execute_block_callable(&block, vec![element], position)?;
                    if verdict.is_truthy() {
                        counted += 1;
                    }
                }
                Ok(Some(Object::Int(counted)))
            }
            // `take_while` keeps the leading run the block answers for, and
            // `drop_while` answers what is left after it.
            "take_while" | "drop_while" => {
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
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                let mut taken = Vec::new();
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    let verdict =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
                    if !verdict.is_truthy() {
                        break;
                    }
                    taken.push(element);
                    index += 1;
                }
                if method_name == "take_while" {
                    return Ok(Some(Object::array(taken)));
                }
                let rest = array_rc.borrow()[index.min(array_rc.borrow().len())..].to_vec();
                Ok(Some(Object::array(rest)))
            }
            // `at` is `[]` with a single index, and nothing else.
            "at" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let index: i64 = match &arguments[0] {
                    Object::Int(index) => *index,
                    Object::Float(index) => *index as i64,
                    other => self
                        .coerce_integer_argument(other, position)?
                        .try_into()
                        .unwrap_or(i64::MAX),
                };
                let array = array_rc.borrow();
                let length = array.len() as i64;
                let resolved = if index < 0 { index + length } else { index };
                if resolved < 0 || resolved >= length {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(array[resolved as usize].clone()))
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
            // `to_a` and `entries` answer the array itself, which is what
            // Ruby returns for an Array that is not a subclass instance.
            "to_a" | "entries" => {
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
            // The in-place variants. Each writes the new elements back into
            // the receiver; the ones Ruby documents as answering nil when
            // nothing changed do so here too.
            "compact!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let kept: Vec<Object> = array_rc
                    .borrow()
                    .iter()
                    .filter(|element| !matches!(element, Object::Nil))
                    .cloned()
                    .collect();
                let changed = kept.len() != array_rc.borrow().len();
                *array_rc.borrow_mut() = kept;
                Ok(Some(if changed {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
            }
            "reverse!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                array_rc.borrow_mut().reverse();
                Ok(Some(receiver.clone()))
            }
            "sort!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut sorted = array_rc.borrow().clone();
                match block {
                    None => sorted.sort_by(compare_for_sort),
                    Some(block) => sorted = self.sort_with_block(sorted, &block, position)?,
                }
                *array_rc.borrow_mut() = sorted;
                Ok(Some(receiver.clone()))
            }
            "sort_by!" => {
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
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                let mut keyed: Vec<(Object, Object)> = Vec::new();
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let key =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
                    keyed.push((key, element));
                }
                keyed.sort_by(|left, right| compare_for_sort(&left.0, &right.0));
                *array_rc.borrow_mut() = keyed.into_iter().map(|(_, element)| element).collect();
                Ok(Some(receiver.clone()))
            }
            "map!" | "collect!" => {
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
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                // Each element is replaced as the block answers for it, so an
                // array left behind by a break or an exception holds the
                // results so far and the originals after them.
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    let value = self.execute_block_body(&block, vec![element])?;
                    array_rc.borrow_mut()[index] = value;
                    index += 1;
                }
                Ok(Some(receiver.clone()))
            }
            "reject!" | "select!" | "filter!" => {
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
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                let rejecting = method_name == "reject!";
                let before = array_rc.borrow().len();
                let kept = self.filter_in_place(array_rc, &block, rejecting, position)?;
                Ok(Some(if kept != before {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
            }
            "uniq!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let elements = array_rc.borrow().clone();
                let unique = self.unique_elements(&elements, block, position)?;
                let changed = unique.len() != elements.len();
                *array_rc.borrow_mut() = unique;
                Ok(Some(if changed {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
            }
            "rotate" | "rotate!" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let by = match arguments.first() {
                    None => 1,
                    Some(Object::Int(count)) => *count,
                    Some(other) => {
                        let coerced = self.coerce_integer_argument(other, position)?;
                        coerced.try_into().unwrap_or(0)
                    }
                };
                let elements = array_rc.borrow().clone();
                let rotated = rotate_elements(&elements, by);
                if method_name == "rotate!" {
                    *array_rc.borrow_mut() = rotated;
                    return Ok(Some(receiver.clone()));
                }
                Ok(Some(Object::array(rotated)))
            }
            "shuffle" | "shuffle!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut elements = array_rc.borrow().clone();
                for index in (1..elements.len()).rev() {
                    let swap = self.next_random_int(index as i64 + 1) as usize;
                    elements.swap(index, swap);
                }
                if method_name == "shuffle!" {
                    *array_rc.borrow_mut() = elements;
                    return Ok(Some(receiver.clone()));
                }
                Ok(Some(Object::array(elements)))
            }
            "sample" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = array_rc.borrow().clone();
                if elements.is_empty() {
                    return Ok(Some(Object::Nil));
                }
                let index = self.next_random_int(elements.len() as i64) as usize;
                Ok(Some(elements[index].clone()))
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

/// `inspect` for a nested array, which prints `[...]` when the array reaches
/// itself rather than recursing forever.
fn inspect_nested(nested: &Rc<RefCell<Vec<Object>>>) -> String {
    let elements = nested.borrow().clone();
    crate::object::render_guarded(Rc::as_ptr(nested) as usize, || inspect_elements(&elements))
        .unwrap_or_else(|| "[...]".to_string())
}

pub(crate) fn inspect_element(element: &Object) -> String {
    match element {
        Object::String(s) => format!("{:?}", s.as_str()),
        Object::Symbol(s) => format!(":{}", s.as_str()),
        Object::Nil => "nil".to_string(),
        Object::Array(nested) => inspect_nested(nested),
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
    /// Walk `array_rc` and keep the elements the block answers for, writing
    /// the result back as it goes. An element the block raises on is kept,
    /// along with everything it has not reached yet, which is what Ruby
    /// leaves behind. Answers how many elements survived.
    fn filter_in_place(
        &mut self,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        block: &Rc<crate::object::BlockStatement>,
        rejecting: bool,
        position: Position,
    ) -> Result<usize, MetorexError> {
        let mut kept: Vec<Object> = Vec::new();
        let mut index = 0;
        while let Some(element) = element_at(array_rc, index) {
            index += 1;
            let verdict = match self.execute_block_callable(block, vec![element.clone()], position)
            {
                Ok(verdict) => verdict,
                Err(error) => {
                    let mut remaining = kept;
                    remaining.push(element);
                    let tail = array_rc.borrow()[index..].to_vec();
                    remaining.extend(tail);
                    *array_rc.borrow_mut() = remaining;
                    return Err(error);
                }
            };
            if verdict.is_truthy() != rejecting {
                kept.push(element);
            }
        }
        let survived = kept.len();
        *array_rc.borrow_mut() = kept;
        Ok(survived)
    }

    /// Order two elements with `<=>`, or with the block when one is given.
    /// A comparison that answers nil is an ArgumentError, which is what Ruby
    /// raises when the two cannot be ordered.
    fn compare_elements(
        &mut self,
        left: &Object,
        right: &Object,
        block: &Option<Rc<crate::object::BlockStatement>>,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let answer = match block {
            Some(block) => {
                self.execute_block_callable(block, vec![left.clone(), right.clone()], position)?
            }
            None => self.send_to_object(left.clone(), "<=>", vec![right.clone()], position)?,
        };
        match answer {
            Object::Int(order) => Ok(order),
            Object::Float(order) => Ok(order as i64),
            _ => {
                let message = format!(
                    "comparison of {} with {} failed",
                    self.builtins().class_of(left).ruby_name(),
                    self.builtins().class_of(right).ruby_name()
                );
                Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    &message,
                    position,
                ))
            }
        }
    }

    /// The elements of `elements` with later duplicates dropped. A block
    /// decides what counts as a duplicate by naming a key for each element.
    pub(crate) fn unique_elements(
        &mut self,
        elements: &[Object],
        block: Option<Rc<crate::object::BlockStatement>>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut seen: Vec<String> = Vec::new();
        let mut unique = Vec::new();
        for element in elements {
            let key = match &block {
                None => element.clone(),
                Some(block) => {
                    self.execute_block_callable(block, vec![element.clone()], position)?
                }
            };
            let rendered = format!("{}", key);
            if !seen.contains(&rendered) {
                seen.push(rendered);
                unique.push(element.clone());
            }
        }
        Ok(unique)
    }

    /// Whether two values are `eql?`, which is stricter than `==`: 1 and 1.0
    /// are equal but not eql, and an object of its own decides by answering
    /// `eql?` itself.
    pub(crate) fn values_eql(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let mut in_flight = Vec::new();
        self.values_eql_within(left, right, &mut in_flight, position)
    }

    /// `values_eql` with the pairs already being compared, so a pair of arrays
    /// that reach themselves is taken as equal rather than recursing forever.
    fn values_eql_within(
        &mut self,
        left: &Object,
        right: &Object,
        in_flight: &mut Vec<(usize, usize)>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        // An object of the program's own making answers `eql?` itself. Only
        // the receiver decides, so a mock on the right of an Array is never
        // asked, which is what Ruby does.
        if matches!(left, Object::Instance(_))
            && crate::vm::native_methods::array_subclass_value(left).is_none()
        {
            if let Some((class, method)) = self.lookup_method(left, "eql?")
                && !method.is_undefined
                && !method.body.is_empty()
            {
                let answer =
                    self.invoke_method(class, method, left.clone(), vec![right.clone()], position)?;
                return Ok(answer.is_truthy());
            }
            // Object's own `eql?` is identity, which is where a class that
            // defines none lands.
            return Ok(identical(left, right));
        }
        let left = &elements_of(left).unwrap_or_else(|| left.clone());
        let right = &elements_of(right).unwrap_or_else(|| right.clone());
        match (left, right) {
            (Object::Array(_), Object::Array(_)) => {}
            (Object::Array(_), _) | (_, Object::Array(_)) => return Ok(false),
            _ => {}
        }
        if let (Object::Array(left_elements), Object::Array(right_elements)) = (left, right) {
            let pair = (
                Rc::as_ptr(left_elements) as usize,
                Rc::as_ptr(right_elements) as usize,
            );
            if pair.0 == pair.1 || in_flight.contains(&pair) {
                return Ok(true);
            }
            let left_elements = left_elements.borrow().clone();
            let right_elements = right_elements.borrow().clone();
            if left_elements.len() != right_elements.len() {
                return Ok(false);
            }
            in_flight.push(pair);
            for (one, other) in left_elements.iter().zip(right_elements.iter()) {
                if !self.values_eql_within(one, other, in_flight, position)? {
                    in_flight.pop();
                    return Ok(false);
                }
            }
            in_flight.pop();
            return Ok(true);
        }
        let same_kind = match (left, right) {
            (Object::Float(_), other) => matches!(other, Object::Float(_)),
            (Object::Int(_) | Object::BigInt(_), other) => {
                matches!(other, Object::Int(_) | Object::BigInt(_))
            }
            _ => true,
        };
        Ok(same_kind && left.equals(right))
    }

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
        // An instance of an Array subclass already is an Array, so its
        // elements are taken directly rather than through `to_ary`.
        if let Some(Object::Array(elements)) =
            crate::vm::native_methods::array_subclass_value(value)
        {
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

/// Copy `elements` into `flat`, descending into nested arrays until `depth`
/// levels have been unwrapped. A negative depth descends all the way.
/// `in_flight` holds the arrays being walked, so an array that contains
/// itself raises rather than recursing forever.
fn flatten_into(
    elements: &[Object],
    depth: i64,
    in_flight: &mut Vec<usize>,
    flat: &mut Vec<Object>,
    position: Position,
) -> Result<(), MetorexError> {
    for element in elements {
        match element {
            Object::Array(nested) if depth != 0 => {
                let address = Rc::as_ptr(nested) as usize;
                if in_flight.contains(&address) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "tries to flatten self",
                        position,
                    ));
                }
                let inner = nested.borrow().clone();
                in_flight.push(address);
                flatten_into(&inner, depth - 1, in_flight, flat, position)?;
                in_flight.pop();
            }
            other => flat.push(other.clone()),
        }
    }
    Ok(())
}

/// `rotate` moves the first `by` elements to the end, counting from the end
/// when negative. An empty array rotates to itself.
fn rotate_elements(elements: &[Object], by: i64) -> Vec<Object> {
    if elements.is_empty() {
        return Vec::new();
    }
    let length = elements.len() as i64;
    let offset = by.rem_euclid(length) as usize;
    let mut rotated = elements[offset..].to_vec();
    rotated.extend_from_slice(&elements[..offset]);
    rotated
}

/// One element of an array, read without holding a borrow, so the block a
/// walk is running may modify the array it is walking.
fn element_at(array_rc: &Rc<RefCell<Vec<Object>>>, index: usize) -> Option<Object> {
    array_rc.borrow().get(index).cloned()
}

/// The backing array of an instance of an Array subclass, so it compares the
/// way the Array it stands for does.
fn elements_of(value: &Object) -> Option<Object> {
    crate::vm::native_methods::array_subclass_value(value)
}

/// Whether two values are the same object, which is what `equal?` reports.
fn identical(left: &Object, right: &Object) -> bool {
    match (left, right) {
        (Object::Instance(one), Object::Instance(other)) => Rc::ptr_eq(one, other),
        (Object::Array(one), Object::Array(other)) => Rc::ptr_eq(one, other),
        (Object::Dict(one), Object::Dict(other)) => Rc::ptr_eq(one, other),
        _ => false,
    }
}
