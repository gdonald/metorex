//! Rational: construction, arithmetic, and conversion. A Rational is an
//! instance of the Rational class carrying `numerator` and `denominator`
//! instance variables, always in lowest terms with a positive denominator.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

/// Euclid's algorithm, used to put a Rational in lowest terms.
pub(crate) fn greatest_common_divisor(
    a: num_bigint::BigInt,
    b: num_bigint::BigInt,
) -> num_bigint::BigInt {
    use num_bigint::BigInt;
    let zero = BigInt::from(0);
    let absolute = |value: BigInt| if value < zero { -value } else { value };
    let (mut a, mut b) = (absolute(a), absolute(b));
    while b != zero {
        let remainder = &a % &b;
        a = b;
        b = remainder;
    }
    if a == zero { BigInt::from(1) } else { a }
}

/// The numerator and denominator of a Rational instance, or None for anything
/// that is not one.
pub(crate) fn rational_parts(
    receiver: &Object,
) -> Option<(num_bigint::BigInt, num_bigint::BigInt)> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    let instance = instance.borrow();
    if instance.class.name() != "Rational" {
        return None;
    }
    let numerator = instance.instance_vars.get("numerator")?.as_big_integer()?;
    let denominator = instance
        .instance_vars
        .get("denominator")?
        .as_big_integer()?;
    Some((numerator, denominator))
}

/// Read `value` as an exact fraction: integers and Rationals directly, floats
/// through their decimal digits so `Rational(0.5)` is (1/2).
fn as_fraction(value: &Object) -> Option<(num_bigint::BigInt, num_bigint::BigInt)> {
    match value {
        Object::Int(_) | Object::BigInt(_) => {
            Some((value.as_big_integer()?, num_bigint::BigInt::from(1)))
        }
        Object::Float(number) if number.is_finite() => {
            let text = format!("{}", number);
            parse_decimal_fraction(&text)
        }
        _ => rational_parts(value),
    }
}

/// Turn a plain decimal string such as "-1.25" into the pair (-125, 100).
fn parse_decimal_fraction(text: &str) -> Option<(num_bigint::BigInt, num_bigint::BigInt)> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if fraction.chars().any(|ch| !ch.is_ascii_digit()) {
        return None;
    }
    let negative = whole.starts_with('-');
    let whole_digits = whole.trim_start_matches(['+', '-']);
    if whole_digits.chars().any(|ch| !ch.is_ascii_digit()) {
        return None;
    }
    if whole_digits.is_empty() && fraction.is_empty() {
        return None;
    }
    let digits = format!("{}{}", whole_digits, fraction);
    let magnitude = num_bigint::BigInt::parse_bytes(digits.as_bytes(), 10)?;
    let denominator = num_bigint::BigInt::from(10).pow(fraction.len() as u32);
    Some((if negative { -magnitude } else { magnitude }, denominator))
}

/// A finite float as an exact fraction, read from its decimal digits so
/// `Rational(0.5)` is (1/2).
pub(crate) fn float_fraction(value: f64) -> (num_bigint::BigInt, num_bigint::BigInt) {
    parse_decimal_fraction(&format!("{}", value)).unwrap_or_else(|| {
        (
            num_bigint::BigInt::from(value as i64),
            num_bigint::BigInt::from(1),
        )
    })
}

/// The exact value of a finite float, which is a fraction over a power of
/// two. `0.6.to_r` is (5404319552844595/9007199254740992), not (3/5).
pub(crate) fn float_exact_fraction(value: f64) -> (num_bigint::BigInt, num_bigint::BigInt) {
    use num_bigint::BigInt;
    if !value.is_finite() || value == 0.0 {
        return (BigInt::from(0), BigInt::from(1));
    }
    // Every finite f64 is a whole mantissa times a power of two, so reading
    // the two straight off the bits gives the exact value rather than
    // whatever a doubling loop can reach before it runs out of precision.
    let bits = value.to_bits();
    let negative = bits >> 63 == 1;
    let raw_exponent = ((bits >> 52) & 0x7ff) as i64;
    let raw_mantissa = bits & ((1u64 << 52) - 1);
    // A subnormal carries no implicit leading one.
    let (mantissa, exponent) = if raw_exponent == 0 {
        (raw_mantissa, -1074i64)
    } else {
        (raw_mantissa | (1u64 << 52), raw_exponent - 1075)
    };
    let mut numerator = BigInt::from(mantissa);
    if negative {
        numerator = -numerator;
    }
    let mut denominator = BigInt::from(1);
    if exponent >= 0 {
        numerator <<= exponent as usize;
    } else {
        denominator <<= (-exponent) as usize;
    }
    let divisor = greatest_common_divisor(numerator.clone(), denominator.clone());
    (numerator / &divisor, denominator / &divisor)
}

