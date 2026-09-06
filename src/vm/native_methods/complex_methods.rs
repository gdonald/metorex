use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;

use super::rational_methods::{complex_parts, format_complex, is_zero};

impl VirtualMachine {
    /// Execute native methods for the Complex class.
    pub(crate) fn call_complex_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Some((real, imaginary)) = complex_parts(receiver) else {
            return Ok(None);
        };
        match method_name {
            "real" => Ok(Some(real)),
            "imaginary" | "imag" => Ok(Some(imaginary)),
            // Ruby renders the two parts with the method it is asked for, so
            // `inspect` shows what each part inspects as.
            "to_s" | "inspect" => {
                let name = if method_name == "to_s" {
                    "to_s"
                } else {
                    "inspect"
                };
                let real = self.render_part(&real, name, position)?;
                let imaginary_negative = self.part_is_negative(&imaginary, position)?;
                let imaginary = self.render_part(&imaginary, name, position)?;
                let rendered = format_complex_parts(&real, &imaginary, imaginary_negative);
                Ok(Some(Object::string(match method_name {
                    "to_s" => rendered,
                    _ => format!("({})", rendered),
                })))
            }
            "real?" => Ok(Some(Object::Bool(false))),
            "zero?" => Ok(Some(Object::Bool(is_zero(&real) && is_zero(&imaginary)))),
            "frozen?" => Ok(Some(Object::Bool(true))),
            "hash" => Ok(Some(Object::string(format_complex(&real, &imaginary)))),
            "==" | "!=" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                // A Complex with a zero imaginary part equals the plain
                // number its real part holds, which is what Ruby compares.
                let equal = match complex_parts(other) {
                    Some((other_real, other_imaginary)) => {
                        self.numbers_equal(&real, &other_real, position)?
                            && self.numbers_equal(&imaginary, &other_imaginary, position)?
                    }
                    None => is_zero(&imaginary) && self.numbers_equal(&real, other, position)?,
                };
                Ok(Some(Object::Bool(if method_name == "!=" {
                    !equal
                } else {
                    equal
                })))
            }
            // `eql?` is equality without conversion, so only another Complex
            // whose parts are the same kind and value counts.
            "eql?" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                let Some((other_real, other_imaginary)) = complex_parts(other) else {
                    return Ok(Some(Object::Bool(false)));
                };
                // The parts are compared by class and by value rather than
                // asked `eql?` themselves, which is what Ruby does here.
                let same_classes = self.builtins().class_of(&real).name()
                    == self.builtins().class_of(&other_real).name()
                    && self.builtins().class_of(&imaginary).name()
                        == self.builtins().class_of(&other_imaginary).name();
                let equal = self.numbers_equal(&real, &other_real, position)?
                    && self.numbers_equal(&imaginary, &other_imaginary, position)?;
                Ok(Some(Object::Bool(same_classes && equal)))
            }
            // The arithmetic, which composes the two parts of each operand.
            "+" | "-" | "*" | "/" | "quo" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                // An operand that is neither a Complex nor a real number is
                // asked to coerce, and the operator is applied to the pair it
                // answers.
                if complex_parts(other).is_none()
                    && !self.is_real_operand(other)
                    && self.responds_to(other, "coerce")
                {
                    let pair = self.send_to_object(
                        other.clone(),
                        "coerce",
                        vec![receiver.clone()],
                        position,
                    )?;
                    if let Object::Array(parts) = &pair {
                        let parts = parts.borrow().clone();
                        if parts.len() == 2 {
                            return self
                                .send_to_object(
                                    parts[0].clone(),
                                    method_name,
                                    vec![parts[1].clone()],
                                    position,
                                )
                                .map(Some);
                        }
                    }
                }
                self.complex_arithmetic(&real, &imaginary, method_name, other, position)
                    .map(Some)
            }
            // `fdiv` divides with both parts as Floats.
            "fdiv" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                let real = self.part_to_float(&real, position)?;
                let imaginary = self.part_to_float(&imaginary, position)?;
                let left = self.make_complex(real, imaginary, position)?;
                let (other_real, other_imaginary) = self.complex_operand_parts(other, position)?;
                let other_real = self.part_to_float(&other_real, position)?;
                let other_imaginary = self.part_to_float(&other_imaginary, position)?;
                let right = self.make_complex(other_real, other_imaginary, position)?;
                self.send_to_object(left, "/", vec![right], position)
                    .map(Some)
            }
            // `**` with a whole exponent multiplies the number out, which
            // keeps exact parts exact.
            "**" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                self.complex_power(&real, &imaginary, other, position)
                    .map(Some)
            }
            // The magnitude and the square of it, which needs no square root.
            "abs" | "magnitude" => {
                let real = self.part_to_float(&real, position)?;
                let imaginary = self.part_to_float(&imaginary, position)?;
                let (Object::Float(real), Object::Float(imaginary)) = (real, imaginary) else {
                    unreachable!("both parts were converted to Floats")
                };
                Ok(Some(Object::Float(real.hypot(imaginary))))
            }
            "abs2" => {
                let real_square = self.multiply_parts(&real, &real, position)?;
                let imaginary_square = self.multiply_parts(&imaginary, &imaginary, position)?;
                self.add_parts(&real_square, &imaginary_square, position)
                    .map(Some)
            }
            // The direction the number points, measured from the positive
            // real axis.
            "arg" | "angle" | "phase" => {
                let real = self.part_to_float(&real, position)?;
                let imaginary = self.part_to_float(&imaginary, position)?;
                let (Object::Float(real), Object::Float(imaginary)) = (real, imaginary) else {
                    unreachable!("both parts were converted to Floats")
                };
                Ok(Some(Object::Float(imaginary.atan2(real))))
            }
            // Two Complexes with nothing on the imaginary axis order by their
            // real parts, and anything else has no order at all.
            "<=>" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                if !is_zero(&imaginary) {
                    return Ok(Some(Object::Nil));
                }
                let other_real = match complex_parts(other) {
                    Some((other_real, other_imaginary)) => {
                        if !is_zero(&other_imaginary) {
                            return Ok(Some(Object::Nil));
                        }
                        other_real
                    }
                    None if self.is_real_operand(other) => other.clone(),
                    None => return Ok(Some(Object::Nil)),
                };
                self.evaluate_binary_operation(
                    &crate::ast::BinaryOp::Spaceship,
                    real,
                    other_real,
                    position,
                )
                .map(Some)
            }
            "polar" => {
                let magnitude = self.send_to_object(receiver.clone(), "abs", vec![], position)?;
                let angle = self.send_to_object(receiver.clone(), "arg", vec![], position)?;
                Ok(Some(Object::array(vec![magnitude, angle])))
            }
            "rect" | "rectangular" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::array(vec![real, imaginary])))
            }
            "conjugate" | "conj" => {
                let negated = self.negate_part(&imaginary, position)?;
                self.make_complex(real, negated, position).map(Some)
            }
            // Negating sends `-@` to each part, so a part of the program's own
            // making decides what its negation is.
            "-@" | "+@" => {
                if method_name == "+@" {
                    return Ok(Some(receiver.clone()));
                }
                let real = self.send_to_object(real, "-@", vec![], position)?;
                let imaginary = self.send_to_object(imaginary, "-@", vec![], position)?;
                self.make_complex(real, imaginary, position).map(Some)
            }
            "to_c" => Ok(Some(receiver.clone())),
            // A Complex is only a real number when nothing is left on the
            // imaginary axis, and Ruby refuses to drop what is there.
            // The two parts share a denominator, which is the least common
            // multiple of theirs, and the numerator is the pair scaled to it.
            "denominator" | "numerator" => {
                let real_denominator =
                    self.send_to_object(real.clone(), "denominator", vec![], position)?;
                let imaginary_denominator =
                    self.send_to_object(imaginary.clone(), "denominator", vec![], position)?;
                let common = self.send_to_object(
                    real_denominator,
                    "lcm",
                    vec![imaginary_denominator],
                    position,
                )?;
                if method_name == "denominator" {
                    return Ok(Some(common));
                }
                let scale = |vm: &mut Self, part: &Object| -> Result<Object, MetorexError> {
                    let scaled = vm.evaluate_binary_operation(
                        &crate::ast::BinaryOp::Multiply,
                        part.clone(),
                        common.clone(),
                        position,
                    )?;
                    vm.send_to_object(scaled, "to_i", vec![], position)
                };
                let real = scale(self, &real)?;
                let imaginary = scale(self, &imaginary)?;
                self.make_complex(real, imaginary, position).map(Some)
            }
            "to_f" | "to_i" | "to_int" | "to_r" | "rationalize" => {
                // The imaginary part is asked whether it is zero, so a number
                // of the program's own making answers for itself. A Float is
                // not exact enough to drop for `to_f` and `to_i`, so
                // `Complex(1, 0.0)` is refused where `Complex(1, 0)` converts.
                // The fraction conversions take it either way.
                let exact_only = matches!(method_name, "to_f" | "to_i" | "to_int" | "rationalize");

                let empty = !(exact_only && matches!(imaginary, Object::Float(_)))
                    && self
                        .send_to_object(imaginary.clone(), "==", vec![Object::Int(0)], position)?
                        .is_truthy();
                if !empty {
                    let message = format!(
                        "can't convert {} into {}",
                        format_complex(&real, &imaginary),
                        match method_name {
                            "to_f" => "Float",
                            "to_i" | "to_int" => "Integer",
                            _ => "Rational",
                        }
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        &message,
                        position,
                    ));
                }
                let name = match method_name {
                    "to_int" => "to_i",
                    other => other,
                };
                self.send_to_object(real, name, arguments.to_vec(), position)
                    .map(Some)
            }
            "finite?" | "infinite?" => {
                let real_answer = self.send_to_object(real, method_name, vec![], position)?;
                let imaginary_answer =
                    self.send_to_object(imaginary, method_name, vec![], position)?;
                if method_name == "finite?" {
                    return Ok(Some(Object::Bool(
                        real_answer.is_truthy() && imaginary_answer.is_truthy(),
                    )));
                }
                // `infinite?` answers 1 when either part runs off the end.
                Ok(Some(match (&real_answer, &imaginary_answer) {
                    (Object::Nil, Object::Nil) => Object::Nil,
                    _ => Object::Int(1),
                }))
            }
            // `coerce` answers two Complexes, since that is what the operators
            // apply to.
            "coerce" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                if complex_parts(other).is_some() {
                    return Ok(Some(Object::array(vec![other.clone(), receiver.clone()])));
                }
                if !self.is_real_operand(other) {
                    let message = format!(
                        "{} can't be coerced into Complex",
                        self.builtins().class_of(other).name()
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &message,
                        position,
                    ));
                }
                let promoted = self.make_complex(other.clone(), Object::Int(0), position)?;
                Ok(Some(Object::array(vec![promoted, receiver.clone()])))
            }
            _ => Ok(None),
        }
    }

    /// `Complex(real)` / `Complex(real, imaginary)`. A String is read as a
    /// complex literal, a Complex operand composes with the other, and
    /// anything non-numeric raises TypeError. `exception: false` answers nil
    /// wherever the value could not be read, though a first argument that is
    /// not a real number still raises.
    pub(crate) fn kernel_complex(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (positional, keywords) = super::kernel_conversion::split_conversion_keywords(arguments);
        let raise = keywords
            .get("exception")
            .map(|value| value.is_truthy())
            .unwrap_or(true);
        if positional.is_empty() || positional.len() > 2 {
            return Err(argument_count_error(
                crate::vm::errors::Arity::Range(1, 2),
                positional.len(),
                position,
            ));
        }
        if positional.len() == 1 {
            return self.complex_from_single(&positional[0], raise, position);
        }
        // With two arguments both must be real numbers. The imaginary one is
        // checked first, since a bad one there is an error `exception: false`
        // swallows, where a first argument that is not a number at all is
        // refused either way.
        let not_a_real = |vm: &mut Self| -> MetorexError {
            let message = "not a real".to_string();
            let _ = vm;
            MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            }
        };
        if !self.is_real_operand(&positional[1]) && !matches!(positional[1], Object::Nil) {
            if !raise {
                return Ok(Object::Nil);
            }
            return Err(not_a_real(self));
        }
        if !self.is_real_operand(&positional[0]) && !matches!(positional[0], Object::Nil) {
            return Err(not_a_real(self));
        }
        // A Numeric of the program's own making that reports itself as not
        // real composes through its own operators: the result is
        // `real + imaginary * Complex(0, 1)`.
        if self.operand_is_unreal(&positional[0], position)?
            || self.operand_is_unreal(&positional[1], position)?
        {
            let unit = self.make_complex(Object::Int(0), Object::Int(1), position)?;
            let scaled = self.invoke_named_method(&positional[1], "*", &[unit], position)?;
            return self.invoke_named_method(&positional[0], "+", &[scaled], position);
        }
        // Two plain real numbers are the two parts as they stand, which keeps
        // a signed zero the sign it was given.
        if complex_parts(&positional[0]).is_none()
            && complex_parts(&positional[1]).is_none()
            && self.is_real_operand(&positional[0])
            && self.is_real_operand(&positional[1])
        {
            return self.make_complex(positional[0].clone(), positional[1].clone(), position);
        }
        let Some(real) = self.complex_operand(&positional[0], raise, position)? else {
            return Ok(Object::Nil);
        };
        let Some(imaginary) = self.complex_operand(&positional[1], raise, position)? else {
            return Ok(Object::Nil);
        };
        // `Complex(a, b)` with complex operands means `a + b * i`.
        let (real_a, imaginary_a) = real;
        let (real_b, imaginary_b) = imaginary;
        let real = self.numeric_operation(&real_a, "-", &imaginary_b, position)?;
        let imaginary = self.numeric_operation(&imaginary_a, "+", &real_b, position)?;
        self.make_complex(real, imaginary, position)
    }

    /// `Complex.polar(modulus, argument)` — the rectangular complex that polar
    /// pair names.
    pub(crate) fn call_complex_class_method(
        &mut self,
        class_rc: &std::rc::Rc<crate::class::Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if class_rc.name() != "Complex" || !matches!(method_name, "polar" | "rectangular" | "rect")
        {
            return Ok(None);
        }
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(argument_count_error(
                crate::vm::errors::Arity::Range(1, 2),
                arguments.len(),
                position,
            ));
        }
        let first = arguments[0].clone();
        let second = arguments.get(1).cloned().unwrap_or(Object::Int(0));
        // `rect` names the two axes directly, so both parts have to be real
        // numbers: a Numeric that reports itself as not real names no point on
        // either one.
        if method_name != "polar" {
            for argument in [&first, &second] {
                if !self.is_real_operand(argument) || self.operand_is_unreal(argument, position)? {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "not a real",
                        position,
                    ));
                }
            }
            return self.make_complex(first, second, position).map(Some);
        }
        // A polar pair is measured along the real line, and a Complex with
        // nothing on the imaginary axis is the number its real part holds.
        let flatten = |vm: &mut Self, value: Object| -> Result<Object, MetorexError> {
            if let Some((real, imaginary)) = complex_parts(&value)
                && is_zero(&imaginary)
            {
                return Ok(real);
            }
            if !vm.is_real_operand(&value) {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "not a real",
                    position,
                ));
            }
            Ok(value)
        };
        let first = flatten(self, first)?;
        let second = flatten(self, second)?;
        let (real, imaginary) = self.polar_parts(first, second, position)?;
        self.make_complex(real, imaginary, position).map(Some)
    }

    /// The one-argument form, which answers the argument itself for a Complex,
    /// for a Numeric that reports itself as not real, and for whatever `to_c`
    /// hands back.
    fn complex_from_single(
        &mut self,
        value: &Object,
        raise: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if complex_parts(value).is_some() {
            return Ok(value.clone());
        }
        if !matches!(value, Object::Nil)
            && self.value_is_numeric(value)
            && self.responds_to(value, "real?")
        {
            let real = self.invoke_named_method(value, "real?", &[], position)?;
            if !real.is_truthy() {
                return Ok(value.clone());
            }
            return self.make_complex(value.clone(), Object::Int(0), position);
        }
        if !self.is_real_operand(value)
            && !matches!(value, Object::Nil | Object::String(_))
            && self.responds_to(value, "to_c")
        {
            return self.invoke_named_method(value, "to_c", &[], position);
        }
        let Some((real, imaginary)) = self.complex_operand(value, raise, position)? else {
            return Ok(Object::Nil);
        };
        self.make_complex(real, imaginary, position)
    }

    /// Whether an operand is a program-defined Numeric that reports itself as
    /// not a real number.
    fn operand_is_unreal(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if !self.is_foreign_numeric(value) || !self.responds_to(value, "real?") {
            return Ok(false);
        }
        let real = self.invoke_named_method(value, "real?", &[], position)?;
        Ok(!real.is_truthy())
    }

    /// Whether a value is a Numeric the program defined rather than one of
    /// metorex's own number kinds.
    fn is_foreign_numeric(&mut self, value: &Object) -> bool {
        matches!(value, Object::Instance(_))
            && complex_parts(value).is_none()
            && super::rational_methods::rational_parts(value).is_none()
            && self.value_is_numeric(value)
    }

    /// Whether a value is a real number metorex can use as a component.
    fn is_real_operand(&mut self, value: &Object) -> bool {
        if matches!(
            value,
            Object::Int(_) | Object::BigInt(_) | Object::Float(_) | Object::String(_)
        ) {
            return true;
        }
        if super::rational_methods::rational_parts(value).is_some()
            || complex_parts(value).is_some()
        {
            return true;
        }
        self.value_is_numeric(value)
    }

    /// One argument to `Complex()`, as its real and imaginary parts. Answers
    /// None when the value could not be read and `exception: false` was given.
    fn complex_operand(
        &mut self,
        value: &Object,
        raise: bool,
        position: Position,
    ) -> Result<Option<(Object, Object)>, MetorexError> {
        if let Some(parts) = complex_parts(value) {
            return Ok(Some(parts));
        }
        let convert_error = |vm: &mut Self, name: String| -> Result<Option<_>, MetorexError> {
            let _ = vm;
            if !raise {
                return Ok(None);
            }
            let message = format!("can't convert {} into Complex", name);
            Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            })
        };
        match value {
            Object::Int(_) | Object::BigInt(_) | Object::Float(_) => {
                Ok(Some((value.clone(), Object::Int(0))))
            }
            _ if super::rational_methods::rational_parts(value).is_some() => {
                Ok(Some((value.clone(), Object::Int(0))))
            }
            Object::String(text) => {
                if text.contains('\0') {
                    if !raise {
                        return Ok(None);
                    }
                    let message = "string contains null byte".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                let Some(parsed) = parse_complex_text(text) else {
                    if !raise {
                        return Ok(None);
                    }
                    let message = format!("invalid value for convert(): \"{}\"", text);
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                let real = self.component_object(parsed.real, position)?;
                let imaginary = self.component_object(parsed.imaginary, position)?;
                if parsed.polar {
                    return self.polar_parts(real, imaginary, position).map(Some);
                }
                Ok(Some((real, imaginary)))
            }
            _ if self.value_is_numeric(value) => Ok(Some((value.clone(), Object::Int(0)))),
            Object::Nil => convert_error(self, "nil".to_string()),
            other => {
                let name = self.builtins().class_of(other).name().to_string();
                convert_error(self, name)
            }
        }
    }

    /// Add or subtract two numeric components. Adding or subtracting zero is
    /// the common case here, and answering the other operand keeps its own
    /// type rather than going through coercion a Rational would not survive.
    fn numeric_operation(
        &mut self,
        left: &Object,
        operator: &str,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A shortcut is only safe when it keeps the kind of number Ruby would
        // have produced, and adding a Float zero answers a Float.
        let exact_zero = |value: &Object| is_zero(value) && !matches!(value, Object::Float(_));
        if exact_zero(right) && !matches!(left, Object::Float(_)) {
            return Ok(left.clone());
        }
        if exact_zero(left) && operator == "+" && !matches!(right, Object::Float(_)) {
            return Ok(right.clone());
        }
        let operation = match operator {
            "-" => crate::ast::BinaryOp::Subtract,
            _ => crate::ast::BinaryOp::Add,
        };
        self.evaluate_binary_operation(&operation, left.clone(), right.clone(), position)
    }

    /// Whether a value's class descends from Numeric.
    fn value_is_numeric(&mut self, value: &Object) -> bool {
        let Some(Object::Class(numeric)) = self.globals().get("Numeric") else {
            return false;
        };
        let value_class = self.builtins().class_of(value);
        self.builtins().is_subclass_of(&value_class, &numeric)
    }

    /// Call a method by name on a value, however it is defined.
    fn invoke_named_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some((class, method)) = self.lookup_method(receiver, method_name) else {
            return Ok(Object::Nil);
        };
        self.invoke_method(
            class,
            method,
            receiver.clone(),
            arguments.to_vec(),
            position,
        )
    }

    /// A numeric component as an f64, which polar form needs for cos and sin.
    fn numeric_to_float(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        match value {
            Object::Int(number) => Ok(*number as f64),
            Object::Float(number) => Ok(*number),
            other => match self.invoke_named_method(other, "to_f", &[], position)? {
                Object::Float(number) => Ok(number),
                Object::Int(number) => Ok(number as f64),
                _ => Ok(0.0),
            },
        }
    }

    /// The rectangular parts of a complex written in polar form.
    fn polar_parts(
        &mut self,
        modulus: Object,
        argument: Object,
        position: Position,
    ) -> Result<(Object, Object), MetorexError> {
        let modulus = self.numeric_to_float(&modulus, position)?;
        let argument = self.numeric_to_float(&argument, position)?;
        Ok((
            Object::Float(modulus * argument.cos()),
            Object::Float(modulus * argument.sin()),
        ))
    }

    /// Turn a parsed component into the Object Ruby would produce for it.
    fn component_object(
        &mut self,
        component: ComplexComponent,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match component {
            ComplexComponent::Integer(value) => Ok(Object::integer(value)),
            ComplexComponent::Float(value) => Ok(Object::Float(value)),
            ComplexComponent::Fraction(numerator, denominator) => {
                self.make_rational(numerator, denominator, position)
            }
        }
    }

    /// Compare two numeric components by value, so `1`, `1.0`, and `(1/1)`
    /// all count as the same number.
    fn numbers_equal(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if let (Object::Int(left), Object::Int(right)) = (left, right) {
            return Ok(left == right);
        }
        let answer = self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Equal,
            left.clone(),
            right.clone(),
            position,
        )?;
        Ok(answer.is_truthy())
    }
}

