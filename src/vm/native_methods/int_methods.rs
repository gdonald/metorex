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
        // Integer#allbits? / #anybits? / #nobits? — how the bits of a mask sit
        // in the receiver. Both sides are compared as arbitrary-width integers,
        // so a bignum and a negative number answer the same way a fixnum does.
        if matches!(method_name, "allbits?" | "anybits?" | "nobits?")
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
            let mask = self.coerce_integer_argument(&arguments[0], position)?;
            let masked = receiver.as_big_integer().expect("integer-kinded") & &mask;
            let zero = num_bigint::BigInt::from(0);
            return Ok(Some(Object::Bool(match method_name {
                "allbits?" => masked == mask,
                "anybits?" => masked != zero,
                _ => masked == zero,
            })));
        }
        // An Integer is the whole of its own fraction, so it is its own
        // numerator over a denominator of one, and the Rational those name.
        if matches!(
            method_name,
            "numerator" | "denominator" | "to_r" | "rationalize"
        ) && matches!(receiver, Object::Int(_) | Object::BigInt(_))
        {
            // `rationalize` takes an optional precision, which cannot move a
            // number that is already exact. The others take nothing at all.
            let allowed = usize::from(method_name == "rationalize");
            if arguments.len() > allowed {
                return Err(method_argument_error(
                    method_name,
                    allowed,
                    arguments.len(),
                    position,
                ));
            }
            let value = receiver.as_big_integer().expect("integer-kinded");
            return match method_name {
                "numerator" => Ok(Some(Object::integer(value))),
                "denominator" => Ok(Some(Object::Int(1))),
                _ => self
                    .make_rational(value, num_bigint::BigInt::from(1), position)
                    .map(Some),
            };
        }
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
        if let Object::BigInt(value) = receiver {
            return self.call_big_int_method(value, method_name, arguments, position);
        }
        let Object::Int(n) = receiver else {
            return Ok(None);
        };
        // These read the same whether or not the receiver fits in a machine
        // word, so the arbitrary-width implementation answers for both.
        if matches!(
            method_name,
            "zero?"
                | "positive?"
                | "negative?"
                | "even?"
                | "odd?"
                | "succ"
                | "next"
                | "pred"
                | "integer?"
        ) {
            let value = Rc::new(num_bigint::BigInt::from(*n));
            return self.call_big_int_method(&value, method_name, arguments, position);
        }
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
            "abs" | "magnitude" => {
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
                Ok(Some(match method_name {
                    "-@" => Object::Int(-*n),
                    _ => Object::Int(*n),
                }))
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
                Ok(Some(Object::Float(*n as f64)))
            }
            // An Integer is already the whole number these ask for.
            "to_i" | "to_int" | "ord" => {
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
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A Float endpoint counts up to the whole number inside it, so
                // `1.upto(3.7)` stops at 3 and `5.downto(2.3)` stops at 3.
                let limit = match &arguments[0] {
                    Object::Int(limit) => *limit,
                    Object::Float(limit) if limit.is_finite() => {
                        let whole = match method_name {
                            "upto" => limit.floor(),
                            _ => limit.ceil(),
                        };
                        whole as i64
                    }
                    other => {
                        let message = format!(
                            "comparison of Integer with {} failed",
                            crate::vm::native_methods::array_methods::inspect_element(other)
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &message,
                            position,
                        ));
                    }
                };
                let limit = &limit;
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

