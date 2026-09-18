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
        (Object::String(x), Object::String(y)) => x.as_str().cmp(&y.as_str()),
        _ => a.to_string().cmp(&b.to_string()),
    }
}

impl VirtualMachine {
    /// Execute native methods for the Array class.
    /// The pieces an array joins from, each with the encoding it is written
    /// in. An element that names an array of its own contributes its own
    /// pieces however deeply they nest, and any other element is asked for
    /// `to_str`, then `to_ary`, then `to_s`, which is the order Ruby tries.
    /// An array that reaches itself is refused.
    fn joined_parts(
        &mut self,
        elements: &[Object],
        in_flight: &mut Vec<usize>,
        position: Position,
    ) -> Result<Vec<(String, String)>, MetorexError> {
        let mut written = Vec::with_capacity(elements.len());
        for element in elements {
            self.join_element_into(element, in_flight, &mut written, position)?;
        }
        Ok(written)
    }

    /// One element's contribution to a join, added to what is written so far.
    fn join_element_into(
        &mut self,
        element: &Object,
        in_flight: &mut Vec<usize>,
        written: &mut Vec<(String, String)>,
        position: Position,
    ) -> Result<(), MetorexError> {
        match element {
            Object::Symbol(name) | Object::String(name) => {
                written.push((name.as_str().to_string(), name.encoding_name()));
                return Ok(());
            }
            _ => {}
        }
        let nested = match element {
            Object::Array(held) => Some(Rc::clone(held)),
            other => match crate::vm::native_methods::array_subclass_value(other) {
                Some(Object::Array(held)) => Some(held),
                _ => None,
            },
        };
        if let Some(held) = nested {
            let address = Rc::as_ptr(&held) as usize;
            if in_flight.contains(&address) {
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "recursive array join",
                    position,
                ));
            }
            in_flight.push(address);
            let inner = held.borrow().clone();
            let joined = self.joined_parts(&inner, in_flight, position);
            in_flight.pop();
            written.extend(joined?);
            return Ok(());
        }
        for name in ["to_str", "to_ary", "to_s"] {
            if !self.responds_to(element, name) {
                continue;
            }
            let answered = self.send_to_object(element.clone(), name, vec![], position)?;
            // Only an array answered by `to_ary` joins as one. Anything else
            // it answers leaves the element for `to_s` to spell.
            if name == "to_ary" {
                if matches!(&answered, Object::Array(_))
                    || crate::vm::native_methods::array_subclass_value(&answered).is_some()
                {
                    return self.join_element_into(&answered, in_flight, written, position);
                }
                continue;
            }
            if let Object::String(text) = answered {
                written.push((text.as_str().to_string(), text.encoding_name()));
                return Ok(());
            }
        }
        Err(crate::vm::errors::simple_exception(
            "NoMethodError",
            &format!(
                "undefined method 'to_str' for an instance of {}",
                self.builtins().class_of(element).name()
            ),
            position,
        ))
    }

    /// The encoding a join writes its answer in: the one the first piece is
    /// written in, widened by the first piece that is not all ASCII. Two
    /// pieces that are each written in an encoding of their own and neither
    /// of which is ASCII cannot be joined at all.
    fn joined_encoding(
        &mut self,
        parts: &[(String, String)],
        position: Position,
    ) -> Result<String, MetorexError> {
        let mut writing = "US-ASCII".to_string();
        let mut all_ascii = true;
        for (at, (text, held)) in parts.iter().enumerate() {
            if at == 0 {
                writing = held.clone();
                all_ascii = text.is_ascii();
                continue;
            }
            if text.is_ascii() {
                continue;
            }
            if all_ascii {
                writing = held.clone();
                all_ascii = false;
                continue;
            }
            if held != &writing {
                let message = format!("incompatible character encodings: {} and {}", writing, held);
                return Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &message,
                    position,
                ));
            }
        }
        Ok(writing)
    }

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
            "initialize",
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
                // An array with nothing in it spells the same in every
                // encoding, and one with something in it is written in the
                // encoding the first element was rendered in.
                let mut writing = "US-ASCII".to_string();
                for element in &elements {
                    let rendered = self.inspected_object(element, position);
                    match rendered {
                        Ok(rendered) => {
                            if parts.is_empty()
                                && let Object::String(text) = &rendered
                            {
                                writing = text.encoding_name();
                            }
                            parts.push(match &rendered {
                                Object::String(text) => text.to_string(),
                                other => format!("{}", other),
                            });
                        }
                        Err(error) => {
                            crate::object::end_rendering();
                            return Err(error);
                        }
                    }
                }
                crate::object::end_rendering();
                let made = crate::object::StringValue::with_encoding(
                    format!("[{}]", parts.join(", ")),
                    writing,
                );
                Ok(Some(Object::String(Rc::new(made))))
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
            // `ary.send :initialize, ...` builds the elements the way
            // `Array.new` does and puts them in place of what is there.
            "initialize" => {
                // A `break` out of the block leaves the array holding what
                // the block had answered up to then, so the elements are put
                // in place whether the run finished or not.
                let mut elements = Vec::new();
                let outcome = self.collect_array_elements(arguments, position, &mut elements);
                if outcome.is_ok() || !elements.is_empty() {
                    let mut held = array_rc.borrow_mut();
                    held.clear();
                    held.extend(elements);
                }
                outcome?;
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
                let other = self.coerce_to_array(&arguments[0], position)?;
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
                        .execute_block_callable(&block, vec![arguments[0].clone()], position)
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
            // `pop` and `shift` take an optional count, and then answer an
            // array of what they removed rather than a single element.
            "pop" | "shift" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let from_front = method_name == "shift";
                let Some(count) = arguments.first() else {
                    let mut array = array_rc.borrow_mut();
                    if array.is_empty() {
                        return Ok(Some(Object::Nil));
                    }
                    return Ok(Some(if from_front {
                        array.remove(0)
                    } else {
                        array.pop().unwrap_or(Object::Nil)
                    }));
                };
                let wanted: i64 = self
                    .coerce_integer_argument(count, position)?
                    .try_into()
                    .unwrap_or(i64::MAX);
                if wanted < 0 {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative array size",
                        position,
                    ));
                }
                let mut array = array_rc.borrow_mut();
                let taken = (wanted as usize).min(array.len());
                let kept = array.len() - taken;
                let removed: Vec<Object> = if from_front {
                    array.drain(..taken).collect()
                } else {
                    array.split_off(kept)
                };
                Ok(Some(Object::array(removed)))
            }
            "[]" | "slice" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let total = array_rc.borrow().len() as i64;
                if arguments.len() == 2 {
                    let start = self.machine_index(&arguments[0], total, position)?;
                    let span = self.machine_index(&arguments[1], 0, position)?;
                    let array = array_rc.borrow();
                    if start < 0 || start > total || span < 0 {
                        return Ok(Some(Object::Nil));
                    }
                    let end = start.saturating_add(span).min(total);
                    return Ok(Some(Object::array(
                        array[start as usize..end as usize].to_vec(),
                    )));
                }
                // An arithmetic sequence names a strided run of the array.
                if let Some(held) = self.sequence_slice(array_rc, &arguments[0], total, position)? {
                    return Ok(Some(held));
                }
                // A Range slices, and each bound goes through `to_int` too.
                if let Some(span) = crate::vm::native_methods::as_range(&arguments[0]) {
                    let (start, span) = self.range_bounds(&span, total, position)?;
                    if start < 0 || start > total {
                        return Ok(Some(Object::Nil));
                    }
                    let array = array_rc.borrow();
                    let end = start.saturating_add(span).min(total);
                    return Ok(Some(Object::array(
                        array[start as usize..end.max(start) as usize].to_vec(),
                    )));
                }
                let index = self.machine_index(&arguments[0], total, position)?;
                let array = array_rc.borrow();
                if index < 0 || index >= total {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(array[index as usize].clone()))
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
                    match self.execute_block_with_control_flow(&block, args, position)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Retry { .. }
                        | super::super::ControlFlow::Continue { .. } => continue,
                        super::super::ControlFlow::Break { value, .. } => {
                            return Ok(Some(value));
                        }
                        super::super::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: super::super::utils::position_to_location(position),
                                home_frame: block.home_frame,
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
                    match self.execute_block_with_control_flow(&block, args, position)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Retry { .. }
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
                                home_frame: block.home_frame,
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
                    let value = self.execute_block_callable(&block, vec![element], position)?;
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
                    let value =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
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
                // Without a block the walk leaves the last match where it
                // found it, so a `$~` set before the call still reads the
                // same afterwards.
                let saved_match = self
                    .globals()
                    .get(crate::vm::native_methods::LAST_MATCH)
                    .unwrap_or(Object::Nil);
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
                        Some(block) => {
                            self.execute_block_callable(block, vec![element], position)?
                        }
                        None => element,
                    });
                }
                if block.is_none() {
                    self.globals_mut()
                        .set(crate::vm::native_methods::LAST_MATCH, saved_match);
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            // The first element the block accepts. When nothing matches, the
            // ifnone argument is called for the answer, and a missing one
            // makes it nil.
            "find" | "detect" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let ifnone = match arguments.first() {
                    Some(Object::Nil) | None => None,
                    Some(other) => Some(other.clone()),
                };
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
                    let value =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
                    if !matches!(value, Object::Bool(false) | Object::Nil) {
                        return Ok(Some(element));
                    }
                }
                match ifnone {
                    Some(ifnone) => self
                        .send_to_object(ifnone, "call", vec![], position)
                        .map(Some),
                    None => Ok(Some(Object::Nil)),
                }
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
                    let value = self.execute_block_callable(&block, args, position)?;
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
            "inject" | "reduce" => {
                if arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                // The last argument names an operator when there are two of
                // them, or when there is one and no block. The rest is the
                // starting value.
                let names_operator =
                    arguments.len() == 2 || (arguments.len() == 1 && block.is_none());
                let operator = match arguments.last().filter(|_| names_operator) {
                    Some(name) => Some(self.coerce_method_name(name, method_name, position)?),
                    None => None,
                };
                if operator.is_some() && arguments.len() == 2 && block.is_some() {
                    let warning =
                        format!("{}given block not used", self.warning_prefix(0, position));
                    self.emit_warning_to_stderr(&warning, position);
                }
                if operator.is_none() && block.is_none() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let seeded = arguments.len() == 2 || (arguments.len() == 1 && operator.is_none());
                let mut accumulator = if seeded {
                    Some(arguments[0].clone())
                } else {
                    array_rc.borrow().first().cloned()
                };
                // The walk reads one element at a time, so a block that grows
                // the array reaches what it added.
                let mut index = if seeded { 0 } else { 1 };
                loop {
                    let Some(element) = array_rc.borrow().get(index).cloned() else {
                        break;
                    };
                    index += 1;
                    let carried = accumulator.clone().unwrap_or(Object::Nil);
                    accumulator = Some(match (&operator, &block) {
                        (Some(operator), _) => {
                            self.send_to_object(carried, operator, vec![element], position)?
                        }
                        (None, Some(block)) => {
                            self.execute_block_callable(block, vec![carried, element], position)?
                        }
                        (None, None) => unreachable!("a block or an operator was required"),
                    });
                }
                Ok(Some(accumulator.unwrap_or(Object::Nil)))
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
                let held = array_rc.borrow().clone();
                let wanted = held.len();
                // Each argument is put through `to_ary`, and one that answers
                // none is walked with `each`, taking only as many values as
                // this array holds so an endless walk still ends.
                let mut columns = Vec::new();
                for argument in arguments {
                    columns.push(self.zip_column(argument, wanted, position)?);
                }
                let mut results = Vec::new();
                for (index, element) in held.iter().enumerate() {
                    let mut tuple = vec![element.clone()];
                    for column in &columns {
                        tuple.push(column.get(index).cloned().unwrap_or(Object::Nil));
                    }
                    results.push(Object::Array(Rc::new(RefCell::new(tuple))));
                }
                // With a block the rows are handed over one at a time and
                // nothing is answered.
                if let Some(Object::Block(block)) = self.pending_block.take() {
                    for row in results {
                        self.execute_block_callable(&block, vec![row], position)?;
                    }
                    return Ok(Some(Object::Nil));
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

                let elements = array.clone();
                drop(array);
                // Each row goes through `to_ary`, so an object standing for an
                // array transposes the way one does.
                let mut row_arrays = Vec::new();
                for element in &elements {
                    row_arrays.push(self.coerce_to_array(element, position)?);
                }

                // Every row must be the same length, which Ruby reports as an
                // IndexError naming the two it found.
                let max_cols = row_arrays.first().map(|row| row.len()).unwrap_or(0);
                if let Some(odd) = row_arrays.iter().find(|row| row.len() != max_cols) {
                    let message = format!(
                        "element size differs ({} should be {})",
                        odd.len(),
                        max_cols
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "IndexError",
                        &message,
                        position,
                    ));
                }

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
                let elements = array_rc.borrow().clone();
                let sorted = self.sort_elements(elements, block, position)?;
                Ok(Some(Object::array(sorted)))
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
                    let key =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
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
            // `ary * other` joins with a separator when the other object
            // names one, and repeats the array when it names a count. Ruby
            // asks for the separator first.
            "*" => {
                if arguments.len() != 1 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Exact(1),
                        arguments.len(),
                        position,
                    ));
                }
                let given = arguments[0].clone();
                if matches!(given, Object::String(_)) {
                    return self.call_array_method(receiver, "join", &[given], position);
                }
                if self.responds_to(&given, "to_str") {
                    let separator = self.send_to_object(given, "to_str", vec![], position)?;
                    return self.call_array_method(receiver, "join", &[separator], position);
                }
                let count = match given {
                    Object::Int(_) => given,
                    other if self.responds_to(&other, "to_int") => {
                        self.send_to_object(other, "to_int", vec![], position)?
                    }
                    other => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &format!(
                                "no implicit conversion of {} into Integer",
                                self.builtins().class_of(&other).name()
                            ),
                            position,
                        ));
                    }
                };
                let Object::Int(count) = count else {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "can't convert to Integer",
                        position,
                    ));
                };
                let Ok(count) = usize::try_from(count) else {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative argument",
                        position,
                    ));
                };
                let source = array_rc.borrow().clone();
                let mut repeated = Vec::with_capacity(source.len() * count);
                for _ in 0..count {
                    repeated.extend(source.iter().cloned());
                }
                Ok(Some(Object::array(repeated)))
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
                // With no separator, or with nil for one, `$,` says what
                // goes between. Reading it warns, since a later release will
                // drop it.
                let default_separator = |vm: &mut Self| -> String {
                    match vm.globals().get(",") {
                        Some(Object::String(held)) => {
                            let text = held.as_str().to_string();
                            vm.emit_warning_to_stderr(
                                "warning: $, is set to non-nil value",
                                position,
                            );
                            text
                        }
                        _ => String::new(),
                    }
                };
                let sep = if arguments.is_empty() || matches!(arguments[0], Object::Nil) {
                    default_separator(self)
                } else {
                    // An empty array never asks the separator for anything,
                    // which is what Ruby does.
                    let given = if array_rc.borrow().is_empty() {
                        Object::Nil
                    } else if matches!(arguments[0], Object::String(_) | Object::Nil) {
                        arguments[0].clone()
                    } else if self.responds_to(&arguments[0], "to_str") {
                        self.send_to_object(arguments[0].clone(), "to_str", vec![], position)?
                    } else {
                        arguments[0].clone()
                    };
                    match &given {
                        Object::String(s) => s.as_str().to_string(),
                        // A nil separator joins with nothing between.
                        Object::Nil => String::new(),
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
                // Each element joins as its `to_s`, so a Symbol contributes
                // the name it is spelled with rather than the leading colon.
                // An element that is itself an array is joined with the same
                // separator, however deeply they nest.
                let elements = array_rc.borrow().clone();
                let mut in_flight = vec![Rc::as_ptr(array_rc) as usize];
                let parts = self.joined_parts(&elements, &mut in_flight, position)?;
                let writing = self.joined_encoding(&parts, position)?;
                let text = parts
                    .iter()
                    .map(|(written, _)| written.as_str())
                    .collect::<Vec<&str>>()
                    .join(&sep);
                let made = crate::object::StringValue::with_encoding(text, writing.clone());
                // A run of bytes joined with others is still a run of bytes,
                // so the result says so rather than reading them as
                // characters.
                if matches!(writing.as_str(), "ASCII-8BIT" | "BINARY") {
                    made.mark_bytes();
                }
                Ok(Some(Object::String(Rc::new(made))))
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
                self.flatten_into(&elements, depth, &mut in_flight, &mut flat, position)?;
                if method_name == "flatten" {
                    return Ok(Some(Object::array(flat)));
                }
                // Told to flatten nothing, the array is left as it was, which
                // is what `flatten!` reports by answering nil.
                let changed = depth != 0
                    && (flat.len() != elements.len()
                        || elements
                            .iter()
                            .any(|element| matches!(element, Object::Array(_))));
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
                // A count too large for a machine word is a RangeError, which
                // is what Ruby raises before it looks at the array at all.
                let Ok(count) = i64::try_from(&count) else {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "bignum too big to convert into `long'",
                        position,
                    ));
                };
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
                    let answer = self.execute_block_callable(&block, vec![element], position)?;
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
            // `slice!` cuts the part a subscript names out of the array and
            // answers it, leaving the rest in place.
            "slice!" => {
                let size = array_rc.borrow().len();
                let Some((from, width, one)) = self.slice_span(arguments, size, position)? else {
                    return Ok(Some(Object::Nil));
                };
                let taken: Vec<Object> = array_rc.borrow_mut().drain(from..from + width).collect();
                Ok(Some(if one {
                    taken.into_iter().next().unwrap_or(Object::Nil)
                } else {
                    Object::array(taken)
                }))
            }
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
                    match self.execute_block_callable(&block, vec![element.clone()], position) {
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
                    self.execute_block_callable(&block, vec![Object::Int(index as i64)], position)?;
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
                    self.execute_block_callable(&block, vec![element], position)?;
                }
                Ok(Some(receiver.clone()))
            }
            // `values_at` reads several positions at once, answering nil for
            // one the array does not reach.
            // `values_at` takes indexes and Ranges, answering nil for a
            // position the array has no element at.
            "values_at" => {
                let array = array_rc.borrow().clone();
                let length = array.len() as i64;
                let mut picked = Vec::new();
                for argument in arguments {
                    if let Object::Range {
                        start,
                        end,
                        exclusive,
                        ..
                    } = argument
                    {
                        let first = match start.as_ref() {
                            Object::Nil => 0,
                            bound => self.index_from(bound, length, position)?,
                        };
                        // A start before the front names nothing. A start past
                        // the end still fills nil for every position the range
                        // covers, which is what Ruby answers.
                        if first < 0 {
                            continue;
                        }
                        let last = match end.as_ref() {
                            Object::Nil => length - 1,
                            bound => {
                                let resolved = self.index_from(bound, length, position)?;
                                if *exclusive { resolved - 1 } else { resolved }
                            }
                        };
                        for index in first..=last.max(first - 1) {
                            picked.push(array.get(index as usize).cloned().unwrap_or(Object::Nil));
                        }
                        continue;
                    }
                    let index = self.index_from(argument, length, position)?;
                    picked.push(
                        usize::try_from(index)
                            .ok()
                            .and_then(|index| array.get(index).cloned())
                            .unwrap_or(Object::Nil),
                    );
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
                // The row's own element is asked `==`, so an object of the
                // program's own answers, and the row itself comes back rather
                // than a copy of it.
                let slot = usize::from(method_name == "rassoc");
                let rows = array_rc.borrow().clone();
                for element in rows {
                    // A row that is not an Array is asked for one, and what it
                    // answers is what comes back when it matches.
                    let row = match &element {
                        Object::Array(_) => element.clone(),
                        other if self.responds_to(other, "to_ary") => {
                            self.send_to_object(other.clone(), "to_ary", vec![], position)?
                        }
                        _ => continue,
                    };
                    let Object::Array(pair) = &row else {
                        continue;
                    };
                    let candidate = pair.borrow().get(slot).cloned();
                    let Some(candidate) = candidate else {
                        continue;
                    };
                    if self.elements_equal(&candidate, &arguments[0], position)? {
                        return Ok(Some(row));
                    }
                }
                Ok(Some(Object::Nil))
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
                    let answer = self.execute_block_callable(&block, vec![element], position)?;
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
                let unique = self.unique_live_elements(array_rc, block, position)?;
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
                let wants_min = matches!(method_name, "min" | "minmax");
                let wants_max = matches!(method_name, "max" | "minmax");
                let mut smallest = elements[0].clone();
                let mut largest = elements[0].clone();
                for element in &elements[1..] {
                    if wants_min && self.compare_elements(element, &smallest, &block, position)? < 0
                    {
                        smallest = element.clone();
                    }
                    if wants_max && self.compare_elements(element, &largest, &block, position)? > 0
                    {
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
            // `union`, `intersection`, and `difference` fold the matching
            // operator over every argument, so each one is coerced and matched
            // the same way `|`, `&`, and `-` are.
            "union" | "intersection" | "difference" => {
                let operator = match method_name {
                    "union" => crate::ast::BinaryOp::BitwiseOr,
                    "intersection" => crate::ast::BinaryOp::BitwiseAnd,
                    _ => crate::ast::BinaryOp::Subtract,
                };
                // With no arguments each answers a copy, which for `union` and
                // `intersection` has its duplicates dropped.
                let mut folded = if arguments.is_empty() && method_name != "difference" {
                    let elements = array_rc.borrow().clone();
                    let mut unique = Vec::new();
                    for element in &elements {
                        if !self.contains_eql(&unique, element, position)? {
                            unique.push(element.clone());
                        }
                    }
                    Object::array(unique)
                } else {
                    Object::array(array_rc.borrow().clone())
                };
                for argument in arguments {
                    folded = self.evaluate_binary_operation(
                        &operator,
                        folded,
                        argument.clone(),
                        position,
                    )?;
                }
                Ok(Some(folded))
            }
            // `fetch(index)` raises when the index names no element, unless a
            // default value or a block says what to answer instead.
            "fetch" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // The block is set aside first, since coercing the index runs
                // `to_int` on an object of the program's own, which would
                // otherwise take the block for itself.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let index: i64 = self
                    .coerce_integer_argument(&arguments[0], position)?
                    .try_into()
                    .unwrap_or(i64::MAX);
                let length = array_rc.borrow().len() as i64;
                let resolved = if index < 0 { index + length } else { index };
                if resolved >= 0 && resolved < length {
                    if block.is_some() {
                        self.pending_block = None;
                        let file = self
                            .current_source_file
                            .clone()
                            .unwrap_or_else(|| "-".to_string());
                        let message = format!(
                            "{}:{}: warning: given block not used\n",
                            file, position.line
                        );
                        self.warn_through_warning_module(message, position)?;
                    }
                    return Ok(Some(array_rc.borrow()[resolved as usize].clone()));
                }
                // A block wins over a default value, and Ruby says so.
                if let Some(block) = block {
                    if arguments.len() == 2 {
                        let file = self
                            .current_source_file
                            .clone()
                            .unwrap_or_else(|| "-".to_string());
                        let message = format!(
                            "{}:{}: warning: block supersedes default value argument\n",
                            file, position.line
                        );
                        self.warn_through_warning_module(message, position)?;
                    }
                    return self
                        .execute_block_callable(&block, vec![arguments[0].clone()], position)
                        .map(Some);
                }
                if let Some(fallback) = arguments.get(1) {
                    return Ok(Some(fallback.clone()));
                }
                let message = format!(
                    "index {} outside of array bounds: {}...{}",
                    index, -length, length
                );
                Err(crate::vm::errors::simple_exception(
                    "IndexError",
                    &message,
                    position,
                ))
            }
            // `fetch_values(*indexes)` fetches each one, so a missing index
            // raises unless the block says what to answer for it.
            "fetch_values" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut fetched = Vec::with_capacity(arguments.len());
                for wanted in arguments {
                    let index: i64 = self
                        .coerce_integer_argument(wanted, position)?
                        .try_into()
                        .unwrap_or(i64::MAX);
                    let length = array_rc.borrow().len() as i64;
                    let resolved = if index < 0 { index + length } else { index };
                    if resolved >= 0 && resolved < length {
                        fetched.push(array_rc.borrow()[resolved as usize].clone());
                        continue;
                    }
                    let Some(block) = &block else {
                        let message = format!(
                            "index {} outside of array bounds: {}...{}",
                            index, -length, length
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "IndexError",
                            &message,
                            position,
                        ));
                    };
                    let block = Rc::clone(block);
                    fetched.push(self.execute_block_callable(
                        &block,
                        vec![wanted.clone()],
                        position,
                    )?);
                }
                Ok(Some(Object::array(fetched)))
            }
            // `insert(index, *objects)` puts the objects before the element at
            // a non-negative index and after it for a negative one, padding
            // with nil when the index is past the end.
            "insert" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                if arguments.len() == 1 {
                    return Ok(Some(receiver.clone()));
                }
                let index: i64 = self
                    .coerce_integer_argument(&arguments[0], position)?
                    .try_into()
                    .unwrap_or(i64::MAX);
                let mut array = array_rc.borrow_mut();
                let length = array.len() as i64;
                let at = if index < 0 { index + length + 1 } else { index };
                if at < 0 {
                    let message = format!(
                        "index {} too small for array; minimum: {}",
                        index,
                        -length - 1
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "IndexError",
                        &message,
                        position,
                    ));
                }
                while (array.len() as i64) < at {
                    array.push(Object::Nil);
                }
                for (offset, value) in arguments[1..].iter().enumerate() {
                    array.insert(at as usize + offset, value.clone());
                }
                drop(array);
                Ok(Some(receiver.clone()))
            }
            // `[]=` in its three forms: one index, a start and a length, and
            // a Range. The span forms splice, and a value that is an Array
            // contributes its elements.
            "[]=" => {
                if arguments.len() < 2 || arguments.len() > 3 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let value = arguments[arguments.len() - 1].clone();
                let length = array_rc.borrow().len() as i64;
                // An instance of a Range subclass names a span the same way a
                // plain Range does.
                let first = super::as_range(&arguments[0]).unwrap_or_else(|| arguments[0].clone());
                // A Range start before the front is a RangeError, while the
                // other forms report an IndexError.
                let out_of_range_error =
                    if arguments.len() == 2 && matches!(&first, Object::Range { .. }) {
                        "RangeError"
                    } else {
                        "IndexError"
                    };
                let (start, span) = if arguments.len() == 3 {
                    let start = self.index_from(&arguments[0], length, position)?;
                    let span: i64 = self
                        .coerce_integer_argument(&arguments[1], position)?
                        .try_into()
                        .unwrap_or(i64::MAX);
                    if span < 0 {
                        let message = format!("negative length ({})", span);
                        return Err(crate::vm::errors::simple_exception(
                            "IndexError",
                            &message,
                            position,
                        ));
                    }
                    (start, Some(span))
                } else if let Object::Range { .. } = &first {
                    let (start, span) = self.range_bounds(&first, length, position)?;
                    (start, Some(span))
                } else {
                    (self.index_from(&arguments[0], length, position)?, None)
                };
                if start < 0 {
                    let message = format!(
                        "index {} too small for array; minimum: {}",
                        start - length,
                        -length
                    );
                    return Err(crate::vm::errors::simple_exception(
                        out_of_range_error,
                        &message,
                        position,
                    ));
                }
                // The replacement is read before the array is borrowed for
                // writing, since `a[0, 2] = a` names the same array on both
                // sides.
                // A span is filled with the elements of an Array, or with the
                // one value anything else stands for. An object that answers
                // `to_ary` contributes its elements too, while an instance of
                // an Array subclass is taken as the array it already is.
                let replacement = match &span {
                    None => Vec::new(),
                    Some(_) => match &value {
                        Object::Array(elements) => elements.borrow().clone(),
                        other => match crate::vm::native_methods::array_subclass_value(other) {
                            Some(Object::Array(elements)) => elements.borrow().clone(),
                            _ if self.responds_to(other, "to_ary") => {
                                match self.send_to_object(
                                    other.clone(),
                                    "to_ary",
                                    vec![],
                                    position,
                                )? {
                                    Object::Array(elements) => elements.borrow().clone(),
                                    converted => vec![converted],
                                }
                            }
                            _ => vec![other.clone()],
                        },
                    },
                };
                let mut array = array_rc.borrow_mut();
                while (array.len() as i64) < start {
                    array.push(Object::Nil);
                }
                match span {
                    None => {
                        let at = start as usize;
                        if at < array.len() {
                            array[at] = value.clone();
                        } else {
                            array.push(value.clone());
                        }
                    }
                    Some(span) => {
                        let at = start as usize;
                        let taken = (span as usize).min(array.len().saturating_sub(at));
                        array.splice(at..at + taken, replacement);
                    }
                }
                drop(array);
                Ok(Some(value))
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
                // A pattern is what decides the answer, so a block handed in
                // alongside one is unused and Ruby says so.
                if pattern.is_some() {
                    self.warn_unused_block(position)?;
                }
                let block = self.pending_block.take();
                let truthy = |v: &Object| !matches!(v, Object::Bool(false) | Object::Nil);
                let mut any_true = false;
                let mut all_true = true;
                let mut true_count = 0usize;
                let mut was_empty = true;
                // The block is free to grow the array it is walking, so each
                // element is read by index rather than through a borrow held
                // across the call.
                let mut index = 0usize;
                loop {
                    let element = {
                        let array = array_rc.borrow();
                        was_empty = was_empty && array.is_empty();
                        match array.get(index) {
                            Some(element) => element.clone(),
                            None => break,
                        }
                    };
                    index += 1;
                    let element = &element;
                    let result = match (&pattern, &block) {
                        (Some(pattern), _) => self.evaluate_binary_operation(
                            &crate::ast::BinaryOp::CaseEqual,
                            pattern.clone(),
                            element.clone(),
                            position,
                        )?,
                        (None, Some(Object::Block(b))) => {
                            let args = vec![element.clone()];
                            self.execute_block_callable(b, args, position)?
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
                    "all?" => was_empty || all_true,
                    "none?" => !any_true,
                    "one?" => true_count == 1,
                    _ => unreachable!(),
                };
                Ok(Some(Object::Bool(value)))
            }
            // `pack` writes the items out as the directives describe them.
            "pack" => {
                // `buffer:` names a String the result is written into, which
                // is the object the call answers.
                let mut arguments = arguments;
                let mut buffer = None;
                if let Some(Object::Dict(entries)) = arguments.last()
                    && let Some(named) = entries.borrow().get(":buffer").cloned()
                {
                    if !matches!(named, Object::String(_)) {
                        let holds = self.builtins().class_of(&named).ruby_name();
                        return Err(simple_exception(
                            "TypeError",
                            &format!("buffer must be String, not {holds}"),
                            position,
                        ));
                    }
                    buffer = Some(named);
                    arguments = &arguments[..arguments.len() - 1];
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let format = match &arguments[0] {
                    Object::String(format) => format.as_str().to_string(),
                    other if self.responds_to(other, "to_str") => {
                        match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                            Object::String(format) => format.as_str().to_string(),
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "String",
                                    other,
                                    position,
                                ));
                            }
                        }
                    }
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                let items = array_rc.borrow().clone();
                let packed = self.array_pack(&items, &format, position)?;
                let Some(buffer) = buffer else {
                    return Ok(Some(packed));
                };
                self.pack_into_buffer(buffer, &format, &packed, position)
                    .map(Some)
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
            // `to_set` collects the elements into a Set, passing each through
            // the block first when one is given.
            "to_set" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A class given as the first argument builds the set instead
                // of Set itself, which Ruby warns about.
                let set_class = match arguments.first() {
                    Some(named @ Object::Class(_)) => {
                        let file = self
                            .current_source_file
                            .clone()
                            .unwrap_or_else(|| "-".to_string());
                        let message = format!(
                            "{}:{}: warning: passing arguments to Enumerable#to_set is deprecated\n",
                            file, position.line
                        );
                        self.warn_through_warning_module(message, position)?;
                        Some(named.clone())
                    }
                    _ => None,
                };
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut collected = Vec::new();
                for element in array_rc.borrow().clone() {
                    collected.push(match &block {
                        None => element,
                        Some(block) => {
                            self.execute_block_callable(block, vec![element], position)?
                        }
                    });
                }
                let set_class = match set_class {
                    Some(named) => named,
                    None => match self.globals().get("Set") {
                        Some(found) => found,
                        None => return Ok(None),
                    },
                };
                self.send_to_object(set_class, "new", vec![Object::array(collected)], position)
                    .map(Some)
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
                let elements = array_rc.borrow().clone();
                let sorted = self.sort_elements(elements, block, position)?;
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
                    let value = self.execute_block_callable(&block, vec![element], position)?;
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
                let unique = self.unique_live_elements(array_rc, block, position)?;
                let changed = unique.len() != array_rc.borrow().len();
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
                // A `random:` keyword names the object each index is drawn
                // from, which Ruby asks for `rand` rather than drawing itself.
                let mut source = None;
                let mut positional = arguments;
                if let Some(Object::Dict(entries)) = arguments.last()
                    && let Some(named) = entries.borrow().get(":random").cloned()
                {
                    source = Some(named);
                    positional = &arguments[..arguments.len() - 1];
                }
                if !positional.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        positional.len(),
                        position,
                    ));
                }
                let mut elements = array_rc.borrow().clone();
                for index in (1..elements.len()).rev() {
                    let swap =
                        self.random_upto(source.as_ref(), index as i64 + 1, position)? as usize;
                    elements.swap(index, swap);
                }
                if method_name == "shuffle!" {
                    *array_rc.borrow_mut() = elements;
                    return Ok(Some(receiver.clone()));
                }
                Ok(Some(Object::array(elements)))
            }
            "sample" => {
                let (count, source) = self.sample_arguments(method_name, arguments, position)?;
                let mut elements = array_rc.borrow().clone();
                let Some(count) = count else {
                    if elements.is_empty() {
                        return Ok(Some(Object::Nil));
                    }
                    let index =
                        self.random_upto(source.as_ref(), elements.len() as i64, position)?
                            as usize;
                    return Ok(Some(elements[index].clone()));
                };
                if count < 0 {
                    return Err(simple_exception(
                        "ArgumentError",
                        "negative sample number",
                        position,
                    ));
                }
                // Drawing without replacement is a partial Fisher-Yates: each
                // pick is swapped to the front, so the stretch already taken
                // is never drawn from again.
                let wanted = (count as usize).min(elements.len());
                for taken in 0..wanted {
                    let remaining = (elements.len() - taken) as i64;
                    let drawn =
                        taken + self.random_upto(source.as_ref(), remaining, position)? as usize;
                    elements.swap(taken, drawn);
                }
                elements.truncate(wanted);
                Ok(Some(Object::array(elements)))
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
        Object::Symbol(s) => crate::object::inspect_symbol(&s.as_str()),
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

    /// Sort by insertion, ordering each pair with the block when one is given
    /// and with `<=>` otherwise. Insertion keeps the comparisons in the order
    /// Ruby makes them, which a block that records them can observe.
    fn sort_elements(
        &mut self,
        elements: Vec<Object>,
        block: Option<Rc<crate::object::BlockStatement>>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut sorted: Vec<Object> = Vec::with_capacity(elements.len());
        for element in elements {
            let mut place = sorted.len();
            for (index, other) in sorted.clone().into_iter().enumerate() {
                if self.compare_elements(&element, &other, &block, position)? < 0 {
                    place = index;
                    break;
                }
            }
            sorted.insert(place, element);
        }
        Ok(sorted)
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
            None => {
                // A class with no `<=>` at all cannot be ordered, and Ruby
                // reports the missing method rather than a failed comparison.
                if matches!(left, Object::Instance(_)) && !self.responds_to(left, "<=>") {
                    let wording = self.receiver_wording_for(left, position);
                    return Err(crate::vm::errors::undefined_method_error_worded(
                        "<=>",
                        left,
                        std::slice::from_ref(right),
                        wording,
                        position,
                    ));
                }
                self.send_to_object(left.clone(), "<=>", vec![right.clone()], position)?
            }
        };
        match answer {
            Object::Int(order) => Ok(order),
            Object::Float(order) => Ok(order as i64),
            // A bignum answers only by its sign, and an object of the
            // program's own is asked how it compares to zero, which is what
            // Ruby reduces a block result to.
            Object::BigInt(ref value) => {
                Ok(num_bigint::BigInt::from(0).cmp(value).reverse() as i64)
            }
            Object::Instance(_) => {
                // Ruby asks a value that is not already a number which side
                // of zero it falls on, reading `>` first and then `<`.
                for (named, order) in [(">", 1), ("<", -1)] {
                    if self
                        .send_to_object(answer.clone(), named, vec![Object::Int(0)], position)?
                        .is_truthy()
                    {
                        return Ok(order);
                    }
                }
                Ok(0)
            }
            _ => Err(comparison_failed(left, right, position)),
        }
    }

    /// The elements of `elements` with later duplicates dropped. A block
    /// decides what counts as a duplicate by naming a key for each element.
    /// The unique elements of an array as it stands, read one index at a
    /// time so an element appended while the walk runs is visited too.
    pub(crate) fn unique_live_elements(
        &mut self,
        array_rc: &Rc<std::cell::RefCell<Vec<Object>>>,
        block: Option<Rc<crate::object::BlockStatement>>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut seen: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
        let mut unique = Vec::new();
        let mut index = 0usize;
        loop {
            let Some(element) = array_rc.borrow().get(index).cloned() else {
                break;
            };
            let key = match &block {
                None => element.clone(),
                Some(block) => {
                    self.execute_block_callable(block, vec![element.clone()], position)?
                }
            };
            let slot = self.dict_slot_in(&seen, &key, false, position)?;
            if !seen.contains_key(&slot) {
                crate::vm::native_methods::remember_key_object(&mut seen, &slot, &key);
                seen.insert(slot, Object::Nil);
                unique.push(element);
            }
            index += 1;
        }
        Ok(unique)
    }

    /// Whether two values are `eql?`, which is stricter than `==`: 1 and 1.0
    /// are equal but not eql, and an object of its own decides by answering
    /// `eql?` itself.
    /// Whether two arrays hold equal elements. Ruby asks each pair with
    /// `equal?` before `==`, and an array that reaches itself compares as
    /// equal rather than recursing forever.
    pub(crate) fn arrays_equal(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let mut in_flight = Vec::new();
        self.arrays_equal_within(left, right, &mut in_flight, position)
    }

    fn arrays_equal_within(
        &mut self,
        left: &Object,
        right: &Object,
        in_flight: &mut Vec<(usize, usize)>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let (Object::Array(one), Object::Array(other)) = (left, right) else {
            return Ok(false);
        };
        let pair = (Rc::as_ptr(one) as usize, Rc::as_ptr(other) as usize);
        if pair.0 == pair.1 || in_flight.contains(&pair) {
            return Ok(true);
        }
        let held = one.borrow().clone();
        let against = other.borrow().clone();
        if held.len() != against.len() {
            return Ok(false);
        }
        in_flight.push(pair);
        let mut same = true;
        for (item, counterpart) in held.iter().zip(against.iter()) {
            if identical(item, counterpart) {
                continue;
            }
            let matched = if matches!((item, counterpart), (Object::Array(_), Object::Array(_))) {
                self.arrays_equal_within(item, counterpart, in_flight, position)?
            } else {
                self.evaluate_binary_operation(
                    &crate::ast::BinaryOp::Equal,
                    item.clone(),
                    counterpart.clone(),
                    position,
                )?
                .is_truthy()
            };
            if !matched {
                same = false;
                break;
            }
        }
        in_flight.pop();
        Ok(same)
    }

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
            && crate::vm::native_methods::string_subclass_value(left).is_none()
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
        // A String subclass holds its text behind the instance, and Ruby
        // compares that text rather than the two objects.
        let left = &crate::vm::native_methods::string_subclass_value(left)
            .unwrap_or_else(|| elements_of(left).unwrap_or_else(|| left.clone()));
        let right = &crate::vm::native_methods::string_subclass_value(right)
            .unwrap_or_else(|| elements_of(right).unwrap_or_else(|| right.clone()));
        if let (Object::String(text), Object::String(other)) = (left, right) {
            use crate::vm::native_methods::string_methods::{binary_bytes, strings_comparable};
            return Ok((binary_bytes(text) == binary_bytes(other)
                || *text.as_str() == *other.as_str())
                && strings_comparable(text, other));
        }
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

