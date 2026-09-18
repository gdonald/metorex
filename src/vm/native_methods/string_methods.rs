//! Native method implementations for the String class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the String class.
    /// The number a slice argument names, which Ruby reads through `to_int`
    /// and refuses when the object names none.
    fn coerce_slice_number(
        &mut self,
        given: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<i64, MetorexError> {
        match given {
            Object::Int(number) => Ok(*number),
            Object::Float(number) => Ok(*number as i64),
            // A number past what a machine word holds names no place in a
            // string, which Ruby reports as a range rather than a type.
            Object::BigInt(_) => Err(crate::vm::errors::simple_exception(
                "RangeError",
                "bignum too big to convert into 'long'",
                position,
            )),
            other if self.answers_to(other, "to_int", position)? => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    Object::Int(number) => Ok(number),
                    Object::Float(number) => Ok(number as i64),
                    _ => Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        given,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                method_name,
                "Integer",
                other,
                position,
            )),
        }
    }

    pub(crate) fn call_string_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::String(string_value) = receiver else {
            return Ok(None);
        };
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
                if held == "EUC-JP" && string_value.holds_bytes() {
                    let bytes = binary_bytes(string_value);
                    let mut at = 0usize;
                    while at < bytes.len() {
                        let width = match super::euc_jp_table::euc_jp_character(&bytes[at..]) {
                            Some((_, width)) => width,
                            None => 1,
                        };
                        if width == 1 && bytes[at] < 0x80 {
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
            // `match` answers the MatchData, and records it as the last match
            // the way every other match does.
            "match" => {
                if arguments.is_empty() {
                    return Err(method_argument_error("match", 1, 0, position));
                }
                // Ruby's String#match hands the work to the pattern, so a
                // Regexp carrying a `match` of its own answers instead.
                if matches!(arguments[0], Object::Regex(_, _))
                    && let Some(class) = self.existing_singleton_class(&arguments[0])
                    && let Some(method) = class.find_method("match")
                    && !method.is_undefined
                {
                    let mut passed = vec![receiver.clone()];
                    passed.extend(arguments[1..].iter().cloned());
                    return self
                        .invoke_method(class, method, arguments[0].clone(), passed, position)
                        .map(Some);
                }
                let (pattern, flags) = match &arguments[0] {
                    Object::Regex(pattern, flags) => {
                        (pattern.as_str().to_string(), flags.as_str().to_string())
                    }
                    Object::String(source) => (source.as_str().to_string(), String::new()),
                    // A pattern written as anything else is asked for the
                    // characters it stands for, which are read as a pattern.
                    other if self.answers_to(other, "to_str", position)? => {
                        match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                            Object::String(source) => (source.as_str().to_string(), String::new()),
                            answered => {
                                return Err(method_argument_type_error(
                                    "match",
                                    "Regexp or String",
                                    &answered,
                                    position,
                                ));
                            }
                        }
                    }
                    other => {
                        return Err(method_argument_type_error(
                            "match",
                            "Regexp or String",
                            other,
                            position,
                        ));
                    }
                };
                let subject = string_value.as_str().to_string();
                let start = match arguments.get(1) {
                    Some(Object::Int(offset)) => {
                        let length = subject.chars().count() as i64;
                        let resolved = if *offset < 0 {
                            offset + length
                        } else {
                            *offset
                        };
                        if resolved < 0 || resolved > length {
                            return Ok(Some(Object::Nil));
                        }
                        subject
                            .char_indices()
                            .nth(resolved as usize)
                            .map(|(index, _)| index)
                            .unwrap_or(subject.len())
                    }
                    _ => 0,
                };
                // The block is taken before the walk, since building the
                // MatchData is itself a call and would consume it.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let found = self.regexp_match_data_in(
                    &pattern,
                    &flags,
                    &subject,
                    start,
                    Some(string_value.encoding_name()),
                    position,
                )?;
                match (found, block) {
                    (Some(data), Some(block)) => self
                        .execute_block_callable(&block, vec![data], position)
                        .map(Some),
                    (Some(data), None) => Ok(Some(data)),
                    (None, _) => Ok(Some(Object::Nil)),
                }
            }
            "match?" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        "match?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let (pattern, flags) = match &arguments[0] {
                    Object::Regex(p, f) => (p.as_str().to_string(), f.as_str().to_string()),
                    Object::String(s) => (s.as_str().to_string(), String::new()),
                    other => {
                        return Err(method_argument_type_error(
                            "match?",
                            "Regexp or String",
                            other,
                            position,
                        ));
                    }
                };
                // A second argument names the character offset to start at,
                // counting from the end when negative.
                let subject = string_value.as_ref();
                let start = match arguments.get(1) {
                    Some(Object::Int(offset)) => {
                        let length = subject.as_str().chars().count() as i64;
                        let resolved = if *offset < 0 {
                            offset + length
                        } else {
                            *offset
                        };
                        if resolved < 0 || resolved > length {
                            return Ok(Some(Object::Bool(false)));
                        }
                        subject
                            .as_str()
                            .char_indices()
                            .nth(resolved as usize)
                            .map(|(index, _)| index)
                            .unwrap_or(subject.as_str().len())
                    }
                    _ => 0,
                };
                let matched = super::compile(&pattern, &flags)
                    .map(|compiled| compiled.find_at(&subject.as_str(), start).is_some())
                    .unwrap_or(false);
                Ok(Some(Object::Bool(matched)))
            }
            // String#encode — metorex strings are always UTF-8 and carry no
            // encoding metadata, so re-encoding is the identity.
            // Text that is nothing but ASCII reads the same in every
            // ASCII-compatible encoding, so a copy of it can be tagged with
            // the one asked for without converting anything. Text that is not
            // needs a conversion metorex does not carry out, and the copy
            // keeps the encoding it had.
            // Metorex holds every string's characters as text, so `encode`
            // answers a copy tagged with the encoding asked for rather than
            // rewriting what it holds.
            "encode" => {
                // The last argument may name what to do with bytes the source
                // encoding cannot read.
                let (positional, replacement) = encode_options(arguments);
                let arguments = positional;
                let Some(named) = arguments.first() else {
                    return Ok(Some(Object::string(string_value.to_text())));
                };
                let Ok(wanted) = self.encoding_name_argument(named, position) else {
                    return Ok(Some(Object::string(string_value.to_text())));
                };
                // An encoding that spells a character in more than one byte
                // is carried as the bytes themselves, since they spell
                // nothing the text model can hold. A second name says what
                // the bytes are to be read as rather than what the string
                // says it is written in.
                let held = match arguments.get(1) {
                    Some(source) => self
                        .encoding_name_argument(source, position)
                        .unwrap_or_else(|_| string_value.encoding_name()),
                    None => string_value.encoding_name(),
                };
                let reads_bytes = string_value.holds_bytes() || arguments.len() > 1;
                let reading = match wide_encoding(&held) {
                    Some(shape) => wide_text(&binary_bytes(string_value), shape),
                    None if held == "ISO-2022-JP" && reads_bytes => {
                        super::euc_jp_table::iso_2022_jp_text(&binary_bytes(string_value))
                    }
                    None if spells_shift_jis(&held) && reads_bytes => {
                        super::shift_jis_table::shift_jis_text(&binary_bytes(string_value))
                    }
                    None if held == "EUC-JP" && reads_bytes => match &replacement {
                        Some(stands_in) => {
                            euc_jp_text_replacing(&binary_bytes(string_value), stands_in)
                        }
                        None => super::euc_jp_table::euc_jp_text(&binary_bytes(string_value)),
                    },
                    None => match latin_text(&binary_bytes(string_value), &held) {
                        Some(spelled) if reads_bytes => spelled,
                        _ => string_value.to_text(),
                    },
                };
                // ISO-2022-JP writes its Japanese runs between escapes, so
                // the bytes are built rather than mapped one for one.
                if wanted == "ISO-2022-JP" {
                    let bytes =
                        super::euc_jp_table::iso_2022_jp_bytes(&reading).map_err(|character| {
                            let message =
                                format!("U+{:04X} from UTF-8 to {}", character as u32, wanted);
                            crate::vm::errors::simple_exception(
                                "Encoding::UndefinedConversionError",
                                &message,
                                position,
                            )
                        })?;
                    let made = crate::object::StringValue::from_bytes(bytes_as_text(&bytes));
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if spells_shift_jis(&wanted) {
                    let bytes =
                        super::shift_jis_table::shift_jis_bytes(&reading).map_err(|character| {
                            let message =
                                format!("U+{:04X} from UTF-8 to {}", character as u32, wanted);
                            crate::vm::errors::simple_exception(
                                "Encoding::UndefinedConversionError",
                                &message,
                                position,
                            )
                        })?;
                    let made = crate::object::StringValue::from_bytes(bytes_as_text(&bytes));
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if wanted == "EUC-JP" {
                    let bytes =
                        super::euc_jp_table::euc_jp_bytes(&reading).map_err(|character| {
                            let message =
                                format!("U+{:04X} from UTF-8 to {}", character as u32, wanted);
                            crate::vm::errors::simple_exception(
                                "Encoding::UndefinedConversionError",
                                &message,
                                position,
                            )
                        })?;
                    let made = crate::object::StringValue::from_bytes(bytes_as_text(&bytes));
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if let Some(spelled) = latin_bytes(&reading, &wanted) {
                    let bytes = spelled.map_err(|character| {
                        let message =
                            format!("U+{:04X} from UTF-8 to {}", character as u32, wanted);
                        crate::vm::errors::simple_exception(
                            "Encoding::UndefinedConversionError",
                            &message,
                            position,
                        )
                    })?;
                    let made = crate::object::StringValue::from_bytes(bytes_as_text(&bytes));
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if let Some(shape) = wide_encoding(&wanted) {
                    let made = Object::String(Rc::new(crate::object::StringValue::from_bytes(
                        bytes_as_text(&wide_bytes(&reading, shape)),
                    )));
                    if let Object::String(copied) = &made {
                        copied.set_encoding(wanted);
                    }
                    return Ok(Some(made));
                }
                let copy = Object::string(reading);
                if let Object::String(copied) = &copy {
                    copied.set_encoding(wanted);
                }
                Ok(Some(copy))
            }
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
                let made = crate::object::StringValue::from_bytes(bytes_as_text(cut));
                made.set_encoding(string_value.encoding_name());
                Ok(Some(Object::String(Rc::new(made))))
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
                let key_bytes = super::pack_format::string_to_bytes(&string_value.to_text());
                let salt_bytes = super::pack_format::string_to_bytes(&salt);
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
                Ok(Some(super::pack_format::bytes_to_string(hashed.to_bytes())))
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
                let held =
                    match super::pack_format::bytes_to_string(&bytes[from.min(bytes.len())..]) {
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
                use super::normalization_table::{composed, decomposed, text_of};
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
                    string_value
                        .replace_text(super::pack_format::bytes_to_string(&bytes).to_string());
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
            // Stream-like predicates so STDOUT/STDERR (stored as String) can be checked
            "tty?" | "isatty" => Ok(Some(Object::Bool(false))),
            "flush" | "sync" | "sync=" | "fsync" => Ok(Some(Object::Nil)),
            // STDOUT/STDERR stream methods (receiver is the "STDOUT"/"STDERR" string).
            "puts" | "print" | "write" => {
                let to_stderr = *string_value.as_str() == *"STDERR";
                let newline = method_name == "puts";
                let mut out = String::new();
                for arg in arguments.iter() {
                    let s = match arg {
                        Object::String(s) => s.as_str().to_string(),
                        other => format!("{}", other),
                    };
                    out.push_str(&s);
                    if newline && !s.ends_with('\n') {
                        out.push('\n');
                    }
                }
                if newline && arguments.is_empty() {
                    out.push('\n');
                }
                if to_stderr {
                    eprint!("{}", out);
                } else {
                    print!("{}", out);
                }
                Ok(Some(Object::Nil))
            }
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
                if width <= current_len {
                    return Ok(Some(Object::string(string_value.to_string())));
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
                    _ => super::pack_format::bytes_to_string(&kept).to_string(),
                };
                let made =
                    crate::object::StringValue::with_encoding(held, string_value.encoding_name());
                if string_value.holds_bytes() {
                    made.mark_bytes();
                }
                Ok(Some(Object::String(Rc::new(made))))
            }
            // `lines` splits on the line separator, keeping it on each piece.
            "split" => {
                if arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // A run of bytes its encoding cannot read has no characters
                // to cut between.
                if !holds_valid_text(string_value) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("invalid byte sequence in {}", string_value.encoding_name()),
                        position,
                    ));
                }
                let limit = match arguments.get(1) {
                    None => 0i64,
                    Some(other) => {
                        let counted = self.coerce_integer_argument(other, position)?;
                        i32::try_from(&counted).map(i64::from).map_err(|_| {
                            crate::vm::errors::simple_exception(
                                "RangeError",
                                &format!("integer {counted} too big to convert to `int'"),
                                position,
                            )
                        })?
                    }
                };
                // Without a pattern of its own, the one `$;` holds decides,
                // and Ruby says so when it holds one.
                let given = match arguments.first() {
                    None | Some(Object::Nil) => match self.globals().get(";") {
                        Some(Object::Nil) | None => None,
                        Some(held) => {
                            let message = "warning: $; is set to non-nil value\n".to_string();
                            self.warn_through_warning_module(message, position)?;
                            Some(held)
                        }
                    },
                    Some(held) => Some(held.clone()),
                };
                let separator = match &given {
                    None => Separator::Whitespace,
                    Some(Object::Regex(pattern, flags)) => {
                        match super::regexp_methods::compile(pattern, flags) {
                            Some(built) => Separator::Pattern(built),
                            // A pattern that is nothing but a look-ahead cuts
                            // in front of every place the run it names starts.
                            None => match looks_ahead(pattern, flags) {
                                Some(built) => Separator::Before(built),
                                None => {
                                    return Err(crate::vm::errors::simple_exception(
                                        "RegexpError",
                                        &format!("invalid pattern: /{pattern}/"),
                                        position,
                                    ));
                                }
                            },
                        }
                    }
                    Some(Object::String(held)) => {
                        if !holds_valid_text(held) {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &format!("invalid byte sequence in {}", held.encoding_name()),
                                position,
                            ));
                        }
                        separator_of(&held.as_str())
                    }
                    Some(other) => {
                        let spelled = self.string_argument(method_name, other, position)?;
                        separator_of(&spelled)
                    }
                };
                let text = string_value.to_text();
                let pieces = split_pieces(&text, &separator, limit);
                let named = string_value.encoding_name();
                let made: Vec<Object> = pieces
                    .into_iter()
                    .map(|piece| {
                        let held = crate::object::StringValue::with_encoding(piece, named.clone());
                        Object::String(Rc::new(held))
                    })
                    .collect();
                // With a block each piece is handed over and the string
                // itself is the answer.
                if let Some(Object::Block(block)) = self.pending_block.take() {
                    for piece in made {
                        self.execute_block_callable(&block, vec![piece], position)?;
                    }
                    return Ok(Some(receiver.clone()));
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(made)))))
            }
            "slice" | "[]" => {
                // One argument is the same lookup a subscript makes, so an
                // Integer, Range, String, or Regexp all read the same way.
                if arguments.len() == 1 {
                    // An object that is neither of the shapes a subscript
                    // reads names a place through `to_int`.
                    let chosen = match &arguments[0] {
                        held @ Object::Instance(_)
                            if crate::vm::native_methods::string_subclass_value(held).is_none()
                                && self.answers_to(held, "to_int", position)? =>
                        {
                            Object::Int(self.coerce_slice_number(held, method_name, position)?)
                        }
                        held => held.clone(),
                    };
                    return self
                        .evaluate_index_operation(receiver.clone(), chosen, position)
                        .map(Some);
                }
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // `held[pattern, group]` answers the group the pattern's
                // match holds, named by number or by name.
                if let Object::Regex(_, _) = &arguments[0] {
                    let matched = self.send_to_object(
                        arguments[0].clone(),
                        "match",
                        vec![receiver.clone()],
                        position,
                    )?;
                    if matches!(matched, Object::Nil) {
                        return Ok(Some(Object::Nil));
                    }
                    return self
                        .send_to_object(matched, "[]", vec![arguments[1].clone()], position)
                        .map(Some);
                }
                // A Float or an object that names a number arrives as one,
                // which is what `to_int` is asked for.
                let start = self.coerce_slice_number(&arguments[0], method_name, position)?;
                let len = self.coerce_slice_number(&arguments[1], method_name, position)?;
                let chars: Vec<char> = string_value.as_str().chars().collect();
                let char_count = chars.len() as i64;
                // A start past the end names no substring at all, where a
                // start exactly at the end names the empty one.
                if start > char_count || (start < 0 && char_count + start < 0) {
                    return Ok(Some(Object::Nil));
                }
                let start_idx = if start < 0 {
                    (char_count + start).max(0) as usize
                } else {
                    start.min(char_count) as usize
                };
                let end_idx = (start_idx as i64 + len).min(char_count).max(0) as usize;
                if start_idx > chars.len() || len < 0 {
                    Ok(Some(Object::Nil))
                } else {
                    let end_idx = end_idx.clamp(start_idx, chars.len());
                    let sliced: String = chars[start_idx..end_idx].iter().collect();
                    Ok(Some(Object::string(sliced)))
                }
            }
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
            "each_char" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        let size = character_count(string_value);
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                vec![],
                                Some(size),
                                position,
                            )
                            .map(Some);
                    }
                };
                for character in encoded_characters(string_value) {
                    self.execute_block_body(&block, vec![character])?;
                }
                Ok(Some(receiver.clone()))
            }
            "to_i" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A base of its own reads the digits of that base, so
                // `"ff".to_i(16)` is 255. Zero means "read the prefix".
                let base = match arguments.first() {
                    None => 10i64,
                    Some(Object::Int(held)) => *held,
                    Some(other) => {
                        let coerced = self.coerce_integer_argument(other, position)?;
                        coerced.try_into().unwrap_or(i64::MAX)
                    }
                };
                if base != 0 && !(2..=36).contains(&base) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("invalid radix {base}"),
                        position,
                    ));
                }
                let held = string_value.to_text();
                Ok(Some(Object::integer(leading_integer(
                    held.as_str(),
                    base as u32,
                ))))
            }
            // `to_c` reads the leading complex value and answers (0+0i)
            // when the string does not start with one.
            "to_c" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let named = string_value.encoding_name();
                if wide_encoding(&named).is_some() {
                    return Err(crate::vm::errors::simple_exception(
                        "Encoding::CompatibilityError",
                        &format!("ASCII incompatible encoding: {named}"),
                        position,
                    ));
                }
                let held = string_value.as_str().to_string();
                let Some(parsed) = super::complex_methods::leading_complex_text(&held) else {
                    return self
                        .make_complex(Object::Int(0), Object::Int(0), position)
                        .map(Some);
                };
                let real = self.component_object(parsed.real, position)?;
                let imaginary = self.component_object(parsed.imaginary, position)?;
                if parsed.polar {
                    let (real, imaginary) = self.polar_parts(real, imaginary, position)?;
                    return self.make_complex(real, imaginary, position).map(Some);
                }
                self.make_complex(real, imaginary, position).map(Some)
            }
            // `to_r` reads the leading rational value and answers (0/1) when
            // the string does not start with one.
            "to_r" => {
                let (numerator, denominator) =
                    super::rational_methods::parse_rational_text(&string_value.as_str());
                self.make_rational(numerator, denominator, position)
                    .map(Some)
            }
            "to_f" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Float(leading_float(
                    &string_value.as_ref().as_str(),
                ))))
            }
            "dup" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // The copy is written in the same encoding, and reads what
                // it holds the same way.
                let copy = Object::string(string_value.as_str().to_string());
                if let Object::String(made) = &copy {
                    made.set_encoding(string_value.encoding_name());
                    if string_value.holds_bytes() {
                        made.mark_bytes();
                    }
                    // A copy of a packed string names the same run of text,
                    // which is what lets `unpack` read a pointer out of it.
                    made.set_pointer(string_value.pointer());
                }
                Ok(Some(copy))
            }
            // `scan` walks every match of a pattern. Without capture
            // groups each match is the text it matched, and with them each is
            // the array of what the groups took.
            "scan" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let (pattern, flags) = match pattern_of(&arguments[0]) {
                    Some(held) => held,
                    None => {
                        let needle = self.string_needle(method_name, &arguments[0], position)?;
                        (crate::regexp::escape(&needle.as_str()), String::new())
                    }
                };
                // `\G` asks each match to begin where the one before it ended,
                // so the walk stops at the first gap.
                let (anchored, source) = match super::regexp_methods::previous_match_split(&pattern)
                {
                    Some((before, after)) if before.is_empty() => (true, after),
                    _ => (false, pattern.clone()),
                };
                let Some(compiled) = super::regexp_methods::compile(&source, &flags) else {
                    return Err(crate::vm::errors::simple_exception(
                        "RegexpError",
                        &format!("invalid pattern: /{}/", source),
                        position,
                    ));
                };
                let subject = string_value.as_str().to_string();
                let groups = compiled.group_count() - 1;
                let mut found: Vec<Object> = Vec::new();
                // Where each match began, so the walk can report it as the
                // last match while the block runs.
                let mut reached: Vec<usize> = Vec::new();
                let mut cursor = 0usize;
                while let Some(captured) = compiled.captures_at(&subject, cursor) {
                    let Some(whole) = captured.get(0) else { break };
                    if anchored && whole.start() != cursor {
                        break;
                    }
                    reached.push(whole.start());
                    if groups == 0 {
                        found.push(Object::string(whole.as_str().to_string()));
                    } else {
                        let taken: Vec<Object> = (1..=groups)
                            .map(|index| match captured.get(index) {
                                Some(held) => Object::string(held.as_str().to_string()),
                                None => Object::Nil,
                            })
                            .collect();
                        found.push(Object::array(taken));
                    }
                    if whole.end() > whole.start() {
                        cursor = whole.end();
                        continue;
                    }
                    // An empty match moves on by one character, and the one at
                    // the very end closes the walk.
                    if whole.end() >= subject.len() {
                        break;
                    }
                    cursor = whole.end() + 1;
                    while cursor < subject.len() && !subject.is_char_boundary(cursor) {
                        cursor += 1;
                    }
                }
                let last = reached.last().copied();
                let answer = match self.pending_block.take() {
                    Some(Object::Block(block)) => {
                        for (index, item) in found.into_iter().enumerate() {
                            // `$~` and the readings taken from it name the
                            // match the block is being handed.
                            let at = reached.get(index).copied().unwrap_or(0);
                            self.regexp_match_data_in(
                                &source, &flags, &subject, at, None, position,
                            )?;
                            // A match with groups arrives as one Array, which
                            // a block of several parameters spreads out.
                            self.execute_block_callable(&block, vec![item], position)?;
                        }
                        receiver.clone()
                    }
                    _ => Object::array(found),
                };
                // The walk leaves the last match behind it, whatever the block
                // matched while it ran.
                match last {
                    Some(at) => {
                        self.regexp_match_data_in(&source, &flags, &subject, at, None, position)?;
                    }
                    None => {
                        self.globals_mut()
                            .set(super::regexp_methods::LAST_MATCH, Object::Nil);
                    }
                }
                Ok(Some(answer))
            }
            // A pattern is matched against characters, so a string whose
            // bytes spell nothing has nothing to match against.
            "gsub" | "sub" if !holds_valid_text(string_value) => {
                Err(broken_text_error(string_value, position))
            }
            "gsub" | "sub" => self
                .substitute(receiver, string_value, method_name, arguments, position)
                .map(Some),

            "empty?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(string_value.as_str().is_empty())))
            }
            _ => self.call_string_set_method(receiver, method_name, arguments, position),
        }
    }
}

