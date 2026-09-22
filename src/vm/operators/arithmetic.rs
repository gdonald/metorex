// Adding, subtracting, multiplying and dividing.

use super::*;

impl VirtualMachine {
    /// Handle addition across supported operand types.
    pub(crate) fn evaluate_addition(
        &self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match (left, right) {
            (Object::Int(a), Object::Int(b)) => match a.checked_add(b) {
                Some(v) => Ok(Object::Int(v)),
                // Too large for an i64, so carry on in arbitrary precision.
                None => Ok(Object::integer(
                    num_bigint::BigInt::from(a) + num_bigint::BigInt::from(b),
                )),
            },
            (
                ref left @ (Object::Int(_) | Object::BigInt(_)),
                ref right @ (Object::Int(_) | Object::BigInt(_)),
            ) => {
                let (a, b) = (
                    left.as_big_integer().expect("integer-kinded"),
                    right.as_big_integer().expect("integer-kinded"),
                );
                Ok(Object::integer(a + b))
            }
            (Object::BigInt(a), Object::Float(b)) => Ok(Object::Float(big_to_float(&a) + b)),
            (Object::Float(a), Object::BigInt(b)) => Ok(Object::Float(a + big_to_float(&b))),
            (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a + b)),
            (Object::Int(a), Object::Float(b)) => Ok(Object::Float((a as f64) + b)),
            (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a + (b as f64))),
            (Object::String(a), Object::String(b)) => {
                // Where either side stands for the bytes it was read from,
                // the answer does too, and the other side is written out in
                // the bytes its own encoding spells it with.
                let holds_bytes = a.holds_bytes() || b.holds_bytes();
                let joined = if holds_bytes {
                    let mut bytes =
                        crate::vm::native_methods::string_methods::binary_bytes(a.as_ref());
                    bytes.extend(crate::vm::native_methods::string_methods::binary_bytes(
                        b.as_ref(),
                    ));
                    let made = crate::object::StringValue::from_bytes(
                        crate::vm::native_methods::pack_format::bytes_to_string(&bytes).to_string(),
                    );
                    Object::String(Rc::new(made))
                } else {
                    let mut combined = a.as_str().to_string();
                    combined.push_str(&b.as_ref().as_str());
                    Object::string(combined)
                };
                // A run recording where each object was made records this
                // one at the place the two were joined.
                if let Object::String(made) = &joined
                    && let Some(written_at) = self.literal_birthplace(position)
                {
                    made.set_created_at(written_at);
                }
                // The result is written in the receiver's encoding, unless
                // the receiver is empty or nothing but ASCII and the other
                // side is not, where that side's reading carries over.
                if let Object::String(made) = &joined {
                    let plain = |side: &crate::object::StringValue| {
                        side.as_str().is_empty()
                            || (side.as_str().is_ascii() && !side.holds_bytes())
                    };
                    let carried = if plain(a.as_ref()) && !plain(b.as_ref()) {
                        b.as_ref()
                    } else {
                        a.as_ref()
                    };
                    made.set_encoding(carried.encoding_name());
                    if holds_bytes {
                        made.mark_bytes();
                    }
                }
                Ok(joined)
            }
            (Object::Array(a), Object::Array(b)) => {
                let mut combined = a.borrow().clone();
                combined.extend(b.borrow().iter().cloned());
                Ok(Object::Array(Rc::new(std::cell::RefCell::new(combined))))
            }
            (lhs, rhs) => Err(binary_type_error(BinaryOp::Add, &lhs, &rhs, position)),
        }
    }

    /// Evaluate numeric binary operations (`-`, `*`, `/`, `%`).
    pub(crate) fn evaluate_numeric_binary(
        &mut self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // An instance of a String subclass repeats the characters it holds,
        // answering a plain String the way Ruby's does.
        if matches!(op, BinaryOp::Multiply)
            && matches!(left, Object::Instance(_))
            && let Some(text) = crate::vm::native_methods::string_subclass_value(&left)
        {
            return self.evaluate_numeric_binary(op, text, right, position);
        }
        match (left, right) {
            // Array difference: elements of the left array not present in
            // the right one, preserving left order.
            (Object::Array(a), Object::Array(b)) if matches!(op, BinaryOp::Subtract) => {
                let b_items = b.borrow();
                let remaining: Vec<Object> = a
                    .borrow()
                    .iter()
                    .filter(|item| !b_items.iter().any(|other| item.equals(other)))
                    .cloned()
                    .collect();
                Ok(Object::Array(Rc::new(std::cell::RefCell::new(remaining))))
            }
            (
                ref left @ (Object::Int(_) | Object::BigInt(_)),
                ref right @ (Object::Int(_) | Object::BigInt(_)),
            ) => {
                let a = left.as_big_integer().expect("integer-kinded");
                let b = right.as_big_integer().expect("integer-kinded");
                integer_arithmetic(op, a, b, position)
            }
            // An Integer receiver refuses a zero divisor whether or not the
            // divisor is a Float, so the bignum path says so.
            (Object::BigInt(a), Object::Float(b)) if matches!(op, BinaryOp::Modulo) => {
                float_modulo(big_to_float(&a), b, position)
            }
            // A negative base raised to a power that is not a whole number
            // has no real root, so the answer is the principal complex one.
            (Object::BigInt(a), Object::Float(b))
                if matches!(op, BinaryOp::Power)
                    && a.sign() == num_bigint::Sign::Minus
                    && b.fract() != 0.0
                    && b.is_finite() =>
            {
                let magnitude = (-big_to_float(&a)).powf(b);
                let angle = std::f64::consts::PI * b;
                self.make_complex(
                    Object::Float(magnitude * angle.cos()),
                    Object::Float(magnitude * angle.sin()),
                    position,
                )
            }
            (Object::BigInt(a), Object::Float(b)) => {
                float_arithmetic(op, big_to_float(&a), b, position)
            }
            (Object::Float(a), Object::BigInt(b)) => {
                float_arithmetic(op, a, big_to_float(&b), position)
            }
            (Object::Float(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - b)),
                BinaryOp::Multiply => Ok(Object::Float(a * b)),
                // Float division by zero follows IEEE 754 and answers an
                // infinity or NaN. Only Integer / Integer raises.
                BinaryOp::Divide => Ok(Object::Float(a / b)),
                BinaryOp::Modulo => float_modulo(a, b, position),
                // A negative base raised to a power that is not a whole
                // number has no real root, so the answer is the principal
                // complex one.
                BinaryOp::Power if a < 0.0 && b.fract() != 0.0 && b.is_finite() => {
                    let magnitude = (-a).powf(b);
                    let angle = std::f64::consts::PI * b;
                    self.make_complex(
                        Object::Float(magnitude * angle.cos()),
                        Object::Float(magnitude * angle.sin()),
                        position,
                    )
                }
                BinaryOp::Power => Ok(Object::Float(a.powf(b))),
                _ => unreachable!(),
            },
            (Object::Int(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float((a as f64) - b)),
                BinaryOp::Multiply => Ok(Object::Float((a as f64) * b)),
                BinaryOp::Divide => Ok(Object::Float((a as f64) / b)),
                BinaryOp::Modulo => float_modulo(a as f64, b, position),
                // A negative base raised to a power that is not a whole
                // number has no real root, so the answer is the principal
                // complex one.
                BinaryOp::Power if a < 0 && b.fract() != 0.0 && b.is_finite() => {
                    let magnitude = (-(a as f64)).powf(b);
                    let angle = std::f64::consts::PI * b;
                    self.make_complex(
                        Object::Float(magnitude * angle.cos()),
                        Object::Float(magnitude * angle.sin()),
                        position,
                    )
                }
                BinaryOp::Power => Ok(Object::Float((a as f64).powf(b))),
                _ => unreachable!(),
            },
            (Object::Float(a), Object::Int(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - (b as f64))),
                BinaryOp::Multiply => Ok(Object::Float(a * (b as f64))),
                BinaryOp::Divide => Ok(Object::Float(a / (b as f64))),
                BinaryOp::Modulo => float_modulo(a, b as f64, position),
                // `powf` rounds once, where repeated multiplication through
                // `powi` drifts, so `10.0 ** 308` matches `(10 ** 308).to_f`.
                BinaryOp::Power => Ok(Object::Float(a.powf(b as f64))),
                _ => unreachable!(),
            },
            // `"ab" * 3` repeats the string, and a negative count is an
            // ArgumentError rather than an empty string.
            (
                Object::String(text),
                given @ (Object::Int(_)
                | Object::Float(_)
                | Object::BigInt(_)
                | Object::Instance(_)),
            ) if matches!(op, BinaryOp::Multiply) => {
                let count = self.repeat_count(&given, position)?;
                let Ok(count) = usize::try_from(count) else {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative argument",
                        position,
                    ));
                };
                // An empty string repeats to nothing however many times it is
                // asked for, so the width of the count never matters there.
                if text.as_str().is_empty() {
                    return Ok(Object::String(std::rc::Rc::new(
                        crate::object::StringValue::with_encoding(
                            String::new(),
                            text.encoding_name(),
                        ),
                    )));
                }
                if text.as_str().len().checked_mul(count).is_none() {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "argument too big",
                        position,
                    ));
                }
                let made = crate::object::StringValue::with_encoding(
                    text.as_str().repeat(count),
                    text.encoding_name(),
                );
                // A string standing for bytes repeats into one that stands
                // for bytes, so the run it spells is the run repeated.
                if text.holds_bytes() {
                    made.mark_bytes();
                }
                Ok(Object::String(std::rc::Rc::new(made)))
            }
            // `[1, 2] * 3` repeats the array and `[1, 2] * ", "` joins it,
            // which Array answers from its own method table.
            (left, right)
                if matches!(op, BinaryOp::Multiply)
                    && (matches!(left, Object::Array(_))
                        || matches!(
                            crate::vm::native_methods::array_subclass_value(&left),
                            Some(Object::Array(_))
                        )) =>
            {
                match self.call_array_method(&left, "*", std::slice::from_ref(&right), position)? {
                    Some(answered) => Ok(answered),
                    None => Err(binary_type_error(op.clone(), &left, &right, position)),
                }
            }
            (lhs, rhs) => Err(binary_type_error(op.clone(), &lhs, &rhs, position)),
        }
    }
}

