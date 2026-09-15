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
        // A subclass instance stands wherever a range does, so an argument
        // that is one is read as the range behind it.
        let arguments: Vec<Object> = arguments
            .iter()
            .map(|held| super::as_range(held).unwrap_or_else(|| held.clone()))
            .collect();
        let arguments = arguments.as_slice();
        match method_name {
            // `bsearch` halves the range at each step, either looking for
            // the smallest element the block says yes to, or for the one the
            // block answers zero for.
            "bsearch" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let bounds = SearchBounds::of(start, end, *exclusive, position)?;
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                self.binary_search(&bounds, &block, position).map(Some)
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
                    // Without a block the walk is handed back, which is what
                    // `(1..3).each` answers. It counts as many values as the
                    // range holds, where the range can say.
                    None => {
                        let counted =
                            match self.send_to_object(receiver.clone(), "size", vec![], position) {
                                Ok(Object::Int(count)) => Some(count),
                                _ => None,
                            };
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                Vec::new(),
                                counted,
                                position,
                            )
                            .map(Some);
                    }
                };

                // A range that counts up from an Integer with no end in sight
                // is walked one value at a time, since there is no list of
                // them to collect. The block stops it with `break`.
                if let Some(first) = endless_int_start(start, end) {
                    let first = &first;
                    let mut current = *first;
                    loop {
                        match self
                            .execute_block_with_control_flow(&block, vec![Object::Int(current)])?
                        {
                            super::super::ControlFlow::Next
                            | super::super::ControlFlow::Value(_)
                            | super::super::ControlFlow::Redo { .. }
                            | super::super::ControlFlow::Retry { .. }
                            | super::super::ControlFlow::Continue { .. } => {}
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
                        current += 1;
                    }
                }

                // A String range with no end follows `succ` for as long as
                // the block keeps asking, the same way the Integer one counts.
                if let Some(first) = endless_string_start(start, end) {
                    let mut current = first;
                    loop {
                        let value = Object::string(current.clone());
                        match self.execute_block_with_control_flow(&block, vec![value])? {
                            super::super::ControlFlow::Next
                            | super::super::ControlFlow::Value(_)
                            | super::super::ControlFlow::Redo { .. }
                            | super::super::ControlFlow::Retry { .. }
                            | super::super::ControlFlow::Continue { .. } => {}
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
                        let stepped = self.send_to_object(
                            Object::string(current.clone()),
                            "succ",
                            vec![],
                            position,
                        )?;
                        match stepped {
                            Object::String(text) => current = text.as_str().to_string(),
                            _ => return Ok(Some(receiver.clone())),
                        }
                    }
                }

                // The values are walked the same way `to_a` collects them,
                // so a String range follows `succ` too.
                let elements = self.range_elements(start, end, *exclusive, position)?;
                for element in elements {
                    match self.execute_block_with_control_flow(&block, vec![element])? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Retry { .. }
                        | super::super::ControlFlow::Continue { .. } => continue,
                        // `break` ends the walk and answers what it carried,
                        // which is what the call reports.
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
            "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                Ok(Some(Object::array(elements)))
            }
            // `size` counts a numeric range without walking it, and answers
            // Infinity when it has no end.
            "size" | "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // Only a range that steps from an Integer has a size. A start
                // that is not a number at all has none to report, while a
                // numeric one that cannot be stepped from is an error.
                if !matches!(
                    start.as_ref(),
                    Object::Int(_) | Object::BigInt(_) | Object::Float(_) | Object::Nil
                ) {
                    // A start that has a successor can be walked, it just has
                    // no size to count. One that has none cannot be walked at
                    // all.
                    if matches!(start.as_ref(), Object::String(_) | Object::Symbol(_))
                        || self.responds_to(start, "succ")
                    {
                        return Ok(Some(Object::Nil));
                    }
                    let message = format!(
                        "can't iterate from {}",
                        self.builtins().class_of(start).ruby_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                if !matches!(start.as_ref(), Object::Int(_) | Object::BigInt(_)) {
                    let message = format!(
                        "can't iterate from {}",
                        self.builtins().class_of(start).ruby_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                let infinite = matches!(end.as_ref(), Object::Nil)
                    || matches!(end.as_ref(), Object::Float(value) if value.is_infinite());
                if infinite {
                    return Ok(Some(Object::Float(f64::INFINITY)));
                }
                let (Some(first), Some(last)) = (numeric_floor(start), numeric_ceiling(end)) else {
                    return Ok(Some(Object::Nil));
                };
                let last = if *exclusive && end_is_whole(end) {
                    last - 1
                } else {
                    last
                };
                Ok(Some(Object::Int((last - first + 1).max(0))))
            }
            // A numeric range answers by comparing the ends, and any other
            // walks its values, which is how a String range refuses one that
            // falls between its endpoints but is not among them.
            "member?" | "include?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Only a range of names is walked; every other kind answers
                // by comparing against the ends, which is what an object with
                // no `succ` of its own needs.
                // A range of numbers answers by comparing against its ends,
                // which is what lets a value coerce itself into the
                // comparison rather than being looked for one step at a time.
                let counts = matches!(
                    start.as_ref(),
                    Object::Int(_) | Object::Float(_) | Object::BigInt(_)
                );
                let walks = !counts
                    && (matches!(start.as_ref(), Object::String(_) | Object::Symbol(_))
                        || self.responds_to(start, "succ"));
                if !walks {
                    let covered =
                        self.range_covers(start, end, *exclusive, &arguments[0], position)?;
                    return Ok(Some(Object::Bool(covered)));
                }
                // A range of names the same length as the value reads like an
                // odometer: each place must stand between the two ends.
                if let (Object::String(low), Object::String(high), Object::String(held)) =
                    (start.as_ref(), end.as_ref(), &arguments[0])
                    && let Some(answer) =
                        place_by_place(&low.as_str(), &high.as_str(), &held.as_str(), *exclusive)
                {
                    return Ok(Some(Object::Bool(answer)));
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                for element in elements {
                    if self.elements_equal(&element, &arguments[0], position)? {
                        return Ok(Some(Object::Bool(true)));
                    }
                }
                Ok(Some(Object::Bool(false)))
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
            // `first` and `last` answer the endpoint, or that many values from
            // the front or the back when given a count.
            "begin" => Ok(Some((**start).clone())),
            "end" => Ok(Some((**end).clone())),
            "first" | "last" => {
                if arguments.is_empty() {
                    // The open side of a range has no first or last value.
                    if method_name == "first" && matches!(start.as_ref(), Object::Nil) {
                        return Err(crate::vm::errors::simple_exception(
                            "RangeError",
                            "cannot get the first element of beginless range",
                            position,
                        ));
                    }
                    if method_name == "last" && matches!(end.as_ref(), Object::Nil) {
                        return Err(crate::vm::errors::simple_exception(
                            "RangeError",
                            "cannot get the last element of endless range",
                            position,
                        ));
                    }
                    return Ok(Some(if method_name == "first" {
                        (**start).clone()
                    } else {
                        (**end).clone()
                    }));
                }
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let wanted: i64 = self
                    .coerce_integer_argument(&arguments[0], position)?
                    .try_into()
                    .unwrap_or(i64::MAX);
                if wanted < 0 {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative array size (or size too big)",
                        position,
                    ));
                }
                if method_name == "last" && matches!(end.as_ref(), Object::Nil) {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot get the last element of endless range",
                        position,
                    ));
                }
                // A range counting up from an Integer with no end has no list
                // to collect, so the wanted values are counted out directly.
                if method_name == "first"
                    && let Some(counted) = endless_int_start(start, end)
                {
                    return Ok(Some(Object::array(
                        (0..wanted)
                            .map(|step| Object::Int(counted + step))
                            .collect(),
                    )));
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                let taken = (wanted as usize).min(elements.len());
                Ok(Some(Object::array(if method_name == "first" {
                    elements[..taken].to_vec()
                } else {
                    elements[elements.len() - taken..].to_vec()
                })))
            }
            // `min` and `max` order the values, and a block decides the order
            // the way it does for an array.
            "min" | "max" | "minmax" => {
                if matches!(start.as_ref(), Object::Nil) && method_name != "max" {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot get the minimum of beginless range",
                        position,
                    ));
                }
                if matches!(end.as_ref(), Object::Nil) && method_name != "min" {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot get the maximum of endless range",
                        position,
                    ));
                }
                let has_block = matches!(self.pending_block, Some(Object::Block(_)));
                // A beginless range has no value to start a comparison from,
                // so a custom order cannot pick a largest value.
                if matches!(start.as_ref(), Object::Nil) && (has_block || !arguments.is_empty()) {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot get the maximum of beginless range with custom comparison method",
                        position,
                    ));
                }
                // Both ends at once are answered by asking for each on its
                // own, which Range does in the core library.
                if method_name == "minmax" && arguments.is_empty() {
                    return Ok(None);
                }
                // Without a block or a count, the ends answer directly, which
                // works for a range too large to walk.
                if !has_block && arguments.is_empty() && method_name != "minmax" {
                    if method_name == "min" {
                        let order = self.evaluate_binary_operation(
                            &crate::ast::BinaryOp::Spaceship,
                            (**start).clone(),
                            (**end).clone(),
                            position,
                        )?;
                        if matches!(order, Object::Int(value) if value > 0) {
                            return Ok(Some(Object::Nil));
                        }
                        // A range that leaves its end out and begins there
                        // holds nothing at all.
                        if *exclusive && matches!(order, Object::Int(0)) {
                            return Ok(Some(Object::Nil));
                        }
                        return Ok(Some((**start).clone()));
                    }
                    // Leaving the end out only makes sense over whole
                    // numbers, where the value below it is known.
                    let last = end.as_big_integer();
                    let ends_beginless = matches!(start.as_ref(), Object::Nil);
                    if *exclusive
                        && last.is_none()
                        && (ends_beginless || counts_as_a_number(end.as_ref()))
                    {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            "cannot exclude non Integer end value",
                            position,
                        ));
                    }
                    let begins_whole = start.as_big_integer().is_some();
                    if ends_beginless {
                        if let (true, Some(reached)) = (*exclusive, last.as_ref()) {
                            return Ok(Some(Object::integer(reached.clone() - 1)));
                        }
                        return Ok(Some((**end).clone()));
                    }
                    let order = self.evaluate_binary_operation(
                        &crate::ast::BinaryOp::Spaceship,
                        (**start).clone(),
                        (**end).clone(),
                        position,
                    )?;
                    if matches!(order, Object::Int(value) if value > 0) {
                        return Ok(Some(Object::Nil));
                    }
                    if !*exclusive {
                        return Ok(Some((**end).clone()));
                    }
                    if let Some(reached) = last {
                        if !begins_whole {
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                "cannot exclude end value with non Integer begin value",
                                position,
                            ));
                        }
                        if matches!(order, Object::Int(0)) {
                            return Ok(Some(Object::Nil));
                        }
                        return Ok(Some(Object::integer(reached - 1)));
                    }
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                let walked = Object::array(elements);
                self.send_to_object(walked, method_name, arguments.to_vec(), position)
                    .map(Some)
            }
            // `cover?` asks whether a value falls between the ends, which
            // needs no walk, and takes another Range too. `===` picks the
            // same answer, which is how a Range reads in a `case`.
            "cover?" | "===" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Range {
                    start: other_start,
                    end: other_end,
                    exclusive: other_exclusive,
                } = &arguments[0]
                {
                    let starts_within = match (start.as_ref(), other_start.as_ref()) {
                        // A range with no start of its own covers one that
                        // has none either, and a range that starts somewhere
                        // covers no beginless range at all.
                        (Object::Nil, _) => true,
                        (_, Object::Nil) => false,
                        _ => self.range_covers(start, end, *exclusive, other_start, position)?,
                    };
                    let ends_within = if *other_exclusive {
                        match (end.as_ref(), other_end.as_ref()) {
                            (Object::Nil, _) => true,
                            (_, Object::Nil) => false,
                            // An exclusive end over whole numbers stops one
                            // short, so the range reaches that number instead.
                            (_, Object::Int(last)) => {
                                let reached = Object::Int(last - 1);
                                self.range_covers(start, end, *exclusive, &reached, position)?
                            }
                            _ => {
                                let order = self.evaluate_binary_operation(
                                    &crate::ast::BinaryOp::Spaceship,
                                    (**other_end).clone(),
                                    (**end).clone(),
                                    position,
                                )?;
                                matches!(order, Object::Int(value) if value <= 0)
                            }
                        }
                    } else {
                        self.range_covers(start, end, *exclusive, other_end, position)?
                    };
                    return Ok(Some(Object::Bool(starts_within && ends_within)));
                }
                let covered = self.range_covers(start, end, *exclusive, &arguments[0], position)?;
                Ok(Some(Object::Bool(covered)))
            }
            // `overlap?` asks whether the two ranges share any value.
            "overlap?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::Range {
                    start: other_start,
                    end: other_end,
                    exclusive: other_exclusive,
                } = &arguments[0]
                else {
                    let message = format!(
                        "wrong argument type {} (expected Range)",
                        self.builtins().class_of(&arguments[0]).ruby_name()
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &message,
                        position,
                    ));
                };
                // A range holding no value shares none, and two ranges over
                // values that do not compare share none either.
                if self.range_holds_nothing(start, end, *exclusive, position)?
                    || self.range_holds_nothing(
                        other_start,
                        other_end,
                        *other_exclusive,
                        position,
                    )?
                {
                    return Ok(Some(Object::Bool(false)));
                }
                if !matches!(start.as_ref(), Object::Nil)
                    && !matches!(other_start.as_ref(), Object::Nil)
                {
                    let compared = self.evaluate_binary_operation(
                        &crate::ast::BinaryOp::Spaceship,
                        (**start).clone(),
                        (**other_start).clone(),
                        position,
                    )?;
                    if matches!(compared, Object::Nil) {
                        return Ok(Some(Object::Bool(false)));
                    }
                }
                let before = self.range_ends_before(end, *exclusive, other_start, position)?;
                let after = self.range_ends_before(other_end, *other_exclusive, start, position)?;
                Ok(Some(Object::Bool(!before && !after)))
            }
            "to_set" => {
                if matches!(end.as_ref(), Object::Nil) {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot convert endless range to a set",
                        position,
                    ));
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                let walked = Object::array(elements);
                self.send_to_object(walked, "to_set", arguments.to_vec(), position)
                    .map(Some)
            }
            "exclude_end?" => Ok(Some(Object::Bool(*exclusive))),
            // The endpoints render through their own `inspect`, and an
            // endless or beginless range leaves its side out.
            "to_s" | "inspect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let separator = if *exclusive { "..." } else { ".." };
                if method_name == "to_s" {
                    let render = |value: &Object| match value {
                        Object::Nil => String::new(),
                        other => format!("{}", other),
                    };
                    return Ok(Some(Object::string(format!(
                        "{}{}{}",
                        render(start),
                        separator,
                        render(end)
                    ))));
                }
                // A range with neither end shows both nils, where one open
                // side alone shows nothing.
                let both_open =
                    matches!(start.as_ref(), Object::Nil) && matches!(end.as_ref(), Object::Nil);
                let first = match start.as_ref() {
                    Object::Nil if !both_open => String::new(),
                    other => self.get_inspect_representation(other, position)?,
                };
                let last = match end.as_ref() {
                    Object::Nil if !both_open => String::new(),
                    other => self.get_inspect_representation(other, position)?,
                };
                Ok(Some(Object::string(format!(
                    "{}{}{}",
                    first, separator, last
                ))))
            }
            // Two ranges are equal when their ends and their exclusivity
            // match. `eql?` compares the ends with `eql?` rather than `==`.
            "==" | "eql?" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                let Object::Range {
                    start: other_start,
                    end: other_end,
                    exclusive: other_exclusive,
                } = other
                else {
                    return Ok(Some(Object::Bool(false)));
                };
                if exclusive != other_exclusive {
                    return Ok(Some(Object::Bool(false)));
                }
                let same = if method_name == "eql?" {
                    self.values_eql(start, other_start, position)?
                        && self.values_eql(end, other_end, position)?
                } else {
                    self.elements_equal(start, other_start, position)?
                        && self.elements_equal(end, other_end, position)?
                };
                Ok(Some(Object::Bool(same)))
            }
            "count" => {
                if !arguments.is_empty() || self.pending_block.is_some() {
                    return Ok(None);
                }
                // A range with no beginning or no end holds endlessly many
                // values, which Ruby reports as Infinity.
                if matches!(start.as_ref(), Object::Nil) || matches!(end.as_ref(), Object::Nil) {
                    return Ok(Some(Object::Float(f64::INFINITY)));
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                Ok(Some(Object::Int(elements.len() as i64)))
            }
            _ => Ok(None),
        }
    }
}

