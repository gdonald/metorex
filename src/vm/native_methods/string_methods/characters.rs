// Characters and code points, and the numbers a string spells.

use super::*;

impl VirtualMachine {
    /// Characters and code points, and the numbers a string spells.
    pub(crate) fn call_string_character_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `hex` and `oct` read a number off the front of the string, in
            // base 16 and base 8, with `oct` honoring a base prefix.
            // The one-way hash the C library computes, with the salt naming
            // which algorithm it uses and what it starts from.
            "crypt" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let salt = match &arguments[0] {
                    Object::String(held) => held.to_text(),
                    other if self.responds_to(other, "to_str") => {
                        match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                            Object::String(held) => held.to_text(),
                            converted => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "String",
                                    &converted,
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
                let key_bytes = crate::vm::native_methods::pack_format::string_to_bytes(
                    &string_value.to_text(),
                );
                let salt_bytes = crate::vm::native_methods::pack_format::string_to_bytes(&salt);
                if key_bytes.contains(&0) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "string contains null byte",
                        position,
                    ));
                }
                if salt_bytes.len() < 2 || salt_bytes[..2].contains(&0) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "salt too short (need >=2 bytes)",
                        position,
                    ));
                }
                let key = std::ffi::CString::new(key_bytes).unwrap_or_default();
                let salt = std::ffi::CString::new(salt_bytes).unwrap_or_default();
                // SAFETY: both strings are NUL-terminated and stay alive for
                // the call, and the answer is the library's own buffer.
                let answered = unsafe { crypt(key.as_ptr(), salt.as_ptr()) };
                if answered.is_null() {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "salt too short (need >=2 bytes)",
                        position,
                    ));
                }
                // SAFETY: `crypt` answers a NUL-terminated string.
                let hashed = unsafe { std::ffi::CStr::from_ptr(answered) };
                Ok(Some(
                    crate::vm::native_methods::pack_format::bytes_to_string(hashed.to_bytes()),
                ))
            }
            "hex" | "oct" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let default_radix = if method_name == "hex" { 16 } else { 8 };
                Ok(Some(Object::Int(leading_radix_number(
                    &string_value.as_str(),
                    default_radix,
                ))))
            }
            // `to_str` is the implicit conversion, which a String answers
            // with itself.
            "to_str" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::String(Rc::clone(string_value))))
            }
            // The code point of each character, which `each_codepoint` walks
            // one at a time.
            // A code point is a character, so a string whose bytes spell
            // nothing has none to walk.
            "codepoints" | "each_codepoint"
                if !holds_valid_text(string_value)
                    && (method_name == "codepoints" || self.pending_block.is_some()) =>
            {
                Err(broken_text_error(string_value, position))
            }
            "codepoints" | "each_codepoint" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let points: Vec<Object> = string_value
                    .as_str()
                    .chars()
                    .map(|character| Object::Int(character as i64))
                    .collect();
                // `codepoints` answers the Array, unless a block is given:
                // then it hands each one over and answers the string.
                if method_name == "codepoints"
                    && !matches!(self.pending_block, Some(Object::Block(_)))
                {
                    return Ok(Some(Object::Array(Rc::new(RefCell::new(points)))));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    let size = points.len() as i64;
                    return self
                        .build_enumerator(
                            receiver.clone(),
                            method_name,
                            vec![],
                            Some(size),
                            position,
                        )
                        .map(Some);
                };
                for point in points {
                    self.execute_block_callable(&block, vec![point], position)?;
                }
                Ok(Some(receiver.clone()))
            }
            "chars" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let chars = encoded_characters(string_value);
                // Given a block, `chars` hands each character over and
                // answers the string, the way `each_char` does.
                if let Some(Object::Block(block)) = self.pending_block.take() {
                    for character in chars {
                        self.execute_block_callable(&block, vec![character], position)?;
                    }
                    return Ok(Some(receiver.clone()));
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(chars)))))
            }
            _ => Ok(None),
        }
    }
}