/// Subtract, multiply, divide, modulo, or raise two exact integers.
fn integer_arithmetic(
    op: &BinaryOp,
    left: num_bigint::BigInt,
    right: num_bigint::BigInt,
    position: Position,
) -> Result<Object, MetorexError> {
    use num_bigint::BigInt;
    match op {
        BinaryOp::Subtract => Ok(Object::integer(left - right)),
        BinaryOp::Multiply => Ok(Object::integer(left * right)),
        // Dividing two integers answers an integer, rounded toward negative
        // infinity rather than toward zero, so `-5 / 2` is -3.
        BinaryOp::Divide => {
            if right == BigInt::from(0) {
                return Err(divide_by_zero_error(position));
            }
            Ok(Object::integer(floored_quotient(&left, &right)))
        }
        // The modulus takes the sign of the divisor, which is what pairs with
        // that division: `-5 % 3` is 1.
        BinaryOp::Modulo => {
            if right == BigInt::from(0) {
                return Err(divide_by_zero_error(position));
            }
            Ok(Object::integer(floored_remainder(&left, &right)))
        }
        BinaryOp::Power => {
            // Zero raised to a negative power is a division by zero.
            if left == BigInt::from(0) && right < BigInt::from(0) {
                return Err(divide_by_zero_error(position));
            }
            // One and minus one stay themselves however far they are raised,
            // so the count never has to be built.
            if left == BigInt::from(1) {
                return Ok(Object::Int(1));
            }
            if left == BigInt::from(-1) && right >= BigInt::from(0) {
                let odd = right.bit(0);
                return Ok(Object::Int(if odd { -1 } else { 1 }));
            }
            // A negative exponent has no exact integer result, and one too
            // large to count is past the limit outright.
            let Ok(exponent) = u32::try_from(&right) else {
                if right > BigInt::from(0) {
                    return Err(exponent_too_large(position));
                }
                return Ok(Object::Float(
                    big_to_float(&left).powf(big_to_float(&right)),
                ));
            };
            // The answer takes one bit for each bit of the base, as many
            // times over as the exponent counts. Past the limit metorex
            // builds numbers up to, it is refused rather than attempted.
            let width = (left.bits() as u128).saturating_mul(exponent as u128);
            if width > WIDEST_INTEGER_BITS {
                return Err(exponent_too_large(position));
            }
            Ok(Object::integer(left.pow(exponent)))
        }
        _ => unreachable!("caller restricts op to the arithmetic set"),
    }
}