/// The parts of a Complex instance, or None for anything that is not one.
pub(crate) fn complex_parts(value: &Object) -> Option<(Object, Object)> {
    let Object::Instance(instance) = value else {
        return None;
    };
    let instance = instance.borrow();
    if instance.class.name() != "Complex" {
        return None;
    }
    Some((
        instance
            .instance_vars
            .get("real")
            .cloned()
            .unwrap_or(Object::Int(0)),
        instance
            .instance_vars
            .get("imaginary")
            .cloned()
            .unwrap_or(Object::Int(0)),
    ))
}

/// Whether a Complex component is exactly zero.
pub(crate) fn is_zero(value: &Object) -> bool {
    match value {
        Object::Int(number) => *number == 0,
        Object::Float(number) => *number == 0.0,
        _ => rational_parts(value)
            .is_some_and(|(numerator, _)| numerator == num_bigint::BigInt::from(0)),
    }
}

/// Render a Complex the way Ruby does: `1+2i`, `1-2i`.
pub(crate) fn format_complex(real: &Object, imaginary: &Object) -> String {
    let rendered = format!("{}", imaginary);
    if rendered.starts_with('-') {
        format!("{}{}i", real, rendered)
    } else {
        format!("{}+{}i", real, rendered)
    }
}

/// Read a string strictly, the way `Rational("...")` does. Unlike
/// `String#to_r`, text that is not wholly a rational is rejected.
pub(crate) fn parse_strict_rational_text(
    text: &str,
) -> Option<(num_bigint::BigInt, num_bigint::BigInt)> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some((left, right)) = trimmed.split_once('/') {
        let (numerator, scale) = parse_decimal_fraction(left)?;
        let denominator = num_bigint::BigInt::parse_bytes(right.trim().as_bytes(), 10)?;
        return Some((numerator, scale * denominator));
    }
    parse_decimal_fraction(trimmed)
}

/// Read the leading rational value of a string the way `String#to_r` does:
/// an optional sign, then digits, an optional `.fraction` or `/denominator`.
/// Text that does not start with a number answers (0, 1).
pub(crate) fn parse_rational_text(text: &str) -> (num_bigint::BigInt, num_bigint::BigInt) {
    use num_bigint::BigInt;
    let trimmed = text.trim();
    if let Some((left, right)) = trimmed.split_once('/') {
        let (numerator, scale) =
            parse_decimal_fraction(left).unwrap_or_else(|| (BigInt::from(0), BigInt::from(1)));
        let denominator = leading_integer(right).unwrap_or(0);
        if denominator == 0 {
            return (BigInt::from(0), BigInt::from(1));
        }
        return (numerator, scale * BigInt::from(denominator));
    }
    // An underscore between digits is a separator rather than part of the
    // number, the same way it is in a literal.
    let mut digits = String::new();
    let mut previous_was_digit = false;
    for (index, ch) in trimmed.char_indices() {
        if ch == '_' && previous_was_digit {
            continue;
        }
        let acceptable =
            ch.is_ascii_digit() || ch == '.' || ((ch == '+' || ch == '-') && index == 0);
        if !acceptable {
            break;
        }
        previous_was_digit = ch.is_ascii_digit();
        digits.push(ch);
    }
    parse_decimal_fraction(&digits).unwrap_or_else(|| (BigInt::from(0), BigInt::from(1)))
}