/// One parsed component of a complex literal, before it becomes an Object.
pub(crate) enum ComplexComponent {
    Integer(num_bigint::BigInt),
    Float(f64),
    Fraction(num_bigint::BigInt, num_bigint::BigInt),
}

/// The two components of a complex literal, and whether they were written in
/// polar form (`modulus@argument`).
pub(crate) struct ParsedComplex {
    pub real: ComplexComponent,
    pub imaginary: ComplexComponent,
    pub polar: bool,
}

/// Strip the `_` digit separators Ruby allows, refusing a run of two or a
/// separator that is not between digits.
fn strip_digit_separators(text: &str) -> Option<String> {
    if !text.contains('_') {
        return Some(text.to_string());
    }
    let characters: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (index, character) in characters.iter().enumerate() {
        if *character != '_' {
            out.push(*character);
            continue;
        }
        let before = index.checked_sub(1).and_then(|i| characters.get(i));
        let after = characters.get(index + 1);
        if !before.is_some_and(char::is_ascii_digit) || !after.is_some_and(char::is_ascii_digit) {
            return None;
        }
    }
    Some(out)
}

/// Read one numeric component: an integer, a float (with optional exponent),
/// or a `numerator/denominator` fraction. The whole text must be consumed.
fn parse_component(text: &str) -> Option<ComplexComponent> {
    let text = strip_digit_separators(text)?;
    let text = text.as_str();
    if text.is_empty() {
        return None;
    }
    if let Some((numerator, denominator)) = text.split_once('/') {
        let numerator = parse_integer_text(numerator)?;
        let denominator = parse_integer_text(denominator)?;
        return Some(ComplexComponent::Fraction(numerator, denominator));
    }
    if text.contains('.') || text.contains('e') || text.contains('E') {
        // Rust accepts `inf` and `nan`, which Ruby's converter does not.
        if !text
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | 'e' | 'E'))
        {
            return None;
        }
        return text.parse::<f64>().ok().map(ComplexComponent::Float);
    }
    parse_integer_text(text).map(ComplexComponent::Integer)
}

