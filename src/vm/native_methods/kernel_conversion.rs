//! The Kernel conversion functions (`Hash()` and friends), reachable both as
//! bare calls and as `Kernel.Name(...)`.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Kernel's conversion functions, which Ruby exposes as private instance
/// methods on Kernel and as public singleton methods on the module.
pub(crate) const KERNEL_CONVERSION_FUNCTIONS: &[&str] = &[
    "Array", "Complex", "Float", "Hash", "Integer", "Rational", "String",
];

pub(crate) fn is_kernel_conversion(name: &str) -> bool {
    KERNEL_CONVERSION_FUNCTIONS.contains(&name)
}

/// Split the parser-marked keyword hash off the end of an argument list.
pub(crate) fn split_conversion_keywords(
    arguments: &[Object],
) -> (Vec<Object>, IndexMap<String, Object>) {
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            let keywords = dict
                .iter()
                .filter(|(key, _)| key.as_str() != "__MX_KWARGS__")
                .map(|(key, value)| {
                    (
                        key.strip_prefix(':').unwrap_or(key).to_string(),
                        value.clone(),
                    )
                })
                .collect();
            return (arguments[..arguments.len() - 1].to_vec(), keywords);
        }
    }
    (arguments.to_vec(), IndexMap::new())
}

/// Whether `error` is one of the conversion failures `exception: false` turns
/// into nil, as opposed to an error raised by the object being converted.
fn is_conversion_failure(error: &MetorexError) -> bool {
    match error {
        MetorexError::UncaughtException {
            exception: Object::Exception(cell),
            ..
        } => matches!(
            cell.borrow().exception_type.as_str(),
            "TypeError" | "ArgumentError" | "FloatDomainError"
        ),
        _ => false,
    }
}