/// The first value of a range that counts up from an Integer and never
/// reaches an end, which is walked one value at a time rather than collected.
/// The first value of a String range with no end, which `each` walks with
/// `succ` for as long as the block keeps asking.
fn endless_string_start(start: &Object, end: &Object) -> Option<String> {
    if !matches!(end, Object::Nil) {
        return None;
    }
    match start {
        Object::String(text) => Some(text.as_str().to_string()),
        _ => None,
    }
}

fn endless_int_start(start: &Object, end: &Object) -> Option<i64> {
    let endless = matches!(end, Object::Nil)
        || matches!(end, Object::Float(value) if value.is_infinite() && *value > 0.0);
    match (endless, start) {
        (true, Object::Int(first)) => Some(*first),
        _ => None,
    }
}

impl VirtualMachine {
    /// The values a range walks: integers count up, and anything else follows
    /// `succ` until it passes the end.
    /// Whether a range holds no value at all, which is the case when it
    /// begins past its end, or ends where it begins and leaves that out.
    fn range_holds_nothing(
        &mut self,
        start: &Object,
        end: &Object,
        exclusive: bool,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if matches!(start, Object::Nil) || matches!(end, Object::Nil) {
            return Ok(false);
        }
        let order = self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Spaceship,
            start.clone(),
            end.clone(),
            position,
        )?;
        match order {
            Object::Int(0) => Ok(exclusive),
            Object::Int(value) => Ok(value > 0),
            _ => Ok(false),
        }
    }

    pub(crate) fn range_elements(
        &mut self,
        start: &Object,
        end: &Object,
        exclusive: bool,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        if let (Object::Int(first), Object::Int(last)) = (start, end) {
            let last = if exclusive { last - 1 } else { *last };
            return Ok((*first..=last).map(Object::Int).collect());
        }
        if matches!(start, Object::Nil) {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                "can't iterate from NilClass",
                position,
            ));
        }
        // A range with no end holds endlessly many values, so collecting them
        // is refused rather than run forever.
        if matches!(end, Object::Nil) {
            return Err(crate::vm::errors::simple_exception(
                "RangeError",
                "cannot convert endless range to an array",
                position,
            ));
        }
        // A String or Symbol answers `succ` natively, and anything else must
        // define one of its own.
        let walkable = matches!(start, Object::String(_) | Object::Symbol(_))
            || self.responds_to(start, "succ");
        if !walkable {
            let message = format!(
                "can't iterate from {}",
                self.builtins().class_of(start).ruby_name()
            );
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &message,
                position,
            ));
        }
        // Two single ASCII characters walk by code point, which is how
        // `("A".."z")` reaches the punctuation between the two alphabets.
        if let (Some(first), Some(last)) = (single_ascii(start), single_ascii(end)) {
            let last = if exclusive {
                last.saturating_sub(1)
            } else {
                last
            };
            let symbols = matches!(start, Object::Symbol(_));
            return Ok((first..=last)
                .filter_map(char::from_u32)
                .map(|letter| {
                    let text = Rc::new(crate::object::StringValue::new(letter.to_string()));
                    if symbols {
                        Object::Symbol(text)
                    } else {
                        Object::String(text)
                    }
                })
                .collect());
        }
        let end_length = name_of(end).map(|text| text.as_str().chars().count());
        let mut walked = Vec::new();
        let mut current = start.clone();
        loop {
            // A name longer than the end's has passed it, whatever the
            // characters say: "Z".succ is "AA", which is past "z".
            if let (Some(limit), Some(text)) = (end_length, name_of(&current))
                && text.as_str().chars().count() > limit
            {
                break;
            }
            let order = self.evaluate_binary_operation(
                &crate::ast::BinaryOp::Spaceship,
                current.clone(),
                end.clone(),
                position,
            )?;
            let Object::Int(order) = order else {
                break;
            };
            if order > 0 || (exclusive && order == 0) {
                break;
            }
            walked.push(current.clone());
            if order == 0 {
                break;
            }
            current = self.send_to_object(current, "succ", vec![], position)?;
        }
        Ok(walked)
    }
}

