// Rounding to a precision, and which way a value between two
// multiples goes.

use super::*;

impl VirtualMachine {
    /// `Integer#ceil` / `#floor` / `#truncate` / `#round`, which differ only in
    /// where they send a value that sits between two multiples. A precision of
    /// zero or more leaves an Integer alone, since it has no digits to drop.
    pub(crate) fn round_to_precision(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // `round` takes a `half:` keyword naming where a value exactly between
        // two multiples goes. The others take the precision alone.
        let (arguments, half) = match method_name {
            "round" => split_rounding_mode(arguments),
            _ => (arguments, None),
        };
        let half = self.rounding_mode(half, position)?;
        if arguments.len() > 1 {
            return Err(method_argument_error(
                method_name,
                1,
                arguments.len(),
                position,
            ));
        }
        let value = receiver.as_big_integer().expect("integer-kinded");
        let Some(argument) = arguments.first() else {
            return Ok(Object::integer(value));
        };
        let precision = self.coerce_precision_argument(argument, position)?;
        if precision >= 0 {
            return Ok(Object::integer(value));
        }
        let Some(digits) = precision
            .checked_neg()
            .and_then(|digits| u32::try_from(digits).ok())
        else {
            let message = format!("integer {} too small to convert to `int'", precision);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("RangeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let step = num_bigint::BigInt::from(10).pow(digits);
        let remainder = &value % &step;
        let zero = num_bigint::BigInt::from(0);
        if remainder == zero {
            return Ok(Object::integer(value));
        }
        let floor = &value
            - if remainder < zero {
                &remainder + &step
            } else {
                remainder.clone()
            };
        let answer = match method_name {
            "floor" => floor,
            "ceil" => floor + &step,
            "truncate" => {
                if value < zero {
                    floor + &step
                } else {
                    floor
                }
            }
            // A value sitting exactly between two multiples goes where the
            // `half:` keyword says, and away from zero when it says nothing.
            _ => {
                let doubled: num_bigint::BigInt = (&value - &floor) * 2;
                let up = floor.clone() + &step;
                match doubled.cmp(&step) {
                    std::cmp::Ordering::Less => floor,
                    std::cmp::Ordering::Greater => up,
                    std::cmp::Ordering::Equal => match half {
                        RoundingMode::Down => {
                            if value < zero {
                                up
                            } else {
                                floor
                            }
                        }
                        RoundingMode::Even => {
                            let multiple = &floor / &step;
                            if &multiple % 2 == zero { floor } else { up }
                        }
                        RoundingMode::Up => {
                            if value < zero {
                                floor
                            } else {
                                up
                            }
                        }
                    },
                }
            }
        };
        Ok(Object::integer(answer))
    }

    /// The `half:` keyword: `:up` sends a value between two multiples away
    /// from zero, `:down` toward it, and `:even` to the even multiple. Ruby
    /// names any other mode in an ArgumentError.
    pub(crate) fn rounding_mode(
        &mut self,
        half: Option<Object>,
        position: Position,
    ) -> Result<RoundingMode, MetorexError> {
        let named = match half {
            None | Some(Object::Nil) => return Ok(RoundingMode::Up),
            Some(Object::Symbol(name) | Object::String(name)) => name.as_str().to_string(),
            Some(other) => self.coerce_name_argument(&other, position)?,
        };
        match named.as_str() {
            "up" => Ok(RoundingMode::Up),
            "down" => Ok(RoundingMode::Down),
            "even" => Ok(RoundingMode::Even),
            _ => {
                let message = format!("invalid rounding mode: {}", named);
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
        }
    }
}

/// Where `round` sends a value sitting exactly between two multiples.
#[derive(Clone, Copy)]
pub(crate) enum RoundingMode {
    Up,
    Down,
    Even,
}

/// Peel the `half:` keyword off a `round` argument list.
pub(crate) fn split_rounding_mode(arguments: &[Object]) -> (&[Object], Option<Object>) {
    let Some(Object::Dict(entries)) = arguments.last() else {
        return (arguments, None);
    };
    let entries = entries.borrow();
    let named: Vec<&String> = entries
        .keys()
        .filter(|key| key.as_str() != crate::vm::param_binding::KWARGS_MARKER)
        .collect();
    if named.len() != 1 || named[0].as_str() != ":half" {
        return (arguments, None);
    }
    let half = entries.get(":half").cloned();
    (&arguments[..arguments.len() - 1], half)
}