fn argument_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("ArgumentError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

fn type_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("TypeError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

impl VirtualMachine {
    /// Run one of the Kernel conversion functions, or answer None when `name`
    /// is not one of them.
    pub(crate) fn call_kernel_conversion(
        &mut self,
        name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match name {
            "Array" => self.kernel_array(arguments, position).map(Some),
            "Complex" => self.kernel_complex(arguments, position).map(Some),
            "Float" => self.kernel_float(arguments, position).map(Some),
            "Hash" => self.kernel_hash(arguments, position).map(Some),
            "Integer" => self.kernel_integer(arguments, position).map(Some),
            "Rational" => self.kernel_rational(arguments, position).map(Some),
            "String" => self.kernel_string(arguments, position).map(Some),
            _ => Ok(None),
        }
    }

    /// `String(arg)` — a String is answered unchanged, anything else is put
    /// through `to_s`, which must exist and must answer a String.
    fn kernel_string(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() != 1 {
            return Err(super::super::errors::method_argument_error(
                "String",
                1,
                arguments.len(),
                position,
            ));
        }
        let value = &arguments[0];
        if matches!(value, Object::String(_)) || self.is_string_subclass_instance(value) {
            return Ok(value.clone());
        }

        // An object that overrides `respond_to?` gets the last word on
        // whether `to_s` may be called at all.
        if let Some((class, method)) = self.lookup_method(value, "respond_to?") {
            let answer = self.invoke_method(
                class,
                method,
                value.clone(),
                vec![Object::Symbol(Rc::new("to_s".to_string()))],
                position,
            )?;
            if !answer.is_truthy() {
                return Err(self.no_string_conversion(value, position));
            }
        }

        let converted = self.string_via_to_s(value, position)?;
        match converted {
            Object::String(_) => Ok(converted),
            other => {
                let source = self.conversion_class_name(value, position);
                let produced = self.conversion_class_name(&other, position);
                Err(type_error(
                    format!(
                        "can't convert {} to String ({}#to_s gives {})",
                        source, source, produced
                    ),
                    position,
                ))
            }
        }
    }

    /// Whether `value` is an instance of a class descending from String.
    fn is_string_subclass_instance(&mut self, value: &Object) -> bool {
        let Object::Instance(instance) = value else {
            return false;
        };
        let mut cursor = Some(Rc::clone(&instance.borrow().class));
        while let Some(class) = cursor {
            if class.name() == "String" {
                return true;
            }
            cursor = class.superclass();
        }
        false
    }

    fn no_string_conversion(&mut self, value: &Object, position: Position) -> MetorexError {
        let source = self.conversion_class_name(value, position);
        type_error(format!("can't convert {} into String", source), position)
    }

    /// Call `to_s`, honouring an undefined `to_s` that `method_missing` picks
    /// up, and refusing the generic fallback when the method was undefined.
    fn string_via_to_s(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some((class, method)) = self.lookup_method(value, "to_s") {
            if !method.is_undefined {
                return self.invoke_method(class, method, value.clone(), vec![], position);
            }
            let Some((missing_class, missing)) = self.lookup_method(value, "method_missing") else {
                return Err(self.no_string_conversion(value, position));
            };
            return self
                .invoke_method(
                    missing_class,
                    missing,
                    value.clone(),
                    vec![Object::Symbol(Rc::new("to_s".to_string()))],
                    position,
                )
                .map_err(|_| self.no_string_conversion(value, position));
        }

        let class = self.builtins().class_of(value);
        if let Some(result) = self.call_native_method(&class, value, "to_s", &[], position)? {
            return Ok(result);
        }
        if let Some(result) = self.call_object_method(value, "to_s", &[], position)? {
            return Ok(result);
        }
        Err(self.no_string_conversion(value, position))
    }

    /// `Rational(numerator, denominator = 1, exception: true)`.
    fn kernel_rational(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (positional, keywords) = split_conversion_keywords(arguments);
        let raise = !matches!(keywords.get("exception"), Some(Object::Bool(false)));
        if positional.is_empty() || positional.len() > 2 {
            return Err(super::super::errors::method_argument_error(
                "Rational",
                1,
                positional.len(),
                position,
            ));
        }

        match self.rational_conversion(&positional, position) {
            Ok(value) => Ok(value),
            // `exception: false` also swallows whatever `to_r` or `to_int`
            // raised, not only the conversion's own complaints.
            Err(_) if !raise => Ok(Object::Nil),
            Err(error) => Err(error),
        }
    }

    fn rational_conversion(
        &mut self,
        positional: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Dividing by a Complex with a non-zero imaginary part gives a
        // Complex, not a Rational: a / (c + di) is (ac - adi) / (c² + d²).
        if let Some(divisor) = positional.get(1)
            && let Some((real, imaginary)) = super::rational_methods::complex_parts(divisor)
            && !super::rational_methods::is_zero(&imaginary)
        {
            return self.rational_over_complex(&positional[0], &real, &imaginary, position);
        }

        let (numerator, scale) = self.rational_operand(&positional[0], position)?;
        let (denominator, denominator_scale) = match positional.get(1) {
            None => (num_bigint::BigInt::from(1), num_bigint::BigInt::from(1)),
            Some(value) => self.rational_operand(value, position)?,
        };
        // Dividing (a/b) by (c/d) is (a*d)/(b*c).
        self.make_rational(numerator * denominator_scale, scale * denominator, position)
    }

    /// `Rational(a, c + di)` with a non-zero `d`.
    fn rational_over_complex(
        &mut self,
        numerator: &Object,
        real: &Object,
        imaginary: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Object::Int(real), Object::Int(imaginary)) = (real, imaginary) else {
            let message = format!(
                "can't convert {} into Rational",
                super::rational_methods::format_complex(real, imaginary)
            );
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("RangeError", message.clone()),
                location: position_to_location(position),
                message,
            });
        };
        let (top, bottom) = self.rational_operand(numerator, position)?;
        let magnitude = real * real + imaginary * imaginary;
        let real_part = self.make_rational(&top * real, &bottom * magnitude, position)?;
        let imaginary_part =
            self.make_rational(-&top * imaginary, &bottom * magnitude, position)?;
        self.make_complex(real_part, imaginary_part, position)
    }

    /// One argument to `Rational()`, as an exact numerator/denominator pair.
    fn rational_operand(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<(num_bigint::BigInt, num_bigint::BigInt), MetorexError> {
        use super::rational_methods::{
            complex_parts, format_complex, is_zero, parse_strict_rational_text, rational_parts,
        };

        match value {
            Object::Int(_) | Object::BigInt(_) => Ok((
                value.as_big_integer().expect("integer-kinded"),
                num_bigint::BigInt::from(1),
            )),
            Object::Float(number) if number.is_finite() => {
                Ok(super::rational_methods::float_fraction(*number))
            }
            // `Rational("1/3")` and `Rational(".52")` are exact; text that is
            // not wholly a rational is refused rather than read leniently the
            // way `String#to_r` reads it.
            Object::String(text) => parse_strict_rational_text(text).ok_or_else(|| {
                argument_error(
                    format!("invalid value for convert(): {:?}", text.as_str()),
                    position,
                )
            }),
            // nil, symbols, and collections have no rational value at all,
            // and never reach a coercion method.
            Object::Nil => Err(type_error(
                "can't convert nil into Rational".to_string(),
                position,
            )),
            Object::Symbol(_)
            | Object::Bool(_)
            | Object::Array(_)
            | Object::Dict(_)
            | Object::Float(_) => Err(type_error(
                format!(
                    "can't convert {} into Rational",
                    self.conversion_class_name(value, position)
                ),
                position,
            )),
            _ => {
                if let Some(parts) = rational_parts(value) {
                    return Ok(parts);
                }
                if let Some((real, imaginary)) = complex_parts(value) {
                    if is_zero(&imaginary) {
                        return self.rational_operand(&real, position);
                    }
                    let message = format!(
                        "can't convert {} into Rational",
                        format_complex(&real, &imaginary)
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("RangeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                self.rational_via_coercion(value, position)
            }
        }
    }

    /// `to_r` first, then `to_int`, matching how Ruby narrows an arbitrary
    /// object down to an exact value.
    fn rational_via_coercion(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<(num_bigint::BigInt, num_bigint::BigInt), MetorexError> {
        let source = self.conversion_class_name(value, position);
        let attempt = self.try_conversion_call(value, "to_r", position);
        let converted = match attempt {
            Ok(converted) => converted,
            Err(_) => {
                return Err(type_error(
                    format!("can't convert {} into Rational", source),
                    position,
                ));
            }
        };
        if let Some(converted) = converted {
            if let Some(parts) = super::rational_methods::rational_parts(&converted) {
                return Ok(parts);
            }
            if let Object::Int(_) | Object::BigInt(_) = converted {
                return Ok((
                    converted.as_big_integer().expect("integer-kinded"),
                    num_bigint::BigInt::from(1),
                ));
            }
            let produced = self.conversion_class_name(&converted, position);
            return Err(type_error(
                format!(
                    "can't convert {} into Rational ({}#to_r gives {})",
                    source, source, produced
                ),
                position,
            ));
        }
        match self.try_conversion_call(value, "to_int", position) {
            Ok(Some(Object::Int(number))) => Ok((
                num_bigint::BigInt::from(number),
                num_bigint::BigInt::from(1),
            )),
            _ => Err(type_error(
                format!("can't convert {} into Rational", source),
                position,
            )),
        }
    }

    /// `Integer(value, base = nil, exception: true)`.
    fn kernel_integer(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (positional, keywords) = split_conversion_keywords(arguments);
        let raise = !matches!(keywords.get("exception"), Some(Object::Bool(false)));
        if positional.is_empty() || positional.len() > 2 {
            return Err(super::super::errors::method_argument_error(
                "Integer",
                1,
                positional.len(),
                position,
            ));
        }

        let result = self.integer_conversion(&positional, position);
        match result {
            Ok(value) => Ok(value),
            Err(error) if !raise && is_conversion_failure(&error) => Ok(Object::Nil),
            Err(error) => Err(error),
        }
    }

    fn integer_conversion(
        &mut self,
        positional: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let base = match positional.get(1) {
            None => None,
            Some(Object::Int(base)) => Some(*base),
            Some(other) => Some(self.coerce_to_integer_base(other, position)?),
        };

        match &positional[0] {
            Object::String(text) => match parse_integer_literal(text, base) {
                Ok(value) => Ok(Object::integer(value)),
                Err(ParseFailure::Malformed) => Err(argument_error(
                    format!("invalid value for Integer(): {:?}", text.as_str()),
                    position,
                )),
            },
            other if base.is_some() => Err(argument_error(
                format!(
                    "base specified for non string value ({})",
                    self.conversion_class_name(other, position)
                ),
                position,
            )),
            Object::Int(_) | Object::BigInt(_) => Ok(positional[0].clone()),
            Object::Float(value) => {
                if value.is_nan() || value.is_infinite() {
                    let label = if value.is_nan() {
                        "NaN".to_string()
                    } else if *value > 0.0 {
                        "Infinity".to_string()
                    } else {
                        "-Infinity".to_string()
                    };
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("FloatDomainError", label.clone()),
                        location: position_to_location(position),
                        message: label,
                    });
                }
                Ok(super::float_methods::float_to_integer(value.trunc()))
            }
            Object::Nil => Err(type_error(
                "can't convert nil into Integer".to_string(),
                position,
            )),
            other => self.integer_via_coercion(other, position),
        }
    }

    /// `to_int` first, then `to_i`. A non-Integer answer from `to_int` is not
    /// fatal on its own: Ruby falls through to `to_i` and only gives up if
    /// that answer is not an Integer either.
    fn integer_via_coercion(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        for name in ["to_int", "to_i"] {
            match self.try_conversion_call(value, name, position)? {
                Some(converted @ (Object::Int(_) | Object::BigInt(_))) => return Ok(converted),
                _ => continue,
            }
        }
        Err(type_error(
            format!(
                "can't convert {} into Integer",
                self.conversion_class_name(value, position)
            ),
            position,
        ))
    }

    /// The base argument accepts anything answering `to_int`.
    fn coerce_to_integer_base(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        if let Some(Object::Int(number)) = self.try_conversion_call(value, "to_int", position)? {
            return Ok(number);
        }
        Err(type_error(
            format!(
                "no implicit conversion of {} into Integer",
                self.conversion_class_name(value, position)
            ),
            position,
        ))
    }

    /// The class name to use in a conversion error. Goes through `class` so
    /// true, false, and nil name TrueClass, FalseClass, and NilClass rather
    /// than Object.
    fn conversion_class_name(&mut self, value: &Object, position: Position) -> String {
        match self.call_object_method(value, "class", &[], position) {
            Ok(Some(Object::Class(class))) => class.ruby_name(),
            _ => self.builtins().class_of(value).ruby_name(),
        }
    }

    /// Call `name` on `value` when it has one, whether the method is defined
    /// in Ruby or lives in a native table. Answers None when the receiver has
    /// no such method, which is what distinguishes "cannot convert" from a
    /// conversion that returned something unusable.
    fn try_conversion_call(
        &mut self,
        value: &Object,
        name: &str,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Some((class, method)) = self.lookup_method(value, name) {
            if method.is_undefined {
                return Ok(None);
            }
            return self
                .invoke_method(class, method, value.clone(), vec![], position)
                .map(Some);
        }
        let class = self.builtins().class_of(value);
        self.call_native_method(&class, value, name, &[], position)
    }

    /// `Hash(arg)` — nil and the empty array become `{}`, a Hash is returned
    /// untouched, and anything else must answer `to_hash` with a Hash.
    /// `Array(arg)` — an Array is answered unchanged. Anything else is put
    /// through `to_ary` and then `to_a`, either of which may be private. A
    /// conversion answering nil moves on to the next one, one answering a
    /// non-Array raises TypeError, and an argument with neither is wrapped in
    /// a one-element Array. nil answers an empty Array.
    fn kernel_array(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() != 1 {
            return Err(super::super::errors::method_argument_error(
                "Array",
                1,
                arguments.len(),
                position,
            ));
        }
        let value = &arguments[0];
        match value {
            Object::Array(_) => return Ok(value.clone()),
            Object::Nil => return Ok(Object::Array(Rc::new(RefCell::new(Vec::new())))),
            _ => {}
        }
        for conversion in ["to_ary", "to_a"] {
            let Some((class, method)) = self.lookup_method(value, conversion) else {
                continue;
            };
            if method.is_undefined {
                continue;
            }
            let converted = self.invoke_method(class, method, value.clone(), vec![], position)?;
            match converted {
                Object::Array(_) => return Ok(converted),
                Object::Nil => continue,
                other => {
                    let source = self.conversion_class_name(value, position);
                    let produced = self.conversion_class_name(&other, position);
                    return Err(type_error(
                        format!(
                            "can't convert {} to Array ({}#{} gives {})",
                            source, source, conversion, produced
                        ),
                        position,
                    ));
                }
            }
        }
        Ok(Object::Array(Rc::new(RefCell::new(vec![value.clone()]))))
    }

    /// `Float(arg)` — a Float is answered unchanged, an Integer converts
    /// exactly, a String is read strictly, and anything else must answer
    /// `to_f` with a Float. `exception: false` answers nil instead of raising.
    fn kernel_float(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (positional, keywords) = split_conversion_keywords(arguments);
        let raise = keywords
            .get("exception")
            .map(|value| value.is_truthy())
            .unwrap_or(true);
        if positional.len() != 1 {
            return Err(super::super::errors::method_argument_error(
                "Float",
                1,
                positional.len(),
                position,
            ));
        }
        let value = &positional[0];
        let refuse = |vm: &mut Self, kind: &str, message: String| -> Result<Object, MetorexError> {
            let _ = vm;
            if !raise {
                return Ok(Object::Nil);
            }
            Err(MetorexError::UncaughtException {
                exception: Object::exception(kind, message.clone()),
                location: position_to_location(position),
                message,
            })
        };
        match value {
            Object::Float(_) => return Ok(value.clone()),
            Object::Int(number) => return Ok(Object::Float(*number as f64)),
            Object::BigInt(number) => {
                return Ok(Object::Float(
                    number.to_string().parse::<f64>().unwrap_or(f64::INFINITY),
                ));
            }
            Object::String(text) => {
                return match parse_strict_float_text(text) {
                    Some(number) => Ok(Object::Float(number)),
                    None => refuse(
                        self,
                        "ArgumentError",
                        format!("invalid value for Float(): {:?}", text.as_str()),
                    ),
                };
            }
            Object::Nil => {
                return refuse(
                    self,
                    "TypeError",
                    "can't convert nil into Float".to_string(),
                );
            }
            _ => {}
        }
        // A Complex is a Float only when it has no imaginary part.
        if let Some((real, imaginary)) = super::rational_methods::complex_parts(value) {
            if !super::rational_methods::is_zero(&imaginary) {
                let rendered = super::rational_methods::format_complex(&real, &imaginary);
                return refuse(
                    self,
                    "RangeError",
                    format!("can't convert {} into Float", rendered),
                );
            }
            return self.kernel_float(&[real], position);
        }
        if super::rational_methods::rational_parts(value).is_some() {
            let converted = self.invoke_conversion(value, "to_f", position)?;
            return Ok(converted);
        }
        let source = self.conversion_class_name(value, position);
        if !self.responds_to(value, "to_f") {
            return refuse(
                self,
                "TypeError",
                format!("can't convert {} into Float", source),
            );
        }
        let converted = self.invoke_conversion(value, "to_f", position)?;
        match converted {
            Object::Float(_) => Ok(converted),
            other => {
                let produced = self.conversion_class_name(&other, position);
                refuse(
                    self,
                    "TypeError",
                    format!(
                        "can't convert {} to Float ({}#to_f gives {})",
                        source, source, produced
                    ),
                )
            }
        }
    }

    /// Call a conversion method on a value.
    fn invoke_conversion(
        &mut self,
        value: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some((class, method)) = self.lookup_method(value, method_name) else {
            return Ok(Object::Nil);
        };
        self.invoke_method(class, method, value.clone(), Vec::new(), position)
    }

    fn kernel_hash(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() != 1 {
            return Err(super::super::errors::method_argument_error(
                "Hash",
                1,
                arguments.len(),
                position,
            ));
        }
        let empty = || Object::Dict(Rc::new(RefCell::new(IndexMap::new())));
        match &arguments[0] {
            Object::Nil => Ok(empty()),
            Object::Array(elements) if elements.borrow().is_empty() => Ok(empty()),
            Object::Dict(_) => Ok(arguments[0].clone()),
            other => {
                let source = self.conversion_class_name(other, position);
                let Some((class, method)) = self.lookup_method(other, "to_hash") else {
                    return Err(type_error(
                        format!("can't convert {} into Hash", source),
                        position,
                    ));
                };
                let converted =
                    self.invoke_method(class, method, other.clone(), vec![], position)?;
                match converted {
                    Object::Dict(_) => Ok(converted),
                    other_result => {
                        let produced = self.conversion_class_name(&other_result, position);
                        Err(type_error(
                            format!(
                                "can't convert {} to Hash ({}#to_hash gives {})",
                                source, source, produced
                            ),
                            position,
                        ))
                    }
                }
            }
        }
    }
}

/// Why a string could not be read as an integer. Magnitude is never a reason,
/// since an integer of any size is representable.
enum ParseFailure {
    Malformed,
}

/// Read `text` the way `Integer()` does: optional surrounding whitespace, a
/// single sign, a radix prefix, and digits separated by lone underscores.
/// `base` of None means "detect from the prefix, otherwise decimal".
fn parse_integer_literal(
    text: &str,
    base: Option<i64>,
) -> Result<num_bigint::BigInt, ParseFailure> {
    if text.contains('\0') {
        return Err(ParseFailure::Malformed);
    }
    let trimmed =
        text.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c'));

    let mut rest = trimmed;
    let mut negative = false;
    if let Some(stripped) = rest.strip_prefix('+') {
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix('-') {
        negative = true;
        rest = stripped;
    }

    let requested = base.unwrap_or(0);
    if requested != 0 && !(2..=36).contains(&requested) {
        return Err(ParseFailure::Malformed);
    }

    // A radix prefix only counts when it agrees with the requested base. For
    // any other base the prefix letter is just a digit, valid or not.
    let lowered = rest.to_ascii_lowercase();
    let prefix_radix = if lowered.starts_with("0x") {
        Some(16)
    } else if lowered.starts_with("0b") {
        Some(2)
    } else if lowered.starts_with("0o") {
        Some(8)
    } else if lowered.starts_with("0d") {
        Some(10)
    } else {
        None
    };

    let mut radix = requested;
    if let Some(prefix) = prefix_radix
        && (requested == 0 || requested == prefix)
    {
        radix = prefix;
        rest = &rest[2..];
    } else if requested == 0 && rest.len() > 1 && rest.starts_with('0') {
        // A bare leading zero means octal, as in `Integer("010") == 8`.
        radix = 8;
        rest = &rest[1..];
    }
    if radix == 0 {
        radix = 10;
    }

    let mut digits = String::with_capacity(rest.len());
    let mut previous_underscore = false;
    for character in rest.chars() {
        if character == '_' {
            if digits.is_empty() || previous_underscore {
                return Err(ParseFailure::Malformed);
            }
            previous_underscore = true;
            continue;
        }
        if character.to_digit(radix as u32).is_none() {
            return Err(ParseFailure::Malformed);
        }
        digits.push(character);
        previous_underscore = false;
    }
    if previous_underscore || digits.is_empty() {
        return Err(ParseFailure::Malformed);
    }

    let magnitude = num_bigint::BigInt::parse_bytes(digits.as_bytes(), radix as u32)
        .ok_or(ParseFailure::Malformed)?;
    Ok(if negative { -magnitude } else { magnitude })
}

/// Read a Float the way `Float("...")` does: strict, with `_` allowed only
/// between digits, whitespace allowed only at the ends, and both decimal and
/// `0x` hexadecimal forms accepted. Text that is not wholly a number, or that
/// holds a null byte, answers None.
pub(crate) fn parse_strict_float_text(text: &str) -> Option<f64> {
    if text.contains('\0') {
        return None;
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (sign, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1.0, rest),
        None => (1.0, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    let magnitude = if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        parse_hexadecimal_float(hex)?
    } else {
        parse_decimal_float(digits)?
    };
    Some(sign * magnitude)
}

/// Strip the `_` separators from a run of digits, refusing one that is not
/// between two digits the predicate accepts.
fn strip_separators(text: &str, is_digit: fn(char) -> bool) -> Option<String> {
    let characters: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (index, character) in characters.iter().enumerate() {
        if *character != '_' {
            out.push(*character);
            continue;
        }
        let before = index.checked_sub(1).and_then(|i| characters.get(i));
        let after = characters.get(index + 1);
        if !before.copied().is_some_and(is_digit) || !after.copied().is_some_and(is_digit) {
            return None;
        }
    }
    Some(out)
}

/// The unsigned decimal form: digits, an optional fraction, an optional
/// `e` exponent. Ruby allows a trailing `.` and a missing fractional part.
fn parse_decimal_float(text: &str) -> Option<f64> {
    let (mantissa, exponent) = match text.find(['e', 'E']) {
        Some(index) => {
            let (mantissa, rest) = text.split_at(index);
            (mantissa, Some(&rest[1..]))
        }
        None => (text, None),
    };
    let mantissa = strip_separators(mantissa, |c| c.is_ascii_digit())?;
    let mantissa = mantissa.strip_suffix('.').unwrap_or(&mantissa);
    if mantissa.is_empty() || !mantissa.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    if mantissa.matches('.').count() > 1 || mantissa.starts_with('.') {
        return None;
    }
    let mantissa: f64 = mantissa.parse().ok()?;
    let Some(exponent) = exponent else {
        return Some(mantissa);
    };
    let exponent = strip_separators(exponent, |c| c.is_ascii_digit())?;
    let (exponent_sign, exponent_digits) = match exponent.strip_prefix('-') {
        Some(rest) => (-1.0_f64, rest),
        None => (1.0_f64, exponent.strip_prefix('+').unwrap_or(&exponent)),
    };
    if exponent_digits.is_empty() || !exponent_digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let exponent: f64 = exponent_digits.parse().ok()?;
    Some(mantissa * 10.0_f64.powf(exponent_sign * exponent))
}

/// The unsigned `0x` form: hex digits, an optional hex fraction, an optional
/// `p` binary exponent written in decimal.
fn parse_hexadecimal_float(text: &str) -> Option<f64> {
    let (mantissa, exponent) = match text.find(['p', 'P']) {
        Some(index) => {
            let (mantissa, rest) = text.split_at(index);
            (mantissa, Some(&rest[1..]))
        }
        None => (text, None),
    };
    let mantissa = strip_separators(mantissa, |c| c.is_ascii_hexdigit())?;
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (mantissa.as_str(), ""),
    };
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    if !fraction.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let mut value = 0.0_f64;
    for digit in whole.chars() {
        value = value * 16.0 + digit.to_digit(16)? as f64;
    }
    let mut place = 1.0 / 16.0;
    for digit in fraction.chars() {
        value += digit.to_digit(16)? as f64 * place;
        place /= 16.0;
    }
    let Some(exponent) = exponent else {
        return Some(value);
    };
    let exponent = strip_separators(exponent, |c| c.is_ascii_digit())?;
    let (exponent_sign, exponent_digits) = match exponent.strip_prefix('-') {
        Some(rest) => (-1.0_f64, rest),
        None => (1.0_f64, exponent.strip_prefix('+').unwrap_or(&exponent)),
    };
    if exponent_digits.is_empty() || !exponent_digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let exponent: f64 = exponent_digits.parse().ok()?;
    Some(value * 2.0_f64.powf(exponent_sign * exponent))
}
