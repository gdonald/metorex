// The bytes a string is made of, read singly and in runs.

use super::*;

impl VirtualMachine {
    /// The bytes a string is made of, read singly and in runs.
    pub(crate) fn call_string_byte_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // String#ord — the codepoint of the first character. An empty
            // String has none, which Ruby reports as an ArgumentError.
            "ord" if !holds_valid_text(string_value) => {
                Err(broken_text_error(string_value, position))
            }
            "ord" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                match string_value.as_str().chars().next() {
                    Some(character) => Ok(Some(Object::Int(character as i64))),
                    None => {
                        let message = "empty string".to_string();
                        Err(MetorexError::UncaughtException {
                            exception: Object::exception("ArgumentError", message.clone()),
                            location: position_to_location(position),
                            message,
                        })
                    }
                }
            }
            // The bytes a String is made of, which is what its length in
            // bytes and each byte-level reader are counted from.
            "bytesize" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(binary_bytes(string_value).len() as i64)))
            }
            // `byteslice` cuts by byte position rather than by character,
            // and what it hands back is tagged the way the whole string is.
            "byteslice" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(1, 2),
                        arguments.len(),
                        position,
                    ));
                }
                let bytes = binary_bytes(string_value);
                let total = bytes.len() as i64;
                let (from, count) = if let Some(span) =
                    crate::vm::native_methods::as_range(&arguments[0])
                    && arguments.len() == 1
                {
                    let Object::Range {
                        start,
                        end,
                        exclusive,
                        ..
                    } = span
                    else {
                        return Ok(Some(Object::Nil));
                    };
                    let opening = match self.span_end_index(start.as_ref(), position)? {
                        Some(number) => {
                            if number < 0 {
                                total + number
                            } else {
                                number
                            }
                        }
                        None => 0,
                    };
                    let closing = match self.span_end_index(end.as_ref(), position)? {
                        Some(number) => {
                            let placed = if number < 0 { total + number } else { number };
                            if exclusive { placed - 1 } else { placed }
                        }
                        None => total - 1,
                    };
                    (opening, (closing - opening + 1).max(0))
                } else {
                    let opening = self.coerce_slice_number(&arguments[0], method_name, position)?;
                    let opening = if opening < 0 {
                        total + opening
                    } else {
                        opening
                    };
                    let wanted = if arguments.len() == 2 {
                        self.coerce_slice_number(&arguments[1], method_name, position)?
                    } else {
                        // A single index names one byte, which has to be
                        // there for the answer to be a string at all.
                        if opening < 0 || opening >= total {
                            return Ok(Some(Object::Nil));
                        }
                        1
                    };
                    (opening, wanted)
                };
                if from < 0 || from > total || count < 0 {
                    return Ok(Some(Object::Nil));
                }
                let stop = (from + count).min(total);
                let cut = &bytes[from as usize..stop as usize];
                Ok(Some(
                    crate::vm::native_methods::string_sets::text_in_encoding(
                        cut,
                        &string_value.encoding_name(),
                    ),
                ))
            }
            "bytes" | "each_byte" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let bytes: Vec<Object> = binary_bytes(string_value)
                    .into_iter()
                    .map(|byte| Object::Int(byte as i64))
                    .collect();
                // `bytes` with a block yields the same way `each_byte`
                // does, and answers the string rather than the Array.
                if method_name == "bytes" && !matches!(self.pending_block, Some(Object::Block(_))) {
                    return Ok(Some(Object::Array(Rc::new(RefCell::new(bytes)))));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    let size = bytes.len() as i64;
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
                // The string may change while it is being walked, and the walk
                // carries on from where it stood into whatever is there now.
                let mut at = 0usize;
                loop {
                    let held = binary_bytes(string_value);
                    let Some(byte) = held.get(at) else {
                        break;
                    };
                    let byte = Object::Int(*byte as i64);
                    at += 1;
                    self.execute_block_callable(&block, vec![byte], position)?;
                }
                Ok(Some(receiver.clone()))
            }
            "getbyte" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
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
                let bytes = binary_bytes(string_value);
                let length = bytes.len() as i64;
                let resolved = if *index < 0 { index + length } else { *index };
                if resolved < 0 || resolved >= length {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(Object::Int(bytes[resolved as usize] as i64)))
            }
            // The first character, or an empty String when there is none.
            "chr" => {
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
                        .chars()
                        .next()
                        .map(String::from)
                        .unwrap_or_default(),
                )))
            }
            // Metorex strings are UTF-8 throughout, so every one of them is
            // valid, and whether it is ASCII is a question about its bytes.
            // A string is valid in the encoding it is tagged with when its
            // bytes spell characters there. Anything read as bytes is valid
            // whatever those bytes are.
            "valid_encoding?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(holds_valid_text(string_value))))
            }
            "ascii_only?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // An encoding that spells even the ASCII letters in more
                // than one byte holds nothing ASCII-only, empty or not.
                let compatible = !matches!(
                    string_value.encoding_name().as_str(),
                    "UTF-16" | "UTF-16BE" | "UTF-16LE" | "UTF-32" | "UTF-32BE" | "UTF-32LE"
                );
                Ok(Some(Object::Bool(
                    compatible && string_value.as_str().is_ascii(),
                )))
            }
            _ => Ok(None),
        }
    }
}