/// The characters a String or Symbol is named with.
fn name_of(value: &Object) -> Option<Rc<crate::object::StringValue>> {
    match value {
        Object::String(text) | Object::Symbol(text) => Some(Rc::clone(text)),
        _ => None,
    }
}

/// The code point of a name that is one ASCII character, or None otherwise.
fn single_ascii(value: &Object) -> Option<u32> {
    let text = name_of(value)?;
    let held = text.to_text();
    let mut letters = held.chars();
    let only = letters.next()?;
    if letters.next().is_some() || !only.is_ascii() {
        return None;
    }
    Some(only as u32)
}

impl VirtualMachine {
    /// Whether a value falls between the ends of a range, without walking it.
    /// Ruby refuses a range whose ends cannot be ordered, which is what
    /// `beg <=> end` answering nil says.
    pub(crate) fn check_range_ends(
        &mut self,
        start: &Object,
        end: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        if matches!(start, Object::Nil) || matches!(end, Object::Nil) {
            return Ok(());
        }
        let order = self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Spaceship,
            start.clone(),
            end.clone(),
            position,
        )?;
        if matches!(order, Object::Int(_)) {
            return Ok(());
        }
        Err(crate::vm::errors::simple_exception(
            "ArgumentError",
            "bad value for range",
            position,
        ))
    }

    fn range_covers(
        &mut self,
        start: &Object,
        end: &Object,
        exclusive: bool,
        value: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if !matches!(start, Object::Nil) {
            // Ruby asks the start where the value stands, so a value that
            // knows how to coerce the start is asked to.
            let order = self.evaluate_binary_operation(
                &crate::ast::BinaryOp::Spaceship,
                start.clone(),
                value.clone(),
                position,
            )?;
            match order {
                Object::Int(order) if order <= 0 => {}
                _ => return Ok(false),
            }
        }
        if matches!(end, Object::Nil) {
            return Ok(true);
        }
        let order = self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Spaceship,
            value.clone(),
            end.clone(),
            position,
        )?;
        Ok(match order {
            Object::Int(order) if exclusive => order < 0,
            Object::Int(order) => order <= 0,
            _ => false,
        })
    }

    /// Whether a range ending at `end` finishes before `value` begins.
    fn range_ends_before(
        &mut self,
        end: &Object,
        exclusive: bool,
        value: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if matches!(end, Object::Nil) || matches!(value, Object::Nil) {
            return Ok(false);
        }
        let order = self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Spaceship,
            end.clone(),
            value.clone(),
            position,
        )?;
        Ok(match order {
            Object::Int(order) if exclusive => order <= 0,
            Object::Int(order) => order < 0,
            _ => false,
        })
    }
}