/// Read a signed run of digits, and nothing else.
fn parse_integer_text(text: &str) -> Option<num_bigint::BigInt> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    num_bigint::BigInt::parse_bytes(text.as_bytes(), 10)
}

/// The index of the sign that splits the real part from the imaginary one: the
/// last `+` or `-` that is neither leading nor part of an exponent.
fn imaginary_sign_index(text: &str) -> Option<usize> {
    let characters: Vec<char> = text.chars().collect();
    for index in (1..characters.len()).rev() {
        if !matches!(characters[index], '+' | '-') {
            continue;
        }
        if matches!(characters[index - 1], 'e' | 'E') {
            continue;
        }
        return Some(index);
    }
    None
}

/// Read a complex literal the way `Complex("...")` does. The whole string must
/// be consumed, so trailing text makes this answer None.
pub(crate) fn parse_complex_text(text: &str) -> Option<ParsedComplex> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some((modulus, argument)) = trimmed.split_once('@') {
        return Some(ParsedComplex {
            real: parse_component(modulus)?,
            imaginary: parse_component(argument)?,
            polar: true,
        });
    }
    let zero = ComplexComponent::Integer(num_bigint::BigInt::from(0));
    let Some(body) = trimmed.strip_suffix(['i', 'I', 'j', 'J']) else {
        return Some(ParsedComplex {
            real: parse_component(trimmed)?,
            imaginary: zero,
            polar: false,
        });
    };
    // `i`, `+i`, and `-i` carry no digits of their own and mean 1i.
    let unit = |sign: i32| ComplexComponent::Integer(num_bigint::BigInt::from(sign));
    match imaginary_sign_index(body) {
        Some(index) => {
            let (real_text, imaginary_text) = body.split_at(index);
            let imaginary = if imaginary_text.len() == 1 {
                unit(if imaginary_text.starts_with('-') {
                    -1
                } else {
                    1
                })
            } else {
                parse_component(imaginary_text)?
            };
            Some(ParsedComplex {
                real: parse_component(real_text)?,
                imaginary,
                polar: false,
            })
        }
        None => {
            let imaginary = match body {
                "" => unit(1),
                "+" => unit(1),
                "-" => unit(-1),
                digits => parse_component(digits)?,
            };
            Some(ParsedComplex {
                real: zero,
                imaginary,
                polar: false,
            })
        }
    }
}

