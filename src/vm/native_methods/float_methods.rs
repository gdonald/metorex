//! Native method implementations for the Float class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Execute native methods for the Float class.
    pub(crate) fn call_float_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Float(f) = receiver else {
            return Ok(None);
        };
        match method_name {
            "round" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let precision = match arguments.first() {
                    None => 0,
                    Some(Object::Int(digits)) => *digits,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            other,
                            position,
                        ));
                    }
                };
                let multiplier = 10_f64.powi(precision as i32);
                let rounded = (f * multiplier).round() / multiplier;
                // Rounding to the digits left of the point, or to none at
                // all, answers a whole number.
                if precision <= 0 {
                    if !rounded.is_finite() {
                        let message = "Infinity".to_string();
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("FloatDomainError", message.clone()),
                            location: position_to_location(position),
                            message,
                        });
                    }
                    return Ok(Some(Object::integer(num_bigint::BigInt::from(
                        rounded as i128,
                    ))));
                }
                Ok(Some(Object::Float(rounded)))
            }
            "nan?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(f.is_nan())))
            }
            "finite?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(f.is_finite())))
            }
            // `infinite?` answers the sign of an infinity, and nil otherwise.
            "infinite?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(if f.is_infinite() {
                    Object::Int(if f.is_sign_negative() { -1 } else { 1 })
                } else {
                    Object::Nil
                }))
            }
            // The unary operators under the names `send` reaches them by.
            "-@" | "+@" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Float(match method_name {
                    "-@" => -*f,
                    _ => *f,
                })))
            }
            "abs" | "magnitude" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Float(f.abs())))
            }
            // `ceil`, `floor`, and `truncate` take a precision: a positive one
            // keeps that many digits after the point and answers a Float, and
            // zero or less answers the whole number those digits sit in.
            "ceil" | "floor" | "truncate" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let round = |value: f64| match method_name {
                    "ceil" => value.ceil(),
                    "floor" => value.floor(),
                    _ => value.trunc(),
                };
                let Some(argument) = arguments.first() else {
                    if !f.is_finite() {
                        return Err(float_domain_error(*f, position));
                    }
                    return Ok(Some(float_to_integer(round(*f))));
                };
                let digits = self.coerce_precision_argument(argument, position)?;
                if digits > 0 {
                    if !f.is_finite() {
                        return Ok(Some(Object::Float(*f)));
                    }
                    let scale = (10f64).powi(digits.min(320) as i32);
                    return Ok(Some(Object::Float(round(f * scale) / scale)));
                }
                if !f.is_finite() {
                    return Err(float_domain_error(*f, position));
                }
                let scale = (10f64).powi(-digits.max(-320) as i32);
                Ok(Some(float_to_integer(round(f / scale) * scale)))
            }

            "to_i" | "to_int" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // A number with no whole part to take, which is what NaN and
                // an infinity are, has no Integer to answer with.
                if !f.is_finite() {
                    let message = if f.is_nan() {
                        "NaN".to_string()
                    } else {
                        Object::Float(*f).to_string()
                    };
                    return Err(crate::vm::errors::simple_exception(
                        "FloatDomainError",
                        &message,
                        position,
                    ));
                }
                Ok(Some(float_to_integer(f.trunc())))
            }
            // `quo` and `fdiv` both divide and answer a Float, and `divmod`
            // reports the floored quotient with the modulus that pairs with it.
            "quo" | "fdiv" | "divmod" | "modulo" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                use crate::ast::BinaryOp;
                let receiver = Object::Float(*f);
                match method_name {
                    "modulo" => self
                        .evaluate_binary_operation(
                            &BinaryOp::Modulo,
                            receiver,
                            arguments[0].clone(),
                            position,
                        )
                        .map(Some),
                    "divmod" => {
                        // A zero divisor leaves no quotient to floor, which
                        // Ruby reports as a division by zero.
                        if matches!(&arguments[0], Object::Float(divisor) if *divisor == 0.0)
                            || matches!(&arguments[0], Object::Int(0))
                        {
                            return Err(crate::vm::errors::simple_exception(
                                "ZeroDivisionError",
                                "divided by 0",
                                position,
                            ));
                        }
                        let quotient = self.evaluate_binary_operation(
                            &BinaryOp::Divide,
                            receiver.clone(),
                            arguments[0].clone(),
                            position,
                        )?;
                        let floored = self.send_to_object(quotient, "floor", vec![], position)?;
                        let modulus = self.evaluate_binary_operation(
                            &BinaryOp::Modulo,
                            receiver,
                            arguments[0].clone(),
                            position,
                        )?;
                        Ok(Some(Object::array(vec![floored, modulus])))
                    }
                    // `quo` and `fdiv` divide by whatever the other number
                    // is, a Rational or a Complex included, which each carry
                    // their own division.
                    _ => {
                        if matches!(&arguments[0], Object::Instance(_)) {
                            return self
                                .evaluate_binary_operation(
                                    &BinaryOp::Divide,
                                    receiver,
                                    arguments[0].clone(),
                                    position,
                                )
                                .map(Some);
                        }
                        let divisor = self.float_value_of(&arguments[0], position)?;
                        Ok(Some(Object::Float(f / divisor)))
                    }
                }
            }
            // The sign predicates, and the magnitude that `abs` also answers.
            "zero?" | "positive?" | "negative?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(match method_name {
                    "zero?" => *f == 0.0,
                    "positive?" => *f > 0.0,
                    _ => *f < 0.0,
                })))
            }
            // `Float#angle` is the direction the number points on the number
            // line: zero for a positive one and Pi for a negative one, which
            // a signed zero follows.
            "angle" | "arg" | "phase" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if f.is_nan() {
                    return Ok(Some(Object::Float(*f)));
                }
                Ok(Some(if f.is_sign_negative() {
                    Object::Float(std::f64::consts::PI)
                } else {
                    Object::Int(0)
                }))
            }
            // The neighboring representable numbers, a single step away.
            "next_float" | "prev_float" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Float(neighboring_float(
                    *f,
                    method_name == "next_float",
                ))))
            }
            // `coerce` answers the pair an operator is applied to, which for a
            // Float is two Floats.
            "coerce" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Ruby reads a String here the way `Float()` reads one.
                let other = match &arguments[0] {
                    Object::String(text) => text.as_str().trim().parse::<f64>().map_err(|_| {
                        let message = format!("invalid value for Float(): {:?}", text.as_str());
                        MetorexError::UncaughtException {
                            exception: Object::exception("ArgumentError", message.clone()),
                            location: position_to_location(position),
                            message,
                        }
                    })?,
                    other => self.float_value_of(other, position)?,
                };
                Ok(Some(Object::array(vec![
                    Object::Float(other),
                    Object::Float(*f),
                ])))
            }
            // The parts of the exact fraction the number stands for. NaN and
            // the infinities have no fraction, and answer themselves.
            "numerator" | "denominator" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if !f.is_finite() {
                    return Ok(Some(match method_name {
                        "numerator" => Object::Float(*f),
                        _ => Object::Int(1),
                    }));
                }
                let (numerator, denominator) = super::rational_methods::float_exact_fraction(*f);
                Ok(Some(Object::integer(match method_name {
                    "numerator" => numerator,
                    _ => denominator,
                })))
            }
            // `to_r` is the float's exact binary value, so `0.6.to_r` is
            // (5404319552844595/9007199254740992) rather than (3/5).
            "to_r" => {
                let (numerator, denominator) = super::rational_methods::float_exact_fraction(*f);
                self.make_rational(numerator, denominator, position)
                    .map(Some)
            }
            "to_f" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(receiver.clone()))
            }
            "to_s" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // Ruby spells the non-finite floats out, and always shows a
                // fractional part, which is what the display form does.
                Ok(Some(Object::string(Object::Float(*f).to_string())))
            }
            _ => Ok(None),
        }
    }
}