impl VirtualMachine {}

impl VirtualMachine {
    /// The elements of an argument that stands for an array: one as it is, and
    /// anything else through `to_ary`.
    /// One index that must fit a machine word, counted from the end when
    /// negative. A value too large for one is a RangeError, which is what
    /// Ruby raises before it looks at the array.
    fn machine_index(
        &mut self,
        value: &Object,
        length: i64,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let index = match value {
            Object::Int(index) => *index,
            // A Float past the width of a machine word names no index at all.
            Object::Float(index)
                if !(-9.223_372_036_854_776e18..9.223_372_036_854_776e18).contains(index) =>
            {
                let message = format!("float {} out of range of integer", index);
                return Err(crate::vm::errors::simple_exception(
                    "RangeError",
                    &message,
                    position,
                ));
            }
            Object::Float(index) => *index as i64,
            other => {
                let wide = self.coerce_integer_argument(other, position)?;
                let Ok(index) = i64::try_from(&wide) else {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "bignum too big to convert into `long'",
                        position,
                    ));
                };
                index
            }
        };
        Ok(if index < 0 { index + length } else { index })
    }

    /// The start and length a Range names over `length` elements, with a
    /// negative bound counted from the end.
    fn range_bounds(
        &mut self,
        range: &Object,
        length: i64,
        position: Position,
    ) -> Result<(i64, i64), MetorexError> {
        let Object::Range {
            start,
            end,
            exclusive,
            ..
        } = range
        else {
            return Ok((0, 0));
        };
        let first = match start.as_ref() {
            Object::Nil => 0,
            bound => self.machine_index(bound, length, position)?,
        };
        let last = match end.as_ref() {
            Object::Nil => length - 1,
            bound => {
                let resolved = self.machine_index(bound, length, position)?;
                if *exclusive { resolved - 1 } else { resolved }
            }
        };
        Ok((first, last.saturating_sub(first).saturating_add(1).max(0)))
    }

    /// One index into an array of `length` elements, counted from the end when
    /// negative and coerced through `to_int` when it is not already an
    /// Integer.
    fn index_from(
        &mut self,
        value: &Object,
        length: i64,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let index: i64 = match value {
            Object::Int(index) => *index,
            Object::Float(index) => *index as i64,
            other => self
                .coerce_integer_argument(other, position)?
                .try_into()
                .unwrap_or(i64::MAX),
        };
        Ok(if index < 0 { index + length } else { index })
    }

    /// The elements an argument to `zip` stands for: an Array, something that
    /// answers `to_ary`, or failing that anything that can be walked with
    /// `each`. An object with neither is refused the way Ruby refuses it.
    /// The values one `zip` argument lines up against, taking at most
    /// `wanted` of them so an endless walk still ends.
    fn zip_column(
        &mut self,
        value: &Object,
        wanted: usize,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if let Object::Array(elements) = value {
            return Ok(elements.borrow().clone());
        }
        if self.answers_to(value, "to_ary", position)?
            && let Object::Array(elements) =
                self.send_to_object(value.clone(), "to_ary", vec![], position)?
        {
            return Ok(elements.borrow().clone());
        }
        if self.answers_to(value, "each", position)? {
            let walk = self.send_to_object(
                value.clone(),
                "to_enum",
                vec![Object::symbol("each".to_string())],
                position,
            )?;
            // `first` stops the walk as soon as it has enough, which is what
            // lets an endless walk be zipped against a finite array.
            let taken =
                self.send_to_object(walk, "first", vec![Object::Int(wanted as i64)], position)?;
            if let Object::Array(elements) = taken {
                return Ok(elements.borrow().clone());
            }
            return Ok(Vec::new());
        }
        let named = match value {
            Object::Bool(true) => "TrueClass".to_string(),
            Object::Bool(false) => "FalseClass".to_string(),
            Object::Nil => "NilClass".to_string(),
            held => self.builtins().class_of(held).ruby_name().to_string(),
        };
        let message = format!("wrong argument type {} (must respond to :each)", named);
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

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
impl VirtualMachine {
    fn flatten_into(
        &mut self,
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
                    self.flatten_into(&inner, depth - 1, in_flight, flat, position)?;
                    in_flight.pop();
                }
                other if depth != 0 => match self.array_for_flatten(other, position)? {
                    Some(inner) => {
                        self.flatten_into(&inner, depth - 1, in_flight, flat, position)?
                    }
                    None => flat.push(other.clone()),
                },
                other => flat.push(other.clone()),
            }
        }
        Ok(())
    }

    /// The array an element spells, where it spells one. `flatten` asks every
    /// element that is not an Array, the way Ruby asks: an object saying it
    /// answers `to_ary`, or one with a `method_missing` to catch the call.
    fn array_for_flatten(
        &mut self,
        element: &Object,
        position: Position,
    ) -> Result<Option<Vec<Object>>, MetorexError> {
        if !matches!(element, Object::Instance(_)) {
            return Ok(None);
        }
        if let Some(held) = crate::vm::native_methods::array_subclass_value(element) {
            if let Object::Array(items) = held {
                return Ok(Some(items.borrow().clone()));
            }
            return Ok(None);
        }
        let named = Object::symbol("to_ary".to_string());
        // A redefined `respond_to?` has the last word, the way Ruby asks it
        // before reaching for a conversion at all.
        if self.lookup_method(element, "respond_to?").is_some()
            && !self
                .send_to_object(
                    element.clone(),
                    "respond_to?",
                    vec![named.clone(), Object::Bool(true)],
                    position,
                )?
                .is_truthy()
        {
            return Ok(None);
        }
        let answered = if self.responds_to(element, "to_ary") {
            self.send_to_object(element.clone(), "to_ary", vec![], position)?
        } else {
            // With no `to_ary` of its own, the object is asked only where it
            // has a `method_missing` to catch the call, and a refusal from
            // that stands for having no conversion at all.
            let mut says_it_does = false;
            if self.lookup_method(element, "respond_to_missing?").is_some() {
                says_it_does = self
                    .send_to_object(
                        element.clone(),
                        "respond_to_missing?",
                        vec![named, Object::Bool(true)],
                        position,
                    )?
                    .is_truthy();
                if !says_it_does {
                    return Ok(None);
                }
            }
            if self.lookup_method(element, "method_missing").is_none() {
                return Ok(None);
            }
            match self.send_to_object(element.clone(), "to_ary", vec![], position) {
                Ok(held) => held,
                Err(problem) => {
                    // An object that said it answers the call keeps the
                    // failure, and one that never claimed to spells no array.
                    if refused_the_call(&problem) && !says_it_does {
                        return Ok(None);
                    }
                    return Err(problem);
                }
            }
        };
        if let Object::Array(items) = &answered {
            return Ok(Some(items.borrow().clone()));
        }
        if let Some(Object::Array(items)) =
            crate::vm::native_methods::array_subclass_value(&answered)
        {
            return Ok(Some(items.borrow().clone()));
        }
        if matches!(answered, Object::Nil) {
            return Ok(None);
        }
        let message = format!(
            "can't convert {} to Array ({}#to_ary gives {})",
            self.builtins().class_of(element).name(),
            self.builtins().class_of(element).name(),
            self.builtins().class_of(&answered).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }
}