/// Ruby's `String#succ`: the rightmost alphanumeric character is bumped, and a
/// carry moves left, growing the string when the leftmost one wraps. A string
/// with no alphanumeric character bumps its last byte instead.
pub(crate) fn successor_of(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut letters: Vec<char> = text.chars().collect();
    let alphanumeric: Vec<usize> = letters
        .iter()
        .enumerate()
        .filter(|(_, letter)| letter.is_ascii_alphanumeric())
        .map(|(index, _)| index)
        .collect();
    if alphanumeric.is_empty() {
        // With no letters or digits the characters count up as the bytes
        // they stand for, carrying from the end. A carry past the front puts
        // one more character there.
        if letters.iter().all(|letter| (*letter as u32) < 256) {
            let mut at = letters.len();
            loop {
                if at == 0 {
                    letters.insert(0, '\u{1}');
                    break;
                }
                at -= 1;
                if letters[at] == '\u{ff}' {
                    letters[at] = '\0';
                    continue;
                }
                letters[at] = char::from_u32(letters[at] as u32 + 1).unwrap_or(letters[at]);
                break;
            }
            return letters.into_iter().collect();
        }
        let last = letters.len() - 1;
        let bumped = (letters[last] as u32).wrapping_add(1);
        if let Some(letter) = char::from_u32(bumped) {
            letters[last] = letter;
        }
        return letters.into_iter().collect();
    }
    for (step, position) in alphanumeric.iter().rev().enumerate() {
        let (next, carried) = bump(letters[*position]);
        letters[*position] = next;
        if !carried {
            return letters.into_iter().collect();
        }
        if step + 1 == alphanumeric.len() {
            // Every character carried, so one more is prepended: "zz" grows
            // into "aaa" and "99" into "100".
            let leading = if letters[*position].is_ascii_digit() {
                '1'
            } else {
                letters[*position]
            };
            letters.insert(*position, leading);
        }
    }
    letters.into_iter().collect()
}

/// One character of a `succ`, answering what it becomes and whether the bump
/// carried past the end of its run.
fn bump(letter: char) -> (char, bool) {
    match letter {
        'z' => ('a', true),
        'Z' => ('A', true),
        '9' => ('0', true),
        other => (char::from_u32(other as u32 + 1).unwrap_or(other), false),
    }
}

/// The Float the leading characters of a string spell, which is 0.0 when they
/// spell none. Underscores separate digits, and a trailing `e`, `.`, or sign
/// that names no digits is left off rather than refused.
fn leading_float(text: &str) -> f64 {
    let letters: Vec<char> = text.trim_start().chars().collect();
    let mut taken = String::new();
    let mut index = 0;
    if matches!(letters.first(), Some('+') | Some('-')) {
        taken.push(letters[0]);
        index = 1;
    }
    // An underscore separates digits, so one that does not sit between two of
    // them ends the number.
    let mut digits = 0;
    while index < letters.len() {
        if letters[index].is_ascii_digit() {
            taken.push(letters[index]);
            digits += 1;
            index += 1;
            continue;
        }
        if letters[index] == '_'
            && digits > 0
            && letters
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_digit())
        {
            index += 1;
            continue;
        }
        break;
    }
    if index < letters.len() && letters[index] == '.' {
        let mut fraction = String::new();
        let mut cursor = index + 1;
        while cursor < letters.len() {
            if letters[cursor].is_ascii_digit() {
                fraction.push(letters[cursor]);
                cursor += 1;
                continue;
            }
            if letters[cursor] == '_'
                && !fraction.is_empty()
                && letters
                    .get(cursor + 1)
                    .is_some_and(|next| next.is_ascii_digit())
            {
                cursor += 1;
                continue;
            }
            break;
        }
        if !fraction.is_empty() || digits > 0 {
            taken.push('.');
            taken.push_str(&fraction);
            digits += fraction.len();
            index = cursor;
        }
    }
    if digits == 0 {
        return 0.0;
    }
    // An exponent counts only when digits follow it.
    if index < letters.len() && (letters[index] == 'e' || letters[index] == 'E') {
        let mut cursor = index + 1;
        let mut exponent = String::new();
        if matches!(letters.get(cursor), Some('+') | Some('-')) {
            exponent.push(letters[cursor]);
            cursor += 1;
        }
        let mut exponent_digits = 0;
        while cursor < letters.len() {
            if letters[cursor].is_ascii_digit() {
                exponent.push(letters[cursor]);
                exponent_digits += 1;
                cursor += 1;
                continue;
            }
            if letters[cursor] == '_'
                && exponent_digits > 0
                && letters
                    .get(cursor + 1)
                    .is_some_and(|next| next.is_ascii_digit())
            {
                cursor += 1;
                continue;
            }
            break;
        }
        if exponent_digits > 0 {
            taken.push('e');
            taken.push_str(&exponent);
        }
    }
    taken.parse().unwrap_or(0.0)
}

