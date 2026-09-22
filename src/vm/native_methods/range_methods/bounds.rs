// The smallest and largest a range holds, and whether it covers a value.

use super::*;

impl VirtualMachine {
    /// The smallest and largest a range holds, and whether it covers a value.
    pub(crate) fn call_range_bounds_method(
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
            ..
        } = receiver
        else {
            return Ok(None);
        };
        match method_name {
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
                    ..
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
                    ..
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
            _ => Ok(None),
        }
    }
}
