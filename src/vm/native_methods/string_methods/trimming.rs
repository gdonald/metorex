// Joining strings, and cutting whitespace off either end.

use super::*;

impl VirtualMachine {
    /// Joining strings, and cutting whitespace off either end.
    pub(crate) fn call_string_trimming_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "+" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Anything that spells itself as text is joined as the text
                // it spells, which is what `to_str` is asked for.
                let given = match &arguments[0] {
                    held @ Object::String(_) => held.clone(),
                    other if self.responds_to(other, "to_str") => {
                        self.send_to_object(other.clone(), "to_str", vec![], position)?
                    }
                    other => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &format!(
                                "no implicit conversion of {} into String",
                                self.builtins().class_of(other).name()
                            ),
                            position,
                        ));
                    }
                };
                let Object::String(rhs) = &given else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                // Two strings written in encodings that cannot be read
                // alongside each other are refused, unless one of them is
                // empty, which carries the other's encoding.
                let joined_encoding = if rhs.as_str().is_empty() {
                    string_value.encoding_name()
                } else if string_value.as_str().is_empty() {
                    rhs.encoding_name()
                } else if !strings_comparable(string_value, rhs) {
                    return Err(clashing_encodings_error(string_value, rhs, position));
                } else if string_value.as_str().is_ascii() && !rhs.as_str().is_ascii() {
                    rhs.encoding_name()
                } else {
                    string_value.encoding_name()
                };
                let mut combined = string_value.as_str().to_string();
                combined.push_str(&rhs.as_str());
                let made = Object::string(combined);
                if let Object::String(built) = &made {
                    built.set_encoding(joined_encoding);
                }
                Ok(Some(made))
            }
            "trim" => {
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
            // `lstrip` and `rstrip` trim one end. Ruby counts a NUL as
            // whitespace at the right end, which `trim_end` does not.
            // `each_line` and `lines` split on a separator, keeping it on the
            // end of each piece the way Ruby does.
            "each_line" | "lines" => {
                self.walk_lines(receiver, string_value, method_name, arguments, position)
            }
            // A string whose bytes spell nothing cannot be trimmed: the
            // scan from the front reports what it found, and the scan from
            // the back reports that the encodings do not go together.
            "lstrip" | "lstrip!" if !holds_valid_text(string_value) => {
                Err(broken_text_error(string_value, position))
            }
            "rstrip" | "rstrip!" | "strip" | "strip!" if !holds_valid_text(string_value) => {
                let message = format!(
                    "incompatible character encodings: {} and US-ASCII",
                    string_value.encoding_name()
                );
                Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &message,
                    position,
                ))
            }
            "lstrip" => {
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
                        .trim_start_matches(|letter: char| letter.is_whitespace() || letter == '\0')
                        .to_string(),
                )))
            }
            "rstrip" => {
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
                        .trim_end_matches(|letter: char| letter.is_whitespace() || letter == '\0')
                        .to_string(),
                )))
            }
            "reverse" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let reversed: String = string_value.as_str().chars().rev().collect();
                Ok(Some(Object::string(reversed)))
            }
            "last" => {
                let chars: Vec<char> = string_value.as_str().chars().collect();
                if chars.is_empty() {
                    Ok(Some(Object::Nil))
                } else {
                    Ok(Some(Object::string(chars.last().unwrap().to_string())))
                }
            }
            _ => Ok(None),
        }
    }
}