impl VirtualMachine {
    /// `+`, `-`, `*`, and `/` between a Complex and whatever it was handed,
    /// composed from the four parts.
    fn complex_arithmetic(
        &mut self,
        real: &Object,
        imaginary: &Object,
        method_name: &str,
        other: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (other_real, other_imaginary) = self.complex_operand_parts(other, position)?;
        match method_name {
            "+" | "-" => {
                let combine = |vm: &mut Self, left: &Object, right: &Object| {
                    if method_name == "+" {
                        vm.add_parts(left, right, position)
                    } else {
                        vm.subtract_parts(left, right, position)
                    }
                };
                let new_real = combine(self, real, &other_real)?;
                let new_imaginary = combine(self, imaginary, &other_imaginary)?;
                self.make_complex(new_real, new_imaginary, position)
            }
            // (a + bi)(c + di) = (ac - bd) + (ad + bc)i
            "*" => {
                let ac = self.multiply_parts(real, &other_real, position)?;
                let bd = self.multiply_parts(imaginary, &other_imaginary, position)?;
                let ad = self.multiply_parts(real, &other_imaginary, position)?;
                let bc = self.multiply_parts(imaginary, &other_real, position)?;
                let new_real = self.subtract_parts(&ac, &bd, position)?;
                let new_imaginary = self.add_parts(&ad, &bc, position)?;
                self.make_complex(new_real, new_imaginary, position)
            }
            // Dividing multiplies by the conjugate of the divisor, which
            // leaves a real denominator to divide each part by.
            _ => {
                let cc = self.multiply_parts(&other_real, &other_real, position)?;
                let dd = self.multiply_parts(&other_imaginary, &other_imaginary, position)?;
                let denominator = self.add_parts(&cc, &dd, position)?;
                let ac = self.multiply_parts(real, &other_real, position)?;
                let bd = self.multiply_parts(imaginary, &other_imaginary, position)?;
                let bc = self.multiply_parts(imaginary, &other_real, position)?;
                let ad = self.multiply_parts(real, &other_imaginary, position)?;
                let real_top = self.add_parts(&ac, &bd, position)?;
                let imaginary_top = self.subtract_parts(&bc, &ad, position)?;
                let new_real = self.divide_parts(&real_top, &denominator, position)?;
                let new_imaginary = self.divide_parts(&imaginary_top, &denominator, position)?;
                self.make_complex(new_real, new_imaginary, position)
            }
        }
    }

