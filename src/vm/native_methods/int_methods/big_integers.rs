// The methods a number past the machine word answers.

use super::*;

impl VirtualMachine {
    /// The Integer methods that have an exact answer for a value past the
    /// i64 range. `times`, `upto`, and `downto` are absent: a loop over that
    /// many values never finishes, so nothing here can stand in for one.
    pub(crate) fn call_big_int_method(
        &mut self,
        value: &Rc<num_bigint::BigInt>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        use num_bigint::BigInt;
        let no_arguments = |expected: usize| -> Result<(), MetorexError> {
            if arguments.len() == expected {
                Ok(())
            } else {
                Err(method_argument_error(
                    method_name,
                    expected,
                    arguments.len(),
                    position,
                ))
            }
        };
        match method_name {
            "to_s" | "inspect" if !arguments.is_empty() => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::Int(base) = arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                if !(2..=36).contains(&base) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("invalid radix {base}"),
                        position,
                    ));
                }
                Ok(Some(ascii_string(value.to_str_radix(base as u32))))
            }
            "to_s" | "inspect" => {
                no_arguments(0)?;
                Ok(Some(ascii_string(value.to_string())))
            }
            "to_i" | "to_int" | "ord" => Ok(Some(Object::BigInt(Rc::clone(value)))),
            "to_f" => {
                no_arguments(0)?;
                Ok(Some(Object::Float(big_to_float(value))))
            }
            "abs" | "magnitude" => {
                no_arguments(0)?;
                Ok(Some(Object::integer(if **value < BigInt::from(0) {
                    -(**value).clone()
                } else {
                    (**value).clone()
                })))
            }
            // The unary operators under the names `send` reaches them by.
            "-@" | "+@" => {
                no_arguments(0)?;
                Ok(Some(match method_name {
                    "-@" => Object::integer(-(**value).clone()),
                    _ => Object::integer((**value).clone()),
                }))
            }
            "~" => {
                no_arguments(0)?;
                Ok(Some(Object::integer(!(**value).clone())))
            }
            "bit_length" => {
                no_arguments(0)?;
                // The magnitude's bit count, which is what Ruby reports for
                // a positive value and for a negative one alike.
                let magnitude = if **value < BigInt::from(0) {
                    !(**value).clone()
                } else {
                    (**value).clone()
                };
                Ok(Some(Object::Int(magnitude.bits() as i64)))
            }
            "<<" | ">>" => {
                no_arguments(1)?;
                let count = self.shift_count(method_name, &arguments[0], position)?;
                let Some(place) = count.written else {
                    let zero = **value == BigInt::from(0);
                    let empties_it = zero || (method_name == "<<") == count.negative;
                    if empties_it {
                        let sign = if **value < BigInt::from(0) { -1 } else { 0 };
                        return Ok(Some(Object::Int(sign)));
                    }
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "shift width too big",
                        position,
                    ));
                };
                let count = place;
                let (shift_left, count) = if count < 0 {
                    (method_name == ">>", count.unsigned_abs())
                } else {
                    (method_name == "<<", count as u64)
                };
                let shifted = if shift_left {
                    (**value).clone() << count
                } else {
                    (**value).clone() >> count
                };
                Ok(Some(Object::integer(shifted)))
            }
            // A number this wide names no character in any encoding.
            "chr" => Err(crate::vm::errors::simple_exception(
                "RangeError",
                "bignum out of char range",
                position,
            )),
            "divmod" => {
                no_arguments(1)?;
                let Some(divisor) = arguments[0].as_big_integer() else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                if divisor == BigInt::from(0) {
                    return Err(divide_by_zero_error(position));
                }
                // Ruby floors the quotient, so the modulus follows the sign
                // of the divisor rather than of the dividend.
                let mut quotient = (**value).clone() / &divisor;
                let mut modulus = (**value).clone() % &divisor;
                if modulus != BigInt::from(0)
                    && (modulus < BigInt::from(0)) != (divisor < BigInt::from(0))
                {
                    quotient -= 1;
                    modulus += &divisor;
                }
                Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(vec![
                    Object::integer(quotient),
                    Object::integer(modulus),
                ])))))
            }
            "zero?" => {
                no_arguments(0)?;
                Ok(Some(Object::Bool(**value == BigInt::from(0))))
            }
            "positive?" => {
                no_arguments(0)?;
                Ok(Some(Object::Bool(**value > BigInt::from(0))))
            }
            "negative?" => {
                no_arguments(0)?;
                Ok(Some(Object::Bool(**value < BigInt::from(0))))
            }
            "even?" | "odd?" => {
                no_arguments(0)?;
                let even = (**value).clone() % BigInt::from(2) == BigInt::from(0);
                Ok(Some(Object::Bool(if method_name == "even?" {
                    even
                } else {
                    !even
                })))
            }
            "succ" | "next" => {
                no_arguments(0)?;
                Ok(Some(Object::integer((**value).clone() + 1)))
            }
            "pred" => {
                no_arguments(0)?;
                Ok(Some(Object::integer((**value).clone() - 1)))
            }
            "hash" => {
                no_arguments(0)?;
                Ok(Some(Object::string(value.to_string())))
            }
            "integer?" => {
                no_arguments(0)?;
                Ok(Some(Object::Bool(true)))
            }
            // Exact division, which answers a Rational for a whole divisor
            // and a Float for one that is not.
            "quo" => {
                no_arguments(1)?;
                if matches!(arguments[0], Object::Float(_)) {
                    let left = value.to_string().parse::<f64>().unwrap_or(f64::NAN);
                    return self
                        .send_to_object(
                            Object::Float(left),
                            "/",
                            vec![arguments[0].clone()],
                            position,
                        )
                        .map(Some);
                }
                let divisor = self.coerce_integer_argument(&arguments[0], position)?;
                if divisor == BigInt::from(0) {
                    return Err(crate::vm::errors::divide_by_zero_error(position));
                }
                self.make_rational((**value).clone(), divisor, position)
                    .map(Some)
            }
            _ => Ok(None),
        }
    }
}

/// Euclid's algorithm, used to put a Rational in lowest terms.
pub(crate) fn greatest_common_divisor(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    if a == 0 { 1 } else { a }
}

/// Shift `value` by `count` bits. A count at or past the width of an i64
/// leaves 0 for a left shift and the sign bit for a right one, which is where
/// the exact result rounds to at that magnitude.
pub(crate) fn shift_integer(value: i64, shift_left: bool, count: u64) -> i64 {
    if count >= i64::BITS as u64 {
        return if shift_left {
            0
        } else if value < 0 {
            -1
        } else {
            0
        };
    }
    if shift_left {
        value.wrapping_shl(count as u32)
    } else {
        value >> count
    }
}

/// The nearest Float to an arbitrary-precision integer.
fn big_to_float(value: &num_bigint::BigInt) -> f64 {
    use std::str::FromStr;
    f64::from_str(&value.to_string()).unwrap_or(f64::INFINITY)
}

/// The whole number an Integer-kinded BigInt stands for, narrowed where it
/// fits so arithmetic on it stays on the fixnum path.
pub(crate) fn from_big(value: num_bigint::BigInt) -> Object {
    match i64::try_from(&value) {
        Ok(narrow) => Object::Int(narrow),
        Err(_) => Object::BigInt(std::rc::Rc::new(value)),
    }
}

/// A Float's whole part as an arbitrary-width integer.
pub(crate) fn float_to_big(value: f64) -> num_bigint::BigInt {
    use num_bigint::ToBigInt;
    value
        .to_bigint()
        .unwrap_or_else(|| num_bigint::BigInt::from(0))
}