/// The most bits a number built by raising one to a power may take, which is
/// the room Ruby gives one before refusing to build it at all.
const WIDEST_INTEGER_BITS: u128 = 16 * 1024 * 1024 * 1024;

/// Ruby's ArgumentError for a power whose answer would be too wide to build.
fn exponent_too_large(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("ArgumentError", "exponent is too large", position)
}

/// The same set of operations on two Floats.
fn float_arithmetic(
    op: &BinaryOp,
    left: f64,
    right: f64,
    _position: Position,
) -> Result<Object, MetorexError> {
    Ok(Object::Float(match op {
        BinaryOp::Subtract => left - right,
        BinaryOp::Multiply => left * right,
        BinaryOp::Divide => left / right,
        BinaryOp::Modulo => {
            return float_modulo(left, right, _position);
        }
        BinaryOp::Power => left.powf(right),
        _ => unreachable!("caller restricts op to the arithmetic set"),
    }))
}

/// The quotient of two integers rounded toward negative infinity, which is the
/// division Ruby pairs with its modulus.
fn floored_quotient(left: &num_bigint::BigInt, right: &num_bigint::BigInt) -> num_bigint::BigInt {
    let quotient = left / right;
    let remainder = left % right;
    if remainder != num_bigint::BigInt::from(0)
        && (remainder < num_bigint::BigInt::from(0)) != (*right < num_bigint::BigInt::from(0))
    {
        quotient - 1
    } else {
        quotient
    }
}

/// The remainder left by that division, which carries the sign of the divisor.
fn floored_remainder(left: &num_bigint::BigInt, right: &num_bigint::BigInt) -> num_bigint::BigInt {
    let remainder = left % right;
    if remainder != num_bigint::BigInt::from(0)
        && (remainder < num_bigint::BigInt::from(0)) != (*right < num_bigint::BigInt::from(0))
    {
        remainder + right
    } else {
        remainder
    }
}

/// Ruby's `%` on Floats leaves a remainder carrying the sign of the divisor,
/// the same way the integer one does, and refuses a zero divisor rather than
/// answering NaN.
pub(crate) fn float_modulo(
    left: f64,
    right: f64,
    position: Position,
) -> Result<Object, MetorexError> {
    if right == 0.0 {
        return Err(divide_by_zero_error(position));
    }
    let remainder = left % right;
    if remainder != 0.0 && (remainder < 0.0) != (right < 0.0) {
        return Ok(Object::Float(remainder + right));
    }
    Ok(Object::Float(remainder))
}
