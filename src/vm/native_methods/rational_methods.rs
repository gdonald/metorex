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
pub(crate) fn greatest_common_divisor(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    if a == 0 { 1 } else { a }
}

/// The numerator and denominator of a Rational instance, or None for anything
/// that is not one.
pub(crate) fn rational_parts(receiver: &Object) -> Option<(i64, i64)> {
    let Object::Instance(instance) = receiver else {
        return None;
    };
    let instance = instance.borrow();
    if instance.class.name() != "Rational" {
        return None;
    }
    let numerator = match instance.instance_vars.get("numerator") {
        Some(Object::Int(value)) => *value,
        _ => return None,
    };
    let denominator = match instance.instance_vars.get("denominator") {
        Some(Object::Int(value)) => *value,
        _ => return None,
    };
    Some((numerator, denominator))
}

/// Read `value` as an exact fraction: integers and Rationals directly, floats
/// through their decimal digits so `Rational(0.5)` is (1/2).
fn as_fraction(value: &Object) -> Option<(i64, i64)> {
    match value {
        Object::Int(number) => Some((*number, 1)),
        Object::Float(number) if number.is_finite() => {
            let text = format!("{}", number);
            parse_decimal_fraction(&text)
        }
        _ => rational_parts(value),
    }
}

/// Turn a plain decimal string such as "-1.25" into the pair (-125, 100).
fn parse_decimal_fraction(text: &str) -> Option<(i64, i64)> {
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
    let magnitude = digits.parse::<i64>().ok()?;
    let denominator = 10i64.checked_pow(fraction.len() as u32)?;
    Some((if negative { -magnitude } else { magnitude }, denominator))
}

/// A finite float as an exact fraction, read from its decimal digits so
/// `Rational(0.5)` is (1/2).
pub(crate) fn float_fraction(value: f64) -> (i64, i64) {
    parse_decimal_fraction(&format!("{}", value)).unwrap_or((value as i64, 1))
}

/// The exact value of a finite float, which is a fraction over a power of
/// two. `0.6.to_r` is (5404319552844595/9007199254740992), not (3/5).
pub(crate) fn float_exact_fraction(value: f64) -> (i64, i64) {
    let mut numerator = value;
    let mut denominator: i64 = 1;
    while numerator.fract() != 0.0 && denominator <= (1i64 << 61) {
        numerator *= 2.0;
        denominator *= 2;
    }
    let numerator = numerator as i64;
    let divisor = greatest_common_divisor(numerator, denominator);
    (numerator / divisor, denominator / divisor)
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
        _ => rational_parts(value).is_some_and(|(numerator, _)| numerator == 0),
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
pub(crate) fn parse_strict_rational_text(text: &str) -> Option<(i64, i64)> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some((left, right)) = trimmed.split_once('/') {
        let (numerator, scale) = parse_decimal_fraction(left)?;
        let denominator = right.parse::<i64>().ok()?;
        return Some((numerator, scale.checked_mul(denominator)?));
    }
    parse_decimal_fraction(trimmed)
}

/// Read the leading rational value of a string the way `String#to_r` does:
/// an optional sign, then digits, an optional `.fraction` or `/denominator`.
/// Text that does not start with a number answers (0, 1).
pub(crate) fn parse_rational_text(text: &str) -> (i64, i64) {
    let trimmed = text.trim();
    if let Some((left, right)) = trimmed.split_once('/') {
        let (numerator, scale) = parse_decimal_fraction(left).unwrap_or((0, 1));
        let denominator = leading_integer(right).unwrap_or(0);
        if denominator == 0 {
            return (0, 1);
        }
        return (numerator, scale.saturating_mul(denominator));
    }
    let mut end = 0;
    for (index, ch) in trimmed.char_indices() {
        let acceptable =
            ch.is_ascii_digit() || ch == '.' || ((ch == '+' || ch == '-') && index == 0);
        if !acceptable {
            break;
        }
        end = index + ch.len_utf8();
    }
    parse_decimal_fraction(&trimmed[..end]).unwrap_or((0, 1))
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
        numerator: i64,
        denominator: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if denominator == 0 {
            let message = "divided by 0".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ZeroDivisionError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        let (mut numerator, mut denominator) = (numerator, denominator);
        if denominator < 0 {
            numerator = -numerator;
            denominator = -denominator;
        }
        let divisor = greatest_common_divisor(numerator, denominator);

        let Some(Object::Class(rational_class)) = self.globals().get("Rational") else {
            return Err(MetorexError::runtime_error(
                "Rational is not defined",
                position_to_location(position),
            ));
        };
        let mut instance = crate::object::Instance::new(rational_class);
        instance.set_var("numerator".to_string(), Object::Int(numerator / divisor));
        instance.set_var(
            "denominator".to_string(),
            Object::Int(denominator / divisor),
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
            Object::Int(number) => (*number, 1),
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
            "numerator" => Ok(Some(Object::Int(numerator))),
            "denominator" => Ok(Some(Object::Int(denominator))),
            "to_r" | "rationalize" => Ok(Some(receiver.clone())),
            // Rational truncates toward zero, so (8/3) is 2 and (-8/3) is -2.
            "to_i" | "to_int" | "truncate" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(numerator / denominator)))
            }
            "to_f" => Ok(Some(Object::Float(numerator as f64 / denominator as f64))),
            "abs" => self
                .make_rational(numerator.abs(), denominator, position)
                .map(Some),
            "zero?" => Ok(Some(Object::Bool(numerator == 0))),
            "negative?" => Ok(Some(Object::Bool(numerator < 0))),
            "positive?" => Ok(Some(Object::Bool(numerator > 0))),
            "to_s" => Ok(Some(Object::string(format!(
                "{}/{}",
                numerator, denominator
            )))),
            "inspect" => Ok(Some(Object::string(format!(
                "({}/{})",
                numerator, denominator
            )))),
            "hash" => Ok(Some(Object::Int(
                numerator.wrapping_mul(31).wrapping_add(denominator),
            ))),
            "frozen?" => Ok(Some(Object::Bool(true))),
            "==" | "eql?" | "!=" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                // `eql?` is stricter than `==`: it wants another Rational,
                // where `==` also matches an equal Integer or Float.
                let equal = match (method_name, as_fraction(other)) {
                    ("eql?", _) if rational_parts(other).is_none() => false,
                    (_, Some((other_numerator, other_denominator))) => {
                        numerator * other_denominator == other_numerator * denominator
                    }
                    (_, None) => false,
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
        (numerator, denominator): (i64, i64),
        operator: &str,
        other: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Object::Float(value) = other {
            let left = numerator as f64 / denominator as f64;
            return self
                .float_binary_operation(left, operator, *value, position)
                .map(Some);
        }

        let Some((other_numerator, other_denominator)) = as_fraction(other) else {
            return Ok(None);
        };

        let (result_numerator, result_denominator) = match operator {
            "+" => (
                numerator * other_denominator + other_numerator * denominator,
                denominator * other_denominator,
            ),
            "-" => (
                numerator * other_denominator - other_numerator * denominator,
                denominator * other_denominator,
            ),
            "*" => (numerator * other_numerator, denominator * other_denominator),
            "/" | "quo" => (numerator * other_denominator, denominator * other_numerator),
            _ => {
                let left = numerator * other_denominator;
                let right = other_numerator * denominator;
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
