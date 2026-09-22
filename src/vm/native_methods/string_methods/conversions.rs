// The number a string spells, read as an Integer, Float, Rational or
// Complex.

use super::*;

impl VirtualMachine {
    /// The number a string spells, read as an Integer, Float, Rational or
    /// Complex.
    pub(crate) fn call_string_conversion_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "each_char" => {
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
                    None => {
                        let size = character_count(string_value);
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                vec![],
                                Some(size),
                                position,
                            )
                            .map(Some);
                    }
                };
                for character in encoded_characters(string_value) {
                    self.execute_block_body(&block, vec![character])?;
                }
                Ok(Some(receiver.clone()))
            }
            "to_i" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A base of its own reads the digits of that base, so
                // `"ff".to_i(16)` is 255. Zero means "read the prefix".
                let base = match arguments.first() {
                    None => 10i64,
                    Some(Object::Int(held)) => *held,
                    Some(other) => {
                        let coerced = self.coerce_integer_argument(other, position)?;
                        coerced.try_into().unwrap_or(i64::MAX)
                    }
                };
                if base != 0 && !(2..=36).contains(&base) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("invalid radix {base}"),
                        position,
                    ));
                }
                let held = string_value.to_text();
                Ok(Some(Object::integer(leading_integer(
                    held.as_str(),
                    base as u32,
                ))))
            }
            // `to_c` reads the leading complex value and answers (0+0i)
            // when the string does not start with one.
            "to_c" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let named = string_value.encoding_name();
                if wide_encoding(&named).is_some() {
                    return Err(crate::vm::errors::simple_exception(
                        "Encoding::CompatibilityError",
                        &format!("ASCII incompatible encoding: {named}"),
                        position,
                    ));
                }
                let held = string_value.as_str().to_string();
                let Some(parsed) =
                    crate::vm::native_methods::complex_methods::leading_complex_text(&held)
                else {
                    return self
                        .make_complex(Object::Int(0), Object::Int(0), position)
                        .map(Some);
                };
                let real = self.component_object(parsed.real, position)?;
                let imaginary = self.component_object(parsed.imaginary, position)?;
                if parsed.polar {
                    let (real, imaginary) = self.polar_parts(real, imaginary, position)?;
                    return self.make_complex(real, imaginary, position).map(Some);
                }
                self.make_complex(real, imaginary, position).map(Some)
            }
            // `to_r` reads the leading rational value and answers (0/1) when
            // the string does not start with one.
            "to_r" => {
                let (numerator, denominator) =
                    crate::vm::native_methods::rational_methods::parse_rational_text(
                        &string_value.as_str(),
                    );
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
                Ok(Some(Object::Float(leading_float(
                    &string_value.as_ref().as_str(),
                ))))
            }
            _ => Ok(None),
        }
    }
}
