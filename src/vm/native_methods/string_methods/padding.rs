// Padding a string out to a width, and the stream methods STDOUT and
// STDERR answer while they are held as strings.

use super::*;

impl VirtualMachine {
    /// Padding a string out to a width, and the stream methods STDOUT and
    /// STDERR answer while they are held as strings.
    pub(crate) fn call_string_padding_method(
        &mut self,
        _receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "ljust" | "rjust" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let width = self.coerce_slice_number(&arguments[0], method_name, position)?;
                let mut pad_encoding = None;
                let pad = if arguments.len() == 2 {
                    // Anything that spells itself as text pads with those
                    // characters, which is what `to_str` is asked for.
                    let given = match &arguments[1] {
                        held @ Object::String(_) => held.clone(),
                        other if self.responds_to(other, "to_str") => {
                            self.send_to_object(other.clone(), "to_str", vec![], position)?
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
                    let Object::String(text) = &given else {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            &arguments[1],
                            position,
                        ));
                    };
                    if text.as_str().is_empty() {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "zero width padding",
                            position,
                        ));
                    }
                    // A pattern written in an encoding this string cannot be
                    // read alongside is refused rather than padded with.
                    if !strings_comparable(string_value, text) {
                        return Err(clashing_encodings_error(string_value, text, position));
                    }
                    pad_encoding = Some(text.encoding_name().to_string());
                    text.as_str().to_string()
                } else {
                    " ".to_string()
                };
                let current_len = string_value.as_str().chars().count() as i64;
                // Already as wide, the answer is a copy, written as the
                // receiver is.
                if width <= current_len {
                    return Ok(Some(Object::String(Rc::new((**string_value).clone()))));
                }
                let pad_chars: Vec<char> = pad.chars().collect();
                let needed = (width - current_len) as usize;
                let mut padding = String::new();
                for i in 0..needed {
                    padding.push(pad_chars[i % pad_chars.len()]);
                }
                let result = if method_name == "ljust" {
                    format!("{}{}", string_value, padding)
                } else {
                    format!("{}{}", padding, string_value)
                };
                let made = Object::string(result);
                // The padded string is written in whichever of the two
                // encodings holds both, which is the pattern's when the
                // padding brought characters the receiver's cannot spell.
                if let Object::String(built) = &made {
                    let wider = match &pad_encoding {
                        Some(name)
                            if *name != string_value.encoding_name() && !padding.is_ascii() =>
                        {
                            name.clone()
                        }
                        _ => string_value.encoding_name(),
                    };
                    built.set_encoding(wider);
                    if string_value.holds_bytes() {
                        built.mark_bytes();
                    }
                }
                Ok(Some(made))
            }
            "strip" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(
                    string_value
                        .as_str()
                        .trim_matches(|letter: char| letter.is_whitespace() || letter == '\0')
                        .to_string(),
                )))
            }
            // chomp([sep]) — strips a trailing separator. With no arg,
            // strips a final \n, \r\n, or \r. With a string arg, strips
            // exactly that suffix. With nil, returns the string unchanged.
            // metorex strings are immutable shared `Rc<String>`s, so the bang
            // form returns the stripped copy rather than mutating in place.
            "chomp" | "chomp!" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let bytes = binary_bytes(string_value);
                // With no separator named, the one `$/` holds decides, and
                // without a value of its own that is the line ending.
                let named = match arguments.first() {
                    None => match self.globals().get("/") {
                        Some(Object::String(held)) => Some(held.to_text()),
                        _ => Some("\n".to_string()),
                    },
                    Some(Object::Nil) => None,
                    Some(Object::String(held)) => Some(held.to_text()),
                    Some(other) => {
                        let spelled = self.string_argument(method_name, other, position)?;
                        Some(spelled)
                    }
                };
                let kept = chomped_bytes(&bytes, named.as_deref(), &string_value.encoding_name());
                // Text keeps reading as text; a run of bytes keeps standing
                // for the bytes it holds.
                let held = match (string_value.holds_bytes(), String::from_utf8(kept.clone())) {
                    (false, Ok(text)) => text,
                    _ => crate::vm::native_methods::pack_format::bytes_to_string(&kept).to_string(),
                };
                let made =
                    crate::object::StringValue::with_encoding(held, string_value.encoding_name());
                if string_value.holds_bytes() {
                    made.mark_bytes();
                }
                Ok(Some(Object::String(Rc::new(made))))
            }
            _ => Ok(None),
        }
    }
}
