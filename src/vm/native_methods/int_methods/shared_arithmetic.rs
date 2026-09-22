// The methods that read the same whichever width the receiver is.

use super::*;

impl VirtualMachine {
    /// The methods that read the same whichever width the receiver is.
    pub(crate) fn call_int_shared_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Integer#size — the bytes the machine representation takes: a word
        // for a number that fits in one, and the bytes a wider one needs.
        if method_name == "size" && matches!(receiver, Object::Int(_) | Object::BigInt(_)) {
            if !arguments.is_empty() {
                return Err(method_argument_error(
                    method_name,
                    0,
                    arguments.len(),
                    position,
                ));
            }
            let magnitude = absolute(receiver.as_big_integer().expect("integer-kinded"));
            let bytes = magnitude.to_bytes_be().1.len().max(1);
            return Ok(Some(Object::Int(std::cmp::max(bytes as i64, 8))));
        }
        // Integer#digits — the place values of the number in a given base,
        // least significant first.
        if method_name == "digits" && matches!(receiver, Object::Int(_) | Object::BigInt(_)) {
            if arguments.len() > 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let base = match arguments.first() {
                Some(argument) => self.coerce_integer_argument(argument, position)?,
                None => num_bigint::BigInt::from(10),
            };
            return self.integer_digits(receiver, base, position).map(Some);
        }
        // Integer#coerce — the pair Ruby applies an operator to, which is two
        // Integers when the argument is one and two Floats otherwise.
        if method_name == "coerce" && matches!(receiver, Object::Int(_) | Object::BigInt(_)) {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let value = receiver.as_big_integer().expect("integer-kinded");
            if let Some(other) = arguments[0].as_big_integer() {
                return Ok(Some(Object::array(vec![
                    Object::integer(other),
                    Object::integer(value),
                ])));
            }
            let other = self.coerce_to_float(&arguments[0], position)?;
            return Ok(Some(Object::array(vec![
                Object::Float(other),
                Object::Float(crate::vm::operators::big_to_float(&value)),
            ])));
        }
        // Integer#divmod — the floored quotient and the modulus that pairs
        // with it, whatever the widths of the two numbers.
        if method_name == "divmod" && matches!(receiver, Object::Int(_) | Object::BigInt(_)) {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let quotient =
                self.integer_division_method(receiver, "div", &arguments[0], position)?;
            let modulus =
                self.integer_division_method(receiver, "modulo", &arguments[0], position)?;
            return Ok(Some(Object::array(vec![quotient, modulus])));
        }
        // Integer#div / #modulo / #fdiv / #remainder — the division operators
        // under the names Ruby also gives them, with `fdiv` always answering a
        // Float and `remainder` taking the sign of the receiver.
        // `ceildiv` rounds the quotient up, which is the floored division of
        // the negated receiver, negated back.
        if method_name == "ceildiv" && matches!(receiver, Object::Int(_) | Object::BigInt(_)) {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let negated = Object::integer(-receiver.as_big_integer().expect("integer-kinded"));
            let quotient =
                self.integer_division_method(&negated, "div", &arguments[0], position)?;
            let quotient = quotient.as_big_integer().ok_or_else(|| {
                MetorexError::runtime_error(
                    "ceildiv expected an integer quotient",
                    crate::vm::utils::position_to_location(position),
                )
            })?;
            return Ok(Some(Object::integer(-quotient)));
        }
        if matches!(method_name, "div" | "modulo" | "fdiv" | "remainder")
            && matches!(receiver, Object::Int(_) | Object::BigInt(_))
        {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            return self
                .integer_division_method(receiver, method_name, &arguments[0], position)
                .map(Some);
        }
        // Integer#gcd / #lcm / #gcdlcm — the divisors two integers share, and
        // the smallest multiple they agree on. Both answers are positive
        // whatever the signs of the two numbers.
        if matches!(method_name, "gcd" | "lcm" | "gcdlcm")
            && matches!(receiver, Object::Int(_) | Object::BigInt(_))
        {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            // Ruby takes an Integer here and nothing else, so a Float is
            // refused rather than truncated.
            let other = match &arguments[0] {
                argument @ (Object::Int(_) | Object::BigInt(_)) => {
                    argument.as_big_integer().expect("integer-kinded")
                }
                other => {
                    let message = format!(
                        "not an integer: {}",
                        crate::vm::native_methods::array_methods::inspect_element(other)
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
            };
            let value = receiver.as_big_integer().expect("integer-kinded");
            let divisor = big_gcd(value.clone(), other.clone());
            let zero = num_bigint::BigInt::from(0);
            let multiple = if divisor == zero {
                zero
            } else {
                absolute(&value * &other / &divisor)
            };
            return Ok(Some(match method_name {
                "gcd" => Object::integer(divisor),
                "lcm" => Object::integer(multiple),
                _ => Object::array(vec![Object::integer(divisor), Object::integer(multiple)]),
            }));
        }
        // Integer#ceil / #floor / #truncate / #round — a precision of zero or
        // more answers the receiver itself, and a negative one rounds to a
        // multiple of that power of ten.
        if matches!(method_name, "ceil" | "floor" | "truncate" | "round")
            && matches!(receiver, Object::Int(_) | Object::BigInt(_))
        {
            return self
                .round_to_precision(receiver, method_name, arguments, position)
                .map(Some);
        }
        Ok(None)
    }
}