    /// A Complex raised to a whole power, by multiplying it out. A negative
    /// exponent divides one by the positive power instead.
    fn complex_power(
        &mut self,
        real: &Object,
        imaginary: &Object,
        exponent: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Object::Int(exponent) = exponent else {
            let message = format!(
                "{} can't be raised to that power yet",
                format_complex(real, imaginary)
            );
            return Err(MetorexError::runtime_error(
                message,
                crate::vm::utils::position_to_location(position),
            ));
        };
        let mut answer = self.make_complex(Object::Int(1), Object::Int(0), position)?;
        let base = self.make_complex(real.clone(), imaginary.clone(), position)?;
        for _ in 0..exponent.unsigned_abs() {
            answer = self.send_to_object(answer, "*", vec![base.clone()], position)?;
        }
        if *exponent < 0 {
            let one = self.make_complex(Object::Int(1), Object::Int(0), position)?;
            return self.send_to_object(one, "/", vec![answer], position);
        }
        Ok(answer)
    }

    /// The two parts of an operand: a Complex gives both, and a real number
    /// gives itself with nothing on the imaginary axis.
    fn complex_operand_parts(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<(Object, Object), MetorexError> {
        if let Some(parts) = complex_parts(value) {
            return Ok(parts);
        }
        if self.is_real_operand(value) {
            return Ok((value.clone(), Object::Int(0)));
        }
        let message = format!(
            "{} can't be coerced into Complex",
            self.builtins().class_of(value).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    fn add_parts(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Add,
            left.clone(),
            right.clone(),
            position,
        )
    }

    fn subtract_parts(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Subtract,
            left.clone(),
            right.clone(),
            position,
        )
    }

    fn multiply_parts(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Multiply,
            left.clone(),
            right.clone(),
            position,
        )
    }