impl VirtualMachine {
    /// An Integer argument: one as it stands, and anything else through
    /// `to_int`. Ruby refuses everything else, `coerce` included, since a bit
    /// operation is not arithmetic between two numbers.
    pub(crate) fn coerce_integer_argument(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<num_bigint::BigInt, MetorexError> {
        if let Some(value) = argument.as_big_integer() {
            return Ok(value);
        }
        let source = self.builtins().class_of(argument).name().to_string();
        let refuse = |message: String| MetorexError::UncaughtException {
            exception: Object::exception("TypeError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        };
        let Some((class, method)) = self.lookup_method(argument, "to_int") else {
            return Err(refuse(format!(
                "no implicit conversion of {} into Integer",
                source
            )));
        };
        let converted =
            self.invoke_method(class, method, argument.clone(), Vec::new(), position)?;
        converted.as_big_integer().ok_or_else(|| {
            refuse(format!(
                "can't convert {} to Integer ({}#to_int gives {})",
                source,
                source,
                self.builtins().class_of(&converted).name()
            ))
        })
    }
}

impl VirtualMachine {
    /// `Integer#ceil` / `#floor` / `#truncate` / `#round`, which differ only in
    /// where they send a value that sits between two multiples. A precision of
    /// zero or more leaves an Integer alone, since it has no digits to drop.
    fn round_to_precision(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // `round` takes a `half:` keyword naming where a value exactly between
        // two multiples goes. The others take the precision alone.
        let (arguments, half) = match method_name {
            "round" => split_rounding_mode(arguments),
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
        let value = receiver.as_big_integer().expect("integer-kinded");
        let Some(argument) = arguments.first() else {
            return Ok(Object::integer(value));
        };
        let precision = self.coerce_precision_argument(argument, position)?;
        if precision >= 0 {
            return Ok(Object::integer(value));
        }
        let Some(digits) = precision
            .checked_neg()
            .and_then(|digits| u32::try_from(digits).ok())
        else {
            let message = format!("integer {} too small to convert to `int'", precision);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("RangeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let step = num_bigint::BigInt::from(10).pow(digits);
        let remainder = &value % &step;
        let zero = num_bigint::BigInt::from(0);
        if remainder == zero {
            return Ok(Object::integer(value));
        }
        let floor = &value
            - if remainder < zero {
                &remainder + &step
            } else {
                remainder.clone()
            };
        let answer = match method_name {
            "floor" => floor,
            "ceil" => floor + &step,
            "truncate" => {
                if value < zero {
                    floor + &step
                } else {
                    floor
                }
            }
            // A value sitting exactly between two multiples goes where the
            // `half:` keyword says, and away from zero when it says nothing.
            _ => {
                let doubled: num_bigint::BigInt = (&value - &floor) * 2;
                let up = floor.clone() + &step;
                match doubled.cmp(&step) {
                    std::cmp::Ordering::Less => floor,
                    std::cmp::Ordering::Greater => up,
                    std::cmp::Ordering::Equal => match half {
                        RoundingMode::Down => {
                            if value < zero {
                                up
                            } else {
                                floor
                            }
                        }
                        RoundingMode::Even => {
                            let multiple = &floor / &step;
                            if &multiple % 2 == zero { floor } else { up }
                        }
                        RoundingMode::Up => {
                            if value < zero {
                                floor
                            } else {
                                up
                            }
                        }
                    },
                }
            }
        };
        Ok(Object::integer(answer))
    }

    /// The `half:` keyword: `:up` sends a value between two multiples away
    /// from zero, `:down` toward it, and `:even` to the even multiple. Ruby
    /// names any other mode in an ArgumentError.
    pub(crate) fn rounding_mode(
        &mut self,
        half: Option<Object>,
        position: Position,
    ) -> Result<RoundingMode, MetorexError> {
        let named = match half {
            None | Some(Object::Nil) => return Ok(RoundingMode::Up),
            Some(Object::Symbol(name) | Object::String(name)) => (*name).clone(),
            Some(other) => self.coerce_name_argument(&other, position)?,
        };
        match named.as_str() {
            "up" => Ok(RoundingMode::Up),
            "down" => Ok(RoundingMode::Down),
            "even" => Ok(RoundingMode::Even),
            _ => {
                let message = format!("invalid rounding mode: {}", named);
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
        }
    }

    /// The precision argument to a rounding method: an Integer as it stands,
    /// and anything else through `to_int`. One too large to count digits with
    /// is a RangeError rather than a number to round by.
    pub(crate) fn coerce_precision_argument(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let refuse = |class_name: &str, message: String| MetorexError::UncaughtException {
            exception: Object::exception(class_name, message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        };
        if matches!(argument, Object::Float(value) if !value.is_finite()) {
            return Err(refuse(
                "RangeError",
                "float infinity out of range of integer".to_string(),
            ));
        }
        let value = self.coerce_integer_argument(argument, position)?;
        i32::try_from(&value).map(i64::from).map_err(|_| {
            refuse(
                "RangeError",
                format!("integer {} out of range of int", value),
            )
        })
    }
}

/// Where `round` sends a value sitting exactly between two multiples.
#[derive(Clone, Copy)]
pub(crate) enum RoundingMode {
    Up,
    Down,
    Even,
}

/// Peel the `half:` keyword off a `round` argument list.
pub(crate) fn split_rounding_mode(arguments: &[Object]) -> (&[Object], Option<Object>) {
    let Some(Object::Dict(entries)) = arguments.last() else {
        return (arguments, None);
    };
    let entries = entries.borrow();
    let named: Vec<&String> = entries
        .keys()
        .filter(|key| key.as_str() != crate::vm::param_binding::KWARGS_MARKER)
        .collect();
    if named.len() != 1 || named[0].as_str() != ":half" {
        return (arguments, None);
    }
    let half = entries.get(":half").cloned();
    (&arguments[..arguments.len() - 1], half)
}

/// The greatest common divisor of two integers, always positive, which is what
/// `gcd` reports whatever signs it was handed.
fn big_gcd(left: num_bigint::BigInt, right: num_bigint::BigInt) -> num_bigint::BigInt {
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
fn absolute(value: num_bigint::BigInt) -> num_bigint::BigInt {
    if value < num_bigint::BigInt::from(0) {
        -value
    } else {
        value
    }
}

impl VirtualMachine {
    /// `div`, `modulo`, `fdiv`, and `remainder`, each of which divides and then
    /// reports a different part of the answer.
    fn integer_division_method(
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
            // asked to coerce before Ruby gives up on it.
            other if self.responds_to(other, "to_f") => {
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

    /// The Float an operand answers through the coercion protocol, when it
    /// takes part in one at all.
    fn coerced_float(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Option<f64>, MetorexError> {
        if !self.responds_to(value, "coerce") {
            return Ok(None);
        }
        let pair = self.send_to_object(value.clone(), "coerce", vec![Object::Int(1)], position)?;
        let Object::Array(parts) = pair else {
            return Ok(None);
        };
        let parts = parts.borrow().clone();
        if parts.len() != 2 {
            return Ok(None);
        }
        match self.send_to_object(parts[1].clone(), "to_f", vec![], position)? {
            Object::Float(number) => Ok(Some(number)),
            Object::Int(number) => Ok(Some(number as f64)),
            _ => Ok(None),
        }
    }

    /// The TypeError Ruby raises for an operand that cannot become a number.
    fn uncoercible(&mut self, value: &Object, position: Position) -> MetorexError {
        let message = format!(
            "{} can't be coerced into Integer",
            self.builtins().class_of(value).name()
        );
        MetorexError::UncaughtException {
            exception: Object::exception("TypeError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        }
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

impl VirtualMachine {
    /// The place values of an integer in `base`, least significant first.
    fn integer_digits(
        &mut self,
        receiver: &Object,
        base: num_bigint::BigInt,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let zero = num_bigint::BigInt::from(0);
        let value = receiver.as_big_integer().expect("integer-kinded");
        if value < zero {
            return Err(crate::vm::errors::simple_exception(
                "Math::DomainError",
                "out of domain",
                position,
            ));
        }
        if base < num_bigint::BigInt::from(2) {
            let message = if base < zero {
                "negative radix".to_string()
            } else {
                format!("invalid radix {}", base)
            };
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &message,
                position,
            ));
        }
        let mut digits = Vec::new();
        let mut remaining = value;
        while remaining > zero {
            digits.push(Object::integer(&remaining % &base));
            remaining /= &base;
        }
        if digits.is_empty() {
            digits.push(Object::Int(0));
        }
        Ok(Object::array(digits))
    }

    /// The Float a `coerce` argument stands for. Ruby reads a String as a
    /// number and asks anything else for `to_f`.
    fn coerce_to_float(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        if let Object::String(text) = argument {
            return text.trim().parse::<f64>().map_err(|_| {
                let message = format!("invalid value for Float(): {:?}", text.as_str());
                MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                }
            });
        }
        if let Object::Float(value) = argument {
            return Ok(*value);
        }
        if self.responds_to(argument, "to_f")
            && let Object::Float(value) =
                self.send_to_object(argument.clone(), "to_f", vec![], position)?
        {
            return Ok(value);
        }
        Err(self.uncoercible(argument, position))
    }
}

impl VirtualMachine {
    /// `Integer.sqrt` and `Integer.try_convert`, which belong to the class.
    pub(crate) fn integer_class_method(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if method_name == "try_convert" {
            // An Integer answers itself, an object with `to_int` answers what
            // that gives, and anything else is not an Integer at all.
            if matches!(argument, Object::Int(_) | Object::BigInt(_)) {
                return Ok(argument.clone());
            }
            if !self.responds_to(argument, "to_int") {
                return Ok(Object::Nil);
            }
            let source = self.builtins().class_of(argument).name().to_string();
            let converted = self.send_to_object(argument.clone(), "to_int", vec![], position)?;
            return match converted {
                Object::Nil | Object::Int(_) | Object::BigInt(_) => Ok(converted),
                other => {
                    let message = format!(
                        "can't convert {} into Integer ({}#to_int gives {})",
                        source,
                        source,
                        self.builtins().class_of(&other).name()
                    );
                    Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    })
                }
            };
        }
        // `sqrt` takes the whole part of the square root, so it answers an
        // Integer for a number of any width.
        let value = match argument {
            Object::Float(number) => num_bigint::BigInt::from(*number as i64),
            other => self.coerce_integer_argument(other, position)?,
        };
        if value < num_bigint::BigInt::from(0) {
            return Err(crate::vm::errors::simple_exception(
                "Math::DomainError",
                "Numerical argument is out of domain - \"isqrt\"",
                position,
            ));
        }
        Ok(Object::integer(integer_square_root(value)))
    }
}

/// The whole part of a square root, found by Newton's method so a number wider
/// than a Float still answers exactly.
fn integer_square_root(value: num_bigint::BigInt) -> num_bigint::BigInt {
    let zero = num_bigint::BigInt::from(0);
    let one = num_bigint::BigInt::from(1);
    if value <= one {
        return value;
    }
    let mut guess = value.clone();
    let mut next = (&guess + &value / &guess) / 2;
    while next < guess {
        guess = next;
        next = (&guess + &value / &guess) / 2;
    }
    let _ = zero;
    guess
}