/// Read a number off the front of `text` in `default_radix`, honoring a base
/// prefix (`0x`, `0b`, `0o`, `0d`) and treating an underscore between digits
/// as a separator. Anything the number does not start with answers 0.
fn leading_radix_number(text: &str, default_radix: u32) -> i64 {
    let trimmed = text.trim_start();
    let (sign, rest) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1i64, rest),
        None => (1i64, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    // `hex` reads only the `0x` prefix, since `0b` and `0d` are themselves
    // hex digits. `oct` reads every base prefix.
    let (radix, rest) = match rest.get(..2).map(str::to_ascii_lowercase).as_deref() {
        Some("0x") => (16, &rest[2..]),
        Some("0b") if default_radix == 8 => (2, &rest[2..]),
        Some("0o") if default_radix == 8 => (8, &rest[2..]),
        Some("0d") if default_radix == 8 => (10, &rest[2..]),
        _ => (default_radix, rest),
    };
    let mut digits = String::new();
    let mut previous_was_digit = false;
    for character in rest.chars() {
        if character == '_' && previous_was_digit {
            previous_was_digit = false;
            continue;
        }
        if !character.is_digit(radix) {
            break;
        }
        previous_was_digit = true;
        digits.push(character);
    }
    match i64::from_str_radix(&digits, radix) {
        Ok(value) => sign * value,
        Err(_) => 0,
    }
}

impl VirtualMachine {
    /// The encoding an argument names, whether it arrives as an Encoding or
    /// as its name in text.
    pub(crate) fn encoding_name_argument(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match value {
            Object::Class(encoding) => Ok(encoding.name().to_string()),
            Object::String(named) => {
                let found = self.send_to_object(
                    self.globals().get("Encoding").unwrap_or(Object::Nil),
                    "find",
                    vec![Object::string(named.to_text())],
                    position,
                )?;
                match found {
                    Object::Class(encoding) => Ok(encoding.name().to_string()),
                    // A special name such as "internal" that nothing is set
                    // to names no encoding, and Ruby reads the bytes as
                    // bytes rather than refusing them.
                    Object::Nil => Ok("ASCII-8BIT".to_string()),
                    _ => Ok(named.as_str().to_string()),
                }
            }
            // Anything that reads as a String names an encoding by the
            // characters it reads as.
            other if self.responds_to(other, "to_str") => {
                let read = self.send_to_object(other.clone(), "to_str", vec![], position)?;
                match read {
                    Object::String(_) => self.encoding_name_argument(&read, position),
                    _ => Err(method_argument_type_error(
                        "force_encoding",
                        "String",
                        other,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                "force_encoding",
                "String",
                other,
                position,
            )),
        }
    }

    /// The Encoding object a name stands for, and UTF-8 where the name is one
    /// metorex does not carry a constant for.
    /// The one frozen string that stands for this text in this encoding.
    /// Every equal string deduplicates to it.
    pub(crate) fn deduped_string(&mut self, held: &crate::object::StringValue) -> Object {
        let key = (held.as_str().to_string(), held.encoding_name());
        if let Some(found) = self.deduped_strings.get(&key) {
            return Object::String(Rc::clone(found));
        }
        let made = crate::object::StringValue::with_encoding(held.to_text(), held.encoding_name());
        if held.holds_bytes() {
            made.mark_bytes();
        }
        // Every place writing the same literal shares one string, so the
        // place it is reported from is the first one that wrote it.
        if let Some(written_at) = held.created_at() {
            made.set_created_at(written_at);
        }
        made.mark_deduplicated();
        let made = Rc::new(made);
        self.deduped_strings.insert(key, Rc::clone(&made));
        Object::String(made)
    }

    pub(crate) fn encoding_object(&mut self, name: &str) -> Object {
        for (constant, display, _) in crate::vm::init::ENCODING_NAMES {
            if display == name
                && let Some(found) = self.globals().get(&format!("Encoding::{}", constant))
            {
                return found;
            }
        }
        self.globals().get("Encoding::UTF_8").unwrap_or(Object::Nil)
    }
}

/// The bytes a string stands for. A string tagged binary holds one character
/// per byte, so its characters are its bytes rather than their UTF-8 form.
pub(crate) fn binary_bytes(string_value: &crate::object::StringValue) -> Vec<u8> {
    if string_value.holds_bytes() {
        return super::pack_format::string_to_bytes(&string_value.as_str());
    }
    match string_value.encoding_name().as_str() {
        "ASCII-8BIT" | "BINARY" => super::pack_format::string_to_bytes(&string_value.as_str()),
        named if wide_encoding(named).is_some() => {
            super::pack_format::string_to_bytes(&string_value.as_str())
        }
        _ => string_value.as_str().as_bytes().to_vec(),
    }
}

/// How an encoding lays a character out: the width of a code unit, whether
/// the high byte comes first, and whether a byte order mark opens the text.
#[derive(Clone, Copy)]
pub(crate) struct WideShape {
    unit: usize,
    big_endian: bool,
    marked: bool,
}

/// The shape of an encoding that spells a character in more than one byte,
/// or None for one that spells it in a single byte.
pub(crate) fn wide_encoding(named: &str) -> Option<WideShape> {
    let shape = match named {
        "UTF-16BE" => (2, true, false),
        "UTF-16LE" => (2, false, false),
        "UTF-16" => (2, true, true),
        "UTF-32BE" => (4, true, false),
        "UTF-32LE" => (4, false, false),
        "UTF-32" => (4, true, true),
        _ => return None,
    };
    Some(WideShape {
        unit: shape.0,
        big_endian: shape.1,
        marked: shape.2,
    })
}

/// Text written out in a wide encoding, one code unit at a time, with the
/// surrogate pair a character above the basic plane calls for.
pub(crate) fn wide_bytes(text: &str, shape: WideShape) -> Vec<u8> {
    let mut units: Vec<u32> = Vec::new();
    if shape.marked {
        units.push(0xfeff);
    }
    for letter in text.chars() {
        let point = letter as u32;
        if shape.unit == 2 && point > 0xffff {
            let carried = point - 0x10000;
            units.push(0xd800 + (carried >> 10));
            units.push(0xdc00 + (carried & 0x3ff));
        } else {
            units.push(point);
        }
    }
    let mut bytes = Vec::with_capacity(units.len() * shape.unit);
    for unit in units {
        let written = unit.to_be_bytes();
        let taken = &written[4 - shape.unit..];
        if shape.big_endian {
            bytes.extend_from_slice(taken);
        } else {
            bytes.extend(taken.iter().rev());
        }
    }
    bytes
}

/// The text a run of bytes spells in a wide encoding, reading the byte order
/// mark when the encoding opens with one.
pub(crate) fn wide_text(bytes: &[u8], shape: WideShape) -> String {
    let mut big_endian = shape.big_endian;
    let mut at = 0;
    let mut units: Vec<u32> = Vec::new();
    while at + shape.unit <= bytes.len() {
        let taken = &bytes[at..at + shape.unit];
        let mut value = 0u32;
        if big_endian {
            for byte in taken {
                value = (value << 8) | *byte as u32;
            }
        } else {
            for byte in taken.iter().rev() {
                value = (value << 8) | *byte as u32;
            }
        }
        at += shape.unit;
        if units.is_empty() && shape.marked && value == 0xfeff {
            continue;
        }
        if units.is_empty() && shape.marked && value == 0xfffe0000 {
            big_endian = !big_endian;
            continue;
        }
        units.push(value);
    }
    let mut text = String::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        index += 1;
        if shape.unit == 2 && (0xd800..0xdc00).contains(&unit) && index < units.len() {
            let low = units[index];
            index += 1;
            let point = 0x10000 + ((unit - 0xd800) << 10) + (low - 0xdc00);
            if let Some(letter) = char::from_u32(point) {
                text.push(letter);
            }
            continue;
        }
        if let Some(letter) = char::from_u32(unit) {
            text.push(letter);
        }
    }
    text
}

/// A run of bytes held as text, one character to a byte, which is how a
/// string tagged with a wide encoding carries what it spells.
pub(crate) fn bytes_as_text(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| *byte as char).collect()
}

/// Which way a case mapping runs.
#[derive(Clone, Copy, PartialEq)]
enum CaseWanted {
    Up,
    Down,
}

/// The options a case mapping was asked for.
struct CaseOptions {
    ascii_only: bool,
    turkic: bool,
    folding: bool,
}

/// Read the Symbols naming how a case mapping should run. Ruby allows one
/// option, and allows Turkic and Lithuanian together.
fn case_options(
    method_name: &str,
    arguments: &[Object],
    position: Position,
) -> Result<CaseOptions, MetorexError> {
    let mut named = Vec::new();
    for argument in arguments {
        let Object::Symbol(name) = argument else {
            let message = format!("invalid option {}", argument);
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &message,
                position,
            ));
        };
        let name = name.as_str().to_string();
        if !matches!(
            name.as_str(),
            "ascii" | "turkic" | "lithuanian" | "fold" | "downcase"
        ) || (name == "fold" && !method_name.starts_with("downcase"))
        {
            let message = format!("invalid option :{}", name);
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &message,
                position,
            ));
        }
        named.push(name);
    }
    // Turkic and Lithuanian are the only pair that go together.
    if named.len() > 2
        || (named.len() == 2
            && !named
                .iter()
                .all(|name| name == "turkic" || name == "lithuanian"))
    {
        let message = "too many options".to_string();
        return Err(crate::vm::errors::simple_exception(
            "ArgumentError",
            &message,
            position,
        ));
    }
    Ok(CaseOptions {
        ascii_only: named.iter().any(|name| name == "ascii"),
        turkic: named.iter().any(|name| name == "turkic"),
        folding: named.iter().any(|name| name == "fold"),
    })
}

/// Text with its letters mapped the way the options ask for.
fn mapped_case(text: &str, wanted: CaseWanted, options: &CaseOptions) -> String {
    if options.ascii_only {
        return match wanted {
            CaseWanted::Up => text.chars().map(|held| held.to_ascii_uppercase()).collect(),
            CaseWanted::Down => text.chars().map(|held| held.to_ascii_lowercase()).collect(),
        };
    }
    if options.turkic {
        // Turkish keeps the dot of an `i` apart from the letter itself, so
        // the two `i`s map to their own pairs.
        return text
            .chars()
            .flat_map(|held| match (wanted, held) {
                (CaseWanted::Down, '\u{130}') => vec!['i'],
                (CaseWanted::Down, 'I') => vec!['\u{131}'],
                (CaseWanted::Up, 'i') => vec!['\u{130}'],
                (CaseWanted::Up, '\u{131}') => vec!['I'],
                (CaseWanted::Up, _) => held.to_uppercase().collect(),
                (CaseWanted::Down, _) => held.to_lowercase().collect(),
            })
            .collect();
    }
    let mapped = match wanted {
        CaseWanted::Up => text.to_uppercase(),
        CaseWanted::Down => text.to_lowercase(),
    };
    // Folding maps a letter onto the letters it compares equal to, which is
    // where a sharp s becomes two of them.
    if options.folding {
        return mapped.replace('\u{df}', "ss");
    }
    mapped
}

/// The first letter raised and the rest lowered. Raising one letter may give
/// several, and only the first of those stays raised.
fn capitalized_case(text: &str, options: &CaseOptions) -> String {
    let mut letters = text.chars();
    let Some(first) = letters.next() else {
        return String::new();
    };
    let raised = mapped_case(&first.to_string(), CaseWanted::Up, options);
    let mut made = String::new();
    let mut raised_letters = raised.chars();
    if let Some(leading) = raised_letters.next() {
        made.push(leading);
    }
    let rest: String = raised_letters.collect();
    made.push_str(&mapped_case(&rest, CaseWanted::Down, options));
    let remainder: String = letters.collect();
    made.push_str(&mapped_case(&remainder, CaseWanted::Down, options));
    made
}

/// Each letter turned the other way.
fn swapped_case(text: &str, options: &CaseOptions) -> String {
    text.chars()
        .flat_map(|letter| {
            if options.ascii_only && !letter.is_ascii() {
                return vec![letter];
            }
            let wanted = if letter.is_uppercase() {
                CaseWanted::Down
            } else if letter.is_lowercase() {
                CaseWanted::Up
            } else {
                return vec![letter];
            };
            mapped_case(&letter.to_string(), wanted, options)
                .chars()
                .collect()
        })
        .collect()
}

/// Whether a string spells characters in the encoding it says it is written
/// in. Only a run of bytes tagged as text can fail to.
pub(crate) fn holds_valid_text(string_value: &crate::object::StringValue) -> bool {
    // An encoding Ruby names without converting anything through it reads its
    // text a byte at a time, so every run of bytes spells characters in it.
    // One that still pairs its bytes is read for that shape all the same.
    let named = string_value.encoding_name();
    if dummy_encoding(&named) && !pairs_its_bytes(&named) {
        return true;
    }
    match named.as_str() {
        "ASCII-8BIT" | "BINARY" => true,
        "US-ASCII" => string_value.as_str().is_ascii(),
        "UTF-8" if string_value.holds_bytes() => {
            String::from_utf8(super::pack_format::string_to_bytes(&string_value.as_str())).is_ok()
        }
        "UTF8-MAC" if string_value.holds_bytes() => {
            String::from_utf8(super::pack_format::string_to_bytes(&string_value.as_str())).is_ok()
        }
        "EUC-JP" => euc_jp_reads(&binary_bytes(string_value)),
        // Shift_JIS spells half-width katakana with one byte of its own, so
        // the shape of a run says more than a plain lead-and-trail pairing.
        named if spells_shift_jis(named) => shift_jis_reads(&binary_bytes(string_value)),
        // A fixed-width encoding reads whole units, so a run of bytes that
        // does not divide into them spells no characters at all, and a half
        // of a surrogate pair standing alone spells none either.
        "UTF-16" | "UTF-16BE" | "UTF-16LE" => utf16_reads(
            &binary_bytes(string_value),
            !string_value.encoding_name().ends_with("LE"),
        ),
        "UTF-32" | "UTF-32BE" | "UTF-32LE" => utf32_reads(
            &binary_bytes(string_value),
            !string_value.encoding_name().ends_with("LE"),
        ),
        named if pairs_its_bytes(named) => paired_bytes_read(&binary_bytes(string_value)),
        _ => true,
    }
}

/// Whether an encoding spells some characters with a lead byte and a trailing
/// one. Metorex reads the shape of such a run rather than the character it
/// stands for, which is what tells a broken run from a whole one.
fn pairs_its_bytes(named: &str) -> bool {
    matches!(
        named,
        "Big5"
            | "CP949"
            | "EUC-KR"
            | "EUC-TW"
            | "GB18030"
            | "GBK"
            | "GB2312"
            | "GB12345"
            | "Shift_JIS"
            | "Windows-31J"
            | "MacJapanese"
            | "Emacs-Mule"
            | "stateless-ISO-2022-JP"
            | "eucJP-ms"
            | "CP51932"
    )
}

