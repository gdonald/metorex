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
            other if self.responds_to(other, "to_int") => {
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
                let held_characters = string_value.to_text();
                let mut characters = held_characters.as_str().chars().peekable();
                while let Some(character) = characters.next() {
                    match character {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        '#' if matches!(characters.peek(), Some('{' | '$' | '@')) => {
                            out.push_str("\\#")
                        }
                        character if character.is_control() || !character.is_ascii() => {
                            out.push_str(&format!("\\u{{{:x}}}", character as u32))
                        }
                        character => out.push(character),
                    }
                }
                out.push('"');
                Ok(Some(Object::string(out)))
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
                let mut out = String::with_capacity(string_value.as_str().len() + 2);
                out.push('"');
                if binary || string_value.holds_bytes() {
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
                for c in string_value.as_str().chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if c.is_control() => out.push_str(&format!("\\x{:02X}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
                Ok(Some(Object::string(out)))
            }
            // `match` answers the MatchData, and records it as the last match
            // the way every other match does.
            "match" => {
                if arguments.is_empty() {
                    return Err(method_argument_error("match", 1, 0, position));
                }
                let (pattern, flags) = match &arguments[0] {
                    Object::Regex(pattern, flags) => {
                        (pattern.as_str().to_string(), flags.as_str().to_string())
                    }
                    Object::String(source) => (source.as_str().to_string(), String::new()),
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
                let Some(named) = arguments.first() else {
                    return Ok(Some(Object::string(string_value.to_text())));
                };
                let Ok(wanted) = self.encoding_name_argument(named, position) else {
                    return Ok(Some(Object::string(string_value.to_text())));
                };
                // An encoding that spells a character in more than one byte
                // is carried as the bytes themselves, since they spell
                // nothing the text model can hold.
                let held = string_value.encoding_name();
                let reading = match wide_encoding(&held) {
                    Some(shape) => wide_text(&binary_bytes(string_value), shape),
                    None => string_value.to_text(),
                };
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
            // `index` answers where a substring or pattern first appears at
            // or after the offset, counted in characters.
            "index" | "rindex" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(1, 2),
                        arguments.len(),
                        position,
                    ));
                }
                let letters: Vec<char> = string_value.as_str().chars().collect();
                let from_end = method_name == "rindex";
                let start = match arguments.get(1) {
                    Some(Object::Int(offset)) => {
                        let counted = if *offset < 0 {
                            *offset + letters.len() as i64
                        } else {
                            *offset
                        };
                        if counted < 0 {
                            return Ok(Some(Object::Nil));
                        }
                        counted as usize
                    }
                    _ => {
                        if from_end {
                            letters.len()
                        } else {
                            0
                        }
                    }
                };
                let found = match &arguments[0] {
                    Object::String(needle) => {
                        character_index(&letters, &needle.as_str(), start, from_end)
                    }
                    Object::Regex(pattern, flags) => {
                        let Some(compiled) = super::compile(pattern, flags) else {
                            return Ok(Some(Object::Nil));
                        };
                        let places: Vec<usize> = compiled
                            .find_iter(&string_value.as_str())
                            .map(|found| string_value.as_str()[..found.start()].chars().count())
                            .collect();
                        if from_end {
                            places.into_iter().rev().find(|place| *place <= start)
                        } else {
                            places.into_iter().find(|place| *place >= start)
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
                Ok(Some(match found {
                    Some(place) => Object::Int(place as i64),
                    None => Object::Nil,
                }))
            }
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
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let separator = match arguments.first() {
                    None => "\n".to_string(),
                    Some(Object::String(text)) => text.as_str().to_string(),
                    Some(Object::Nil) => String::new(),
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                let held = string_value.to_text();
                let text = held.as_str();
                let pieces: Vec<Object> = if separator.is_empty() {
                    if text.is_empty() {
                        Vec::new()
                    } else {
                        vec![Object::string(text.to_string())]
                    }
                } else {
                    let mut collected = Vec::new();
                    let mut rest = text;
                    while let Some(cut) = rest.find(&separator) {
                        let end = cut + separator.len();
                        collected.push(Object::string(rest[..end].to_string()));
                        rest = &rest[end..];
                    }
                    if !rest.is_empty() {
                        collected.push(Object::string(rest.to_string()));
                    }
                    collected
                };
                // Each line is written the way the whole string was, which
                // a block reading them has to see.
                for piece in &pieces {
                    if let Object::String(line) = piece {
                        line.set_encoding(string_value.encoding_name());
                        if string_value.holds_bytes() {
                            line.mark_bytes();
                        }
                    }
                }
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
                let chars: Vec<Object> = string_value
                    .as_str()
                    .chars()
                    .map(|c| Object::string(c.to_string()))
                    .collect();
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
                let held = if from == 0 {
                    string_value.as_str().to_string()
                } else {
                    // The offset counts bytes, so the run left is taken from
                    // the bytes rather than from the characters.
                    let bytes = binary_bytes(string_value);
                    match super::pack_format::bytes_to_string(&bytes[from..]) {
                        Object::String(rest) => rest.as_str().to_string(),
                        _ => String::new(),
                    }
                };
                let read = self.string_unpack(&held, &format, position)?;
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
            "force_encoding" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let named = self.encoding_name_argument(&arguments[0], position)?;
                // Re-tagging says how to read the bytes the string already
                // holds, so from here on they are read as bytes rather than
                // as the characters they were written as.
                let fixed_width = matches!(
                    named.as_str(),
                    "UTF-16" | "UTF-16BE" | "UTF-16LE" | "UTF-32" | "UTF-32BE" | "UTF-32LE"
                );
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
                let s: &str = &string_value.as_ref().as_str();
                let result: String = match arguments.first() {
                    None => {
                        if let Some(stripped) = s.strip_suffix("\r\n") {
                            stripped.to_string()
                        } else if let Some(stripped) = s.strip_suffix('\n') {
                            stripped.to_string()
                        } else if let Some(stripped) = s.strip_suffix('\r') {
                            stripped.to_string()
                        } else {
                            s.to_string()
                        }
                    }
                    Some(Object::Nil) => s.to_string(),
                    Some(Object::String(sep)) => s
                        .strip_suffix(&*sep.as_ref().as_str())
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| s.to_string()),
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                Ok(Some(Object::string(result)))
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
                // Ruby's limit: a positive one caps the number of fields and
                // leaves the rest in the last, zero (or none) drops trailing
                // empty fields, and a negative one keeps them.
                let limit = match arguments.get(1) {
                    None => 0i64,
                    Some(Object::Int(count)) => *count,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            other,
                            position,
                        ));
                    }
                };
                let mut parts: Vec<String> = if arguments.is_empty() {
                    string_value
                        .as_str()
                        .split_whitespace()
                        .map(|piece| piece.to_string())
                        .collect()
                } else {
                    match &arguments[0] {
                        Object::String(separator) if limit > 0 => string_value
                            .as_str()
                            .splitn(limit as usize, &*separator.as_str())
                            .map(|piece| piece.to_string())
                            .collect(),
                        Object::String(separator) => string_value
                            .as_str()
                            .split(&*separator.as_str())
                            .map(|piece| piece.to_string())
                            .collect(),
                        Object::Regex(pattern, flags) => {
                            // Build the same regex used by Regex literal eval.
                            let pat = pattern.as_str();
                            let flag_str = flags.as_str();
                            let translated = super::regexp_methods::uniquify_group_names(pat).0;
                            let mut builder = regex::RegexBuilder::new(&translated);
                            builder.multi_line(true);
                            if flag_str.contains('i') {
                                builder.case_insensitive(true);
                            }
                            if flag_str.contains('m') {
                                builder.dot_matches_new_line(true);
                            }
                            if flag_str.contains('x') {
                                builder.ignore_whitespace(true);
                            }
                            match builder.build() {
                                Ok(re) if limit > 0 => re
                                    .splitn(&string_value.as_str(), limit as usize)
                                    .map(|piece| piece.to_string())
                                    .collect(),
                                Ok(re) => re
                                    .split(&string_value.as_str())
                                    .map(|piece| piece.to_string())
                                    .collect(),
                                Err(e) => {
                                    return Err(MetorexError::runtime_error(
                                        format!("invalid regex for split: {}", e),
                                        position_to_location(position),
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String or Regexp",
                                &arguments[0],
                                position,
                            ));
                        }
                    }
                };
                if limit == 0 {
                    while parts.last().is_some_and(|piece| piece.is_empty()) {
                        parts.pop();
                    }
                }
                let parts: Vec<Object> = parts.into_iter().map(Object::string).collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(parts)))))
            }
            "slice" | "[]" => {
                // One argument is the same lookup a subscript makes, so an
                // Integer, Range, String, or Regexp all read the same way.
                if arguments.len() == 1 {
                    return self
                        .evaluate_index_operation(receiver.clone(), arguments[0].clone(), position)
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
                        let size = string_value.as_str().chars().count() as i64;
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
                for ch in string_value.as_str().chars() {
                    let char_str = Object::string(ch.to_string());
                    let args = vec![char_str];
                    self.execute_block_body(&block, args)?;
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
                // `"ff".to_i(16)` is 255.
                let base = match arguments.first() {
                    None => 10u32,
                    Some(Object::Int(held)) if (2..=36).contains(held) => *held as u32,
                    Some(Object::Int(held)) => {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &format!("invalid radix {}", held),
                            position,
                        ));
                    }
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            other,
                            position,
                        ));
                    }
                };
                let held_trimmed = string_value.to_text();
                let trimmed = held_trimmed.as_str().trim();
                let digits: String = trimmed
                    .chars()
                    .take_while(|held| held.is_digit(base) || *held == '-' || *held == '+')
                    .collect();
                // Digits that do not fit a machine word still name a number,
                // so the wider type carries them rather than answering zero.
                match i64::from_str_radix(&digits, base) {
                    Ok(held) => Ok(Some(Object::Int(held))),
                    Err(_) => match num_bigint::BigInt::parse_bytes(digits.as_bytes(), base) {
                        Some(held) => Ok(Some(Object::integer(held))),
                        None => Ok(Some(Object::Int(0))),
                    },
                }
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
                let (pattern, flags) = match &arguments[0] {
                    Object::String(text) => (regex::escape(&text.as_str()), String::new()),
                    Object::Regex(pattern, flags) => {
                        (pattern.as_str().to_string(), flags.as_str().to_string())
                    }
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Regexp",
                            other,
                            position,
                        ));
                    }
                };
                let translated = super::regexp_methods::uniquify_group_names(&pattern).0;
                let mut builder = regex::RegexBuilder::new(&translated);
                builder.multi_line(true);
                if flags.contains('i') {
                    builder.case_insensitive(true);
                }
                if flags.contains('m') {
                    builder.dot_matches_new_line(true);
                }
                if flags.contains('x') {
                    builder.ignore_whitespace(true);
                }
                let compiled = match builder.build() {
                    Ok(compiled) => compiled,
                    Err(problem) => {
                        return Err(MetorexError::runtime_error(
                            format!("invalid regex for scan: {}", problem),
                            position_to_location(position),
                        ));
                    }
                };
                let subject = string_value.as_str().to_string();
                let groups = compiled.captures_len() - 1;
                let mut found: Vec<Object> = Vec::new();
                // Where each match began, so the walk can report it as the
                // last match while the block runs.
                let mut reached: Vec<usize> = Vec::new();
                for captured in compiled.captures_iter(&subject) {
                    reached.push(captured.get(0).map(|held| held.start()).unwrap_or(0));
                    if groups == 0 {
                        let whole = captured.get(0).map(|held| held.as_str()).unwrap_or("");
                        found.push(Object::string(whole.to_string()));
                        continue;
                    }
                    let taken: Vec<Object> = (1..=groups)
                        .map(|index| match captured.get(index) {
                            Some(held) => Object::string(held.as_str().to_string()),
                            None => Object::Nil,
                        })
                        .collect();
                    found.push(Object::array(taken));
                }
                match self.pending_block.take() {
                    Some(Object::Block(block)) => {
                        for (index, item) in found.into_iter().enumerate() {
                            // `$~` and the readings taken from it name the
                            // match the block is being handed.
                            let at = reached.get(index).copied().unwrap_or(0);
                            self.regexp_match_data_in(
                                &pattern, &flags, &subject, at, None, position,
                            )?;
                            self.execute_block_body(&block, vec![item])?;
                        }
                        Ok(Some(receiver.clone()))
                    }
                    _ => Ok(Some(Object::array(found))),
                }
            }
            // A pattern is matched against characters, so a string whose
            // bytes spell nothing has nothing to match against.
            "gsub" | "sub" if !holds_valid_text(string_value) => {
                Err(broken_text_error(string_value, position))
            }
            "gsub" | "sub" => {
                // A block form takes the pattern alone and answers each
                // replacement from what the block returns for that match.
                if arguments.len() == 1
                    && let Some(Object::Block(block)) = self.pending_block.take()
                {
                    let (pattern, flags) = match &arguments[0] {
                        Object::String(text) => (regex::escape(&text.as_str()), String::new()),
                        Object::Regex(pattern, flags) => {
                            (pattern.as_str().to_string(), flags.as_str().to_string())
                        }
                        other => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String or Regexp",
                                other,
                                position,
                            ));
                        }
                    };
                    let translated = super::regexp_methods::uniquify_group_names(&pattern).0;
                    let mut builder = regex::RegexBuilder::new(&translated);
                    builder.multi_line(true);
                    if flags.contains('i') {
                        builder.case_insensitive(true);
                    }
                    if flags.contains('m') {
                        builder.dot_matches_new_line(true);
                    }
                    if flags.contains('x') {
                        builder.ignore_whitespace(true);
                    }
                    let compiled = match builder.build() {
                        Ok(compiled) => compiled,
                        Err(problem) => {
                            return Err(MetorexError::runtime_error(
                                format!("invalid regex for {}: {}", method_name, problem),
                                position_to_location(position),
                            ));
                        }
                    };
                    let subject = string_value.as_str().to_string();
                    let mut built = String::new();
                    let mut cut = 0;
                    let once = method_name == "sub";
                    for (replaced, found) in compiled.find_iter(&subject).enumerate() {
                        if once && replaced == 1 {
                            break;
                        }
                        built.push_str(&subject[cut..found.start()]);
                        let answered = self.execute_block_body(
                            &block,
                            vec![Object::string(found.as_str().to_string())],
                        )?;
                        match &answered {
                            Object::String(text) => built.push_str(&text.as_str()),
                            other => built.push_str(&other.to_string()),
                        }
                        cut = found.end();
                    }
                    built.push_str(&subject[cut..]);
                    return Ok(Some(Object::string(built)));
                }
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let replacement = match &arguments[1] {
                    Object::String(s) => s.as_str().to_string(),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            &arguments[1],
                            position,
                        ));
                    }
                };
                let limit = if method_name == "sub" { 1 } else { 0 };
                match &arguments[0] {
                    Object::String(s) => {
                        let pattern = s.as_str().to_string();
                        // A String pattern matches literally, and the match it
                        // finds is recorded the way a Regexp one is.
                        let escaped = regex::escape(&pattern);
                        let subject = string_value.as_str().to_string();
                        self.regexp_match_data(&escaped, "", &subject, 0, position)?;
                        // The whole match is all a literal pattern can name,
                        // so `\0` and `\&` stand for the pattern itself.
                        let replacement = expand_whole_match(&replacement, &pattern);
                        let result = if limit == 0 {
                            string_value.as_str().replace(&pattern, &replacement)
                        } else {
                            string_value
                                .as_str()
                                .replacen(&pattern, &replacement, limit)
                        };
                        Ok(Some(Object::string(result)))
                    }
                    // Regexp pattern: compile, honour the `i` flag, and apply
                    // either a single substitution (`sub`) or a global one
                    // (`gsub`). `\Z` / `\z` come from Ruby; the `regex` crate
                    // accepts them.
                    Object::Regex(pattern, flags) => {
                        let written =
                            super::regexp_methods::uniquify_group_names(pattern.as_str()).0;
                        let re_pattern = if flags.contains('i') {
                            format!("(?i){}", written)
                        } else {
                            written
                        };
                        let (source, flags) =
                            (pattern.as_str().to_string(), flags.as_str().to_string());
                        let subject = string_value.as_str().to_string();
                        self.regexp_match_data(&source, &flags, &subject, 0, position)?;
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => {
                                let written = replacement_for_regex(&replacement);
                                let result = if limit == 0 {
                                    re.replace_all(
                                        &string_value.as_ref().as_str(),
                                        written.as_str(),
                                    )
                                    .into_owned()
                                } else {
                                    re.replacen(
                                        &string_value.as_ref().as_str(),
                                        limit,
                                        written.as_str(),
                                    )
                                    .into_owned()
                                };
                                Ok(Some(Object::string(result)))
                            }
                            Err(_) => Ok(Some(Object::string(string_value.as_str().to_string()))),
                        }
                    }
                    other => Err(method_argument_type_error(
                        method_name,
                        "String or Regexp",
                        other,
                        position,
                    )),
                }
            }
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

