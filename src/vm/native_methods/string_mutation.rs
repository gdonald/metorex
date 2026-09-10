//! The String methods that change what a string holds.
//!
//! Every reference to a string sees the change, which is why the text sits
//! behind a cell. A method whose name ends in `!` answers nil when it found
//! nothing to change, and every one of them refuses a frozen string.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;

/// The methods that change the string they are called on.
pub(crate) const MUTATING_STRING_METHODS: &[&str] = &[
    "<<",
    "concat",
    "replace",
    "prepend",
    "insert",
    "clear",
    "setbyte",
    "slice!",
    "sub!",
    "gsub!",
    "squeeze!",
    "delete!",
    "tr!",
    "tr_s!",
    "strip!",
    "lstrip!",
    "rstrip!",
    "chomp!",
    "chop!",
    "upcase!",
    "downcase!",
    "capitalize!",
    "swapcase!",
    "reverse!",
    "succ!",
    "next!",
    "encode!",
    "unicode_normalize!",
    "scrub!",
];

impl VirtualMachine {
    /// Run one of the methods that changes a string, or answer None when the
    /// name is not one of them.
    pub(crate) fn call_string_mutation(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if !MUTATING_STRING_METHODS.contains(&method_name) {
            return Ok(None);
        }
        let Object::String(target) = receiver else {
            return Ok(None);
        };
        if target.is_frozen() {
            return Err(self.frozen_modification_error(receiver, position));
        }
        match method_name {
            "<<" | "concat" => {
                if method_name == "<<" && arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let mut added = String::new();
                for argument in arguments {
                    added.push_str(&self.appended_text(argument, position)?);
                }
                target.append_text(&added);
                Ok(Some(receiver.clone()))
            }
            "replace" => {
                let text = self.one_string_argument(method_name, arguments, position)?;
                // The replacement's encoding comes with its text.
                if let Some(Object::String(source)) = arguments.first() {
                    target.set_encoding(source.encoding_name());
                }
                target.replace_text(text);
                Ok(Some(receiver.clone()))
            }
            "prepend" => {
                let mut front = String::new();
                for argument in arguments {
                    front.push_str(&self.one_string_value(method_name, argument, position)?);
                }
                target.update_text(|held| format!("{}{}", front, held));
                Ok(Some(receiver.clone()))
            }
            "insert" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let Object::Int(index) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                let added = self.one_string_value(method_name, &arguments[1], position)?;
                let held = target.to_text();
                let letters: Vec<char> = held.chars().collect();
                // A negative index counts back from the end, and one past the
                // end there means "after the last character".
                let at = if *index < 0 {
                    *index + letters.len() as i64 + 1
                } else {
                    *index
                };
                if at < 0 || at > letters.len() as i64 {
                    let message = format!("index {} out of string", index);
                    return Err(crate::vm::errors::simple_exception(
                        "IndexError",
                        &message,
                        position,
                    ));
                }
                let at = at as usize;
                let mut made: String = letters[..at].iter().collect();
                made.push_str(&added);
                made.extend(letters[at..].iter());
                target.replace_text(made);
                Ok(Some(receiver.clone()))
            }
            "clear" => {
                target.replace_text(String::new());
                Ok(Some(receiver.clone()))
            }
            "setbyte" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let (Object::Int(index), Object::Int(value)) = (&arguments[0], &arguments[1])
                else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                let mut bytes = super::pack_format::string_to_bytes(&target.as_str());
                let at = if *index < 0 {
                    *index + bytes.len() as i64
                } else {
                    *index
                };
                if at < 0 || at >= bytes.len() as i64 {
                    let message = format!("index {} out of string", index);
                    return Err(crate::vm::errors::simple_exception(
                        "IndexError",
                        &message,
                        position,
                    ));
                }
                bytes[at as usize] = (value.rem_euclid(256)) as u8;
                if let Object::String(made) = super::pack_format::bytes_to_string(&bytes) {
                    target.replace_text(made.to_text());
                }
                Ok(Some(Object::Int(*value)))
            }
            "slice!" => self.cut_from_string(receiver, arguments, position),
            // These answer the string itself rather than nil, and the tag
            // the answer carries comes back with the text.
            "encode!" | "scrub!" | "unicode_normalize!" => {
                let plain = method_name.trim_end_matches('!');
                let answered = self.call_string_method(receiver, plain, arguments, position)?;
                if let Some(Object::String(made)) = answered {
                    target.set_encoding(made.encoding_name());
                    target.replace_text(made.to_text());
                }
                Ok(Some(receiver.clone()))
            }
            _ => {
                // The rest answer what the method without the `!` answers,
                // and put that back into the string when it differs.
                let plain = method_name.trim_end_matches('!');
                let before = target.to_text();
                let answered = self.call_string_method(receiver, plain, arguments, position)?;
                let Some(Object::String(made)) = answered else {
                    return Ok(Some(Object::Nil));
                };
                let after = made.to_text();
                // Most of these answer nil when they found nothing to change.
                // Reversing and stepping to the next string always answer the
                // string itself.
                let always_self = matches!(method_name, "reverse!" | "succ!" | "next!");
                if after == before && !always_self {
                    return Ok(Some(Object::Nil));
                }
                target.replace_text(after);
                Ok(Some(receiver.clone()))
            }
        }
    }

    /// The text `<<` and `concat` add. An Integer names a character by its
    /// code point rather than by its digits.
    fn appended_text(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::Int(code) = argument {
            let Some(letter) = u32::try_from(*code).ok().and_then(char::from_u32) else {
                let message = format!("{} out of char range", code);
                return Err(crate::vm::errors::simple_exception(
                    "RangeError",
                    &message,
                    position,
                ));
            };
            return Ok(letter.to_string());
        }
        self.one_string_value("<<", argument, position)
    }

    /// The one String an argument names, asking it for `to_str` when it is
    /// not already one.
    fn one_string_value(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = argument {
            return Ok(text.to_text());
        }
        if self.responds_to(argument, "to_str") {
            let named = self.send_to_object(argument.clone(), "to_str", vec![], position)?;
            if let Object::String(text) = named {
                return Ok(text.to_text());
            }
        }
        Err(method_argument_type_error(
            method_name,
            "String",
            argument,
            position,
        ))
    }

    fn one_string_argument(
        &mut self,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<String, MetorexError> {
        if arguments.len() != 1 {
            return Err(method_argument_error(
                method_name,
                1,
                arguments.len(),
                position,
            ));
        }
        self.one_string_value(method_name, &arguments[0], position)
    }

    /// `slice!` takes the part `slice` would answer out of the string and
    /// hands it back, leaving what was around it.
    fn cut_from_string(
        &mut self,
        receiver: &Object,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::String(target) = receiver else {
            return Ok(None);
        };
        let held = target.to_text();
        let taken = self.call_string_method(receiver, "slice", arguments, position)?;
        let Some(Object::String(cut)) = taken.clone() else {
            return Ok(Some(Object::Nil));
        };
        let removed = cut.to_text();
        // Where the part sits decides what is left, and only an index form
        // says outright where that is.
        let letters: Vec<char> = held.chars().collect();
        let start = match arguments.first() {
            Some(Object::Int(index)) => {
                let at = if *index < 0 {
                    *index + letters.len() as i64
                } else {
                    *index
                };
                usize::try_from(at).unwrap_or(0)
            }
            Some(Object::Range { start, .. }) => match start.as_ref() {
                Object::Int(index) => {
                    let at = if *index < 0 {
                        *index + letters.len() as i64
                    } else {
                        *index
                    };
                    usize::try_from(at).unwrap_or(0)
                }
                _ => 0,
            },
            _ => held
                .find(&removed)
                .map(|byte_offset| held[..byte_offset].chars().count())
                .unwrap_or(0),
        };
        let width = removed.chars().count();
        let mut left: String = letters[..start.min(letters.len())].iter().collect();
        left.extend(letters[(start + width).min(letters.len())..].iter());
        target.replace_text(left);
        Ok(taken)
    }
}