/// The first whole value at or above a numeric range's start, which is what
/// counting one begins from.
fn numeric_floor(value: &Object) -> Option<i64> {
    match value {
        Object::Int(number) => Some(*number),
        Object::BigInt(number) => i64::try_from(number.as_ref()).ok(),
        Object::Float(number) => Some(number.ceil() as i64),
        _ => None,
    }
}

/// The last whole value at or below a numeric range's end.
fn numeric_ceiling(value: &Object) -> Option<i64> {
    match value {
        Object::Int(number) => Some(*number),
        Object::BigInt(number) => i64::try_from(number.as_ref()).ok(),
        Object::Float(number) => Some(number.floor() as i64),
        _ => None,
    }
}

/// Whether an end is a whole number, which decides whether an exclusive range
/// loses its last value.
fn end_is_whole(value: &Object) -> bool {
    match value {
        Object::Int(_) | Object::BigInt(_) => true,
        Object::Float(number) => number.fract() == 0.0,
        _ => false,
    }
}

/// Whether a name stands inside a range of names of the same length, read one
/// place at a time. Ruby reads the places as bytes, so a character spelled
/// with several of them is read one byte at a time. Answers None where the
/// three do not line up that way.
fn place_by_place(low: &str, high: &str, held: &str, exclusive: bool) -> Option<bool> {
    let low: Vec<u8> = low.bytes().collect();
    let high: Vec<u8> = high.bytes().collect();
    let held: Vec<u8> = held.bytes().collect();
    if low.len() != high.len() || low.len() != held.len() {
        return None;
    }
    for ((low, high), held) in low.iter().zip(high.iter()).zip(held.iter()) {
        if held < low || held > high {
            return Some(false);
        }
    }
    if exclusive && held == high {
        return Some(false);
    }
    Some(true)
}