/// The Integer a whole Float names. A magnitude past the i64 range keeps its
/// exact value rather than saturating, the way Ruby's does.
pub(super) fn float_to_integer(value: f64) -> Object {
    if !value.is_finite() {
        // Infinities and NaN have no integer value.
        return Object::Int(0);
    }
    // A whole f64 formats without an exponent at any magnitude, so its digits
    // are the exact value.
    let digits = format!("{:.0}", value);
    match num_bigint::BigInt::parse_bytes(digits.as_bytes(), 10) {
        Some(exact) => Object::integer(exact),
        None => Object::Int(value as i64),
    }
}

/// The representable Float one step away from `value`, in the direction the
/// caller asks for. The step is the gap between neighboring doubles, which
/// widens with the magnitude, so the bits are walked rather than a fixed
/// amount added.
fn neighboring_float(value: f64, upward: bool) -> f64 {
    if value.is_nan() {
        return value;
    }
    if value.is_infinite() {
        // Past the end there is nowhere further to go, and coming back from it
        // lands on the largest number there is.
        return if value.is_sign_positive() == upward {
            value
        } else if upward {
            -f64::MAX
        } else {
            f64::MAX
        };
    }
    if value == 0.0 {
        let smallest = f64::from_bits(1);
        return if upward { smallest } else { -smallest };
    }
    let bits = value.to_bits();
    let stepped = if (value > 0.0) == upward {
        bits + 1
    } else {
        bits - 1
    };
    f64::from_bits(stepped)
}

/// The FloatDomainError a number with no whole part to take reports, which is
/// what NaN and the infinities are.
fn float_domain_error(value: f64, position: Position) -> MetorexError {
    let message = if value.is_nan() {
        "NaN".to_string()
    } else {
        Object::Float(value).to_string()
    };
    crate::vm::errors::simple_exception("FloatDomainError", &message, position)
}