fn leading_integer(text: &str) -> Option<i64> {
    let digits: String = text.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    digits.parse::<i64>().ok()
}

impl VirtualMachine {
    /// Build a Rational in lowest terms with a positive denominator. Ruby
    /// freezes every Rational, so the instance answers `frozen?` with true.
    pub(crate) fn make_rational(
        &mut self,
        numerator: impl Into<num_bigint::BigInt>,
        denominator: impl Into<num_bigint::BigInt>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use num_bigint::BigInt;
        let numerator: BigInt = numerator.into();
        let denominator: BigInt = denominator.into();
        if denominator == BigInt::from(0) {
            let message = "divided by 0".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ZeroDivisionError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        let (mut numerator, mut denominator) = (numerator, denominator);
        if denominator < BigInt::from(0) {
            numerator = -numerator;
            denominator = -denominator;
        }
        let divisor = greatest_common_divisor(numerator.clone(), denominator.clone());

        let Some(Object::Class(rational_class)) = self.globals().get("Rational") else {
            return Err(MetorexError::runtime_error(
                "Rational is not defined",
                position_to_location(position),
            ));
        };
        let mut instance = crate::object::Instance::new(rational_class);
        instance.set_var(
            "numerator".to_string(),
            Object::integer(numerator / &divisor),
        );
        instance.set_var(
            "denominator".to_string(),
            Object::integer(denominator / &divisor),
        );
        instance.frozen = true;
        Ok(Object::Instance(Rc::new(std::cell::RefCell::new(instance))))
    }

    /// An Integer or Float as an exact Rational, for arithmetic where the
    /// other operand is one.
    pub(crate) fn promote_to_rational(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (numerator, denominator) = match value {
            Object::Int(_) | Object::BigInt(_) => (
                value.as_big_integer().expect("integer-kinded"),
                num_bigint::BigInt::from(1),
            ),
            Object::Float(number) => float_fraction(*number),
            _ => return Ok(value.clone()),
        };
        self.make_rational(numerator, denominator, position)
    }

    /// Build a Complex from its two components.
    pub(crate) fn make_complex(
        &mut self,
        real: Object,
        imaginary: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(Object::Class(complex_class)) = self.globals().get("Complex") else {
            return Err(MetorexError::runtime_error(
                "Complex is not defined",
                position_to_location(position),
            ));
        };
        let mut instance = crate::object::Instance::new(complex_class);
        instance.set_var("real".to_string(), real);
        instance.set_var("imaginary".to_string(), imaginary);
        Ok(Object::Instance(Rc::new(std::cell::RefCell::new(instance))))
    }

    /// Execute native methods for the Rational class.
    pub(crate) fn call_rational_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Some((numerator, denominator)) = rational_parts(receiver) else {
            return Ok(None);
        };

        match method_name {
            // The unary operators, which a sign in front of a Rational reaches.
            "-@" | "+@" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let numerator = match method_name {
                    "-@" => -numerator,
                    _ => numerator,
                };
                self.make_rational(numerator, denominator, position)
                    .map(Some)
            }
            // A Rational compared against something that is not a number
            // answers by asking that object instead.
            "==" if arguments.len() == 1
                && matches!(&arguments[0], Object::Instance(instance)
                    if !matches!(instance.borrow().class.name(), "Rational" | "Complex")) =>
            {
                let answer = self.send_to_object(
                    arguments[0].clone(),
                    "==",
                    vec![receiver.clone()],
                    position,
                )?;
                Ok(Some(Object::Bool(answer.is_truthy())))
            }
            "numerator" => Ok(Some(Object::integer(numerator))),
            "denominator" => Ok(Some(Object::integer(denominator))),
            // `floor`, `ceil`, `truncate`, and `round` take a precision: a
            // positive one keeps that many decimal places and answers a
            // Rational, and zero or less answers the Integer those places sit
            // in.
            "floor" | "ceil" | "truncate" | "round" => {
                // `round` takes a `half:` keyword naming where a value exactly
                // between two multiples goes.
                let (arguments, half) = match method_name {
                    "round" => crate::vm::native_methods::split_rounding_mode(arguments),
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
                // Ruby takes an Integer precision here and nothing else, so an
                // object carrying `to_int` is refused rather than converted.
                let digits = match arguments.first() {
                    None => 0,
                    Some(Object::Int(digits)) => *digits,
                    Some(_) => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            "not an integer",
                            position,
                        ));
                    }
                };
                let ten = num_bigint::BigInt::from(10);
                // Move the decimal point by the precision, round there, and
                // move it back.
                let (scaled_numerator, scaled_denominator) = if digits > 0 {
                    (&numerator * ten.pow(digits as u32), denominator.clone())
                } else {
                    (
                        numerator.clone(),
                        &denominator * ten.pow(digits.unsigned_abs() as u32),
                    )
                };
                let rounded =
                    round_fraction(&scaled_numerator, &scaled_denominator, method_name, &half);
                if digits > 0 {
                    return self
                        .make_rational(rounded, ten.pow(digits as u32), position)
                        .map(Some);
                }
                Ok(Some(Object::integer(
                    rounded * ten.pow(digits.unsigned_abs() as u32),
                )))
            }
            "to_r" | "rationalize" => Ok(Some(receiver.clone())),
            // Rational truncates toward zero, so (8/3) is 2 and (-8/3) is -2.
            "to_i" | "to_int" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::integer(numerator / denominator)))
            }
            // The two sides divide with the precision their widths carry, so a
            // fraction of two bignums answers the ratio between them.
            "to_f" => Ok(Some(Object::Float(crate::vm::native_methods::exact_ratio(
                &numerator,
                &denominator,
            )))),
            "abs" => {
                let magnitude = if numerator < num_bigint::BigInt::from(0) {
                    -numerator
                } else {
                    numerator
                };
                self.make_rational(magnitude, denominator, position)
                    .map(Some)
            }
            "zero?" => Ok(Some(Object::Bool(numerator == num_bigint::BigInt::from(0)))),
            "negative?" => Ok(Some(Object::Bool(numerator < num_bigint::BigInt::from(0)))),
            "positive?" => Ok(Some(Object::Bool(numerator > num_bigint::BigInt::from(0)))),
            "to_s" => Ok(Some(Object::string(format!(
                "{}/{}",
                numerator, denominator
            )))),
            "inspect" => Ok(Some(Object::string(format!(
                "({}/{})",
                numerator, denominator
            )))),
            "hash" => Ok(Some(Object::integer(numerator * 31 + denominator))),
            "frozen?" => Ok(Some(Object::Bool(true))),
            "==" | "eql?" | "!=" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                // `eql?` is stricter than `==`: it wants another Rational,
                // where `==` also matches an equal Integer or Float.
                // Ruby compares a Rational against a Float by rounding the
                // Rational to a Float, so `0.7.to_r == 0.7` holds even though
                // no fraction with a power-of-ten denominator equals 0.7.
                let equal = match (method_name, other) {
                    ("eql?", _) if rational_parts(other).is_none() => false,
                    (_, Object::Float(number)) => {
                        crate::vm::native_methods::exact_ratio(&numerator, &denominator) == *number
                    }
                    _ => match as_fraction(other) {
                        Some((other_numerator, other_denominator)) => {
                            numerator * other_denominator == other_numerator * denominator
                        }
                        None => false,
                    },
                };
                Ok(Some(Object::Bool(if method_name == "!=" {
                    !equal
                } else {
                    equal
                })))
            }
            "+" | "-" | "*" | "/" | "quo" | "<" | "<=" | ">" | ">=" | "<=>" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                self.rational_binary_operation(
                    (numerator, denominator),
                    method_name,
                    other,
                    position,
                )
            }
            _ => Ok(None),
        }
    }

    /// Arithmetic and ordering against an Integer, Float, or Rational. A Float
    /// operand makes the whole expression a Float, as it does in Ruby.
    fn rational_binary_operation(
        &mut self,
        (numerator, denominator): (num_bigint::BigInt, num_bigint::BigInt),
        operator: &str,
        other: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Object::Float(value) = other {
            let left = big_to_float(&numerator) / big_to_float(&denominator);
            return self
                .float_binary_operation(left, operator, *value, position)
                .map(Some);
        }

        let Some((other_numerator, other_denominator)) = as_fraction(other) else {
            return Ok(None);
        };

        let (result_numerator, result_denominator) = match operator {
            "+" => (
                &numerator * &other_denominator + &other_numerator * &denominator,
                &denominator * &other_denominator,
            ),
            "-" => (
                &numerator * &other_denominator - &other_numerator * &denominator,
                &denominator * &other_denominator,
            ),
            "*" => (
                &numerator * &other_numerator,
                &denominator * &other_denominator,
            ),
            "/" | "quo" => (
                &numerator * &other_denominator,
                &denominator * &other_numerator,
            ),
            _ => {
                let left = &numerator * &other_denominator;
                let right = &other_numerator * &denominator;
                let ordering = left.cmp(&right);
                return Ok(Some(match operator {
                    "<" => Object::Bool(ordering.is_lt()),
                    "<=" => Object::Bool(ordering.is_le()),
                    ">" => Object::Bool(ordering.is_gt()),
                    ">=" => Object::Bool(ordering.is_ge()),
                    _ => Object::Int(ordering as i64),
                }));
            }
        };
        self.make_rational(result_numerator, result_denominator, position)
            .map(Some)
    }

    fn float_binary_operation(
        &mut self,
        left: f64,
        operator: &str,
        right: f64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        Ok(match operator {
            "+" => Object::Float(left + right),
            "-" => Object::Float(left - right),
            "*" => Object::Float(left * right),
            "/" | "quo" => Object::Float(left / right),
            "<" => Object::Bool(left < right),
            "<=" => Object::Bool(left <= right),
            ">" => Object::Bool(left > right),
            ">=" => Object::Bool(left >= right),
            _ => match left.partial_cmp(&right) {
                Some(ordering) => Object::Int(ordering as i64),
                None => {
                    return Err(MetorexError::runtime_error(
                        "comparison of Rational with Float failed",
                        position_to_location(position),
                    ));
                }
            },
        })
    }
}

