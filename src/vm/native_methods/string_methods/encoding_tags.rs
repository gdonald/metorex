// The encoding a string says it is written in, and what changing that
// tag does to it.

use super::*;

impl VirtualMachine {
    /// The encoding a string says it is written in, and what changing that
    /// tag does to it.
    pub(crate) fn call_string_encoding_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `unpack` reads the bytes back as the directives describe them,
            // and `unpack1` answers the first of them.
            "unpack" | "unpack1" => {
                // `offset:` names where in the bytes the unpacking starts.
                let mut positional = arguments;
                let mut from = 0usize;
                if let Some(Object::Dict(options)) = arguments.last() {
                    let named = options.borrow().get(":offset").cloned();
                    if let Some(held) = named {
                        let counted: i64 = self
                            .coerce_integer_argument(&held, position)?
                            .try_into()
                            .unwrap_or(i64::MAX);
                        if counted < 0 {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "offset can't be negative",
                                position,
                            ));
                        }
                        let bytes = binary_bytes(string_value).len() as i64;
                        if counted > bytes {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "offset outside of string",
                                position,
                            ));
                        }
                        from = counted as usize;
                    }
                    positional = &arguments[..arguments.len() - 1];
                }
                let arguments = positional;
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let format = match &arguments[0] {
                    Object::String(format) => format.as_str().to_string(),
                    other if self.responds_to(other, "to_str") => {
                        match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                            Object::String(format) => format.as_str().to_string(),
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "String",
                                    other,
                                    position,
                                ));
                            }
                        }
                    }
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                // Every directive reads bytes, so the string is handed over
                // as the bytes it holds rather than as the characters they
                // spell. An offset counts bytes too.
                let bytes = binary_bytes(string_value);
                let held = match crate::vm::native_methods::pack_format::bytes_to_string(
                    &bytes[from.min(bytes.len())..],
                ) {
                    Object::String(rest) => rest.as_str().to_string(),
                    _ => String::new(),
                };
                let read = self.string_unpack(&held, &format, string_value.pointer(), position)?;
                if method_name == "unpack1" {
                    return Ok(Some(read.into_iter().next().unwrap_or(Object::Nil)));
                }
                Ok(Some(Object::array(read)))
            }
            // `force_encoding` tags the string as being in another encoding
            // without touching what it holds, which is what Ruby does for a
            // string whose bytes are already right for the new one.
            // The characters of a literal written in escapes stand for the
            // bytes they named, which is what the literal says of itself.
            // `"text".freeze` written out in the source stands for one
            // frozen string, which every place writing it shares.
            "__frozen_literal__" => Ok(Some(self.deduped_string(string_value))),
            "__mutable_literal__" => {
                string_value.take_chill();
                Ok(Some(receiver.clone()))
            }
            "__holds_bytes__" => {
                let held = string_value.as_str().to_string();
                let made = crate::object::StringValue::from_bytes(held);
                // The literal is still written in the source's encoding; only
                // the characters stand for bytes.
                made.set_encoding(string_value.encoding_name());
                Ok(Some(Object::String(Rc::new(made))))
            }
            // A literal in a source written in bytes stands for those bytes.
            "__binary_literal__" => {
                let held = string_value.as_str().to_string();
                Ok(Some(Object::String(Rc::new(
                    crate::object::StringValue::from_bytes(held),
                ))))
            }
            // `unicode_normalize` answers the text put into one of the four
            // forms Unicode names, and `unicode_normalized?` whether it is
            // already in one.
            "unicode_normalize" | "unicode_normalized?" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let named = string_value.encoding_name();
                if !matches!(named.as_str(), "UTF-8" | "US-ASCII" | "UTF8-MAC") {
                    return Err(crate::vm::errors::simple_exception(
                        "Encoding::CompatibilityError",
                        &format!("Unicode Normalization not appropriate for {named}"),
                        position,
                    ));
                }
                let form = match arguments.first() {
                    None => "nfc".to_string(),
                    Some(Object::Symbol(held) | Object::String(held)) => held.as_str().to_string(),
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Symbol",
                            other,
                            position,
                        ));
                    }
                };
                if !matches!(form.as_str(), "nfc" | "nfd" | "nfkc" | "nfkd") {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("invalid normalization form {form}"),
                        position,
                    ));
                }
                use crate::vm::native_methods::normalization_table::{
                    composed, decomposed, text_of,
                };
                let wholly = form.starts_with("nfk");
                let points = decomposed(&string_value.as_str(), wholly);
                let points = if form.ends_with('c') {
                    composed(&points)
                } else {
                    points
                };
                let written = text_of(&points);
                if method_name == "unicode_normalized?" {
                    return Ok(Some(Object::Bool(written == *string_value.as_str())));
                }
                let made = crate::object::StringValue::with_encoding(written, named);
                Ok(Some(Object::String(Rc::new(made))))
            }
            "force_encoding" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if string_value.is_frozen() {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let named = self.encoding_name_argument(&arguments[0], position)?;
                // Re-tagging says how to read the bytes the string already
                // holds, so from here on they are read as bytes rather than
                // as the characters they were written as.
                let fixed_width = matches!(
                    named.as_str(),
                    "UTF-16" | "UTF-16BE" | "UTF-16LE" | "UTF-32" | "UTF-32BE" | "UTF-32LE"
                );
                // Read as bytes, each byte is a character of its own, so the
                // text is spread out to say so.
                let by_the_byte = matches!(named.as_str(), "ASCII-8BIT" | "BINARY");
                if by_the_byte && !string_value.holds_bytes() {
                    let bytes = binary_bytes(string_value);
                    string_value.replace_text(
                        crate::vm::native_methods::pack_format::bytes_to_string(&bytes).to_string(),
                    );
                    string_value.mark_bytes();
                }
                // Read back through an encoding that spells what the bytes
                // hold, the string is text again.
                if !by_the_byte
                    && !fixed_width
                    && string_value.holds_bytes()
                    && matches!(named.as_str(), "UTF-8" | "US-ASCII")
                    && let Ok(text) = String::from_utf8(binary_bytes(string_value))
                {
                    string_value.replace_text(text);
                    string_value.clear_bytes();
                }
                string_value.set_encoding(named);
                if fixed_width {
                    string_value.mark_bytes();
                }
                Ok(Some(receiver.clone()))
            }
            // The encoding this string says it is in.
            "encoding" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let named = string_value.encoding_name();
                Ok(Some(self.encoding_object(&named)))
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
                Ok(Some(Object::Int(character_count(string_value))))
            }
            _ => Ok(None),
        }
    }
}
