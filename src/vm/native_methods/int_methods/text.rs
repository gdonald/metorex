// The character a code names, and the digits a number reads back as.

use super::*;

impl VirtualMachine {
    /// The character a code names, and the digits a number reads back as.
    pub(crate) fn call_int_text_method(
        &mut self,
        receiver: &Object,
        n: &i64,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `chr` names the character a code point stands for.
            // `chr` names the character a number stands for. With no
            // encoding, a number under 128 is ASCII and one up to 255 is a
            // byte of its own, and anything wider needs an encoding named.
            "chr" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A number too wide for a machine word names no character in
                // any encoding.
                let code = match receiver {
                    Object::Int(code) => *code,
                    Object::BigInt(_) => {
                        return Err(crate::vm::errors::simple_exception(
                            "RangeError",
                            "bignum out of char range",
                            position,
                        ));
                    }
                    _ => return Ok(None),
                };
                let out_of_range = || {
                    let message = format!("{} out of char range", code);
                    crate::vm::errors::simple_exception("RangeError", &message, position)
                };
                if code < 0 {
                    return Err(out_of_range());
                }
                let named = match arguments.first() {
                    Some(held) => {
                        let written = match held {
                            Object::String(text) => text.to_text(),
                            other => self.get_string_representation(other, position)?,
                        };
                        Some(
                            crate::vm::native_methods::string_methods::canonical_encoding_name(
                                &written,
                            ),
                        )
                    }
                    // Written with no encoding, a code in the ASCII range is
                    // ASCII and one above it is a byte, whatever encoding the
                    // program reads text in.
                    None => {
                        if (0..=127).contains(&code) {
                            return Ok(Some(one_byte_string(code as u8, "US-ASCII")));
                        }
                        if (128..=255).contains(&code) {
                            return Ok(Some(one_byte_string(code as u8, "ASCII-8BIT")));
                        }
                        match self.globals().get("__Encoding_default_internal") {
                            Some(Object::Nil) | None => return Err(out_of_range()),
                            Some(held) => {
                                let written = self.get_string_representation(&held, position)?;
                                Some(
                                    crate::vm::native_methods::string_methods::canonical_encoding_name(
                                        &written,
                                    ),
                                )
                            }
                        }
                    }
                };
                let named = named.unwrap_or_else(|| "US-ASCII".to_string());
                let Some(made) = character_in_encoding(code, &named) else {
                    return Err(out_of_range());
                };
                Ok(Some(made))
            }
            "to_s" | "inspect" if !arguments.is_empty() => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A base of its own writes the digits of that base, so
                // `255.to_s(16)` is "ff".
                let Object::Int(base) = arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                if !(2..=36).contains(&base) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("invalid radix {base}"),
                        position,
                    ));
                }
                let written = num_bigint::BigInt::from(*n).to_str_radix(base as u32);
                Ok(Some(ascii_string(written)))
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
                Ok(Some(ascii_string(n.to_string())))
            }
            _ => Ok(None),
        }
    }
}
