// Whether a string holds, opens with, or closes on another.

use super::*;

impl VirtualMachine {
    /// Whether a string holds, opens with, or closes on another.
    pub(crate) fn call_string_searching_method(
        &mut self,
        _receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "include?" | "contains?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Text written in an encoding this string cannot be read
                // alongside is refused rather than searched for.
                if let Object::String(other) = &arguments[0]
                    && !strings_comparable(string_value, other)
                {
                    return Err(clashing_encodings_error(string_value, other, position));
                }
                // Anything that spells itself as text is read as the text it
                // spells, which is how a wrapper around a String is searched
                // for.
                let wanted = self.string_argument(method_name, &arguments[0], position)?;
                let held = string_value.as_str().contains(wanted.as_str());
                Ok(Some(Object::Bool(held)))
            }
            "start_with?" | "end_with?" => {
                // With no arguments nothing matches, which is what Ruby
                // answers rather than refusing the call.
                let mut result = false;
                for arg in arguments {
                    // An argument that is not a String is asked for one, which
                    // is what `to_str` answers.
                    let arg = &match arg {
                        Object::String(_) => arg.clone(),
                        other if self.responds_to(other, "to_str") => {
                            self.send_to_object(other.clone(), "to_str", vec![], position)?
                        }
                        other => other.clone(),
                    };
                    match arg {
                        Object::String(s) => {
                            // Text written in an encoding this string cannot
                            // be read alongside is refused rather than
                            // compared.
                            if !strings_comparable(string_value, s) {
                                return Err(clashing_encodings_error(string_value, s, position));
                            }
                            // A match has to start where a character does, so
                            // the ends are compared by character rather than
                            // by byte. An encoding of more than one byte to a
                            // character is read as characters first.
                            let fits =
                                if let Some(shape) = wide_encoding(&string_value.encoding_name()) {
                                    let letters: Vec<char> =
                                        wide_text(&binary_bytes(string_value), shape)
                                            .chars()
                                            .collect();
                                    let wanted = binary_bytes(s);
                                    (1..=letters.len()).any(|count| {
                                        let taken: String = if method_name == "start_with?" {
                                            letters[..count].iter().collect()
                                        } else {
                                            letters[letters.len() - count..].iter().collect()
                                        };
                                        wide_bytes(&taken, shape) == wanted
                                    })
                                } else {
                                    let held: Vec<char> = string_value.as_str().chars().collect();
                                    let wanted: Vec<char> = s.as_str().chars().collect();
                                    wanted.len() <= held.len()
                                        && if method_name == "start_with?" {
                                            held[..wanted.len()] == wanted[..]
                                        } else {
                                            held[held.len() - wanted.len()..] == wanted[..]
                                        }
                                };
                            if fits {
                                result = true;
                                break;
                            }
                        }
                        // `start_with?` also takes a Regexp, which matches at
                        // the front of the string and records the match the
                        // way any other match does.
                        Object::Regex(pattern, flags) if method_name == "start_with?" => {
                            let anchored = format!("\\A(?:{})", pattern);
                            let subject = string_value.as_str().to_string();
                            let found =
                                self.regexp_match_data(&anchored, flags, &subject, 0, position)?;
                            if found.is_some() {
                                result = true;
                                break;
                            }
                        }
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                arg,
                                position,
                            ));
                        }
                    }
                }
                Ok(Some(Object::Bool(result)))
            }
            "starts_with?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::String(prefix) => Ok(Some(Object::Bool(
                        string_value.as_str().starts_with(&*prefix.as_str()),
                    ))),
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
            }
            "ends_with?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::String(suffix) => Ok(Some(Object::Bool(
                        string_value.as_str().ends_with(&*suffix.as_str()),
                    ))),
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
            }
            _ => Ok(None),
        }
    }
}