/// Whether a failure is the one an object raises for a call it has no method
/// for, which is what a `method_missing` that passes the call along raises.
fn refused_the_call(problem: &MetorexError) -> bool {
    matches!(problem, MetorexError::UncaughtException { exception, .. }
        if matches!(exception, Object::Exception(held)
            if held.borrow().exception_type == "NoMethodError"))
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

/// The ArgumentError Ruby raises when two values cannot be ordered.
fn comparison_failed(left: &Object, right: &Object, position: Position) -> MetorexError {
    // An instance reports its own class; every other kind reports the class
    // Ruby names it with, which `class_of` does not know for true and nil.
    let name_of = |value: &Object| match value {
        Object::Instance(instance) => instance.borrow().class.ruby_name(),
        other => crate::vm::native_methods::define_method::ruby_class_name(other).to_string(),
    };
    let message = format!(
        "comparison of {} with {} failed",
        name_of(left),
        name_of(right)
    );
    crate::vm::errors::simple_exception("ArgumentError", &message, position)
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
        // A float is its own bits, which is how an array holding NaN equals
        // another holding the same NaN even though NaN equals nothing.
        (Object::Float(one), Object::Float(other)) => one.to_bits() == other.to_bits(),
        _ => false,
    }
}

impl VirtualMachine {
    /// The count and the `random:` source `Array#sample` was called with.
    fn sample_arguments(
        &mut self,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<(Option<i64>, Option<Object>), MetorexError> {
        let mut positional = arguments;
        let mut source = None;
        if let Some(Object::Dict(entries)) = arguments.last() {
            let named = entries.borrow().get(":random").cloned();
            if let Some(named) = named {
                source = Some(named);
                positional = &arguments[..arguments.len() - 1];
            }
        }
        if positional.len() > 1 {
            return Err(method_argument_error(
                method_name,
                1,
                positional.len(),
                position,
            ));
        }
        let count = match positional.first() {
            None => None,
            Some(Object::Int(count)) => Some(*count),
            Some(other) => {
                let counted = self.coerce_integer_argument(other, position)?;
                Some(counted.try_into().unwrap_or(i64::MAX))
            }
        };
        Ok((count, source))
    }

    /// An index below `limit`, drawn either from the interpreter's own
    /// generator or from the object a `random:` keyword named. Ruby asks such
    /// an object for `rand(limit)` and refuses an answer outside the range.
    pub(crate) fn random_upto(
        &mut self,
        source: Option<&Object>,
        limit: i64,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let Some(source) = source else {
            return Ok(self.next_random_int(limit));
        };
        if limit <= 0 {
            return Ok(0);
        }
        let drawn =
            self.send_to_object(source.clone(), "rand", vec![Object::Int(limit)], position)?;
        let index = match drawn {
            Object::Int(index) => index,
            Object::Float(index) => index.trunc() as i64,
            other => {
                let drawn = self.coerce_integer_argument(&other, position)?;
                drawn.try_into().unwrap_or(i64::MAX)
            }
        };
        if index < 0 || index >= limit {
            return Err(simple_exception(
                "RangeError",
                &format!("random number too big {index}"),
                position,
            ));
        }
        Ok(index)
    }
}

impl VirtualMachine {
    /// Write a packed result into the String a `buffer:` keyword named. The
    /// writing starts at the end of what the buffer holds, or at the offset a
    /// leading `@` directive names, and the buffer is the answer.
    fn pack_into_buffer(
        &mut self,
        buffer: Object,
        format: &str,
        packed: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Object::String(held) = &buffer else {
            return Ok(buffer);
        };
        if held.is_frozen() {
            return Err(self.frozen_modification_error(&buffer, position));
        }
        let existing = super::pack_format::string_to_bytes(&held.as_str());
        let written = match packed {
            Object::String(text) => super::pack_format::string_to_bytes(&text.as_str()),
            _ => Vec::new(),
        };
        let (start, tail) = match leading_pack_offset(format) {
            Some(offset) => (offset, written.get(offset..).unwrap_or(&[]).to_vec()),
            None => (existing.len(), written),
        };
        let mut made = existing;
        made.resize(start, 0);
        made.extend_from_slice(&tail);
        held.replace_text(made.iter().map(|byte| *byte as char).collect::<String>());
        held.mark_bytes();
        Ok(buffer)
    }
}

/// The absolute offset a format's leading `@` directive names, where it has
/// one. Any other format writes at the end of what the buffer holds.
fn leading_pack_offset(format: &str) -> Option<usize> {
    let digits = format.strip_prefix('@')?;
    let counted: String = digits.chars().take_while(char::is_ascii_digit).collect();
    counted.parse().ok()
}

impl VirtualMachine {
    /// The run of elements a `slice!` subscript names: where it starts, how
    /// wide it is, and whether it named one element rather than a run. None
    /// when the subscript reaches past the array, which answers nil.
    fn slice_span(
        &mut self,
        arguments: &[Object],
        size: usize,
        position: Position,
    ) -> Result<Option<(usize, usize, bool)>, MetorexError> {
        let total = size as i64;
        match arguments.len() {
            1 => {
                let Some(Object::Range {
                    start,
                    end,
                    exclusive,
                    ..
                }) = crate::vm::native_methods::as_range(&arguments[0])
                else {
                    let index = self.coerce_integer_argument(&arguments[0], position)?;
                    let at = i64::try_from(&index).unwrap_or(i64::MAX);
                    let at = if at < 0 { at + total } else { at };
                    if at < 0 || at >= total {
                        return Ok(None);
                    }
                    return Ok(Some((at as usize, 1, true)));
                };
                let opening = match start.as_ref() {
                    Object::Nil => 0,
                    held => {
                        let asked = self.coerce_integer_argument(held, position)?;
                        let asked = i64::try_from(&asked).unwrap_or(i64::MAX);
                        if asked < 0 { asked + total } else { asked }
                    }
                };
                if opening < 0 || opening > total {
                    return Ok(None);
                }
                let closing = match end.as_ref() {
                    Object::Nil => total - 1,
                    held => {
                        let asked = self.coerce_integer_argument(held, position)?;
                        let asked = i64::try_from(&asked).unwrap_or(i64::MAX);
                        let placed = if asked < 0 { asked + total } else { asked };
                        if exclusive { placed - 1 } else { placed }
                    }
                };
                let width = (closing - opening + 1).clamp(0, total - opening);
                Ok(Some((opening as usize, width as usize, false)))
            }
            2 => {
                let asked = self.coerce_integer_argument(&arguments[0], position)?;
                let wanted = self.coerce_integer_argument(&arguments[1], position)?;
                let at = i64::try_from(&asked).unwrap_or(i64::MAX);
                let wanted = i64::try_from(&wanted).unwrap_or(i64::MAX);
                if wanted < 0 {
                    return Ok(None);
                }
                let at = if at < 0 { at + total } else { at };
                if at < 0 || at > total {
                    return Ok(None);
                }
                Ok(Some((at as usize, wanted.min(total - at) as usize, false)))
            }
            given => Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(1, 2),
                given,
                position,
            )),
        }
    }
}

