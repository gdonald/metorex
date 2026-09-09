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
            "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(string_value.chars().count() as i64)))
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
                let mut out = String::with_capacity(string_value.len() + 2);
                out.push('"');
                let mut characters = string_value.chars().peekable();
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
                Ok(Some(Object::binary_string(string_value.as_str())))
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
                // Ruby's String#inspect output for the common cases.
                let mut out = String::with_capacity(string_value.len() + 2);
                out.push('"');
                for c in string_value.chars() {
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
                        let length = subject.chars().count() as i64;
                        let resolved = if *offset < 0 {
                            offset + length
                        } else {
                            *offset
                        };
                        if resolved < 0 || resolved > length {
                            return Ok(Some(Object::Bool(false)));
                        }
                        subject
                            .char_indices()
                            .nth(resolved as usize)
                            .map(|(index, _)| index)
                            .unwrap_or(subject.len())
                    }
                    _ => 0,
                };
                let matched = super::compile(&pattern, &flags)
                    .map(|compiled| compiled.find_at(subject, start).is_some())
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
            "encode" => {
                let copy = Object::string(string_value.as_str());
                if string_value.as_str().is_ascii()
                    && let Some(named) = arguments.first()
                    && let Ok(wanted) = self.encoding_name_argument(named, position)
                    && let Object::String(copied) = &copy
                {
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
                let letters: Vec<char> = string_value.chars().collect();
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
                    Object::String(needle) => character_index(&letters, needle, start, from_end),
                    Object::Regex(pattern, flags) => {
                        let Some(compiled) = super::compile(pattern, flags) else {
                            return Ok(Some(Object::Nil));
                        };
                        let places: Vec<usize> = compiled
                            .find_iter(string_value)
                            .map(|found| string_value[..found.start()].chars().count())
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
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(string_value.to_uppercase())))
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
                let next = successor_of(string_value.as_ref().as_str());
                Ok(Some(Object::string(next)))
            }
            "downcase" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(string_value.to_lowercase())))
            }
            // `capitalize` raises the first letter and lowers the rest, and
            // `swapcase` turns each letter the other way.
            "capitalize" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut letters = string_value.chars();
                let capitalized = match letters.next() {
                    None => String::new(),
                    Some(first) => first
                        .to_uppercase()
                        .chain(letters.flat_map(|letter| letter.to_lowercase()))
                        .collect(),
                };
                Ok(Some(Object::string(capitalized)))
            }
            "swapcase" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let swapped: String = string_value
                    .chars()
                    .flat_map(|letter| {
                        if letter.is_uppercase() {
                            letter.to_lowercase().collect::<Vec<_>>()
                        } else if letter.is_lowercase() {
                            letter.to_uppercase().collect::<Vec<_>>()
                        } else {
                            vec![letter]
                        }
                    })
                    .collect();
                Ok(Some(Object::string(swapped)))
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
                match &arguments[0] {
                    Object::String(rhs) => {
                        let mut combined = string_value.as_str().to_string();
                        combined.push_str(rhs);
                        Ok(Some(Object::string(combined)))
                    }
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
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
                Ok(Some(Object::string(string_value.trim().to_string())))
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
                let text = string_value.as_ref().as_str();
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
            "lstrip" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(string_value.trim_start().to_string())))
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
                let reversed: String = string_value.chars().rev().collect();
                Ok(Some(Object::string(reversed)))
            }
            "last" => {
                let chars: Vec<char> = string_value.chars().collect();
                if chars.is_empty() {
                    Ok(Some(Object::Nil))
                } else {
                    Ok(Some(Object::string(chars.last().unwrap().to_string())))
                }
            }
            // String#ord — the codepoint of the first character. An empty
            // String has none, which Ruby reports as an ArgumentError.
            "ord" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                match string_value.chars().next() {
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
                Ok(Some(Object::Int(string_value.len() as i64)))
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
                let bytes: Vec<Object> = string_value
                    .bytes()
                    .map(|byte| Object::Int(byte as i64))
                    .collect();
                if method_name == "bytes" {
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
                for byte in bytes {
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
                let length = string_value.len() as i64;
                let resolved = if *index < 0 { index + length } else { *index };
                if resolved < 0 || resolved >= length {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(Object::Int(
                    string_value.as_bytes()[resolved as usize] as i64,
                )))
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
                        .chars()
                        .next()
                        .map(String::from)
                        .unwrap_or_default(),
                )))
            }
            // Metorex strings are UTF-8 throughout, so every one of them is
            // valid, and whether it is ASCII is a question about its bytes.
            "valid_encoding?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(true)))
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
                Ok(Some(Object::Bool(string_value.is_ascii())))
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
                    string_value,
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
                    .chars()
                    .map(|character| Object::Int(character as i64))
                    .collect();
                if method_name == "codepoints" {
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
                    .chars()
                    .map(|c| Object::string(c.to_string()))
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(chars)))))
            }
            // `unpack` reads the bytes back as the directives describe them,
            // and `unpack1` answers the first of them.
            "unpack" | "unpack1" => {
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
                let held = string_value.as_str().to_string();
                let read = self.string_unpack(&held, &format, position)?;
                if method_name == "unpack1" {
                    return Ok(Some(read.into_iter().next().unwrap_or(Object::Nil)));
                }
                Ok(Some(Object::array(read)))
            }
            // `force_encoding` tags the string as being in another encoding
            // without touching what it holds, which is what Ruby does for a
            // string whose bytes are already right for the new one.
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
                string_value.set_encoding(named);
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
                Ok(Some(Object::Int(string_value.chars().count() as i64)))
            }
            // Stream-like predicates so STDOUT/STDERR (stored as String) can be checked
            "tty?" | "isatty" => Ok(Some(Object::Bool(false))),
            "flush" | "sync" | "sync=" | "fsync" => Ok(Some(Object::Nil)),
            // STDOUT/STDERR stream methods (receiver is the "STDOUT"/"STDERR" string).
            "puts" | "print" | "write" => {
                let to_stderr = string_value.as_str() == "STDERR";
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
                let width = match &arguments[0] {
                    Object::Int(n) => *n,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let pad = if arguments.len() == 2 {
                    match &arguments[1] {
                        Object::String(s) if !s.is_empty() => s.as_str().to_string(),
                        Object::String(_) => {
                            return Err(MetorexError::runtime_error(
                                format!("zero width padding for {}", method_name),
                                crate::vm::utils::position_to_location(position),
                            ));
                        }
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                &arguments[1],
                                position,
                            ));
                        }
                    }
                } else {
                    " ".to_string()
                };
                let current_len = string_value.chars().count() as i64;
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
                Ok(Some(Object::string(result)))
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
                Ok(Some(Object::string(string_value.trim().to_string())))
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
                let s: &str = string_value.as_ref();
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
                        .strip_suffix(sep.as_ref().as_str())
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
                        .split_whitespace()
                        .map(|piece| piece.to_string())
                        .collect()
                } else {
                    match &arguments[0] {
                        Object::String(separator) if limit > 0 => string_value
                            .splitn(limit as usize, separator.as_str())
                            .map(|piece| piece.to_string())
                            .collect(),
                        Object::String(separator) => string_value
                            .split(separator.as_str())
                            .map(|piece| piece.to_string())
                            .collect(),
                        Object::Regex(pattern, flags) => {
                            // Build the same regex used by Regex literal eval.
                            let pat = pattern.as_str();
                            let flag_str = flags.as_str();
                            let mut builder = regex::RegexBuilder::new(pat);
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
                                    .splitn(string_value.as_str(), limit as usize)
                                    .map(|piece| piece.to_string())
                                    .collect(),
                                Ok(re) => re
                                    .split(string_value.as_str())
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
                let (start, len) = match (&arguments[0], &arguments[1]) {
                    (Object::Int(s), Object::Int(l)) => (*s, *l),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Integer",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let chars: Vec<char> = string_value.chars().collect();
                let char_count = chars.len() as i64;
                let start_idx = if start < 0 {
                    (char_count + start).max(0) as usize
                } else {
                    start.min(char_count) as usize
                };
                let end_idx = (start_idx as i64 + len).min(char_count).max(0) as usize;
                if start_idx > chars.len() {
                    Ok(Some(Object::Nil))
                } else {
                    let sliced: String =
                        chars[start_idx..end_idx.min(chars.len())].iter().collect();
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
                match &arguments[0] {
                    Object::String(substr) => {
                        Ok(Some(Object::Bool(string_value.contains(substr.as_str()))))
                    }
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
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
                            if method_name == "start_with?" {
                                if string_value.starts_with(s.as_str()) {
                                    result = true;
                                    break;
                                }
                            } else if string_value.ends_with(s.as_str()) {
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
                        string_value.starts_with(prefix.as_str()),
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
                    Object::String(suffix) => {
                        Ok(Some(Object::Bool(string_value.ends_with(suffix.as_str()))))
                    }
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
                        let size = string_value.chars().count() as i64;
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
                for ch in string_value.chars() {
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
                let trimmed = string_value.trim();
                let digits: String = trimmed
                    .chars()
                    .take_while(|held| held.is_digit(base) || *held == '-' || *held == '+')
                    .collect();
                let n = i64::from_str_radix(&digits, base).unwrap_or(0);
                Ok(Some(Object::Int(n)))
            }
            // `to_r` reads the leading rational value and answers (0/1) when
            // the string does not start with one.
            "to_r" => {
                let (numerator, denominator) =
                    super::rational_methods::parse_rational_text(string_value);
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
                Ok(Some(Object::Float(leading_float(string_value.as_ref()))))
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
                Ok(Some(Object::string(string_value.as_str().to_string())))
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
                    Object::String(text) => (regex::escape(text.as_str()), String::new()),
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
                let mut builder = regex::RegexBuilder::new(&pattern);
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
                for captured in compiled.captures_iter(&subject) {
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
                        for item in found {
                            self.execute_block_body(&block, vec![item])?;
                        }
                        Ok(Some(receiver.clone()))
                    }
                    _ => Ok(Some(Object::array(found))),
                }
            }
            "gsub" | "sub" => {
                // A block form takes the pattern alone and answers each
                // replacement from what the block returns for that match.
                if arguments.len() == 1
                    && let Some(Object::Block(block)) = self.pending_block.take()
                {
                    let (pattern, flags) = match &arguments[0] {
                        Object::String(text) => (regex::escape(text.as_str()), String::new()),
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
                    let mut builder = regex::RegexBuilder::new(&pattern);
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
                            Object::String(text) => built.push_str(text),
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
                        let result = if limit == 0 {
                            string_value.replace(&pattern, &replacement)
                        } else {
                            string_value.replacen(&pattern, &replacement, limit)
                        };
                        Ok(Some(Object::string(result)))
                    }
                    // Regexp pattern: compile, honour the `i` flag, and apply
                    // either a single substitution (`sub`) or a global one
                    // (`gsub`). `\Z` / `\z` come from Ruby; the `regex` crate
                    // accepts them. Replacement string back-refs (`\1`, etc.)
                    // are preserved by `regex`'s default replace semantics.
                    Object::Regex(pattern, flags) => {
                        let re_pattern = if flags.contains('i') {
                            format!("(?i){}", pattern)
                        } else {
                            pattern.as_str().to_string()
                        };
                        let (source, flags) =
                            (pattern.as_str().to_string(), flags.as_str().to_string());
                        let subject = string_value.as_str().to_string();
                        self.regexp_match_data(&source, &flags, &subject, 0, position)?;
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => {
                                let result = if limit == 0 {
                                    re.replace_all(string_value.as_ref(), replacement.as_str())
                                        .into_owned()
                                } else {
                                    re.replacen(string_value.as_ref(), limit, replacement.as_str())
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
                Ok(Some(Object::Bool(string_value.is_empty())))
            }
            _ => Ok(None),
        }
    }
}

/// Ruby's `String#succ`: the rightmost alphanumeric character is bumped, and a
/// carry moves left, growing the string when the leftmost one wraps. A string
/// with no alphanumeric character bumps its last byte instead.
fn successor_of(text: &str) -> String {
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
                    vec![Object::string(named.as_str())],
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
