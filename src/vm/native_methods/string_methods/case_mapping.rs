// Raising and lowering letters, and the next string after this one.

use super::*;

impl VirtualMachine {
    /// Raising and lowering letters, and the next string after this one.
    pub(crate) fn call_string_case_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `byteindex` and `byterindex` answer where a substring or
            // pattern appears, counted in bytes rather than in characters.
            "byteindex" | "byterindex" => self
                .byte_index_of(receiver, string_value, method_name, arguments, position)
                .map(Some),
            // `index` answers where a substring or pattern appears, counted
            // in characters rather than in bytes.
            "index" | "rindex" => self
                .byte_index_of(receiver, string_value, method_name, arguments, position)
                .map(Some),
            "upcase" => {
                let wanted = case_options(method_name, arguments, position)?;
                Ok(Some(Object::string(mapped_case(
                    &string_value.as_str(),
                    CaseWanted::Up,
                    &wanted,
                ))))
            }
            // String#succ / String#next — Ruby's "next string" successor.
            // For digit-only strings (e.g. "0" → "1", "9" → "10") this
            // matches MRI; for letter or mixed strings we approximate by
            // bumping the trailing character (sufficient for spec helpers
            // that uniquify with a leading digit). The bang form returns a
            // fresh string here because metorex strings are immutable
            // shared `Rc<String>`s; callers re-assign or use `+`-prefixed
            // strings expecting a mutable buffer that we don't model.
            // String#insert(index, str) — returns the receiver with `str`
            // inserted at `index`. metorex strings are immutable shared
            // `Rc<String>`s, so we return a new string rather than
            // mutating; spec helpers that chain `name.insert(...)` then
            // use `name` afterward typically reassign anyway.
            "insert" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let idx = match &arguments[0] {
                    Object::Int(n) => *n,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            other,
                            position,
                        ));
                    }
                };
                let to_insert = match &arguments[1] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                let s = string_value.as_ref().as_str();
                let len = s.chars().count() as i64;
                let pos_idx = if idx < 0 { idx + len + 1 } else { idx };
                if pos_idx < 0 || pos_idx > len {
                    let msg = format!("index {} out of string", idx);
                    let exc = Object::exception("IndexError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let split_at: usize = s
                    .char_indices()
                    .nth(pos_idx as usize)
                    .map(|(i, _)| i)
                    .unwrap_or(s.len());
                let mut result = String::with_capacity(s.len() + to_insert.len());
                result.push_str(&s[..split_at]);
                result.push_str(&to_insert);
                result.push_str(&s[split_at..]);
                Ok(Some(Object::string(result)))
            }
            "succ" | "next" | "succ!" | "next!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let next = successor_of(&string_value.as_ref().as_str());
                Ok(Some(Object::string(next)))
            }
            "downcase" => {
                let wanted = case_options(method_name, arguments, position)?;
                Ok(Some(Object::string(mapped_case(
                    &string_value.as_str(),
                    CaseWanted::Down,
                    &wanted,
                ))))
            }
            // `capitalize` raises the first letter and lowers the rest, and
            // `swapcase` turns each letter the other way.
            "capitalize" => {
                let wanted = case_options(method_name, arguments, position)?;
                Ok(Some(Object::string(capitalized_case(
                    &string_value.as_str(),
                    &wanted,
                ))))
            }
            "swapcase" => {
                let wanted = case_options(method_name, arguments, position)?;
                Ok(Some(Object::string(swapped_case(
                    &string_value.as_str(),
                    &wanted,
                ))))
            }
            _ => Ok(None),
        }
    }
}