/// Ruby treats a range end as numeric when it is an Integer, a Float, or one
/// of the Numeric classes the prelude defines as instances.
fn counts_as_a_number(held: &Object) -> bool {
    match held {
        Object::Int(_) | Object::BigInt(_) | Object::Float(_) => true,
        Object::Instance(instance) => {
            matches!(instance.borrow().class.name(), "Complex" | "Rational")
        }
        _ => false,
    }
}

/// What a binary search walks: whole numbers, or the bit patterns of the
/// Floats in order, which are the same order as the Floats themselves.
enum SearchBounds {
    /// A whole-number range, where None at either end reaches without limit.
    Whole {
        low: Option<i64>,
        high: Option<i64>,
        exclusive: bool,
    },
    /// A Float range, walked over the whole numbers its bit patterns map to.
    Fractional {
        low: i64,
        high: i64,
        exclusive: bool,
    },
}

/// Which way a step of the search goes, and whether the block said the
/// element it was handed is one it wants.
struct SearchAnswer {
    smaller: bool,
    settled: Option<Object>,
    satisfied: bool,
}

impl SearchBounds {
    /// The bounds a range names, or the TypeError a range of something other
    /// than numbers raises.
    fn of(
        start: &Object,
        end: &Object,
        exclusive: bool,
        position: Position,
    ) -> Result<Self, MetorexError> {
        let refuse = |held: &Object| {
            let named = match held {
                Object::String(_) => "String",
                Object::Symbol(_) => "Symbol",
                Object::Instance(instance) => {
                    return refuse_binary_search(instance.borrow().class.name(), position);
                }
                other => other.type_name(),
            };
            refuse_binary_search(named, position)
        };
        let whole_of = |held: &Object| match held {
            Object::Nil => Ok(None),
            Object::Int(number) => Ok(Some(Some(*number))),
            _ => Err(()),
        };
        let fraction_of = |held: &Object| match held {
            Object::Nil => Ok(None),
            Object::Int(number) => Ok(Some(*number as f64)),
            Object::Float(number) => Ok(Some(*number)),
            _ => Err(()),
        };
        if let (Ok(low), Ok(high)) = (whole_of(start), whole_of(end)) {
            return Ok(SearchBounds::Whole {
                low: low.flatten(),
                high: high.flatten(),
                exclusive,
            });
        }
        let (Ok(low), Ok(high)) = (fraction_of(start), fraction_of(end)) else {
            return Err(match fraction_of(start) {
                Err(()) => refuse(start),
                Ok(_) => refuse(end),
            });
        };
        Ok(SearchBounds::Fractional {
            low: fraction_as_whole(low.unwrap_or(f64::NEG_INFINITY)),
            high: fraction_as_whole(high.unwrap_or(f64::INFINITY)),
            exclusive,
        })
    }
}