impl VirtualMachine {
    /// The run of elements an arithmetic sequence names, or None when the
    /// subscript is not one.
    fn sequence_slice(
        &mut self,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        value: &Object,
        total: i64,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if !self.is_arithmetic_sequence(value) {
            return Ok(None);
        }
        let step = match self.send_to_object(value.clone(), "step", vec![], position)? {
            Object::Int(held) => held,
            other => self.machine_index(&other, 0, position)?,
        };
        if step == 0 {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "step can't be 0",
                position,
            ));
        }
        let opening = self.send_to_object(value.clone(), "begin", vec![], position)?;
        let closing = self.send_to_object(value.clone(), "end", vec![], position)?;
        let excluded = matches!(
            self.send_to_object(value.clone(), "exclude_end?", vec![], position)?,
            Object::Bool(true)
        );
        // A sequence stepping by one reads exactly the way the range it was
        // built from reads, down to answering nil for a start past the end.
        if step == 1 {
            let span = Object::Range {
                start: Box::new(opening),
                end: Box::new(closing),
                exclusive: excluded,
                mark: std::rc::Rc::new(()),
            };
            let (start, width) = self.range_bounds(&span, total, position)?;
            if start < 0 || start > total {
                return Ok(Some(Object::Nil));
            }
            let held = array_rc.borrow();
            let end = start.saturating_add(width).min(total);
            return Ok(Some(Object::array(
                held[start as usize..end.max(start) as usize].to_vec(),
            )));
        }
        let resolved = |vm: &mut Self, held: &Object| -> Result<Option<i64>, MetorexError> {
            match held {
                Object::Nil => Ok(None),
                other => {
                    let counted = vm.machine_index(other, 0, position)?;
                    Ok(Some(if counted < 0 {
                        counted + total
                    } else {
                        counted
                    }))
                }
            }
        };
        let (low, high) = if step > 0 {
            let low = resolved(self, &opening)?.unwrap_or(0);
            let high = match resolved(self, &closing)? {
                Some(held) => held - i64::from(excluded),
                None => total - 1,
            };
            (low, high)
        } else {
            // A sequence running downward reads the same span the other way
            // about, so the bounds trade places.
            let closing_held = resolved(self, &closing)?.map(|held| held + i64::from(excluded));
            let low = closing_held.unwrap_or(0);
            let high = match resolved(self, &opening)? {
                Some(held) if closing_held.is_none() => held.min(total - 1),
                Some(held) => held,
                None => total - 1,
            };
            (low, high)
        };
        if low < 0 || low > total || high - low >= total {
            let shown = self.send_to_object(value.clone(), "inspect", vec![], position)?;
            let message = format!("{} out of range", shown);
            return Err(crate::vm::errors::simple_exception(
                "RangeError",
                &message,
                position,
            ));
        }
        let top = high.min(total - 1);
        let mut taken = Vec::new();
        if top >= low {
            let held = array_rc.borrow();
            if step > 0 {
                let mut at = low;
                while at <= top {
                    taken.push(held[at as usize].clone());
                    at += step;
                }
            } else {
                let mut at = top;
                while at >= low {
                    taken.push(held[at as usize].clone());
                    at += step;
                }
            }
        }
        Ok(Some(Object::array(taken)))
    }

    /// Whether a value is an arithmetic sequence, which names a run of the
    /// array with a gap between the elements it takes.
    fn is_arithmetic_sequence(&self, value: &Object) -> bool {
        let Object::Instance(instance) = value else {
            return false;
        };
        let mut cursor = Some(Rc::clone(&instance.borrow().class));
        while let Some(class) = cursor {
            if class.name() == "Enumerator::ArithmeticSequence" {
                return true;
            }
            cursor = class.superclass();
        }
        false
    }
}
