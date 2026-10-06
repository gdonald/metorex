// Dividing, and the divisors two numbers share.

use super::*;

impl VirtualMachine {
    /// `div`, `modulo`, `fdiv`, and `remainder`, each of which divides and then
    /// reports a different part of the answer.
    pub(crate) fn integer_division_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use crate::ast::BinaryOp;
        // Every one of these divides, so an operand that cannot become a
        // number is refused up front and named the way Ruby names it.
        let numeric = matches!(
            argument,
            Object::Int(_) | Object::BigInt(_) | Object::Float(_)
        ) || crate::vm::native_methods::rational_parts(argument).is_some();
        if !numeric && !self.responds_to(argument, "coerce") {
            return Err(self.uncoercible(argument, position));
        }
        match method_name {
            // A Float divisor still answers a whole number, which is the
            // quotient rounded toward negative infinity.
            "div" => {
                // A value of the program's own answers the division itself:
                // the pair its `coerce` hands back is asked for `div` rather
                // than for `/`.
                let coerces = matches!(argument, Object::Instance(held)
                    if !matches!(held.borrow().class.name(), "Rational" | "Complex"));
                if coerces && self.answers_to(argument, "coerce", position)? {
                    let pair = self.send_to_object(
                        argument.clone(),
                        "coerce",
                        vec![receiver.clone()],
                        position,
                    )?;
                    if let Object::Array(parts) = &pair
                        && parts.borrow().len() == 2
                    {
                        let (first, second) = {
                            let held = parts.borrow();
                            (held[0].clone(), held[1].clone())
                        };
                        return self.send_to_object(first, "div", vec![second], position);
                    }
                }
                // A zero divisor is refused before the division runs, so a
                // Float zero is reported the same way an Integer one is.
                if matches!(argument, Object::Float(divisor) if *divisor == 0.0) {
                    return Err(crate::vm::errors::simple_exception(
                        "ZeroDivisionError",
                        "divided by 0",
                        position,
                    ));
                }
                let quotient = self.evaluate_binary_operation(
                    &BinaryOp::Divide,
                    receiver.clone(),
                    argument.clone(),
                    position,
                )?;
                match quotient {
                    // A NaN quotient names no number at all, which Ruby
                    // reports apart from a division by zero.
                    Object::Float(value) if value.is_nan() => Err(
                        crate::vm::errors::simple_exception("FloatDomainError", "NaN", position),
                    ),
                    Object::Float(value) if !value.is_finite() => {
                        Err(crate::vm::errors::simple_exception(
                            "ZeroDivisionError",
                            "divided by 0",
                            position,
                        ))
                    }
                    Object::Float(value) => Ok(Object::integer(
                        format!("{:.0}", value.floor()).parse().unwrap_or_default(),
                    )),
                    other => self.send_to_object(other, "floor", vec![], position),
                }
            }
            "modulo" => self.evaluate_binary_operation(
                &BinaryOp::Modulo,
                receiver.clone(),
                argument.clone(),
                position,
            ),
            "fdiv" => {
                // Two integers divide with the extra bits their widths carry,
                // so a pair of bignums answers the ratio rather than the
                // infinity each would round to on its own.
                if let (Some(value), Some(divisor)) =
                    (receiver.as_big_integer(), argument.as_big_integer())
                {
                    return Ok(Object::Float(exact_ratio(&value, &divisor)));
                }
                let divisor = self.float_value_of(argument, position)?;
                let value = match receiver {
                    Object::BigInt(value) => crate::vm::operators::big_to_float(value),
                    Object::Int(value) => *value as f64,
                    _ => unreachable!("caller restricts the receiver to integers"),
                };
                Ok(Object::Float(value / divisor))
            }
            // `remainder` truncates the division where `modulo` floors it, so
            // what is left carries the sign of the receiver rather than the
            // sign of the divisor. The two differ by one divisor.
            _ => {
                // Two integers divide exactly, and truncating that division is
                // the remainder Rust's own `%` leaves.
                if let (Some(value), Some(divisor)) =
                    (receiver.as_big_integer(), argument.as_big_integer())
                {
                    if divisor == num_bigint::BigInt::from(0) {
                        return Err(crate::vm::errors::simple_exception(
                            "ZeroDivisionError",
                            "divided by 0",
                            position,
                        ));
                    }
                    return Ok(Object::integer(value % divisor));
                }
                // A Rational divides exactly, so the truncated quotient it
                // answers gives the remainder directly.
                if crate::vm::native_methods::rational_parts(argument).is_some() {
                    let quotient = self.evaluate_binary_operation(
                        &BinaryOp::Divide,
                        receiver.clone(),
                        argument.clone(),
                        position,
                    )?;
                    let truncated = self.send_to_object(quotient, "truncate", vec![], position)?;
                    let product = self.evaluate_binary_operation(
                        &BinaryOp::Multiply,
                        argument.clone(),
                        truncated,
                        position,
                    )?;
                    return self.evaluate_binary_operation(
                        &BinaryOp::Subtract,
                        receiver.clone(),
                        product,
                        position,
                    );
                }
                let modulus = self.evaluate_binary_operation(
                    &BinaryOp::Modulo,
                    receiver.clone(),
                    argument.clone(),
                    position,
                )?;
                let differing_signs = self.numbers_differ_in_sign(receiver, argument, position)?;
                let zero = self.evaluate_binary_operation(
                    &BinaryOp::Equal,
                    modulus.clone(),
                    Object::Int(0),
                    position,
                )?;
                if differing_signs && !zero.is_truthy() {
                    return self.evaluate_binary_operation(
                        &BinaryOp::Subtract,
                        modulus,
                        argument.clone(),
                        position,
                    );
                }
                Ok(modulus)
            }
        }
    }

    /// Whether two numbers sit on opposite sides of zero, which is what tells
    /// a truncated remainder apart from a floored modulus.
    fn numbers_differ_in_sign(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        use crate::ast::BinaryOp;
        let negative = |vm: &mut Self, value: &Object| -> Result<bool, MetorexError> {
            let answer = vm.evaluate_binary_operation(
                &BinaryOp::Less,
                value.clone(),
                Object::Int(0),
                position,
            )?;
            Ok(answer.is_truthy())
        };
        Ok(negative(self, left)? != negative(self, right)?)
    }

    /// A divisor as a Float, which is what `fdiv` divides by. Anything that is
    /// not a number is refused the way the operators refuse it.
    pub(crate) fn float_value_of(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        match value {
            Object::Int(number) => Ok(*number as f64),
            Object::BigInt(number) => Ok(crate::vm::operators::big_to_float(number)),
            Object::Float(number) => Ok(*number),
            // A Rational answers its own Float value, and anything else is
            // asked to coerce before Ruby gives up on it. A core value such
            // as nil or a String is not a number whatever `to_f` it has.
            other @ Object::Instance(_) if self.responds_to(other, "to_f") => {
                match self.send_to_object(other.clone(), "to_f", vec![], position)? {
                    Object::Float(number) => Ok(number),
                    Object::Int(number) => Ok(number as f64),
                    _ => Err(self.uncoercible(other, position)),
                }
            }
            other => match self.coerced_float(other, position)? {
                Some(number) => Ok(number),
                None => Err(self.uncoercible(other, position)),
            },
        }
    }
}

