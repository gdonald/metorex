// Building a Complex from what `Complex()` was given.

use super::*;

impl VirtualMachine {
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
        let (positional, keywords) =
            crate::vm::native_methods::kernel_conversion::split_conversion_keywords(arguments);
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
        // Text spelling a number must be text the numbers can be read from,
        // which text in an encoding without ASCII is not.
        for held in positional.iter() {
            if let Object::String(text) = held {
                let named = text.encoding_name();
                if crate::vm::native_methods::string_methods::wide_encoding(&named).is_some() {
                    let message = format!("ASCII incompatible encoding: {named}");
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception(
                            "Encoding::CompatibilityError",
                            message.clone(),
                        ),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
            }
        }
        if positional.len() == 1 {
            return self.complex_from_single(&positional[0], raise, position);
        }
        // A String stands for the number it spells, which is read out rather
        // than taken as a part of its own.
        let spelled = |held: &Object| matches!(held, Object::String(_));
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
        if !spelled(&positional[1])
            && !self.is_real_operand(&positional[1])
            && !matches!(positional[1], Object::Nil)
        {
            if !raise {
                return Ok(Object::Nil);
            }
            return Err(not_a_real(self));
        }
        if !spelled(&positional[0])
            && !self.is_real_operand(&positional[0])
            && !matches!(positional[0], Object::Nil)
        {
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
            && !spelled(&positional[0])
            && !spelled(&positional[1])
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
    pub(crate) fn complex_from_single(
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
    pub(crate) fn operand_is_unreal(
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
    pub(crate) fn is_foreign_numeric(&mut self, value: &Object) -> bool {
        matches!(value, Object::Instance(_))
            && complex_parts(value).is_none()
            && crate::vm::native_methods::rational_methods::rational_parts(value).is_none()
            && self.value_is_numeric(value)
    }

    /// Whether a value is a real number metorex can use as a component.
    pub(crate) fn is_real_operand(&mut self, value: &Object) -> bool {
        if matches!(
            value,
            Object::Int(_) | Object::BigInt(_) | Object::Float(_) | Object::String(_)
        ) {
            return true;
        }
        if crate::vm::native_methods::rational_methods::rational_parts(value).is_some()
            || complex_parts(value).is_some()
        {
            return true;
        }
        self.value_is_numeric(value)
    }

    /// One argument to `Complex()`, as its real and imaginary parts. Answers
    /// None when the value could not be read and `exception: false` was given.
    pub(crate) fn complex_operand(
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
            _ if crate::vm::native_methods::rational_methods::rational_parts(value).is_some() => {
                Ok(Some((value.clone(), Object::Int(0))))
            }
            Object::String(text) => {
                if text.as_str().contains('\0') {
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
                let Some(parsed) = parse_complex_text(&text.as_str()) else {
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
    pub(crate) fn numeric_operation(
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
    pub(crate) fn value_is_numeric(&mut self, value: &Object) -> bool {
        let Some(Object::Class(numeric)) = self.globals().get("Numeric") else {
            return false;
        };
        let value_class = self.builtins().class_of(value);
        self.builtins().is_subclass_of(&value_class, &numeric)
    }

    /// Call a method by name on a value, however it is defined.
    pub(crate) fn invoke_named_method(
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
    pub(crate) fn numeric_to_float(
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
    pub(crate) fn polar_parts(
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
    pub(crate) fn component_object(
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
    pub(crate) fn numbers_equal(
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