/// The TypeError a range of something a binary search cannot halve raises.
fn refuse_binary_search(named: &str, position: Position) -> MetorexError {
    let message = format!("can't do binary search for {}", named);
    crate::vm::errors::simple_exception("TypeError", &message, position)
}

/// A Float as the whole number its bits stand for, in the same order the
/// Floats themselves are in, so a search can halve the gap between two.
fn fraction_as_whole(value: f64) -> i64 {
    let bits = value.to_bits() as i64;
    if bits < 0 { i64::MIN - bits } else { bits }
}

/// The Float a whole number from `fraction_as_whole` stands for.
fn whole_as_fraction(value: i64) -> f64 {
    let bits = if value < 0 { i64::MIN - value } else { value };
    f64::from_bits(bits as u64)
}

impl VirtualMachine {
    /// Halve the range until the block settles on an element, answering nil
    /// when it never does.
    fn binary_search(
        &mut self,
        bounds: &SearchBounds,
        block: &Rc<crate::object::BlockStatement>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match bounds {
            SearchBounds::Whole {
                low,
                high,
                exclusive,
            } => {
                let (low, high) =
                    self.whole_search_limits(*low, *high, *exclusive, block, position)?;
                let Some((low, high)) = low.zip(high) else {
                    return Ok(Object::Nil);
                };
                self.halve(low, high, block, position, &Object::Int)
            }
            SearchBounds::Fractional {
                low,
                high,
                exclusive,
            } => {
                let high = if *exclusive {
                    *high
                } else {
                    high.saturating_add(1)
                };
                self.halve(*low, high, block, position, &|held| {
                    Object::Float(whole_as_fraction(held))
                })
            }
        }
    }