/// Whether every lead byte in a run is followed by the byte it pairs with.
/// A byte under 0x80 stands for itself.
fn paired_bytes_read(bytes: &[u8]) -> bool {
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] < 0x80 {
            at += 1;
            continue;
        }
        if at + 1 >= bytes.len() {
            return false;
        }
        at += 2;
    }
    true
}

/// Whether a run of bytes spells characters in UTF-16: whole units, with each
/// high half of a surrogate pair followed by a low one.
fn utf16_reads(bytes: &[u8], big_endian: bool) -> bool {
    if !bytes.len().is_multiple_of(2) {
        return false;
    }
    let unit_at = |at: usize| -> u16 {
        let (first, second) = (bytes[at] as u16, bytes[at + 1] as u16);
        if big_endian {
            (first << 8) | second
        } else {
            (second << 8) | first
        }
    };
    let mut at = 0;
    while at < bytes.len() {
        let unit = unit_at(at);
        if (0xdc00..0xe000).contains(&unit) {
            return false;
        }
        if (0xd800..0xdc00).contains(&unit) {
            if at + 3 >= bytes.len() || !(0xdc00..0xe000).contains(&unit_at(at + 2)) {
                return false;
            }
            at += 4;
            continue;
        }
        at += 2;
    }
    true
}

/// Whether a run of bytes spells characters in UTF-32: whole units, each one
/// a code point Unicode names.
fn utf32_reads(bytes: &[u8], big_endian: bool) -> bool {
    if !bytes.len().is_multiple_of(4) {
        return false;
    }
    bytes.chunks(4).all(|unit| {
        let point = if big_endian {
            u32::from_be_bytes([unit[0], unit[1], unit[2], unit[3]])
        } else {
            u32::from_le_bytes([unit[0], unit[1], unit[2], unit[3]])
        };
        char::from_u32(point).is_some()
    })
}

/// Whether a run of bytes spells characters in EUC-JP. A byte under 0x80
/// stands for itself, 0x8E opens a half-width katakana pair, 0x8F opens a
/// three-byte run, and everything else pairs two bytes from 0xA1 to 0xFE.
fn euc_jp_reads(bytes: &[u8]) -> bool {
    let mut at = 0;
    while at < bytes.len() {
        let held = bytes[at];
        let width = match held {
            0x00..=0x7f => 1,
            0x8e => 2,
            0x8f => 3,
            0xa1..=0xfe => 2,
            _ => return false,
        };
        if at + width > bytes.len() {
            return false;
        }
        let trailing = &bytes[at + 1..at + width];
        let fits = match held {
            0x8e => trailing.iter().all(|byte| (0xa1..=0xdf).contains(byte)),
            _ => trailing.iter().all(|byte| (0xa1..=0xfe).contains(byte)),
        };
        if !fits {
            return false;
        }
        at += width;
    }
    true
}

/// Whether an encoding spells the ASCII letters one byte to a letter.
pub(crate) fn encoding_is_ascii_compatible(named: &str) -> bool {
    !matches!(
        named,
        "UTF-16" | "UTF-16BE" | "UTF-16LE" | "UTF-32" | "UTF-32BE" | "UTF-32LE"
    )
}

/// The ArgumentError Ruby raises for a string whose bytes spell nothing in
/// the encoding it is tagged with.
pub(crate) fn broken_text_error(
    string_value: &crate::object::StringValue,
    position: Position,
) -> MetorexError {
    let message = format!("invalid byte sequence in {}", string_value.encoding_name());
    crate::vm::errors::simple_exception("ArgumentError", &message, position)
}

/// Whether two strings are written in encodings that cannot be joined. Text
/// that is nothing but ASCII goes with anything.
pub(crate) fn encodings_clash(
    left: &crate::object::StringValue,
    right: &crate::object::StringValue,
) -> bool {
    if left.encoding_name() == right.encoding_name() {
        return false;
    }
    !left.as_str().is_ascii() && !right.as_str().is_ascii()
}

/// Whether two strings are written in encodings Ruby will compare. Empty
/// text goes with anything, as does text that is nothing but ASCII when the
/// other encoding spells ASCII one byte to a letter.
pub(crate) fn strings_comparable(
    left: &crate::object::StringValue,
    right: &crate::object::StringValue,
) -> bool {
    if left.as_str().is_empty() || right.as_str().is_empty() {
        return true;
    }
    if left.encoding_name() == right.encoding_name() {
        return true;
    }
    // Text tagged as raw bytes says nothing about the characters behind it,
    // so it compares against anything rather than against its own tag alone.
    let reads_as_bytes =
        |named: &str| matches!(named, "ASCII-8BIT" | "BINARY" | "ASCII-8BIT (BINARY)");
    if reads_as_bytes(&left.encoding_name()) || reads_as_bytes(&right.encoding_name()) {
        return true;
    }
    let left_plain =
        left.as_str().is_ascii() && encoding_is_ascii_compatible(&left.encoding_name());
    let right_plain =
        right.as_str().is_ascii() && encoding_is_ascii_compatible(&right.encoding_name());
    if left_plain && right_plain {
        return true;
    }
    if left_plain && encoding_is_ascii_compatible(&right.encoding_name()) {
        return true;
    }
    right_plain && encoding_is_ascii_compatible(&left.encoding_name())
}

/// The Encoding::CompatibilityError Ruby raises for joining text written in
/// two encodings that do not go together.
pub(crate) fn clashing_encodings_error(
    left: &crate::object::StringValue,
    right: &crate::object::StringValue,
    position: Position,
) -> MetorexError {
    let message = format!(
        "incompatible character encodings: {} and {}",
        left.encoding_name(),
        right.encoding_name()
    );
    crate::vm::errors::simple_exception("Encoding::CompatibilityError", &message, position)
}

/// How many bytes the character opening a run takes in UTF-8, or 0 when the
/// run does not open a whole one.
fn utf8_sequence_width(bytes: &[u8]) -> usize {
    let Some(first) = bytes.first().copied() else {
        return 0;
    };
    let width = match first {
        0x00..=0x7f => return 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return 0,
    };
    if bytes.len() < width {
        return 0;
    }
    if bytes[1..width]
        .iter()
        .all(|held| (0x80..=0xbf).contains(held))
    {
        width
    } else {
        0
    }
}

/// How many characters a string holds, counted the way the encoding it is
/// tagged with reads them. A string that carries text rather than a run of
/// bytes is counted by its characters, since relabelling it leaves the text
/// as it was.
pub(crate) fn character_count(string_value: &crate::object::StringValue) -> i64 {
    let named = string_value.encoding_name();
    match named.as_str() {
        // A run of bytes has one character to the byte however it is tagged.
        "ASCII-8BIT" | "BINARY" => binary_bytes(string_value).len() as i64,
        // A fixed-width encoding reads whole units, and a unit left short at
        // the end still counts as the one broken character it spells.
        "UTF-16" | "UTF-16BE" | "UTF-16LE" if string_value.holds_bytes() => {
            utf16_unit_count(&binary_bytes(string_value), named.ends_with("BE"))
        }
        "UTF-32" | "UTF-32BE" | "UTF-32LE" if string_value.holds_bytes() => {
            binary_bytes(string_value).len().div_ceil(4) as i64
        }
        // Shift_JIS pairs some of its bytes, so a run of them spells fewer
        // characters than it has bytes.
        held if spells_shift_jis(held) && string_value.holds_bytes() => {
            shift_jis_characters(&binary_bytes(string_value)).len() as i64
        }
        _ => string_value.as_str().chars().count() as i64,
    }
}

/// How many characters a run of UTF-16 bytes spells. A high surrogate paired
/// with a low one stands for a single character, and one left on its own
/// stands for itself.
fn utf16_unit_count(bytes: &[u8], big_endian: bool) -> i64 {
    let unit_at = |at: usize| -> u16 {
        let (first, second) = (bytes[at] as u16, bytes[at + 1] as u16);
        if big_endian {
            (first << 8) | second
        } else {
            (second << 8) | first
        }
    };
    let mut counted = 0i64;
    let mut at = 0usize;
    while at + 1 < bytes.len() {
        let unit = unit_at(at);
        let paired = (0xd800..0xdc00).contains(&unit)
            && at + 3 < bytes.len()
            && (0xdc00..0xe000).contains(&unit_at(at + 2));
        at += if paired { 4 } else { 2 };
        counted += 1;
    }
    if at < bytes.len() {
        counted += 1;
    }
    counted
}

/// The number a string starts with, the way `String#to_i` reads one: leading
/// whitespace, one sign, a radix prefix that agrees with the base, then
/// digits with lone underscores between them. Reading stops at the first
/// character the base does not name, and a string that starts with none is
/// zero.
pub(crate) fn leading_integer(text: &str, base: u32) -> num_bigint::BigInt {
    let mut rest = text.trim_start_matches(|c: char| c.is_whitespace());
    let mut negative = false;
    if let Some(stripped) = rest.strip_prefix('+') {
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix('-') {
        negative = true;
        rest = stripped;
    }

    let lowered = rest.to_ascii_lowercase();
    let prefix_radix = if lowered.starts_with("0x") {
        Some(16)
    } else if lowered.starts_with("0b") {
        Some(2)
    } else if lowered.starts_with("0o") {
        Some(8)
    } else if lowered.starts_with("0d") {
        Some(10)
    } else {
        None
    };

    let mut radix = base;
    match prefix_radix {
        Some(prefix) if base == 0 || base == prefix => {
            radix = prefix;
            rest = &rest[2..];
        }
        // Only a base left to the string reads a bare leading zero as octal.
        _ if base == 0 && rest.len() > 1 && rest.starts_with('0') => {
            radix = 8;
            rest = &rest[1..];
        }
        _ => {}
    }
    if radix == 0 {
        radix = 10;
    }

    let mut digits = String::with_capacity(rest.len());
    let mut previous_underscore = false;
    for character in rest.chars() {
        if character == '_' {
            // A pair of them ends the number, and so does one before any
            // digit at all.
            if digits.is_empty() || previous_underscore {
                break;
            }
            previous_underscore = true;
            continue;
        }
        if character.to_digit(radix).is_none() {
            break;
        }
        digits.push(character);
        previous_underscore = false;
    }
    let magnitude = num_bigint::BigInt::parse_bytes(digits.as_bytes(), radix).unwrap_or_default();
    if negative { -magnitude } else { magnitude }
}

impl VirtualMachine {
    /// The encoding `inspect` writes its answer in: the one a program named
    /// as the internal or the external encoding, or US-ASCII when that one
    /// spells ASCII differently from ASCII itself.
    pub(crate) fn inspect_result_encoding(&mut self) -> String {
        let named = match self.globals().get("__Encoding_default_internal") {
            Some(Object::Class(held)) => held.name().to_string(),
            _ => match self.globals().get("__Encoding_default_external") {
                Some(Object::Class(held)) => held.name().to_string(),
                _ => crate::object::string_value::DEFAULT_ENCODING.to_string(),
            },
        };
        if encoding_is_ascii_compatible(&named) {
            named
        } else {
            "US-ASCII".to_string()
        }
    }
}

/// The arguments an `encode` was written with, split from what it says to put
/// in place of a byte the source encoding cannot read. `invalid: :replace`
/// asks for that, and `replace:` names the text, which defaults to `"?"`.
fn encode_options(arguments: &[Object]) -> (&[Object], Option<String>) {
    let Some(Object::Dict(pairs)) = arguments.last() else {
        return (arguments, None);
    };
    let held = pairs.borrow();
    if !held.contains_key("__MX_KWARGS__") {
        return (arguments, None);
    }
    let replacing = matches!(
        held.get(":invalid"),
        Some(Object::Symbol(named)) if &*named.as_str() == "replace"
    ) || matches!(
        held.get(":undef"),
        Some(Object::Symbol(named)) if &*named.as_str() == "replace"
    );
    let stands_in = match held.get(":replace") {
        Some(Object::String(text)) => text.as_str().to_string(),
        _ => "?".to_string(),
    };
    drop(held);
    let rest = &arguments[..arguments.len() - 1];
    (rest, replacing.then_some(stands_in))
}

/// The text a run of EUC-JP bytes spells, with a byte that opens no character
/// written as the text that stands in for one.
fn euc_jp_text_replacing(bytes: &[u8], stands_in: &str) -> String {
    let mut written = String::with_capacity(bytes.len());
    let mut at = 0usize;
    while at < bytes.len() {
        match super::euc_jp_table::euc_jp_character(&bytes[at..]) {
            Some((character, width)) => {
                written.push(character);
                at += width;
            }
            None => {
                written.push_str(stands_in);
                at += 1;
            }
        }
    }
    written
}

/// Whether Ruby names this encoding without converting anything through it.
/// A string tagged with one has a character to the byte, since nothing reads
/// its bytes as text.
/// A symbol written the way `inspect` writes one, reading its name through
/// the encoding it carries. A name that does not read as plain text is
/// quoted, with the bytes it holds written out.
pub(crate) fn symbol_inspect_text(value: &crate::object::StringValue) -> String {
    let named = value.encoding_name();
    let bytes = binary_bytes(value);
    // An encoding metorex names without reading text through it spells its
    // name byte by byte.
    if dummy_encoding(&named) {
        let written: String = bytes.iter().map(|byte| format!("\\x{byte:02X}")).collect();
        return format!(":\"{written}\"");
    }
    // An encoding that does not spell ASCII the way ASCII does is read
    // through, and the characters it names are quoted.
    if let Some(shape) = wide_encoding(&named) {
        return format!(":{:?}", wide_text(&bytes, shape));
    }
    if matches!(named.as_str(), "ASCII-8BIT" | "BINARY") && !bytes.is_ascii() {
        let mut written = String::from(":\"");
        for byte in &bytes {
            if byte.is_ascii_graphic() || *byte == b' ' {
                written.push(char::from(*byte));
            } else {
                written.push_str(&format!("\\x{byte:02X}"));
            }
        }
        written.push('"');
        return written;
    }
    crate::object::inspect_symbol(&value.as_str())
}

pub(crate) fn dummy_encoding(named: &str) -> bool {
    crate::vm::init::ENCODING_NAMES
        .iter()
        .any(|(_, canonical, dummy)| *dummy && *canonical == named)
}

/// Where each Shift_JIS character starts. A byte in the lead ranges is read
/// together with the byte after it, and every other byte stands alone.
fn shift_jis_characters(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut characters = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        let leads = matches!(bytes[at], 0x81..=0x9f | 0xe0..=0xfc) && at + 1 < bytes.len();
        let width = if leads { 2 } else { 1 };
        characters.push(bytes[at..at + width].to_vec());
        at += width;
    }
    characters
}

