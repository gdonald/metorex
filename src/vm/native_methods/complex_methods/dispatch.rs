// The methods a Complex answers, and the class methods that build one.

use super::*;

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
            "hash" => Ok(Some(Object::Int(
                crate::vm::native_methods::object_methods::hashing::seeded_text_hash(
                    &format_complex(&real, &imaginary),
                ),
            ))),
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
                // Text reads as a number only where `Complex()` parses it,
                // never as an operand of the arithmetic.
                if matches!(other, Object::String(_)) {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "String can't be coerced into Complex",
                        position,
                    ));
                }
                // A number written in Ruby says whether it stands on the
                // real line, which decides how it joins the arithmetic.
                if matches!(other, Object::Instance(_))
                    && complex_parts(other).is_none()
                    && crate::vm::native_methods::rational_methods::rational_parts(other).is_none()
                    && self.value_is_numeric(other)
                    && self.responds_to(other, "real?")
                {
                    let stands_real = self
                        .send_to_object(other.clone(), "real?", Vec::new(), position)?
                        .is_truthy();
                    // Dividing by a number keeps the exact answer, which is
                    // what `quo` gives where `/` would truncate.
                    let applied = if method_name == "/" {
                        "quo"
                    } else {
                        method_name
                    };
                    if stands_real {
                        // Multiplying and dividing reach both parts. Adding
                        // and subtracting reach the real part alone, which is
                        // what the operand coerces against.
                        if matches!(method_name, "*" | "/" | "quo") {
                            let scaled_real = self.send_to_object(
                                real.clone(),
                                applied,
                                vec![other.clone()],
                                position,
                            )?;
                            let scaled_imaginary = self.send_to_object(
                                imaginary.clone(),
                                applied,
                                vec![other.clone()],
                                position,
                            )?;
                            return self
                                .make_complex(scaled_real, scaled_imaginary, position)
                                .map(Some);
                        }
                        let pair = self.send_to_object(
                            other.clone(),
                            "coerce",
                            vec![real.clone()],
                            position,
                        )?;
                        if let Object::Array(parts) = &pair {
                            let parts = parts.borrow().clone();
                            if parts.len() == 2 {
                                let joined = self.send_to_object(
                                    parts[0].clone(),
                                    applied,
                                    vec![parts[1].clone()],
                                    position,
                                )?;
                                return self
                                    .make_complex(joined, imaginary.clone(), position)
                                    .map(Some);
                            }
                        }
                    } else if self.responds_to(other, "coerce") {
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
                                        applied,
                                        vec![parts[1].clone()],
                                        position,
                                    )
                                    .map(Some);
                            }
                        }
                    }
                }
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
                // A String spells out a number for `Complex()` but is not one
                // to divide by.
                if matches!(other, Object::String(_))
                    || (!self.is_real_operand(other) && complex_parts(other).is_none())
                {
                    return Err(method_argument_type_error(
                        method_name,
                        "Numeric",
                        other,
                        position,
                    ));
                }
                // A divisor standing on the real line divides each part on
                // its own, so an infinite part stays infinite.
                if complex_parts(other).is_none() {
                    let divisor = self.part_to_float(other, position)?;
                    let left_real = self.part_to_float(&real, position)?;
                    let left_imaginary = self.part_to_float(&imaginary, position)?;
                    let new_real = self.divide_parts(&left_real, &divisor, position)?;
                    let new_imaginary = self.divide_parts(&left_imaginary, &divisor, position)?;
                    return self
                        .make_complex(new_real, new_imaginary, position)
                        .map(Some);
                }
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
                // Text reads as a number only where `Complex()` parses it,
                // never as an operand of the arithmetic.
                if matches!(other, Object::String(_)) {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "String can't be coerced into Complex",
                        position,
                    ));
                }
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
                // `coerce` takes a real number to pair with, so a String is
                // refused even though `Complex("2")` reads one, and so is a
                // Numeric that says it is not real.
                let real_number = !matches!(other, Object::String(_))
                    && self.is_real_operand(other)
                    && !matches!(
                        self.send_to_object(other.clone(), "real?", vec![], position),
                        Ok(Object::Bool(false))
                    );
                if !real_number {
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
}