    /// The limits a whole-number search runs between. An end left open is
    /// found by stepping out from the other one until the block turns.
    fn whole_search_limits(
        &mut self,
        low: Option<i64>,
        high: Option<i64>,
        exclusive: bool,
        block: &Rc<crate::object::BlockStatement>,
        position: Position,
    ) -> Result<(Option<i64>, Option<i64>), MetorexError> {
        match (low, high) {
            (Some(low), Some(high)) => {
                let high = if exclusive {
                    high
                } else {
                    high.saturating_add(1)
                };
                Ok((Some(low), Some(high)))
            }
            (Some(low), None) => {
                let mut step = 1i64;
                let mut reach = low.saturating_add(step);
                for _ in 0..64 {
                    let answer = self.ask_block(block, Object::Int(reach), position)?;
                    if answer.settled.is_some() || answer.smaller {
                        // The element reached is one the search keeps, so the
                        // limit sits one past it.
                        return Ok((Some(low), Some(reach.saturating_add(1))));
                    }
                    step = step.saturating_mul(2);
                    reach = low.saturating_add(step);
                }
                Ok((Some(low), Some(reach.saturating_add(1))))
            }
            (None, Some(high)) => {
                let high = if exclusive {
                    high
                } else {
                    high.saturating_add(1)
                };
                let mut step = 1i64;
                let mut reach = high.saturating_sub(step);
                for _ in 0..64 {
                    let answer = self.ask_block(block, Object::Int(reach), position)?;
                    if answer.settled.is_some() || !answer.smaller {
                        return Ok((Some(reach), Some(high)));
                    }
                    step = step.saturating_mul(2);
                    reach = high.saturating_sub(step);
                }
                Ok((Some(reach), Some(high)))
            }
            (None, None) => Ok((None, None)),
        }
    }

