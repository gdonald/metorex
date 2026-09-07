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

                // The values are walked the same way `to_a` collects them,
                // so a String range follows `succ` too.
                let elements = self.range_elements(start, end, *exclusive, position)?;
                for element in elements {
                    match self.execute_block_with_control_flow(&block, vec![element])? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Continue { .. } => continue,
                        super::super::ControlFlow::Break { .. } => break,
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
                let walks = matches!(start.as_ref(), Object::String(_) | Object::Symbol(_))
                    || self.responds_to(start, "succ");
                if !walks {
                    let covered =
                        self.range_covers(start, end, *exclusive, &arguments[0], position)?;
                    return Ok(Some(Object::Bool(covered)));
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
            "begin" => Ok(Some(start.as_ref().clone())),
            "end" => Ok(Some(end.as_ref().clone())),
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
                        start.as_ref().clone()
                    } else {
                        end.as_ref().clone()
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
                // Without a block or a count, the ends answer directly, which
                // works for a range too large to walk.
                if !has_block && arguments.is_empty() && method_name != "minmax" {
                    if method_name == "min" {
                        let order = self.evaluate_binary_operation(
                            &crate::ast::BinaryOp::Spaceship,
                            start.as_ref().clone(),
                            end.as_ref().clone(),
                            position,
                        )?;
                        if matches!(order, Object::Int(value) if value > 0) {
                            return Ok(Some(Object::Nil));
                        }
                        return Ok(Some(start.as_ref().clone()));
                    }
                    let order = self.evaluate_binary_operation(
                        &crate::ast::BinaryOp::Spaceship,
                        start.as_ref().clone(),
                        end.as_ref().clone(),
                        position,
                    )?;
                    if matches!(order, Object::Int(value) if value > 0) {
                        return Ok(Some(Object::Nil));
                    }
                    if !*exclusive {
                        return Ok(Some(end.as_ref().clone()));
                    }
                    // The largest value below an exclusive integer end is the
                    // one before it, which needs no walk.
                    if let Some(last) = end.as_big_integer() {
                        if matches!(order, Object::Int(0)) {
                            return Ok(Some(Object::Nil));
                        }
                        return Ok(Some(Object::integer(last - 1)));
                    }
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                let walked = Object::array(elements);
                self.send_to_object(walked, method_name, arguments.to_vec(), position)
                    .map(Some)
            }
            // `cover?` asks whether a value falls between the ends, which
            // needs no walk, and takes another Range too.
            "cover?" => {
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
                    let starts_within =
                        self.range_covers(start, end, *exclusive, other_start, position)?;
                    let ends_within = if *other_exclusive {
                        match (end.as_ref(), other_end.as_ref()) {
                            (Object::Nil, _) => true,
                            (_, Object::Nil) => false,
                            _ => {
                                let order = self.evaluate_binary_operation(
                                    &crate::ast::BinaryOp::Spaceship,
                                    other_end.as_ref().clone(),
                                    end.as_ref().clone(),
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
                let before = self.range_ends_before(end, *exclusive, other_start, position)?;
                let after = self.range_ends_before(other_end, *other_exclusive, start, position)?;
                Ok(Some(Object::Bool(!before && !after)))
            }
            "reverse_each" => {
                let elements = self.range_elements(start, end, *exclusive, position)?;
                let walked = Object::array(elements);
                self.send_to_object(walked, "reverse_each", arguments.to_vec(), position)
                    .map(Some)
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
                    let text = Rc::new(letter.to_string());
                    if symbols {
                        Object::Symbol(text)
                    } else {
                        Object::String(text)
                    }
                })
                .collect());
        }
        let end_length = name_of(end).map(|text| text.chars().count());
        let mut walked = Vec::new();
        let mut current = start.clone();
        loop {
            // A name longer than the end's has passed it, whatever the
            // characters say: "Z".succ is "AA", which is past "z".
            if let (Some(limit), Some(text)) = (end_length, name_of(&current))
                && text.chars().count() > limit
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
fn name_of(value: &Object) -> Option<Rc<String>> {
    match value {
        Object::String(text) | Object::Symbol(text) => Some(Rc::clone(text)),
        _ => None,
    }
}

/// The code point of a name that is one ASCII character, or None otherwise.
fn single_ascii(value: &Object) -> Option<u32> {
    let text = name_of(value)?;
    let mut letters = text.chars();
    let only = letters.next()?;
    if letters.next().is_some() || !only.is_ascii() {
        return None;
    }
    Some(only as u32)
}

impl VirtualMachine {
    /// Whether a value falls between the ends of a range, without walking it.
    fn range_covers(
        &mut self,
        start: &Object,
        end: &Object,
        exclusive: bool,
        value: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if !matches!(start, Object::Nil) {
            let order = self.evaluate_binary_operation(
                &crate::ast::BinaryOp::Spaceship,
                value.clone(),
                start.clone(),
                position,
            )?;
            match order {
                Object::Int(value) if value >= 0 => {}
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