/// The characters a string spells, each tagged with the string's own
/// encoding. What counts as a character depends on that encoding: a run of
/// bytes and an encoding Ruby converts nothing through both split at every
/// byte, Shift_JIS reads a lead byte together with the one after it, and the
/// rest read text.
pub(crate) fn encoded_characters(string_value: &crate::object::StringValue) -> Vec<Object> {
    let named = string_value.encoding_name();
    let from_bytes = |bytes: &[u8]| {
        let spelled: String = bytes.iter().map(|byte| *byte as char).collect();
        let made = crate::object::StringValue::from_bytes(spelled);
        made.set_encoding(named.clone());
        Object::String(Rc::new(made))
    };
    let per_byte = || {
        binary_bytes(string_value)
            .iter()
            .map(|byte| from_bytes(&[*byte]))
            .collect()
    };
    match named.as_str() {
        "ASCII-8BIT" | "BINARY" => per_byte(),
        held if dummy_encoding(held) => per_byte(),
        "Shift_JIS" | "Windows-31J" | "MacJapanese" => {
            shift_jis_characters(&binary_bytes(string_value))
                .iter()
                .map(|character| from_bytes(character))
                .collect()
        }
        _ => string_value
            .as_str()
            .chars()
            .map(|character| {
                let made =
                    crate::object::StringValue::with_encoding(character.to_string(), named.clone());
                Object::String(Rc::new(made))
            })
            .collect(),
    }
}

/// The characters the bytes 0xA0 through 0xFF stand for in a single-byte
/// encoding, where they differ from Latin-1. Every other byte in these
/// encodings stands for the character of the same number.
fn latin_exceptions(named: &str) -> Option<&'static [(u8, char)]> {
    match named {
        "ISO-8859-1" => Some(&[]),
        "Windows-1252" => Some(&[
            (0x80, '\u{20ac}'),
            (0x82, '\u{201a}'),
            (0x83, '\u{0192}'),
            (0x84, '\u{201e}'),
            (0x85, '\u{2026}'),
            (0x86, '\u{2020}'),
            (0x87, '\u{2021}'),
            (0x88, '\u{02c6}'),
            (0x89, '\u{2030}'),
            (0x8a, '\u{0160}'),
            (0x8b, '\u{2039}'),
            (0x8c, '\u{0152}'),
            (0x8e, '\u{017d}'),
            (0x91, '\u{2018}'),
            (0x92, '\u{2019}'),
            (0x93, '\u{201c}'),
            (0x94, '\u{201d}'),
            (0x95, '\u{2022}'),
            (0x96, '\u{2013}'),
            (0x97, '\u{2014}'),
            (0x98, '\u{02dc}'),
            (0x99, '\u{2122}'),
            (0x9a, '\u{0161}'),
            (0x9b, '\u{203a}'),
            (0x9c, '\u{0153}'),
            (0x9e, '\u{017e}'),
            (0x9f, '\u{0178}'),
        ]),
        "ISO-8859-15" => Some(&[
            (0xa4, '\u{20ac}'),
            (0xa6, '\u{0160}'),
            (0xa8, '\u{0161}'),
            (0xb4, '\u{017d}'),
            (0xb8, '\u{017e}'),
            (0xbc, '\u{0152}'),
            (0xbd, '\u{0153}'),
            (0xbe, '\u{0178}'),
        ]),
        "IBM720" => Some(&[
            (0x82, '\u{00e9}'),
            (0x83, '\u{00e2}'),
            (0x85, '\u{00e0}'),
            (0x87, '\u{00e7}'),
            (0x88, '\u{00ea}'),
            (0x89, '\u{00eb}'),
            (0x8a, '\u{00e8}'),
            (0x8b, '\u{00ef}'),
            (0x8c, '\u{00ee}'),
            (0x91, '\u{0651}'),
            (0x92, '\u{0652}'),
            (0x93, '\u{00f4}'),
            (0x94, '\u{00a4}'),
            (0x95, '\u{0640}'),
            (0x96, '\u{00fb}'),
            (0x97, '\u{00f9}'),
            (0x98, '\u{0621}'),
            (0x99, '\u{0622}'),
            (0x9a, '\u{0623}'),
            (0x9b, '\u{0624}'),
            (0x9c, '\u{00a3}'),
            (0x9d, '\u{0625}'),
            (0x9e, '\u{0626}'),
            (0x9f, '\u{0627}'),
            (0xa0, '\u{0628}'),
            (0xa1, '\u{0629}'),
            (0xa2, '\u{062a}'),
            (0xa3, '\u{062b}'),
            (0xa4, '\u{062c}'),
            (0xa5, '\u{062d}'),
            (0xa6, '\u{062e}'),
            (0xa7, '\u{062f}'),
            (0xa8, '\u{0630}'),
            (0xa9, '\u{0631}'),
            (0xaa, '\u{0632}'),
            (0xab, '\u{0633}'),
            (0xac, '\u{0634}'),
            (0xad, '\u{0635}'),
            (0xae, '\u{00ab}'),
            (0xaf, '\u{00bb}'),
            (0xb0, '\u{2591}'),
            (0xb1, '\u{2592}'),
            (0xb2, '\u{2593}'),
            (0xb3, '\u{2502}'),
            (0xb4, '\u{2524}'),
            (0xb5, '\u{2561}'),
            (0xb6, '\u{2562}'),
            (0xb7, '\u{2556}'),
            (0xb8, '\u{2555}'),
            (0xb9, '\u{2563}'),
            (0xba, '\u{2551}'),
            (0xbb, '\u{2557}'),
            (0xbc, '\u{255d}'),
            (0xbd, '\u{255c}'),
            (0xbe, '\u{255b}'),
            (0xbf, '\u{2510}'),
            (0xc0, '\u{2514}'),
            (0xc1, '\u{2534}'),
            (0xc2, '\u{252c}'),
            (0xc3, '\u{251c}'),
            (0xc4, '\u{2500}'),
            (0xc5, '\u{253c}'),
            (0xc6, '\u{255e}'),
            (0xc7, '\u{255f}'),
            (0xc8, '\u{255a}'),
            (0xc9, '\u{2554}'),
            (0xca, '\u{2569}'),
            (0xcb, '\u{2566}'),
            (0xcc, '\u{2560}'),
            (0xcd, '\u{2550}'),
            (0xce, '\u{256c}'),
            (0xcf, '\u{2567}'),
            (0xd0, '\u{2568}'),
            (0xd1, '\u{2564}'),
            (0xd2, '\u{2565}'),
            (0xd3, '\u{2559}'),
            (0xd4, '\u{2558}'),
            (0xd5, '\u{2552}'),
            (0xd6, '\u{2553}'),
            (0xd7, '\u{256b}'),
            (0xd8, '\u{256a}'),
            (0xd9, '\u{2518}'),
            (0xda, '\u{250c}'),
            (0xdb, '\u{2588}'),
            (0xdc, '\u{2584}'),
            (0xdd, '\u{258c}'),
            (0xde, '\u{2590}'),
            (0xdf, '\u{2580}'),
            (0xe0, '\u{0636}'),
            (0xe1, '\u{0637}'),
            (0xe2, '\u{0638}'),
            (0xe3, '\u{0639}'),
            (0xe4, '\u{063a}'),
            (0xe5, '\u{0641}'),
            (0xe6, '\u{00b5}'),
            (0xe7, '\u{0642}'),
            (0xe8, '\u{0643}'),
            (0xe9, '\u{0644}'),
            (0xea, '\u{0645}'),
            (0xeb, '\u{0646}'),
            (0xec, '\u{0647}'),
            (0xed, '\u{0648}'),
            (0xee, '\u{0649}'),
            (0xef, '\u{064a}'),
            (0xf0, '\u{2261}'),
            (0xf1, '\u{064b}'),
            (0xf2, '\u{064c}'),
            (0xf3, '\u{064d}'),
            (0xf4, '\u{064e}'),
            (0xf5, '\u{064f}'),
            (0xf6, '\u{0650}'),
            (0xf7, '\u{2248}'),
            (0xf8, '\u{00b0}'),
            (0xf9, '\u{2219}'),
            (0xfa, '\u{00b7}'),
            (0xfb, '\u{221a}'),
            (0xfc, '\u{207f}'),
            (0xfd, '\u{00b2}'),
            (0xfe, '\u{25a0}'),
            (0xff, '\u{00a0}'),
        ]),
        "IBM437" => Some(&[
            (0x80, '\u{00c7}'),
            (0x81, '\u{00fc}'),
            (0x82, '\u{00e9}'),
            (0x83, '\u{00e2}'),
            (0x84, '\u{00e4}'),
            (0x85, '\u{00e0}'),
            (0x86, '\u{00e5}'),
            (0x87, '\u{00e7}'),
            (0x88, '\u{00ea}'),
            (0x89, '\u{00eb}'),
            (0x8a, '\u{00e8}'),
            (0x8b, '\u{00ef}'),
            (0x8c, '\u{00ee}'),
            (0x8d, '\u{00ec}'),
            (0x8e, '\u{00c4}'),
            (0x8f, '\u{00c5}'),
            (0x90, '\u{00c9}'),
            (0x91, '\u{00e6}'),
            (0x92, '\u{00c6}'),
            (0x93, '\u{00f4}'),
            (0x94, '\u{00f6}'),
            (0x95, '\u{00f2}'),
            (0x96, '\u{00fb}'),
            (0x97, '\u{00f9}'),
            (0x98, '\u{00ff}'),
            (0x99, '\u{00d6}'),
            (0x9a, '\u{00dc}'),
            (0x9b, '\u{00a2}'),
            (0x9c, '\u{00a3}'),
            (0x9d, '\u{00a5}'),
            (0x9e, '\u{20a7}'),
            (0x9f, '\u{0192}'),
            (0xa0, '\u{00e1}'),
            (0xa1, '\u{00ed}'),
            (0xa2, '\u{00f3}'),
            (0xa3, '\u{00fa}'),
            (0xa4, '\u{00f1}'),
            (0xa5, '\u{00d1}'),
            (0xa6, '\u{00aa}'),
            (0xa7, '\u{00ba}'),
            (0xa8, '\u{00bf}'),
            (0xa9, '\u{2310}'),
            (0xaa, '\u{00ac}'),
            (0xab, '\u{00bd}'),
            (0xac, '\u{00bc}'),
            (0xad, '\u{00a1}'),
            (0xae, '\u{00ab}'),
            (0xaf, '\u{00bb}'),
            (0xb0, '\u{2591}'),
            (0xb1, '\u{2592}'),
            (0xb2, '\u{2593}'),
            (0xb3, '\u{2502}'),
            (0xb4, '\u{2524}'),
            (0xb5, '\u{2561}'),
            (0xb6, '\u{2562}'),
            (0xb7, '\u{2556}'),
            (0xb8, '\u{2555}'),
            (0xb9, '\u{2563}'),
            (0xba, '\u{2551}'),
            (0xbb, '\u{2557}'),
            (0xbc, '\u{255d}'),
            (0xbd, '\u{255c}'),
            (0xbe, '\u{255b}'),
            (0xbf, '\u{2510}'),
            (0xc0, '\u{2514}'),
            (0xc1, '\u{2534}'),
            (0xc2, '\u{252c}'),
            (0xc3, '\u{251c}'),
            (0xc4, '\u{2500}'),
            (0xc5, '\u{253c}'),
            (0xc6, '\u{255e}'),
            (0xc7, '\u{255f}'),
            (0xc8, '\u{255a}'),
            (0xc9, '\u{2554}'),
            (0xca, '\u{2569}'),
            (0xcb, '\u{2566}'),
            (0xcc, '\u{2560}'),
            (0xcd, '\u{2550}'),
            (0xce, '\u{256c}'),
            (0xcf, '\u{2567}'),
            (0xd0, '\u{2568}'),
            (0xd1, '\u{2564}'),
            (0xd2, '\u{2565}'),
            (0xd3, '\u{2559}'),
            (0xd4, '\u{2558}'),
            (0xd5, '\u{2552}'),
            (0xd6, '\u{2553}'),
            (0xd7, '\u{256b}'),
            (0xd8, '\u{256a}'),
            (0xd9, '\u{2518}'),
            (0xda, '\u{250c}'),
            (0xdb, '\u{2588}'),
            (0xdc, '\u{2584}'),
            (0xdd, '\u{258c}'),
            (0xde, '\u{2590}'),
            (0xdf, '\u{2580}'),
            (0xe0, '\u{03b1}'),
            (0xe1, '\u{00df}'),
            (0xe2, '\u{0393}'),
            (0xe3, '\u{03c0}'),
            (0xe4, '\u{03a3}'),
            (0xe5, '\u{03c3}'),
            (0xe6, '\u{00b5}'),
            (0xe7, '\u{03c4}'),
            (0xe8, '\u{03a6}'),
            (0xe9, '\u{0398}'),
            (0xea, '\u{03a9}'),
            (0xeb, '\u{03b4}'),
            (0xec, '\u{221e}'),
            (0xed, '\u{03c6}'),
            (0xee, '\u{03b5}'),
            (0xef, '\u{2229}'),
            (0xf0, '\u{2261}'),
            (0xf1, '\u{00b1}'),
            (0xf2, '\u{2265}'),
            (0xf3, '\u{2264}'),
            (0xf4, '\u{2320}'),
            (0xf5, '\u{2321}'),
            (0xf6, '\u{00f7}'),
            (0xf7, '\u{2248}'),
            (0xf8, '\u{00b0}'),
            (0xf9, '\u{2219}'),
            (0xfa, '\u{00b7}'),
            (0xfb, '\u{221a}'),
            (0xfc, '\u{207f}'),
            (0xfd, '\u{00b2}'),
            (0xfe, '\u{25a0}'),
            (0xff, '\u{00a0}'),
        ]),
        "macCyrillic" => Some(&[
            (0x80, '\u{0410}'),
            (0x81, '\u{0411}'),
            (0x82, '\u{0412}'),
            (0x83, '\u{0413}'),
            (0x84, '\u{0414}'),
            (0x85, '\u{0415}'),
            (0x86, '\u{0416}'),
            (0x87, '\u{0417}'),
            (0x88, '\u{0418}'),
            (0x89, '\u{0419}'),
            (0x8a, '\u{041a}'),
            (0x8b, '\u{041b}'),
            (0x8c, '\u{041c}'),
            (0x8d, '\u{041d}'),
            (0x8e, '\u{041e}'),
            (0x8f, '\u{041f}'),
            (0x90, '\u{0420}'),
            (0x91, '\u{0421}'),
            (0x92, '\u{0422}'),
            (0x93, '\u{0423}'),
            (0x94, '\u{0424}'),
            (0x95, '\u{0425}'),
            (0x96, '\u{0426}'),
            (0x97, '\u{0427}'),
            (0x98, '\u{0428}'),
            (0x99, '\u{0429}'),
            (0x9a, '\u{042a}'),
            (0x9b, '\u{042b}'),
            (0x9c, '\u{042c}'),
            (0x9d, '\u{042d}'),
            (0x9e, '\u{042e}'),
            (0x9f, '\u{042f}'),
            (0xa0, '\u{2020}'),
            (0xa1, '\u{00b0}'),
            (0xa2, '\u{0490}'),
            (0xa4, '\u{00a7}'),
            (0xa5, '\u{2022}'),
            (0xa6, '\u{00b6}'),
            (0xa7, '\u{0406}'),
            (0xa8, '\u{00ae}'),
            (0xaa, '\u{2122}'),
            (0xab, '\u{0402}'),
            (0xac, '\u{0452}'),
            (0xad, '\u{2260}'),
            (0xae, '\u{0403}'),
            (0xaf, '\u{0453}'),
            (0xb0, '\u{221e}'),
            (0xb2, '\u{2264}'),
            (0xb3, '\u{2265}'),
            (0xb4, '\u{0456}'),
            (0xb6, '\u{0491}'),
            (0xb7, '\u{0408}'),
            (0xb8, '\u{0404}'),
            (0xb9, '\u{0454}'),
            (0xba, '\u{0407}'),
            (0xbb, '\u{0457}'),
            (0xbc, '\u{0409}'),
            (0xbd, '\u{0459}'),
            (0xbe, '\u{040a}'),
            (0xbf, '\u{045a}'),
            (0xc0, '\u{0458}'),
            (0xc1, '\u{0405}'),
            (0xc2, '\u{00ac}'),
            (0xc3, '\u{221a}'),
            (0xc4, '\u{0192}'),
            (0xc5, '\u{2248}'),
            (0xc6, '\u{2206}'),
            (0xc7, '\u{00ab}'),
            (0xc8, '\u{00bb}'),
            (0xc9, '\u{2026}'),
            (0xca, '\u{00a0}'),
            (0xcb, '\u{040b}'),
            (0xcc, '\u{045b}'),
            (0xcd, '\u{040c}'),
            (0xce, '\u{045c}'),
            (0xcf, '\u{0455}'),
            (0xd0, '\u{2013}'),
            (0xd1, '\u{2014}'),
            (0xd2, '\u{201c}'),
            (0xd3, '\u{201d}'),
            (0xd4, '\u{2018}'),
            (0xd5, '\u{2019}'),
            (0xd6, '\u{00f7}'),
            (0xd7, '\u{201e}'),
            (0xd8, '\u{040e}'),
            (0xd9, '\u{045e}'),
            (0xda, '\u{040f}'),
            (0xdb, '\u{045f}'),
            (0xdc, '\u{2116}'),
            (0xdd, '\u{0401}'),
            (0xde, '\u{0451}'),
            (0xdf, '\u{044f}'),
            (0xe0, '\u{0430}'),
            (0xe1, '\u{0431}'),
            (0xe2, '\u{0432}'),
            (0xe3, '\u{0433}'),
            (0xe4, '\u{0434}'),
            (0xe5, '\u{0435}'),
            (0xe6, '\u{0436}'),
            (0xe7, '\u{0437}'),
            (0xe8, '\u{0438}'),
            (0xe9, '\u{0439}'),
            (0xea, '\u{043a}'),
            (0xeb, '\u{043b}'),
            (0xec, '\u{043c}'),
            (0xed, '\u{043d}'),
            (0xee, '\u{043e}'),
            (0xef, '\u{043f}'),
            (0xf0, '\u{0440}'),
            (0xf1, '\u{0441}'),
            (0xf2, '\u{0442}'),
            (0xf3, '\u{0443}'),
            (0xf4, '\u{0444}'),
            (0xf5, '\u{0445}'),
            (0xf6, '\u{0446}'),
            (0xf7, '\u{0447}'),
            (0xf8, '\u{0448}'),
            (0xf9, '\u{0449}'),
            (0xfa, '\u{044a}'),
            (0xfb, '\u{044b}'),
            (0xfc, '\u{044c}'),
            (0xfd, '\u{044d}'),
            (0xfe, '\u{044e}'),
            (0xff, '\u{00a4}'),
        ]),
        "ISO-8859-9" => Some(&[
            (0xd0, '\u{011e}'),
            (0xdd, '\u{0130}'),
            (0xde, '\u{015e}'),
            (0xf0, '\u{011f}'),
            (0xfd, '\u{0131}'),
            (0xfe, '\u{015f}'),
        ]),
        _ => None,
    }
}