    /// Dividing two parts keeps them exact: two Integers answer the Rational
    /// between them rather than the whole number the division would floor to.
    fn divide_parts(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if matches!(left, Object::Int(_) | Object::BigInt(_))
            && matches!(right, Object::Int(_) | Object::BigInt(_))
        {
            return self.send_to_object(left.clone(), "quo", vec![right.clone()], position);
        }
        self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Divide,
            left.clone(),
            right.clone(),
            position,
        )
    }

    fn negate_part(&mut self, value: &Object, position: Position) -> Result<Object, MetorexError> {
        self.subtract_parts(&Object::Int(0), value, position)
    }

    /// A part as a Float, which is what the magnitude and the angle measure.
    fn part_to_float(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.send_to_object(value.clone(), "to_f", vec![], position)
    }
}

impl VirtualMachine {
    /// One part of a Complex as text, through the method the caller names.
    fn render_part(
        &mut self,
        value: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<String, MetorexError> {
        match self.send_to_object(value.clone(), method_name, vec![], position)? {
            Object::String(text) => Ok((*text).clone()),
            other => Ok(other.to_string()),
        }
    }

    /// Whether a part sits below zero, which decides the sign between the two
    /// halves. A negative zero counts, since Ruby shows `1-0.0i`.
    fn part_is_negative(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if let Object::Float(number) = value {
            // NaN carries a sign bit that varies by platform, and Ruby prints
            // it with a `+` either way, so only a real negative counts. A
            // negative zero keeps its sign, which `-0.0 < 0` would miss.
            return Ok(!number.is_nan() && number.is_sign_negative());
        }
        let answer = self.send_to_object(value.clone(), "<", vec![Object::Int(0)], position)?;
        Ok(answer.is_truthy())
    }
}

/// Join the two rendered halves the way Ruby does: a sign between them, and a
/// `*` before the `i` when the imaginary half does not end in a digit.
fn format_complex_parts(real: &str, imaginary: &str, negative: bool) -> String {
    let magnitude = imaginary.strip_prefix('-').unwrap_or(imaginary);
    let separator = if negative { "-" } else { "+" };
    let suffix = match magnitude.chars().last() {
        Some(last) if last.is_ascii_digit() => "i",
        _ => "*i",
    };
    format!("{}{}{}{}", real, separator, magnitude, suffix)
}
