//! Native method implementations for the Integer class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the Integer class.
    pub(crate) fn call_int_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Object::BigInt(value) = receiver {
            return self.call_big_int_method(value, method_name, arguments, position);
        }
        let Object::Int(n) = receiver else {
            return Ok(None);
        };
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
                let Object::Int(count) = arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                // A negative count shifts the other way, as Ruby's does.
                let (shift_left, count) = if count < 0 {
                    (method_name == ">>", count.unsigned_abs())
                } else {
                    (method_name == "<<", count as u64)
                };
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
            "abs" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(n.abs())))
            }
            "to_r" | "rationalize" => self.make_rational(*n, 1, position).map(Some),
            "to_f" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Float(*n as f64)))
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
                Ok(Some(receiver.clone()))
            }
            "size" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(8)))
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
                Ok(Some(Object::String(Rc::new(n.to_string()))))
            }
            "times" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    // Without a block, Ruby returns an Enumerator. We
                    // approximate by returning the integer range as an Array
                    // so chained calls like `n.times.map { ... }` work.
                    None => {
                        let nums: Vec<Object> = (0..*n).map(Object::Int).collect();
                        return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(nums)))));
                    }
                };
                for i in 0..*n {
                    let args = vec![Object::Int(i)];
                    match self.execute_block_with_control_flow(&block, args)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        super::super::ControlFlow::Break { value, .. } => {
                            return Ok(Some(value));
                        }
                        super::super::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: super::super::utils::position_to_location(position),
                            });
                        }
                        super::super::ControlFlow::Exception {
                            exception,
                            position,
                        } => {
                            return Err(MetorexError::runtime_error(
                                format!(
                                    "Uncaught exception: {}",
                                    super::super::utils::format_exception(&exception)
                                ),
                                super::super::utils::position_to_location(position),
                            ));
                        }
                    }
                }
                Ok(Some(Object::Int(*n)))
            }
            // `upto(limit)` / `downto(limit)` — yield each integer from the
            // receiver to `limit` inclusive, answering the receiver. Without
            // a block they answer the sequence as an Array, matching how
            // `times` stands in for an Enumerator here.
            // `quo(other)` — exact division, so Integer / Integer answers a
            // Rational rather than truncating.
            "quo" => {
                let Some(Object::Int(divisor)) = arguments.first() else {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                if *divisor == 0 {
                    return Err(super::super::errors::divide_by_zero_error(position));
                }
                let (mut numerator, mut denominator) = (*n, *divisor);
                if denominator < 0 {
                    numerator = -numerator;
                    denominator = -denominator;
                }
                let divisor = greatest_common_divisor(numerator.abs(), denominator);
                let Some(Object::Class(rational_class)) = self.globals().get("Rational") else {
                    return Ok(None);
                };
                let mut instance = crate::object::Instance::new(rational_class);
                instance.set_var("numerator".to_string(), Object::Int(numerator / divisor));
                instance.set_var(
                    "denominator".to_string(),
                    Object::Int(denominator / divisor),
                );
                Ok(Some(Object::Instance(Rc::new(std::cell::RefCell::new(
                    instance,
                )))))
            }
            "upto" | "downto" => {
                let Some(Object::Int(limit)) = arguments.first() else {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                let sequence: Vec<i64> = if method_name == "upto" {
                    (*n..=*limit).collect()
                } else {
                    (*limit..=*n).rev().collect()
                };
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        let values: Vec<Object> = sequence.into_iter().map(Object::Int).collect();
                        return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                            values,
                        )))));
                    }
                };
                for value in sequence {
                    let args = vec![Object::Int(value)];
                    match self.execute_block_with_control_flow(&block, args)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        super::super::ControlFlow::Break { value, .. } => {
                            return Ok(Some(value));
                        }
                        super::super::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: super::super::utils::position_to_location(position),
                            });
                        }
                        super::super::ControlFlow::Exception {
                            exception,
                            position,
                        } => {
                            return Err(MetorexError::runtime_error(
                                format!(
                                    "Uncaught exception: {}",
                                    super::super::utils::format_exception(&exception)
                                ),
                                super::super::utils::position_to_location(position),
                            ));
                        }
                    }
                }
                Ok(Some(Object::Int(*n)))
            }
            _ => Ok(None),
        }
    }

    /// The Integer methods that have an exact answer for a value past the
    /// i64 range. `times`, `upto`, and `downto` are absent: a loop over that
    /// many values never finishes, so nothing here can stand in for one.
    fn call_big_int_method(
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
            "to_s" | "inspect" => {
                no_arguments(0)?;
                Ok(Some(Object::String(Rc::new(value.to_string()))))
            }
            "to_i" | "to_int" | "truncate" | "floor" | "ceil" | "round" => {
                Ok(Some(Object::BigInt(Rc::clone(value))))
            }
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
                let Object::Int(count) = arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
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
                Ok(Some(Object::String(Rc::new(value.to_string()))))
            }
            "integer?" => {
                no_arguments(0)?;
                Ok(Some(Object::Bool(true)))
            }
            _ => Ok(None),
        }
    }
}

/// Euclid's algorithm, used to put a Rational in lowest terms.
fn greatest_common_divisor(a: i64, b: i64) -> i64 {
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
fn shift_integer(value: i64, shift_left: bool, count: u64) -> i64 {
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
