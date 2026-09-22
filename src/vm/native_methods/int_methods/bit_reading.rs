// The bits `Integer#[]` names.

use super::*;

impl VirtualMachine {
    /// The bits `Integer#[]` names: one by position, a run from a position
    /// and a length, or the run a Range spans.
    pub(crate) fn integer_bits_at(
        &mut self,
        value: &num_bigint::BigInt,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        match arguments.len() {
            1 => {
                let Some(Object::Range {
                    start,
                    end,
                    exclusive,
                    ..
                }) = crate::vm::native_methods::as_range(&arguments[0])
                else {
                    let index = self.bit_position_argument(&arguments[0], position)?;
                    return Ok(from_big(single_bit(value, &index)));
                };
                let opening = match start.as_ref() {
                    Object::Nil => None,
                    held => Some(self.bit_position_argument(held, position)?),
                };
                let closing = match end.as_ref() {
                    Object::Nil => None,
                    held => Some(self.bit_position_argument(held, position)?),
                };
                match (opening, closing) {
                    // A range with no lower bound reaches every bit below the
                    // upper one, which names a number at all only when they
                    // are all zero.
                    (None, Some(last)) => {
                        let width = if exclusive { last.clone() } else { last + 1 };
                        let mask = mask_of(&width);
                        if value & &mask != num_bigint::BigInt::from(0) {
                            let message = "The beginless range for Integer#[] results in infinity"
                                .to_string();
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &message,
                                position,
                            ));
                        }
                        Ok(Object::Int(0))
                    }
                    (Some(first), None) => Ok(from_big(shifted_right(value, &first))),
                    (Some(first), Some(last)) => {
                        let width = if exclusive {
                            &last - &first
                        } else {
                            &last - &first + 1
                        };
                        Ok(from_big(masked_run(value, &first, &width)))
                    }
                    (None, None) => Ok(from_big(value.clone())),
                }
            }
            2 => {
                let first = self.bit_position_argument(&arguments[0], position)?;
                let width = self.bit_position_argument(&arguments[1], position)?;
                Ok(from_big(masked_run(value, &first, &width)))
            }
            given => Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(1, 2),
                given,
                position,
            )),
        }
    }

    /// A bit position, which a Float names by its whole part and anything
    /// else through `to_int`. An endless Float names no position at all.
    fn bit_position_argument(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<num_bigint::BigInt, MetorexError> {
        if let Object::Float(value) = argument {
            if value.is_infinite() {
                let message = format!("{}Infinity", if *value < 0.0 { "-" } else { "" });
                return Err(crate::vm::errors::simple_exception(
                    "FloatDomainError",
                    &message,
                    position,
                ));
            }
            if value.is_nan() {
                return Err(crate::vm::errors::simple_exception(
                    "FloatDomainError",
                    "NaN",
                    position,
                ));
            }
            return Ok(float_to_big(value.trunc()));
        }
        self.coerce_integer_argument(argument, position)
    }
}

/// `1 << width`, less one: the mask a run of that many bits reads through.
fn mask_of(width: &num_bigint::BigInt) -> num_bigint::BigInt {
    let Ok(narrow) = u64::try_from(width) else {
        return num_bigint::BigInt::from(0);
    };
    (num_bigint::BigInt::from(1) << narrow as usize) - 1
}

/// The value shifted so the named bit sits lowest. A negative position shifts
/// the other way, which is what reaches the bits above the ones written.
fn shifted_right(value: &num_bigint::BigInt, by: &num_bigint::BigInt) -> num_bigint::BigInt {
    let zero = num_bigint::BigInt::from(0);
    if *by < zero {
        let Ok(narrow) = u64::try_from(&-by) else {
            return zero;
        };
        return value << narrow as usize;
    }
    match u64::try_from(by) {
        Ok(narrow) if narrow < 1 << 20 => value >> narrow as usize,
        // Past the width of the value every bit reads as the sign bit.
        _ => {
            if *value < zero {
                num_bigint::BigInt::from(-1)
            } else {
                zero
            }
        }
    }
}

/// The single bit at a position, where a negative position names none.
pub(crate) fn single_bit(
    value: &num_bigint::BigInt,
    at: &num_bigint::BigInt,
) -> num_bigint::BigInt {
    let zero = num_bigint::BigInt::from(0);
    if *at < zero {
        return zero;
    }
    shifted_right(value, at) & num_bigint::BigInt::from(1)
}

/// A run of bits starting at a position. A width below one reads every bit
/// from there up, the way Ruby ignores a negative length.
fn masked_run(
    value: &num_bigint::BigInt,
    from: &num_bigint::BigInt,
    width: &num_bigint::BigInt,
) -> num_bigint::BigInt {
    let shifted = shifted_right(value, from);
    if *width < num_bigint::BigInt::from(1) {
        return shifted;
    }
    shifted & mask_of(width)
}