/// The text a run of bytes spells in a single-byte encoding.
pub(crate) fn latin_text(bytes: &[u8], named: &str) -> Option<String> {
    let exceptions = latin_exceptions(named)?;
    Some(
        bytes
            .iter()
            .map(|byte| {
                exceptions
                    .iter()
                    .find(|(at, _)| at == byte)
                    .map(|(_, spelled)| *spelled)
                    .unwrap_or(*byte as char)
            })
            .collect(),
    )
}

/// The bytes text spells in a single-byte encoding, or the first character
/// the encoding has no byte for.
pub(crate) fn latin_bytes(text: &str, named: &str) -> Option<Result<Vec<u8>, char>> {
    let exceptions = latin_exceptions(named)?;
    let mut bytes = Vec::with_capacity(text.len());
    for character in text.chars() {
        if let Some((at, _)) = exceptions.iter().find(|(_, spelled)| *spelled == character) {
            bytes.push(*at);
            continue;
        }
        let point = character as u32;
        // A byte an exception has taken over no longer stands for the Latin-1
        // character of the same number.
        if point > 0xff || exceptions.iter().any(|(at, _)| *at as u32 == point) {
            return Some(Err(character));
        }
        bytes.push(point as u8);
    }
    Some(Ok(bytes))
}

/// The escape a character is written as where the encoding an answer is
/// written in has no room for it.
pub(crate) fn escaped_point(character: char) -> String {
    let point = character as u32;
    if point > 0xffff {
        format!("\\u{{{point:X}}}")
    } else {
        format!("\\u{point:04X}")
    }
}

/// Text with every character an ASCII answer has no room for written as an
/// escape. Ruby writes a string this way where it would otherwise have to
/// refuse text in an encoding the answer is not being written in.
pub(crate) fn escaped_text(string_value: &crate::object::StringValue) -> Object {
    // An encoding that spells a character in more than one byte is carried as
    // the bytes themselves, so the characters are read back out of them.
    let held = match wide_encoding(&string_value.encoding_name()) {
        Some(shape) => wide_text(&binary_bytes(string_value), shape),
        None => string_value.to_text(),
    };
    let mut out = String::with_capacity(held.len());
    for character in held.chars() {
        match character {
            character if character.is_ascii() && !character.is_control() => out.push(character),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if character.is_control() => {
                out.push_str(&format!("\\x{:02X}", character as u32))
            }
            character => out.push_str(&escaped_point(character)),
        }
    }
    Object::String(Rc::new(crate::object::StringValue::with_encoding(
        out, "US-ASCII",
    )))
}

/// The letter Ruby writes a character as after a backslash, for the few that
/// have a name of their own.
fn named_escape(character: char) -> Option<char> {
    Some(match character {
        '\u{7}' => 'a',
        '\u{8}' => 'b',
        '\t' => 't',
        '\n' => 'n',
        '\u{b}' => 'v',
        '\u{c}' => 'f',
        '\r' => 'r',
        '\u{1b}' => 'e',
        _ => return None,
    })
}

/// The name Ruby knows an encoding by, whatever spelling or case it was named
/// with. A name nothing in the table matches is left as it was given.
pub(crate) fn canonical_encoding_name(named: &str) -> String {
    let wanted = named.replace('-', "_").to_ascii_uppercase();
    let same = |held: &str| held.replace('-', "_").to_ascii_uppercase() == wanted;
    crate::vm::init::ENCODING_NAMES
        .iter()
        .find(|(spelled, canonical, _)| same(spelled) || same(canonical))
        .map(|(_, canonical, _)| canonical.to_string())
        .unwrap_or_else(|| named.to_string())
}

/// Whether an encoding name is one of the Shift_JIS family.
pub(crate) fn spells_shift_jis(named: &str) -> bool {
    matches!(named, "Shift_JIS" | "Windows-31J" | "MacJapanese")
}

/// Whether a run of bytes spells characters in Shift_JIS. A byte under 0x80
/// stands for itself, 0xA1 through 0xDF is a half-width katakana of its own,
/// and 0x81 through 0x9F or 0xE0 through 0xEF opens a pair.
pub(crate) fn shift_jis_reads(bytes: &[u8]) -> bool {
    let mut at = 0;
    while at < bytes.len() {
        let held = bytes[at];
        let width = match held {
            0x00..=0x7f | 0xa1..=0xdf => 1,
            0x81..=0x9f | 0xe0..=0xef => 2,
            _ => return false,
        };
        if width == 2 {
            match bytes.get(at + 1) {
                Some(0x40..=0x7e | 0x80..=0xfc) => {}
                _ => return false,
            }
        }
        at += width;
    }
    true
}

/// Whether a run of bytes spells characters in an encoding. Used where bytes
/// arrive from outside the program already, such as a name read off the file
/// system, rather than as the text a String holds.
pub(crate) fn encoding_reads_bytes(bytes: &[u8], named: &str) -> bool {
    match named {
        "ASCII-8BIT" | "BINARY" => true,
        "US-ASCII" => bytes.is_ascii(),
        "UTF-8" | "CESU-8" => std::str::from_utf8(bytes).is_ok(),
        "EUC-JP" => euc_jp_reads(bytes),
        "Shift_JIS" | "Windows-31J" | "MacJapanese" => shift_jis_reads(bytes),
        "UTF-16" | "UTF-16BE" | "UTF-16LE" => bytes.len().is_multiple_of(2),
        "UTF-32" | "UTF-32BE" | "UTF-32LE" => bytes.len().is_multiple_of(4),
        _ => true,
    }
}

// The C library's one-way hash, which `String#crypt` answers with.
unsafe extern "C" {
    fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char;
}

impl VirtualMachine {
    /// Where a needle sits in a string, counted in bytes. `byterindex` reads
    /// from the offset backwards, and `byteindex` from the offset forwards.
    fn byte_index_of(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(1, 2),
                arguments.len(),
                position,
            ));
        }
        // `index` and `rindex` count in characters, `byteindex` and
        // `byterindex` in bytes.
        let in_bytes = method_name.starts_with("byte");
        let text = string_value.as_str().to_string();
        let bytes = if in_bytes {
            binary_bytes(string_value)
        } else {
            text.as_bytes().to_vec()
        };
        // Where each position starts, in the bytes searched, with one more
        // for the end of the string.
        let places: Vec<usize> = if in_bytes {
            (0..=bytes.len()).collect()
        } else {
            text.char_indices()
                .map(|(at, _)| at)
                .chain(std::iter::once(text.len()))
                .collect()
        };
        let total = places.len() as i64 - 1;
        let from_end = method_name.ends_with("rindex");
        let searching_pattern = pattern_of(&arguments[0]).is_some();
        let refuse_offset = |vm: &mut Self| {
            if searching_pattern {
                vm.globals_mut().set(
                    crate::vm::native_methods::regexp_methods::LAST_MATCH,
                    Object::Nil,
                );
            }
            Object::Nil
        };
        let offset = match arguments.get(1) {
            None => {
                if from_end {
                    total
                } else {
                    0
                }
            }
            Some(held) => {
                let asked = self.integer_argument(method_name, held, position)?;
                let placed = if asked < 0 { asked + total } else { asked };
                if placed < 0 {
                    return Ok(refuse_offset(self));
                }
                if from_end {
                    placed.min(total)
                } else {
                    if placed > total {
                        return Ok(refuse_offset(self));
                    }
                    placed
                }
            }
        };
        if in_bytes {
            crate::vm::native_methods::string_mutation::check_character_boundary(
                &bytes,
                offset as usize,
                position,
            )?;
        }
        let opening = places[offset as usize];
        if let Some((pattern, flags)) = pattern_of(&arguments[0]) {
            self.refuse_mixed_pattern_encoding(string_value, &arguments[0], position)?;
            let found =
                self.byte_index_by_pattern(&pattern, &flags, &text, opening, from_end, position)?;
            return Ok(match found {
                Object::Int(at) => Object::Int(position_of(&places, at as usize, in_bytes, &text)),
                other => other,
            });
        }
        let needle = self.string_needle(method_name, &arguments[0], position)?;
        if self
            .compatible_encoding(receiver, &Object::String(Rc::clone(&needle)))
            .is_none()
        {
            let message = format!(
                "incompatible character encodings: {} and {}",
                string_value.encoding_name(),
                needle.encoding_name()
            );
            return Err(crate::vm::errors::simple_exception(
                "Encoding::CompatibilityError",
                &message,
                position,
            ));
        }
        let wanted = if in_bytes {
            binary_bytes(&needle)
        } else {
            needle.as_str().as_bytes().to_vec()
        };
        Ok(match byte_run_index(&bytes, &wanted, opening, from_end) {
            Some(at) => Object::Int(position_of(&places, at, in_bytes, &text)),
            None => Object::Nil,
        })
    }

    /// The needle a byte search was given: a String, or what `to_str` makes
    /// of the argument. Nothing else names one.
    fn string_needle(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<Rc<crate::object::StringValue>, MetorexError> {
        if let Object::String(text) = argument {
            return Ok(Rc::clone(text));
        }
        if let Some(Object::String(text)) =
            crate::vm::native_methods::string_subclass_value(argument)
        {
            return Ok(text);
        }
        let _ = method_name;
        let named = value_name(self, argument);
        let refuse = |vm: &Self| {
            let _ = vm;
            let message = format!("no implicit conversion of {} into String", named);
            crate::vm::errors::simple_exception("TypeError", &message, position)
        };
        if !self.answers_to(argument, "to_str", position)? {
            return Err(refuse(self));
        }
        match self.send_to_object(argument.clone(), "to_str", vec![], position)? {
            Object::String(text) => Ok(text),
            _ => Err(refuse(self)),
        }
    }

    /// A pattern written in an encoding the subject cannot be read alongside
    /// matches nothing at all, which Ruby reports rather than answering nil.
    fn refuse_mixed_pattern_encoding(
        &mut self,
        string_value: &Rc<crate::object::StringValue>,
        pattern: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let subject = Object::String(Rc::clone(string_value));
        if self.compatible_encoding(&subject, pattern).is_some() {
            return Ok(());
        }
        let named = match self.send_to_object(pattern.clone(), "encoding", vec![], position)? {
            Object::Class(held) | Object::Module(held) => held.name().to_string(),
            other => other.to_string(),
        };
        let message = format!(
            "incompatible encoding regexp match ({} regexp with {} string)",
            named,
            string_value.encoding_name()
        );
        Err(crate::vm::errors::simple_exception(
            "Encoding::CompatibilityError",
            &message,
            position,
        ))
    }

    /// The byte offset a pattern matches at, reading from `offset` forwards or
    /// backwards. `\G` in the pattern stands for that offset itself.
    fn byte_index_by_pattern(
        &mut self,
        pattern: &str,
        flags: &str,
        text: &str,
        offset: usize,
        from_end: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let found = match crate::vm::native_methods::regexp_methods::previous_match_split(pattern) {
            Some((before, after)) => {
                self.anchored_match_start(&before, &after, flags, text, offset, from_end)
            }
            None => plain_match_start(pattern, flags, text, offset, from_end),
        };
        match found {
            Some(at) => {
                self.regexp_match_data(pattern, flags, text, at, position)?;
                Ok(Object::Int(at as i64))
            }
            None => {
                self.globals_mut().set(
                    crate::vm::native_methods::regexp_methods::LAST_MATCH,
                    Object::Nil,
                );
                Ok(Object::Nil)
            }
        }
    }

    /// Where a pattern carrying `\G` matches: the part before it has to end
    /// at the offset, and the part after it has to start there.
    fn anchored_match_start(
        &mut self,
        before: &str,
        after: &str,
        flags: &str,
        text: &str,
        offset: usize,
        from_end: bool,
    ) -> Option<usize> {
        let tail = crate::vm::native_methods::regexp_methods::compile(after, flags)?;
        let held = tail.find_at(text, offset)?;
        if held.start() != offset {
            return None;
        }
        let head = crate::vm::native_methods::regexp_methods::compile(
            &format!(r"(?:{})\z", before),
            flags,
        )?;
        let opening = &text[..offset];
        let mut starts: Vec<usize> = Vec::new();
        for at in 0..=offset {
            if !opening.is_char_boundary(at) {
                continue;
            }
            if head
                .find_at(opening, at)
                .is_some_and(|found| found.start() == at)
            {
                starts.push(at);
            }
        }
        if from_end {
            starts.pop()
        } else {
            starts.first().copied()
        }
    }
}