/// The greatest common divisor of two integers, always positive, which is what
/// `gcd` reports whatever signs it was handed.
pub(crate) fn big_gcd(left: num_bigint::BigInt, right: num_bigint::BigInt) -> num_bigint::BigInt {
    let (mut left, mut right) = (absolute(left), absolute(right));
    let zero = num_bigint::BigInt::from(0);
    while right != zero {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    left
}

/// The magnitude of an integer, which `num_bigint` leaves to a trait the rest
/// of this module does not import.
pub(crate) fn absolute(value: num_bigint::BigInt) -> num_bigint::BigInt {
    if value < num_bigint::BigInt::from(0) {
        -value
    } else {
        value
    }
}

/// Two integers divided with the precision their widths carry: the quotient is
/// taken with extra bits and then scaled back, so a pair of bignums that would
/// each round to infinity still answers the ratio between them.
pub(crate) fn exact_ratio(value: &num_bigint::BigInt, divisor: &num_bigint::BigInt) -> f64 {
    use crate::vm::operators::big_to_float;
    let zero = num_bigint::BigInt::from(0);
    if *divisor == zero {
        return match value.cmp(&zero) {
            std::cmp::Ordering::Greater => f64::INFINITY,
            std::cmp::Ordering::Less => f64::NEG_INFINITY,
            std::cmp::Ordering::Equal => f64::NAN,
        };
    }
    if *value == zero {
        return 0.0;
    }
    // Shift the numerator until the quotient carries a Float's worth of bits,
    // whatever the two magnitudes are, then scale the answer back down.
    const SIGNIFICANT_BITS: i64 = 64;
    let value_bits = value.bits() as i64;
    let divisor_bits = divisor.bits() as i64;
    let shift = (SIGNIFICANT_BITS + divisor_bits - value_bits).max(0);
    let scaled = (value << shift as u32) / divisor;
    crate::vm::native_methods::scale_by_power_of_two(big_to_float(&scaled), -shift)
}