/// Where a needle sits among `letters`, counted in characters. The walk runs
/// forward from `start`, or backward from it when the caller asked for the
/// last place instead of the first.
fn character_index(letters: &[char], needle: &str, start: usize, from_end: bool) -> Option<usize> {
    let wanted: Vec<char> = needle.chars().collect();
    if wanted.is_empty() {
        return Some(start.min(letters.len()));
    }
    if wanted.len() > letters.len() {
        return None;
    }
    let last = letters.len() - wanted.len();
    let places: Vec<usize> = (0..=last)
        .filter(|place| letters[*place..*place + wanted.len()] == wanted[..])
        .collect();
    if from_end {
        places.into_iter().rev().find(|place| *place <= start)
    } else {
        places.into_iter().find(|place| *place >= start)
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
                    _ => Ok(named.as_str().to_string()),
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
        made.freeze();
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
    match string_value.encoding_name().as_str() {
        "ASCII-8BIT" | "BINARY" => true,
        "US-ASCII" => string_value.as_str().is_ascii(),
        "UTF-8" if string_value.holds_bytes() => {
            String::from_utf8(super::pack_format::string_to_bytes(&string_value.as_str())).is_ok()
        }
        "EUC-JP" if string_value.holds_bytes() => {
            euc_jp_reads(&super::pack_format::string_to_bytes(&string_value.as_str()))
        }
        // A fixed-width encoding reads whole units, so a run of bytes that
        // does not divide into them spells no characters at all.
        "UTF-16" | "UTF-16BE" | "UTF-16LE" => binary_bytes(string_value).len().is_multiple_of(2),
        "UTF-32" | "UTF-32BE" | "UTF-32LE" => binary_bytes(string_value).len().is_multiple_of(4),
        _ => true,
    }
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

/// Ruby writes a back-reference in a replacement as `\1`, the whole match as
/// `\&` or `\0`, and a named group as `\k<name>`. The regex crate reads
/// `${1}` and `${name}` instead, and takes a bare `$` as the start of one.
pub(crate) fn replacement_for_regex(written: &str) -> String {
    let mut out = String::new();
    let mut letters = written.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '$' => out.push_str("$$"),
            '\\' => match letters.next() {
                Some(digit) if digit.is_ascii_digit() => {
                    out.push_str("${");
                    out.push(digit);
                    out.push('}');
                }
                Some('&') => out.push_str("${0}"),
                Some('k') if letters.peek() == Some(&'<') => {
                    letters.next();
                    let mut name = String::new();
                    for letter in letters.by_ref() {
                        if letter == '>' {
                            break;
                        }
                        name.push(letter);
                    }
                    out.push_str("${");
                    out.push_str(&name);
                    out.push('}');
                }
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            },
            other => out.push(other),
        }
    }
    out
}

/// The replacement a literal pattern takes, where the only back-reference
/// that can be named is the whole match.
fn expand_whole_match(written: &str, matched: &str) -> String {
    let mut out = String::new();
    let mut letters = written.chars().peekable();
    while let Some(letter) = letters.next() {
        if letter != '\\' {
            out.push(letter);
            continue;
        }
        match letters.next() {
            Some('&') | Some('0') => out.push_str(matched),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
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
