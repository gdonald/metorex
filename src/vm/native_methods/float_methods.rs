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
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let precision = match &arguments[0] {
                    Object::Int(p) => *p,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            &arguments[0],
                            position,
                        ));
                    }
                };

                if precision < 0 {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "Float.round precision must be non-negative, got {}",
                            precision
                        ),
                        position_to_location(position),
                    ));
                }

                let multiplier = 10_f64.powi(precision as i32);
                let rounded = (f * multiplier).round() / multiplier;
                Ok(Some(Object::Float(rounded)))
            }
            "abs" => {
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
            "ceil" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(float_to_integer(f.ceil())))
            }
            "floor" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(float_to_integer(f.floor())))
            }
            "to_i" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(float_to_integer(f.trunc())))
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
                Ok(Some(Object::String(std::rc::Rc::new(f.to_string()))))
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