/// The pattern and flags a value stands for, reading a Regexp subclass
/// through the pattern it keeps.
fn pattern_of(value: &Object) -> Option<(String, String)> {
    match value {
        Object::Regex(pattern, flags) => Some((pattern.to_string(), flags.to_string())),
        other => match crate::vm::native_methods::regexp_subclass_value(other) {
            Some(Object::Regex(pattern, flags)) => Some((pattern.to_string(), flags.to_string())),
            _ => None,
        },
    }
}

/// Where a pattern with no `\G` matches, reading from the offset forwards or
/// backwards.
fn plain_match_start(
    pattern: &str,
    flags: &str,
    text: &str,
    offset: usize,
    from_end: bool,
) -> Option<usize> {
    let compiled = crate::vm::native_methods::regexp_methods::compile(pattern, flags)?;
    if !from_end {
        return compiled.find_at(text, offset).map(|found| found.start());
    }
    let mut at = offset;
    loop {
        if text.is_char_boundary(at)
            && compiled
                .find_at(text, at)
                .is_some_and(|found| found.start() == at)
        {
            return Some(at);
        }
        if at == 0 {
            return None;
        }
        at -= 1;
    }
}

/// Where one run of bytes sits inside another, reading from the offset
/// forwards or backwards.
fn byte_run_index(bytes: &[u8], wanted: &[u8], offset: usize, from_end: bool) -> Option<usize> {
    let last = bytes.len().checked_sub(wanted.len())?;
    if from_end {
        let mut at = offset.min(last);
        loop {
            if bytes[at..at + wanted.len()] == *wanted {
                return Some(at);
            }
            if at == 0 {
                return None;
            }
            at -= 1;
        }
    }
    (offset..=last).find(|at| bytes[*at..*at + wanted.len()] == *wanted)
}

/// The name Ruby gives a value when it reports a conversion it could not
/// make: `nil`, `true`, and `false` name themselves, and everything else
/// names its class.
fn value_name(vm: &VirtualMachine, held: &Object) -> String {
    match held {
        Object::Nil => "nil".to_string(),
        Object::Bool(true) => "true".to_string(),
        Object::Bool(false) => "false".to_string(),
        other => vm.builtins().class_of(other).name().to_string(),
    }
}

/// The position a byte offset stands at: the offset itself when positions are
/// counted in bytes, and the number of characters before it otherwise.
fn position_of(places: &[usize], at: usize, in_bytes: bool, text: &str) -> i64 {
    if in_bytes {
        return at as i64;
    }
    match places.binary_search(&at) {
        Ok(found) => found as i64,
        Err(_) => text[..at].chars().count() as i64,
    }
}

impl VirtualMachine {
    /// `sub` and `gsub`: the string with the first match, or every match,
    /// replaced by what the argument or the block answers.
    fn substitute(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let held_block = match self.pending_block.take() {
            Some(Object::Block(block)) => Some(block),
            other => {
                self.pending_block = other;
                None
            }
        };
        // A replacement written alongside a block is the one that counts, so
        // the block is left unused.
        let block = if arguments.len() > 1 {
            None
        } else {
            held_block
        };
        if arguments.len() == 1 && block.is_none() {
            // A pattern on its own, with nothing to put in place of what it
            // matches, answers an Enumerator over every match. One match is
            // not enough to walk, so `sub` asks for a replacement instead.
            if !method_name.starts_with("gsub") {
                return Err(crate::vm::errors::argument_count_error(
                    crate::vm::errors::Arity::Exact(2),
                    arguments.len(),
                    position,
                ));
            }
            return self.make_enumerator(receiver, method_name, arguments, position);
        }
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Range(1, 2),
                arguments.len(),
                position,
            ));
        }
        let global = method_name.starts_with("gsub");
        let (pattern, flags) = match pattern_of(&arguments[0]) {
            Some(held) => held,
            None => {
                let needle = self.string_needle(method_name, &arguments[0], position)?;
                (crate::regexp::escape(&needle.as_str()), String::new())
            }
        };
        // `\G` asks the match to begin where the one before it ended.
        let (anchored, source) = match super::regexp_methods::previous_match_split(&pattern) {
            Some((before, after)) if before.is_empty() => (true, after),
            _ => (false, pattern.clone()),
        };
        let Some(compiled) = super::regexp_methods::compile(&source, &flags) else {
            return Err(crate::vm::errors::simple_exception(
                "RegexpError",
                &format!("invalid pattern: /{}/", source),
                position,
            ));
        };
        let subject = string_value.as_str().to_string();
        let mut answer = AnswerText {
            built: String::new(),
            encoding: string_value.encoding_name(),
            holds_only_ascii: true,
        };
        let mut cursor = 0usize;
        let mut last: Option<usize> = None;
        while cursor <= subject.len() {
            let Some(captured) = compiled.captures_at(&subject, cursor) else {
                break;
            };
            let Some(whole) = captured.get(0) else { break };
            if anchored && whole.start() != cursor {
                break;
            }
            if super::regexp_methods::line_start_past_the_end(
                &compiled,
                &subject,
                whole.start(),
                whole.end(),
            ) {
                break;
            }
            answer.add(
                &subject[cursor..whole.start()],
                &string_value.encoding_name(),
                position,
            )?;
            last = Some(whole.start());
            self.regexp_match_data_in(&source, &flags, &subject, whole.start(), None, position)?;
            let (piece, piece_encoding) = self.replacement_text(
                receiver,
                arguments.get(1),
                block.as_ref(),
                &subject,
                &captured,
                position,
            )?;
            answer.add(&piece, &piece_encoding, position)?;
            if whole.end() > whole.start() {
                cursor = whole.end();
            } else {
                // An empty match moves on by one character, carrying that
                // character into the answer.
                if whole.end() >= subject.len() {
                    cursor = whole.end();
                    break;
                }
                let mut next = whole.end() + 1;
                while next < subject.len() && !subject.is_char_boundary(next) {
                    next += 1;
                }
                answer.add(
                    &subject[whole.end()..next],
                    &string_value.encoding_name(),
                    position,
                )?;
                cursor = next;
            }
            if !global {
                break;
            }
        }
        if last.is_none() {
            self.globals_mut()
                .set(super::regexp_methods::LAST_MATCH, Object::Nil);
            return Ok(Object::String(Rc::new(
                crate::object::StringValue::with_encoding(subject, string_value.encoding_name()),
            )));
        }
        let tail = subject[cursor.min(subject.len())..].to_string();
        answer.add(&tail, &string_value.encoding_name(), position)?;
        // The block may have matched patterns of its own, so the last match
        // this call made is put back as the one `$~` names.
        if let Some(start) = last {
            self.regexp_match_data_in(&source, &flags, &subject, start, None, position)?;
        }
        let made = crate::object::StringValue::with_encoding(answer.built, &answer.encoding);
        if string_value.holds_bytes() || answer.encoding == "ASCII-8BIT" {
            made.mark_bytes();
        }
        Ok(Object::String(Rc::new(made)))
    }

    /// `each_line` and `lines`: the pieces a separator cuts a string into,
    /// handed to a block or collected into an Array. A separator of `nil`
    /// leaves the string whole, an empty one cuts it into paragraphs, and
    /// `chomp:` takes the separator back off each piece.
    fn walk_lines(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Reading the separator may run a `to_str` of the program's own, and
        // a call made there would take the block this walk was given.
        let held_block = self.pending_block.take();
        let options = self.line_walk_options(method_name, arguments, position);
        self.pending_block = held_block;
        let (given, chomp) = options?;
        let named = string_value.encoding_name();
        // An encoding that spells ASCII over several bytes has no newline to
        // look for, so the string is one line whatever the separator says.
        let whole_only = named.starts_with("UTF-16") || named.starts_with("UTF-32");
        if !whole_only && dummy_encoding(&named) {
            return Err(crate::vm::errors::simple_exception(
                "Encoding::ConverterNotFoundError",
                &format!("code converter not found ({} to UTF-8)", named),
                position,
            ));
        }
        let separator = match given {
            None => None,
            Some(_) if whole_only => None,
            Some(text) => Some(text),
        };
        let held = string_value.to_text();
        let text = held.as_str();
        let pieces: Vec<String> = match separator.as_deref() {
            None => {
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![text.to_string()]
                }
            }
            Some("") => paragraphs_of(text),
            Some(separator) => {
                let mut collected = Vec::new();
                let mut rest = text;
                while let Some(cut) = rest.find(separator) {
                    let end = cut + separator.len();
                    collected.push(rest[..end].to_string());
                    rest = &rest[end..];
                }
                if !rest.is_empty() {
                    collected.push(rest.to_string());
                }
                collected
            }
        };
        let pieces: Vec<Object> = pieces
            .into_iter()
            .map(|piece| {
                let piece = match chomp {
                    false => piece,
                    true => chomped_line(piece, separator.as_deref()),
                };
                // Each line is written the way the whole string was, which a
                // block reading them has to see.
                let line = crate::object::StringValue::with_encoding(piece, &named);
                if string_value.holds_bytes() {
                    line.mark_bytes();
                }
                Object::String(Rc::new(line))
            })
            .collect();
        if method_name == "lines" {
            self.warn_unused_block(position)?;
            return Ok(Some(Object::array(pieces)));
        }
        let Some(Object::Block(block)) = self.pending_block.take() else {
            return self
                .make_enumerator(receiver, method_name, arguments, position)
                .map(Some);
        };
        for piece in pieces {
            self.execute_block_callable(&block, vec![piece], position)?;
        }
        Ok(Some(receiver.clone()))
    }

    /// The separator a line walk cuts on, None where the string stays whole,
    /// alongside whether `chomp:` asked for the separator to come back off.
    /// Written without a separator, the walk takes the one `$/` names.
    fn line_walk_options(
        &mut self,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<(Option<String>, bool), MetorexError> {
        let mut positional = arguments;
        let mut chomp = false;
        if let Some(last) = arguments.last()
            && let Object::Dict(pairs) = last
            && pairs.borrow().contains_key("__MX_KWARGS__")
        {
            chomp = pairs
                .borrow()
                .get(":chomp")
                .is_some_and(|value| value.is_truthy());
            positional = &arguments[..arguments.len() - 1];
        }
        if positional.len() > 1 {
            return Err(method_argument_error(
                method_name,
                1,
                positional.len(),
                position,
            ));
        }
        let separator = match positional.first() {
            None => match self.globals().get("/") {
                Some(Object::String(text)) => Some(text.as_str().to_string()),
                Some(Object::Nil) => None,
                _ => Some("\n".to_string()),
            },
            Some(Object::Nil) => None,
            Some(other) => Some(
                self.string_needle(method_name, other, position)?
                    .as_str()
                    .to_string(),
            ),
        };
        Ok((separator, chomp))
    }

    /// What one match is replaced by: what the block answers, what the Hash
    /// holds under the matched text, or the replacement String with its
    /// backslash sequences filled in.
    fn replacement_text(
        &mut self,
        subject_string: &Object,
        replacement: Option<&Object>,
        block: Option<&Rc<crate::object::BlockStatement>>,
        subject: &str,
        captured: &crate::regexp::Captures<'_, '_>,
        position: Position,
    ) -> Result<(String, String), MetorexError> {
        let whole = captured.get(0).expect("a match was found");
        if let Some(block) = block {
            let _ = subject_string;
            let answered = self.execute_block_callable(
                block,
                vec![Object::string(whole.as_str().to_string())],
                position,
            )?;
            return self.text_of(answered, position);
        }
        let held = replacement.expect("a replacement or a block was given");
        if let Some(Object::Dict(_)) = crate::vm::native_methods::as_dict(held) {
            let key = Object::string(whole.as_str().to_string());
            let found = self.send_to_object(held.clone(), "[]", vec![key], position)?;
            if matches!(found, Object::Nil) {
                return Ok((
                    String::new(),
                    crate::object::string_value::DEFAULT_ENCODING.to_string(),
                ));
            }
            return self.text_of(found, position);
        }
        let written = self.string_needle("sub", held, position)?;
        let filled = fill_replacement(&written.as_str(), subject, captured);
        Ok((filled, written.encoding_name()))
    }

    /// The text a value stands for and the encoding it is written in, taking
    /// `to_s` from anything that is not already a String.
    fn text_of(
        &mut self,
        held: Object,
        position: Position,
    ) -> Result<(String, String), MetorexError> {
        match held {
            Object::String(text) => Ok((text.as_str().to_string(), text.encoding_name())),
            other => match self.send_to_object(other, "to_s", vec![], position)? {
                Object::String(text) => Ok((text.as_str().to_string(), text.encoding_name())),
                shown => Ok((
                    shown.to_string(),
                    crate::object::string_value::DEFAULT_ENCODING.to_string(),
                )),
            },
        }
    }
}

