// The halving search `bsearch` runs.

use super::*;

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

    pub(crate) fn range_covers(
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
    pub(crate) fn range_ends_before(
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

    /// Halve the range until the block settles on an element, answering nil
    /// when it never does.
    pub(crate) fn binary_search(
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
    pub(crate) fn whole_search_limits(
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
    pub(crate) fn halve(
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
    pub(crate) fn ask_block(
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

/// The first whole value at or above a numeric range's start, which is what
/// counting one begins from.
pub(crate) fn numeric_floor(value: &Object) -> Option<i64> {
    match value {
        Object::Int(number) => Some(*number),
        Object::BigInt(number) => i64::try_from(number.as_ref()).ok(),
        Object::Float(number) => Some(number.ceil() as i64),
        _ => None,
    }
}

/// The last whole value at or below a numeric range's end.
pub(crate) fn numeric_ceiling(value: &Object) -> Option<i64> {
    match value {
        Object::Int(number) => Some(*number),
        Object::BigInt(number) => i64::try_from(number.as_ref()).ok(),
        Object::Float(number) => Some(number.floor() as i64),
        _ => None,
    }
}

/// Whether an end is a whole number, which decides whether an exclusive range
/// loses its last value.
pub(crate) fn end_is_whole(value: &Object) -> bool {
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
pub(crate) fn place_by_place(low: &str, high: &str, held: &str, exclusive: bool) -> Option<bool> {
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
pub(crate) fn counts_as_a_number(held: &Object) -> bool {
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
pub(crate) enum SearchBounds {
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
pub(crate) struct SearchAnswer {
    smaller: bool,
    settled: Option<Object>,
    satisfied: bool,
}

impl SearchBounds {
    /// The bounds a range names, or the TypeError a range of something other
    /// than numbers raises.
    pub(crate) fn of(
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
pub(crate) fn refuse_binary_search(named: &str, position: Position) -> MetorexError {
    let message = format!("can't do binary search for {}", named);
    crate::vm::errors::simple_exception("TypeError", &message, position)
}

/// A Float as the whole number its bits stand for, in the same order the
/// Floats themselves are in, so a search can halve the gap between two.
pub(crate) fn fraction_as_whole(value: f64) -> i64 {
    let bits = value.to_bits() as i64;
    if bits < 0 { i64::MIN - bits } else { bits }
}

/// The Float a whole number from `fraction_as_whole` stands for.
pub(crate) fn whole_as_fraction(value: i64) -> f64 {
    let bits = if value < 0 { i64::MIN - value } else { value };
    f64::from_bits(bits as u64)
}
