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
            "to_s" => Ok(Some(Object::string(format_complex(&real, &imaginary)))),
            "inspect" => Ok(Some(Object::string(format!(
                "({})",
                format_complex(&real, &imaginary)
            )))),
            "real?" => Ok(Some(Object::Bool(false))),
            "zero?" => Ok(Some(Object::Bool(is_zero(&real) && is_zero(&imaginary)))),
            "frozen?" => Ok(Some(Object::Bool(true))),
            "hash" => Ok(Some(Object::string(format_complex(&real, &imaginary)))),
            "==" | "eql?" | "!=" => {
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
        if method_name != "polar" {
            return self.make_complex(first, second, position).map(Some);
        }
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
        if is_zero(right) {
            return Ok(left.clone());
        }
        if is_zero(left) && operator == "+" {
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