    /// The halving itself, over whole numbers that stand for the elements.
    fn halve(
        &mut self,
        mut low: i64,
        high: i64,
        block: &Rc<crate::object::BlockStatement>,
        position: Position,
        element: &dyn Fn(i64) -> Object,
    ) -> Result<Object, MetorexError> {
        let opening = high;
        let mut high = high;
        let mut satisfied = false;
        while low < high {
            let middle = (low as i128 + (high as i128 - low as i128) / 2) as i64;
            let answer = self.ask_block(block, element(middle), position)?;
            if let Some(found) = answer.settled {
                return Ok(found);
            }
            satisfied |= answer.satisfied;
            if answer.smaller {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        if low >= opening || !satisfied {
            return Ok(Object::Nil);
        }
        Ok(element(low))
    }

    /// Hand one element to the block and read which way the search goes.
    fn ask_block(
        &mut self,
        block: &Rc<crate::object::BlockStatement>,
        element: Object,
        position: Position,
    ) -> Result<SearchAnswer, MetorexError> {
        let answer = self.execute_block_callable(block, vec![element.clone()], position)?;
        Ok(match answer {
            Object::Bool(true) => SearchAnswer {
                smaller: true,
                settled: None,
                satisfied: true,
            },
            Object::Bool(false) | Object::Nil => SearchAnswer {
                smaller: false,
                settled: None,
                satisfied: false,
            },
            Object::Int(number) => {
                if number == 0 {
                    SearchAnswer {
                        smaller: false,
                        settled: Some(element),
                        satisfied: true,
                    }
                } else {
                    SearchAnswer {
                        smaller: number < 0,
                        settled: None,
                        satisfied: false,
                    }
                }
            }
            Object::Float(number) => {
                if number == 0.0 {
                    SearchAnswer {
                        smaller: false,
                        settled: Some(element),
                        satisfied: true,
                    }
                } else {
                    SearchAnswer {
                        smaller: number < 0.0,
                        settled: None,
                        satisfied: false,
                    }
                }
            }
            other => {
                let message = format!(
                    "wrong argument type {} (must be numeric, true, false or nil)",
                    self.builtins().class_of(&other).name()
                );
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ));
            }
        })
    }
}
