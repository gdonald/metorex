// How a string reads back: its length, its source form, and the bytes
// behind it.

use super::*;

impl VirtualMachine {
    /// How a string reads back: its length, its source form, and the bytes
    /// behind it.
    pub(crate) fn call_string_inspection_method(
        &mut self,
        _receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // A number is read off ASCII digits, so an encoding that does not
            // spell those one byte to a letter has no number in it.
            "to_f" | "to_r" | "to_c"
                if !encoding_is_ascii_compatible(&string_value.encoding_name()) =>
            {
                let message = format!(
                    "ASCII incompatible encoding: {}",
                    string_value.encoding_name()
                );
                Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &message,
                    position,
                ))
            }
            "length" => {
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
            // `dump` renders the string as source that reads back as itself.
            // It escapes what `inspect` does, plus every non-printable and
            // non-ASCII character, and the `#` that would start an
            // interpolation.
            "dump" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut out = String::with_capacity(string_value.as_str().len() + 2);
                out.push('"');
                let held = string_value.encoding_name();
                // An encoding that spells no ASCII is dumped as the bytes it
                // holds, since a character of its own has no spelling the
                // dump could be read back through.
                if !encoding_is_ascii_compatible(&held) {
                    for byte in binary_bytes(string_value) {
                        match byte {
                            b'"' => out.push_str("\\\""),
                            b'\\' => out.push_str("\\\\"),
                            held if held.is_ascii_graphic() || held == b' ' => {
                                out.push(char::from(held))
                            }
                            held => match named_escape(char::from(held)) {
                                Some(named) => {
                                    out.push('\\');
                                    out.push(named);
                                }
                                None => out.push_str(&format!("\\x{held:02X}")),
                            },
                        }
                    }
                    out.push('"');
                    out.push_str(&format!(".force_encoding(\"{held}\")"));
                    let made = crate::object::StringValue::with_encoding(out, "US-ASCII");
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                // An encoding that spells a character in more than one byte is
                // carried as the bytes themselves, so the characters are read
                // back out of them.
                let held_characters = match wide_encoding(&held) {
                    Some(shape) => wide_text(&binary_bytes(string_value), shape),
                    None => string_value.to_text(),
                };
                let spells_unicode = held.starts_with("UTF-") || held == "CESU-8";
                let mut characters = held_characters.as_str().chars().peekable();
                while let Some(character) = characters.next() {
                    if let Some(named) = named_escape(character) {
                        out.push('\\');
                        out.push(named);
                        continue;
                    }
                    match character {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '#' if matches!(characters.peek(), Some('{' | '$' | '@')) => {
                            out.push_str("\\#")
                        }
                        character if character.is_ascii() && !character.is_control() => {
                            out.push(character)
                        }
                        // Text in an encoding that spells the whole of Unicode
                        // names the character itself; text in any other names
                        // the bytes it is spelled with.
                        character if spells_unicode && !character.is_ascii() => {
                            out.push_str(&escaped_point(character))
                        }
                        character => {
                            let mut spelling = [0u8; 4];
                            let bytes: Vec<u8> = if (character as u32) < 0x100 {
                                vec![character as u32 as u8]
                            } else {
                                character.encode_utf8(&mut spelling).as_bytes().to_vec()
                            };
                            for byte in bytes {
                                out.push_str(&format!("\\x{byte:02X}"));
                            }
                        }
                    }
                }
                out.push('"');
                // Text in an encoding that spells ASCII its own way says which
                // encoding to read the dump back in.
                if !encoding_is_ascii_compatible(&held) {
                    out.push_str(&format!(".force_encoding(\"{held}\")"));
                }
                // A dump is nothing but ASCII, so it is written in the
                // encoding it was dumped from when that spells ASCII the same
                // way, and in US-ASCII when it does not.
                let writing = if encoding_is_ascii_compatible(&held) {
                    held
                } else {
                    "US-ASCII".to_string()
                };
                let made = crate::object::StringValue::with_encoding(out, writing);
                Ok(Some(Object::String(Rc::new(made))))
            }
            // `b` answers a copy of the text tagged as a run of bytes. It is
            // a new string, so tagging it leaves the receiver alone.
            "b" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // The bytes behind the text are what a binary copy holds,
                // one character to a byte.
                let held = bytes_as_text(&binary_bytes(string_value));
                Ok(Some(Object::String(Rc::new(
                    crate::object::StringValue::from_bytes(held),
                ))))
            }
            "inspect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // Render with double-quotes and minimal escaping. Mirrors
                // Ruby's String#inspect output for the common cases. A run of
                // bytes shows the bytes themselves, since the characters they
                // would spell are not what the string holds.
                let binary = matches!(
                    string_value.encoding_name().as_str(),
                    "ASCII-8BIT" | "BINARY"
                );
                // A string whose characters stand for bytes is written as
                // those bytes only where they spell nothing in the encoding it
                // is tagged with; where they spell text, it is written as the
                // text they spell.
                let spells_nothing = string_value.holds_bytes()
                    && wide_encoding(&string_value.encoding_name()).is_none()
                    && !holds_valid_text(string_value);
                let mut out = String::with_capacity(string_value.as_str().len() + 2);
                out.push('"');
                if binary || spells_nothing {
                    let bytes = binary_bytes(string_value);
                    let mut at = 0;
                    while at < bytes.len() {
                        let byte = bytes[at];
                        // A byte that opens a character the encoding can spell
                        // is written as that character; anything else is
                        // written as the byte it is.
                        let width = if binary {
                            0
                        } else {
                            utf8_sequence_width(&bytes[at..])
                        };
                        if width > 1
                            && let Ok(text) = std::str::from_utf8(&bytes[at..at + width])
                        {
                            out.push_str(text);
                            at += width;
                            continue;
                        }
                        match byte {
                            b'"' => out.push_str("\\\""),
                            b'\\' => out.push_str("\\\\"),
                            b'\n' => out.push_str("\\n"),
                            b'\r' => out.push_str("\\r"),
                            b'\t' => out.push_str("\\t"),
                            0x20..=0x7e => out.push(byte as char),
                            _ => out.push_str(&format!("\\x{byte:02X}")),
                        }
                        at += 1;
                    }
                    out.push('"');
                    return Ok(Some(Object::string(out)));
                }
                let held = string_value.encoding_name();
                // A character an encoding spells in bytes of its own is named
                // by those bytes together, since the answer cannot show the
                // character itself.
                if (held == "EUC-JP" || spells_shift_jis(&held)) && string_value.holds_bytes() {
                    let bytes = binary_bytes(string_value);
                    let mut at = 0usize;
                    while at < bytes.len() {
                        let decoded = if held == "EUC-JP" {
                            crate::vm::native_methods::euc_jp_table::euc_jp_character(&bytes[at..])
                        } else {
                            crate::vm::native_methods::shift_jis_table::shift_jis_character(
                                &bytes[at..],
                            )
                        };
                        let width = decoded.map_or(1, |(_, width)| width);
                        // A byte standing alone past ASCII is named by itself.
                        if width == 1 && bytes[at] >= 0x80 {
                            out.push_str(&format!("\\x{:02X}", bytes[at]));
                            at += 1;
                            continue;
                        }
                        if width == 1 {
                            match bytes[at] {
                                b'"' => out.push_str("\\\""),
                                b'\\' => out.push_str("\\\\"),
                                b'\n' => out.push_str("\\n"),
                                b'\r' => out.push_str("\\r"),
                                b'\t' => out.push_str("\\t"),
                                byte @ 0x20..=0x7e => out.push(byte as char),
                                byte => out.push_str(&format!("\\x{byte:02X}")),
                            }
                            at += 1;
                            continue;
                        }
                        out.push_str("\\x{");
                        for byte in &bytes[at..at + width] {
                            out.push_str(&format!("{byte:02X}"));
                        }
                        out.push('}');
                        at += width;
                    }
                    out.push('"');
                    let made = crate::object::StringValue::with_encoding(
                        out,
                        self.inspect_result_encoding(),
                    );
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                // A character prints as itself when the string is in the
                // encoding the answer is written in, and when it is ASCII in
                // an encoding that spells ASCII the same way. Anything else
                // is escaped, since the answer could not spell it.
                let writing = self.inspect_result_encoding();
                let spells_unicode = held.starts_with("UTF-") || held == "CESU-8";
                // An encoding that spells a character in more than one byte is
                // carried as the bytes themselves, so the characters are read
                // back out of them.
                let held_text = match wide_encoding(&held) {
                    Some(shape) => wide_text(&binary_bytes(string_value), shape),
                    // A string whose characters stand for bytes has its text
                    // read back out of them, since the characters they spell
                    // are not the characters it holds.
                    None if spells_unicode && string_value.holds_bytes() => {
                        String::from_utf8(binary_bytes(string_value))
                            .unwrap_or_else(|_| string_value.to_text())
                    }
                    None => string_value.to_text(),
                };
                let mut letters = held_text.chars().peekable();
                while let Some(c) = letters.next() {
                    if let Some(named) = named_escape(c) {
                        out.push('\\');
                        out.push(named);
                        continue;
                    }
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        // A `#` that opens an interpolation is escaped, where
                        // one followed by anything else stands for itself.
                        '#' if matches!(letters.peek(), Some('$' | '@' | '{')) => {
                            out.push_str("\\#")
                        }
                        c if (held == writing || c.is_ascii()) && !c.is_control() => out.push(c),
                        // Text in an encoding that spells the whole of Unicode
                        // names the character itself; text in any other names
                        // the bytes it is spelled with.
                        c if spells_unicode => out.push_str(&escaped_point(c)),
                        c => {
                            let mut spelling = [0u8; 4];
                            for byte in c.encode_utf8(&mut spelling).as_bytes() {
                                out.push_str(&format!("\\x{byte:02X}"));
                            }
                        }
                    }
                }
                out.push('"');
                let made =
                    crate::object::StringValue::with_encoding(out, self.inspect_result_encoding());
                Ok(Some(Object::String(Rc::new(made))))
            }
            _ => Ok(None),
        }
    }
}
