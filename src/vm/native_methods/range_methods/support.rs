// The values a range is built from, and how they step.

use super::*;

impl VirtualMachine {
    /// The values a range walks: integers count up, and anything else follows
    /// `succ` until it passes the end.
    /// Whether a range holds no value at all, which is the case when it
    /// begins past its end, or ends where it begins and leaves that out.
    pub(crate) fn range_holds_nothing(
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

/// The first value of a range that counts up from an Integer and never
/// reaches an end, which is walked one value at a time rather than collected.
/// The first value of a String range with no end, which `each` walks with
/// `succ` for as long as the block keeps asking.
pub(crate) fn endless_string_start(start: &Object, end: &Object) -> Option<String> {
    if !matches!(end, Object::Nil) {
        return None;
    }
    match start {
        Object::String(text) => Some(text.as_str().to_string()),
        _ => None,
    }
}

pub(crate) fn endless_int_start(start: &Object, end: &Object) -> Option<i64> {
    let endless = matches!(end, Object::Nil)
        || matches!(end, Object::Float(value) if value.is_infinite() && *value > 0.0);
    match (endless, start) {
        (true, Object::Int(first)) => Some(*first),
        _ => None,
    }
}

/// The characters a String or Symbol is named with.
pub(crate) fn name_of(value: &Object) -> Option<Rc<crate::object::StringValue>> {
    match value {
        Object::String(text) | Object::Symbol(text) => Some(Rc::clone(text)),
        _ => None,
    }
}

/// The code point of a name that is one ASCII character, or None otherwise.
pub(crate) fn single_ascii(value: &Object) -> Option<u32> {
    let text = name_of(value)?;
    let held = text.to_text();
    let mut letters = held.chars();
    let only = letters.next()?;
    if letters.next().is_some() || !only.is_ascii() {
        return None;
    }
    Some(only as u32)
}