/// An answer built one piece at a time, each piece written in an encoding of
/// its own. An answer holding only ASCII so far takes on the encoding of the
/// first piece that holds more, and a later piece that cannot be read in that
/// encoding is a CompatibilityError.
pub(crate) struct AnswerText {
    built: String,
    encoding: String,
    holds_only_ascii: bool,
}

impl AnswerText {
    /// An answer with nothing in it yet, written in the encoding the text it
    /// is built from was written in.
    pub(crate) fn new(encoding: &str) -> Self {
        Self {
            built: String::new(),
            encoding: encoding.to_string(),
            holds_only_ascii: true,
        }
    }

    /// What has been built so far.
    pub(crate) fn text(self) -> String {
        self.built
    }

    /// The encoding the pieces settled on.
    pub(crate) fn encoding(&self) -> String {
        self.encoding.clone()
    }

    pub(crate) fn add(
        &mut self,
        piece: &str,
        encoding: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if piece.is_empty() {
            return Ok(());
        }
        // Text in an encoding that spells ASCII over several bytes never
        // joins text in another, however little either of them holds.
        if self.encoding != encoding
            && (!crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(encoding)
                || !crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(
                    &self.encoding,
                ))
        {
            return Err(crate::vm::errors::simple_exception(
                "Encoding::CompatibilityError",
                &format!(
                    "incompatible character encodings: {} and {}",
                    self.encoding, encoding
                ),
                position,
            ));
        }
        if !piece.is_ascii() {
            if self.holds_only_ascii {
                self.encoding = encoding.to_string();
                self.holds_only_ascii = false;
            } else if self.encoding != encoding {
                return Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &format!(
                        "incompatible character encodings: {} and {}",
                        self.encoding, encoding
                    ),
                    position,
                ));
            }
        } else if self.holds_only_ascii
            && !crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(
                &self.encoding,
            )
        {
            self.encoding = encoding.to_string();
        }
        self.built.push_str(piece);
        Ok(())
    }
}

/// The paragraphs a string holds: each one runs up to a break of two or more
/// newlines, keeps two of them, and the rest of the break is dropped.
fn paragraphs_of(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut collected = Vec::new();
    let mut start = 0;
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'\n' {
            at += 1;
            continue;
        }
        let mut run = at;
        while run < bytes.len() && bytes[run] == b'\n' {
            run += 1;
        }
        if run - at < 2 {
            at = run;
            continue;
        }
        collected.push(text[start..at + 2].to_string());
        start = run;
        at = run;
    }
    if start < text.len() {
        collected.push(text[start..].to_string());
    }
    collected
}

/// One line with its separator taken back off. Written without a separator,
/// the walk cuts on newlines and takes a carriage return off with them.
fn chomped_line(piece: String, separator: Option<&str>) -> String {
    match separator {
        Some("\n") | None => piece
            .strip_suffix('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .unwrap_or(&piece)
            .to_string(),
        Some(separator) => piece.strip_suffix(separator).unwrap_or(&piece).to_string(),
    }
}

/// A replacement string with its backslash sequences filled in from the
/// match: the numbered groups, the whole match, what sits either side of it,
/// and the last group that took part.
fn fill_replacement(
    written: &str,
    subject: &str,
    captured: &crate::regexp::Captures<'_, '_>,
) -> String {
    let letters: Vec<char> = written.chars().collect();
    let whole = captured.get(0).expect("a match was found");
    let mut built = String::new();
    let mut at = 0;
    while at < letters.len() {
        if letters[at] != '\\' || at + 1 >= letters.len() {
            built.push(letters[at]);
            at += 1;
            continue;
        }
        let marker = letters[at + 1];
        at += 2;
        match marker {
            '0'..='9' => {
                let index = marker as usize - '0' as usize;
                if let Some(part) = captured.get(index) {
                    built.push_str(part.as_str());
                }
            }
            '&' => built.push_str(whole.as_str()),
            '`' => built.push_str(&subject[..whole.start()]),
            '\'' => built.push_str(&subject[whole.end()..]),
            '+' => {
                // The last group that took part in the match, which is not
                // always the last one written.
                let found = (1..captured.len())
                    .rev()
                    .find_map(|index| captured.get(index));
                if let Some(part) = found {
                    built.push_str(part.as_str());
                }
            }
            '\\' => built.push('\\'),
            'k' if letters.get(at) == Some(&'<') => {
                let Some(closing) = letters[at + 1..].iter().position(|held| *held == '>') else {
                    built.push('\\');
                    built.push('k');
                    continue;
                };
                let name: String = letters[at + 1..at + 1 + closing].iter().collect();
                if let Some(part) = captured.name(&name) {
                    built.push_str(part.as_str());
                }
                at += closing + 2;
            }
            other => {
                built.push('\\');
                built.push(other);
            }
        }
    }
    built
}

/// The bytes a `chomp` keeps. Without a separator the string is left as it
/// is, an empty one takes off every line ending at the end, and the line
/// ending itself takes off one of any of the three shapes it has.
fn chomped_bytes(bytes: &[u8], named: Option<&str>, encoding: &str) -> Vec<u8> {
    let Some(named) = named else {
        return bytes.to_vec();
    };
    let spelled = |text: &str| bytes_in_encoding(text, encoding);
    if named.is_empty() {
        let (carriage, line) = (spelled("\r\n"), spelled("\n"));
        let mut kept = bytes;
        loop {
            if let Some(shorter) = kept.strip_suffix(carriage.as_slice()) {
                kept = shorter;
                continue;
            }
            if let Some(shorter) = kept.strip_suffix(line.as_slice()) {
                kept = shorter;
                continue;
            }
            return kept.to_vec();
        }
    }
    if named == "\n" {
        for ending in ["\r\n", "\n", "\r"] {
            if let Some(kept) = bytes.strip_suffix(spelled(ending).as_slice()) {
                return kept.to_vec();
            }
        }
        return bytes.to_vec();
    }
    match bytes.strip_suffix(spelled(named).as_slice()) {
        Some(kept) => kept.to_vec(),
        None => bytes.to_vec(),
    }
}

/// The bytes an encoding spells a run of text with, or None where metorex
/// carries no table for it.
pub(crate) fn spelled_bytes(text: &str, named: &str) -> Option<Vec<u8>> {
    if let Some(shape) = wide_encoding(named) {
        return Some(wide_bytes(text, shape));
    }
    if named == "EUC-JP" {
        return super::euc_jp_table::euc_jp_bytes(text).ok();
    }
    if named == "ISO-2022-JP" {
        return super::euc_jp_table::iso_2022_jp_bytes(text).ok();
    }
    if spells_shift_jis(named) {
        return super::shift_jis_table::shift_jis_bytes(text).ok();
    }
    match latin_bytes(text, named) {
        Some(Ok(spelled)) => Some(spelled),
        _ => None,
    }
}

/// The bytes an encoding spells a run of text with, falling back on the ones
/// the text is held as where the encoding has no table of its own.
pub(crate) fn bytes_in_encoding(text: &str, named: &str) -> Vec<u8> {
    spelled_bytes(text, named).unwrap_or_else(|| text.as_bytes().to_vec())
}

/// What a `split` cuts on.
enum Separator {
    /// Runs of whitespace, with none kept at either end.
    Whitespace,
    /// Between every character.
    Characters,
    /// A run of text, matched as it stands.
    Literal(String),
    /// A pattern, whose groups are kept alongside the pieces.
    Pattern(crate::regexp::Pattern),
    /// The place a run starts, cut in front of rather than taken out.
    Before(crate::regexp::Pattern),
}

/// The run a pattern that is nothing but a look-ahead names.
fn looks_ahead(pattern: &str, flags: &str) -> Option<crate::regexp::Pattern> {
    let inside = pattern.strip_prefix("(?=")?.strip_suffix(')')?;
    if inside.contains(')') && !inside.contains('(') {
        return None;
    }
    super::regexp_methods::compile(inside, flags)
}

/// The separator a written one stands for. A single space asks for the
/// whitespace reading, and an empty one cuts between characters.
fn separator_of(written: &str) -> Separator {
    if written == " " {
        return Separator::Whitespace;
    }
    if written.is_empty() {
        return Separator::Characters;
    }
    Separator::Literal(written.to_string())
}

/// The pieces a split answers. A count of one keeps the whole text, a
/// positive count leaves the rest in the last piece, zero drops the empty
/// pieces at the end, and a negative one keeps them.
fn split_pieces(text: &str, separator: &Separator, limit: i64) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if limit == 1 {
        return vec![text.to_string()];
    }
    let mut pieces = match separator {
        Separator::Whitespace => split_on_whitespace(text, limit),
        Separator::Characters => split_between_characters(text, limit),
        Separator::Literal(held) => split_on_literal(text, held, limit),
        Separator::Pattern(held) => split_on_pattern(text, held, limit),
        Separator::Before(held) => split_before(text, held, limit),
    };
    if limit == 0 {
        while pieces.last().is_some_and(|piece| piece.is_empty()) {
            pieces.pop();
        }
    }
    pieces
}

/// Words, with the whitespace between them dropped along with any at either
/// end. A count leaves the rest of the text, spaces and all, in the last
/// piece.
fn split_on_whitespace(text: &str, limit: i64) -> Vec<String> {
    let mut pieces = Vec::new();
    let bytes = text.as_bytes();
    let mut skipping = true;
    let mut from = 0usize;
    let mut ends = 0usize;
    let mut fields = 1i64;
    let mut at = 0usize;
    while at < bytes.len() {
        let width = next_character(text, at).min(text.len()) - at;
        let space = spells_space(bytes[at]);
        if skipping {
            if space {
                from = at + width;
            } else {
                ends = at + width;
                skipping = false;
                if limit > 0 && limit <= fields {
                    break;
                }
            }
        } else if space {
            pieces.push(text[from..ends].to_string());
            skipping = true;
            from = at + width;
            fields += 1;
        } else {
            ends = at + width;
        }
        at += width;
    }
    if !text.is_empty() && (limit != 0 || text.len() > from) {
        pieces.push(text[from..].to_string());
    }
    pieces
}

/// Whether a byte is one of the six characters Ruby counts as whitespace.
fn spells_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// One piece for each character.
fn split_between_characters(text: &str, limit: i64) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    for (at, held) in text.char_indices() {
        if limit > 0 && pieces.len() as i64 == limit - 1 {
            pieces.push(text[at..].to_string());
            return pieces;
        }
        pieces.push(held.to_string());
    }
    if limit != 0 {
        pieces.push(String::new());
    }
    pieces
}

/// Pieces cut on a run of text matched as it stands.
fn split_on_literal(text: &str, held: &str, limit: i64) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut from = 0usize;
    let mut at = 0usize;
    while let Some(found) = text[at..].find(held) {
        let cut = at + found;
        pieces.push(text[from..cut].to_string());
        from = cut + held.len();
        at = from;
        if limit > 0 && pieces.len() as i64 >= limit - 1 {
            break;
        }
    }
    pieces.push(text[from..].to_string());
    pieces
}

/// Pieces cut on a pattern, with the text each of its groups matched kept
/// alongside them. A pattern that matches nothing at all cuts between
/// characters, which is what Ruby does for an empty match.
fn split_on_pattern(text: &str, held: &crate::regexp::Pattern, limit: i64) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    let mut fields = 1i64;
    let mut from = 0usize;
    let mut at = 0usize;
    let mut after_empty = false;
    while at <= text.len() {
        let Some(found) = held.captures_at(text, at) else {
            break;
        };
        let whole = found.get(0).expect("a match holds its whole run");
        let (starts, ends) = (whole.start(), whole.end());
        if at == starts && starts == ends {
            if !after_empty {
                // An empty match at the reading point steps on by one
                // character before it counts as a cut.
                at = next_character(text, at);
                after_empty = true;
                continue;
            }
            let ends_at = next_character(text, from);
            pieces.push(text[from..ends_at].to_string());
            from = at;
        } else {
            pieces.push(text[from..starts].to_string());
            from = ends;
            at = ends;
        }
        after_empty = false;
        for group in 1..found.len() {
            if let Some(taken) = found.get(group) {
                pieces.push(taken.as_str().to_string());
            }
        }
        fields += 1;
        if limit > 0 && limit <= fields {
            break;
        }
    }
    if text.len() > from || limit != 0 {
        pieces.push(text[from..].to_string());
    }
    pieces
}

/// Pieces cut in front of every place a run starts, which is what a pattern
/// written as a look-ahead asks for.
fn split_before(text: &str, held: &crate::regexp::Pattern, limit: i64) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut from = 0usize;
    let mut at = 0usize;
    let mut fields = 1i64;
    while at <= text.len() {
        let Some(found) = held.find_at(text, at) else {
            break;
        };
        let starts = found.start();
        if starts > from {
            pieces.push(text[from..starts].to_string());
            from = starts;
            fields += 1;
            if limit > 0 && limit <= fields {
                break;
            }
        }
        at = next_character(text, starts);
    }
    if text.len() > from || limit != 0 {
        pieces.push(text[from..].to_string());
    }
    pieces
}

/// Where the character at `at` ends.
fn next_character(text: &str, at: usize) -> usize {
    if at >= text.len() {
        return text.len() + 1;
    }
    text[at..]
        .chars()
        .next()
        .map(|held| at + held.len_utf8())
        .unwrap_or(text.len())
}