/// The nearest Float to an arbitrary-precision integer.
fn big_to_float(value: &num_bigint::BigInt) -> f64 {
    use std::str::FromStr;
    f64::from_str(&value.to_string()).unwrap_or(f64::INFINITY)
}

/// The whole number a fraction rounds to, in the direction the method names.
fn round_fraction(
    numerator: &num_bigint::BigInt,
    denominator: &num_bigint::BigInt,
    method_name: &str,
    half: &crate::vm::native_methods::RoundingMode,
) -> num_bigint::BigInt {
    let zero = num_bigint::BigInt::from(0);
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder == zero {
        return quotient;
    }
    let negative = (numerator < &zero) != (denominator < &zero);
    match method_name {
        "truncate" => quotient,
        "floor" => {
            if negative {
                quotient - 1
            } else {
                quotient
            }
        }
        "ceil" => {
            if negative {
                quotient
            } else {
                quotient + 1
            }
        }
        // A value exactly between two whole numbers goes where the `half:`
        // keyword says, and away from zero when it says nothing.
        _ => {
            use crate::vm::native_methods::RoundingMode;
            let doubled: num_bigint::BigInt = remainder.clone() * 2;
            let magnitude = if doubled < zero { -doubled } else { doubled };
            let divisor = if *denominator < zero {
                -denominator.clone()
            } else {
                denominator.clone()
            };
            let away = if negative {
                quotient.clone() - 1
            } else {
                quotient.clone() + 1
            };
            match magnitude.cmp(&divisor) {
                std::cmp::Ordering::Less => quotient,
                std::cmp::Ordering::Greater => away,
                std::cmp::Ordering::Equal => match half {
                    RoundingMode::Down => quotient,
                    RoundingMode::Even => {
                        if &quotient % 2 == zero {
                            quotient
                        } else {
                            away
                        }
                    }
                    RoundingMode::Up => away,
                },
            }
        }
    }
}
