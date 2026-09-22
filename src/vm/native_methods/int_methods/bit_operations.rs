// Dividing with a remainder, shifting, and the width a number reads in.

use super::*;

impl VirtualMachine {
    /// Dividing with a remainder, shifting, and the width a number reads in.
    pub(crate) fn call_int_bit_operation(
        &mut self,
        _receiver: &Object,
        n: &i64,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // Integer#divmod — the floored quotient and the modulus, as a
            // two-element Array. The signs follow the divisor, as Ruby's do.
            "divmod" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::Int(0) => {
                        let message = "divided by 0".to_string();
                        Err(MetorexError::UncaughtException {
                            exception: Object::exception("ZeroDivisionError", message.clone()),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        })
                    }
                    Object::Int(divisor) => {
                        let quotient = n.div_euclid(*divisor);
                        let remainder = n - quotient * divisor;
                        // `div_euclid` floors toward zero for a negative
                        // divisor, so correct it back to Ruby's floor.
                        let (quotient, remainder) =
                            if remainder != 0 && (remainder < 0) != (*divisor < 0) {
                                (quotient - 1, remainder + divisor)
                            } else {
                                (quotient, remainder)
                            };
                        Ok(Some(Object::Array(std::rc::Rc::new(
                            std::cell::RefCell::new(vec![
                                Object::Int(quotient),
                                Object::Int(remainder),
                            ]),
                        ))))
                    }
                    // Ruby answers an Integer quotient and a Float modulus
                    // when the divisor is a Float.
                    Object::Float(divisor) => {
                        let value = *n as f64;
                        let quotient = (value / divisor).floor();
                        Ok(Some(Object::Array(std::rc::Rc::new(
                            std::cell::RefCell::new(vec![
                                Object::Int(quotient as i64),
                                Object::Float(value - quotient * divisor),
                            ]),
                        ))))
                    }
                    other => Err(method_argument_type_error(
                        method_name,
                        "Integer or Float",
                        other,
                        position,
                    )),
                }
            }
            // Bit operations. A shift count past the width of the value
            // saturates rather than wrapping, which is what an arbitrary
            // precision result would round to at these magnitudes.
            "<<" | ">>" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let count = self.shift_count(method_name, &arguments[0], position)?;
                // A count wider than any machine word shifts everything out,
                // or fills with the sign, depending on which way it goes.
                let Some(place) = count.written else {
                    // Shifting zero leaves zero however far it goes.
                    let empties_it = *n == 0 || (method_name == "<<") == count.negative;
                    if empties_it {
                        return Ok(Some(Object::Int(if *n < 0 { -1 } else { 0 })));
                    }
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "shift width too big",
                        position,
                    ));
                };
                let count = place;
                // A negative count shifts the other way, as Ruby's does.
                let (shift_left, count) = if count < 0 {
                    (method_name == ">>", count.unsigned_abs())
                } else {
                    (method_name == "<<", count as u64)
                };
                // A left shift is exact in Ruby, so a result too wide for an
                // i64 keeps its value rather than dropping the high bits.
                if shift_left && count < 1_000_000 {
                    let widened = num_bigint::BigInt::from(*n) << count as usize;
                    return Ok(Some(Object::integer(widened)));
                }
                Ok(Some(Object::Int(shift_integer(*n, shift_left, count))))
            }
            "~" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(!*n)))
            }
            "bit_length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let magnitude = if *n < 0 { !*n } else { *n };
                Ok(Some(Object::Int(
                    (i64::BITS - magnitude.leading_zeros()) as i64,
                )))
            }
            _ => Ok(None),
        }
    }
}
