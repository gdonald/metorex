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
use crate::vm::native_methods::as_range;

/// The methods that change the string they are called on.
pub(crate) const MUTATING_STRING_METHODS: &[&str] = &[
    "<<",
    "concat",
    "replace",
    "__native_replace__",
    "prepend",
    "insert",
    "clear",
    "setbyte",
    "bytesplice",
    "append_as_bytes",
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
    "delete_prefix!",
    "delete_suffix!",
    "upcase!",
    "downcase!",
    "capitalize!",
    "swapcase!",
    "reverse!",
    "succ!",
    "next!",
    "[]=",
    "__borrow__",
    "__release__",
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
        // Saying that something else is reading the string is not itself a
        // change, so it is answered before the checks below.
        match method_name {
            "__borrow__" | "__release__" => {
                target.set_borrowed(method_name == "__borrow__");
                return Ok(Some(receiver.clone()));
            }
            _ => {}
        }
        // A string whose bytes its encoding already reads has nothing for
        // `scrub!` to put aside, so a frozen one is left as it is.
        if method_name == "scrub!"
            && target.is_frozen()
            && super::string_methods::holds_valid_text(target)
        {
            return Ok(Some(receiver.clone()));
        }
        if target.is_frozen() {
            return Err(self.frozen_modification_error(receiver, position));
        }
        // A string handed back with notice that it will be frozen in a later
        // release says so the first time it is changed.
        if let Some(notice) = target.take_chill()
            && self.warning_category_enabled("deprecated")
        {
            self.emit_warning_to_stderr(&notice, position);
            self.report_string_birthplace(target, position);
        }
        if target.is_borrowed() {
            let message = "can't modify string; temporarily locked".to_string();
            return Err(crate::vm::errors::simple_exception(
                "RuntimeError",
                &message,
                position,
            ));
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
                    match argument {
                        // Text written in an encoding this one cannot be read
                        // alongside is refused, and text that goes with it
                        // carries its own reading into the result.
                        Object::String(other) => {
                            // Text that stands for bytes carries that over,
                            // so a byte the encoding cannot read stays the
                            // byte it was rather than being written out.
                            if other.holds_bytes() {
                                target.mark_bytes();
                            }
                            match appended_encoding(target, other) {
                                Some(Some(named)) => {
                                    target.set_encoding(named);
                                }
                                Some(None) => {}
                                None => {
                                    return Err(
                                        crate::vm::native_methods::string_methods::clashing_encodings_error(
                                            target, other, position,
                                        ),
                                    );
                                }
                            }
                            added.push_str(&other.to_text());
                        }
                        // A number names a codepoint, which has to be one the
                        // string's own encoding can spell.
                        Object::Int(_) | Object::BigInt(_) => {
                            let (letter, binary) = self.appended_codepoint(
                                &target.encoding_name(),
                                argument,
                                position,
                            )?;
                            if binary {
                                target.set_encoding("ASCII-8BIT");
                                target.mark_bytes();
                            }
                            added.push(letter);
                        }
                        other => added.push_str(&self.one_string_value("<<", other, position)?),
                    }
                }
                target.append_text(&added);
                Ok(Some(receiver.clone()))
            }
            "replace" | "__native_replace__" => {
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
                let index = &self.integer_argument(method_name, &arguments[0], position)?;
                // Text written in an encoding this one cannot be read
                // alongside is refused, and text that goes with it carries
                // its own reading into the result.
                if let Object::String(other) = &arguments[1] {
                    if crate::vm::native_methods::string_methods::encodings_clash(target, other) {
                        return Err(
                            crate::vm::native_methods::string_methods::clashing_encodings_error(
                                target, other, position,
                            ),
                        );
                    }
                    if target.as_str().is_ascii() && !other.as_str().is_ascii() {
                        target.set_encoding(other.encoding_name());
                        if other.holds_bytes() {
                            target.mark_bytes();
                        }
                    }
                }
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
            // `append_as_bytes` puts the bytes on the end without reading
            // them in any encoding, so the string it is called on keeps the
            // encoding it had however broken the result is.
            "append_as_bytes" => {
                let mut bytes = super::string_methods::binary_bytes(target);
                for given in arguments {
                    match given {
                        Object::String(text) => {
                            bytes.extend(super::string_methods::binary_bytes(text));
                        }
                        Object::Int(number) => bytes.push(number.rem_euclid(256) as u8),
                        Object::BigInt(number) => {
                            let wrapped = ((number.as_ref() % 256u32) + 256u32) % 256u32;
                            let digits = wrapped.to_u32_digits().1;
                            bytes.push(digits.first().copied().unwrap_or(0) as u8);
                        }
                        other => {
                            let message = format!(
                                "wrong argument type {} (expected String or Integer)",
                                self.builtins().class_of(other).name()
                            );
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                &message,
                                position,
                            ));
                        }
                    }
                }
                target.replace_text(super::string_methods::bytes_as_text(&bytes));
                target.mark_bytes();
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
                let index = self.integer_argument(method_name, &arguments[0], position)?;
                let value = self.integer_argument(method_name, &arguments[1], position)?;
                let (index, value) = (&index, &value);
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
                    // The bytes written may not spell a letter in the string's
                    // encoding any more, so what it holds stands for bytes.
                    target.mark_bytes();
                }
                Ok(Some(Object::Int(*value)))
            }
            "bytesplice" => self.splice_string_bytes(receiver, arguments, position),
            "slice!" => self.cut_from_string(receiver, arguments, position),
            "[]=" => self
                .assign_string_part(receiver, target, arguments, position)
                .map(Some),
            // These answer the string itself rather than nil, and the tag
            // the answer carries comes back with the text.
            "encode!" | "scrub!" | "unicode_normalize!" => {
                let plain = method_name.trim_end_matches('!');
                let answered = self.call_string_method(receiver, plain, arguments, position)?;
                if let Some(Object::String(made)) = answered {
                    // A string already reading the way it was asked to is
                    // left alone, so a frozen one that needs no change is
                    // not refused.
                    if made.to_text() == target.to_text()
                        && made.encoding_name() == target.encoding_name()
                    {
                        return Ok(Some(receiver.clone()));
                    }
                    target.set_encoding(made.encoding_name());
                    if made.holds_bytes() {
                        target.mark_bytes();
                    }
                    target.replace_text(made.to_text());
                }
                Ok(Some(receiver.clone()))
            }
            // A substitution answers nil when it matched nothing, and the
            // encoding the answer settled on comes back with the text.
            "sub!" | "gsub!" => {
                let plain = method_name.trim_end_matches('!');
                let before = target.to_text();
                // The string is held still while the block decides what to
                // put in place of each match.
                target.set_borrowed(true);
                let outcome = self.call_string_method(receiver, plain, arguments, position);
                target.set_borrowed(false);
                let answered = outcome?;
                let Some(Object::String(made)) = answered else {
                    // A pattern on its own answers an Enumerator over the
                    // matches rather than a string.
                    return Ok(Some(answered.unwrap_or(Object::Nil)));
                };
                let after = made.to_text();
                if after == before && made.encoding_name() == target.encoding_name() {
                    return Ok(Some(Object::Nil));
                }
                target.set_encoding(made.encoding_name());
                if made.holds_bytes() {
                    target.mark_bytes();
                }
                target.replace_text(after);
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
    /// The letter a codepoint names, alongside whether the string it is
    /// appended to now stands for bytes. A codepoint the string's encoding
    /// cannot spell is refused.
    fn appended_codepoint(
        &mut self,
        encoding: &str,
        argument: &Object,
        position: Position,
    ) -> Result<(char, bool), MetorexError> {
        let refuse = |code: &str| {
            crate::vm::errors::simple_exception(
                "RangeError",
                &format!("{} out of char range", code),
                position,
            )
        };
        let Object::Int(code) = argument else {
            return Err(refuse(&argument.to_string()));
        };
        let code = *code;
        if code < 0 {
            return Err(refuse(&code.to_string()));
        }
        let plain = matches!(encoding, "US-ASCII" | "ASCII" | "ANSI_X3.4-1968");
        // One of the high bytes written into an ASCII string makes it a run
        // of bytes rather than text, which is what Ruby answers.
        if plain && (128..=255).contains(&code) {
            return Ok((char::from(code as u8), true));
        }
        if plain && code > 127 {
            return Err(refuse(&code.to_string()));
        }
        // An encoding whose letters are not Unicode spells only what a single
        // byte stands for, so a codepoint past that is refused.
        let narrow = !matches!(
            encoding,
            "UTF-8" | "UTF-16LE" | "UTF-16BE" | "UTF-32LE" | "UTF-32BE"
        ) && !plain;
        if narrow
            && code > 127
            && !matches!(encoding, "ASCII-8BIT" | "BINARY" | "ASCII-8BIT (BINARY)")
        {
            return Err(refuse(&code.to_string()));
        }
        let Some(letter) = u32::try_from(code).ok().and_then(char::from_u32) else {
            return Err(refuse(&code.to_string()));
        };
        Ok((letter, false))
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
            Some(Object::Range { start, .. }) => {
                // An end written as something that answers `to_int` names a
                // place the same way a number does.
                let index = match start.as_ref() {
                    Object::Int(index) => Some(*index),
                    held if self.answers_to(held, "to_int", position)? => {
                        match self.send_to_object(held.clone(), "to_int", vec![], position)? {
                            Object::Int(index) => Some(index),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                match index {
                    Some(index) => {
                        let at = if index < 0 {
                            index + letters.len() as i64
                        } else {
                            index
                        };
                        usize::try_from(at).unwrap_or(0)
                    }
                    None => 0,
                }
            }
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

    /// `bytesplice` replaces a byte range of the receiver with bytes taken
    /// from another string, counting positions in bytes rather than in
    /// characters.
    fn splice_string_bytes(
        &mut self,
        receiver: &Object,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::String(target) = receiver else {
            return Ok(None);
        };
        // The three shapes Ruby accepts: a range and a string, a range with
        // a range into that string, an index and a length with a string, and
        // the same pair for the string itself.
        let (span, replacement, source_span) = match arguments.len() {
            2 => (SpliceSpan::Whole(arguments[0].clone()), &arguments[1], None),
            3 if as_range(&arguments[0]).is_some() => (
                SpliceSpan::Whole(arguments[0].clone()),
                &arguments[1],
                Some(SpliceSpan::Whole(arguments[2].clone())),
            ),
            3 => (
                SpliceSpan::Counted(arguments[0].clone(), arguments[1].clone()),
                &arguments[2],
                None,
            ),
            5 => (
                SpliceSpan::Counted(arguments[0].clone(), arguments[1].clone()),
                &arguments[2],
                Some(SpliceSpan::Counted(
                    arguments[3].clone(),
                    arguments[4].clone(),
                )),
            ),
            given => {
                let message = format!(
                    "wrong number of arguments (given {}, expected 2, 3, or 5)",
                    given
                );
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    &message,
                    position,
                ));
            }
        };
        let Object::String(source) = replacement else {
            let message = format!(
                "no implicit conversion of {} into String",
                self.builtins().class_of(replacement).name()
            );
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &message,
                position,
            ));
        };
        let target_bytes = super::string_methods::binary_bytes(target);
        let source_bytes = super::string_methods::binary_bytes(source);
        let (at, width) = self.splice_bounds(&span, target_bytes.len(), position)?;
        check_character_boundary(&target_bytes, at, position)?;
        check_character_boundary(&target_bytes, at + width, position)?;
        let (from, taken) = match &source_span {
            Some(span) => {
                let (from, taken) = self.splice_bounds(span, source_bytes.len(), position)?;
                check_character_boundary(&source_bytes, from, position)?;
                check_character_boundary(&source_bytes, from + taken, position)?;
                (from, taken)
            }
            None => (0, source_bytes.len()),
        };
        let mut spliced = target_bytes[..at].to_vec();
        spliced.extend_from_slice(&source_bytes[from..from + taken]);
        spliced.extend_from_slice(&target_bytes[at + width..]);
        if let Some(Some(named)) = appended_encoding(target, source) {
            target.set_encoding(named);
        }
        match String::from_utf8(spliced.clone()) {
            Ok(text) if !target.holds_bytes() && target.encoding_name() == "UTF-8" => {
                target.replace_text(text)
            }
            _ => {
                target.replace_text(super::string_methods::bytes_as_text(&spliced));
                target.mark_bytes();
            }
        }
        Ok(Some(receiver.clone()))
    }

    /// The byte offset and byte width a splice argument names, with the
    /// out-of-range readings Ruby raises on.
    fn splice_bounds(
        &mut self,
        span: &SpliceSpan,
        total: usize,
        position: Position,
    ) -> Result<(usize, usize), MetorexError> {
        let total = total as i64;
        match span {
            SpliceSpan::Counted(index, length) => {
                let asked = self.integer_argument("bytesplice", index, position)?;
                let wanted = self.integer_argument("bytesplice", length, position)?;
                let at = if asked < 0 { asked + total } else { asked };
                if at < 0 || at > total {
                    let message = format!("index {} out of string", asked);
                    return Err(crate::vm::errors::simple_exception(
                        "IndexError",
                        &message,
                        position,
                    ));
                }
                if wanted < 0 {
                    let message = format!("negative length {}", wanted);
                    return Err(crate::vm::errors::simple_exception(
                        "IndexError",
                        &message,
                        position,
                    ));
                }
                Ok((at as usize, wanted.min(total - at) as usize))
            }
            SpliceSpan::Whole(value) => {
                let Some(Object::Range {
                    start,
                    end,
                    exclusive,
                    ..
                }) = as_range(value)
                else {
                    let message = format!(
                        "wrong argument type {} (expected Range)",
                        self.builtins().class_of(value).name()
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &message,
                        position,
                    ));
                };
                let opening = match self.span_end_index(start.as_ref(), position)? {
                    Some(number) if number < 0 => number + total,
                    Some(number) => number,
                    None => 0,
                };
                if opening < 0 || opening > total {
                    let message = format!("{} out of range", value);
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        &message,
                        position,
                    ));
                }
                let closing = match self.span_end_index(end.as_ref(), position)? {
                    Some(number) => {
                        let placed = if number < 0 { number + total } else { number };
                        if exclusive { placed } else { placed + 1 }
                    }
                    None => total,
                };
                let width = (closing - opening).clamp(0, total - opening);
                Ok((opening as usize, width as usize))
            }
        }
    }
}

/// The encoding an append answers with: `Some(None)` to keep the one the
/// string already has, `Some(Some(name))` to take the other's, and `None`
/// when the two cannot be read alongside each other.
fn appended_encoding(
    target: &crate::object::StringValue,
    other: &crate::object::StringValue,
) -> Option<Option<String>> {
    if other.as_str().is_empty() || target.encoding_name() == other.encoding_name() {
        return Some(None);
    }
    if target.as_str().is_empty() {
        return Some(Some(other.encoding_name()));
    }
    let reads_alongside_ascii = |named: &str| {
        !matches!(
            named,
            "UTF-16LE" | "UTF-16BE" | "UTF-16" | "UTF-32LE" | "UTF-32BE" | "UTF-32"
        )
    };
    if !reads_alongside_ascii(&target.encoding_name())
        || !reads_alongside_ascii(&other.encoding_name())
    {
        return None;
    }
    if other.as_str().is_ascii() {
        return Some(None);
    }
    if target.as_str().is_ascii() {
        return Some(Some(other.encoding_name()));
    }
    None
}

/// Which shape a splice position came in as: a whole Range, or an index
/// paired with a byte count.
enum SpliceSpan {
    Whole(Object),
    Counted(Object, Object),
}

/// A splice offset has to fall between characters, which for a multi-byte
/// encoding is not every byte position.
pub(crate) fn check_character_boundary(
    bytes: &[u8],
    offset: usize,
    position: Position,
) -> Result<(), MetorexError> {
    if offset >= bytes.len() || bytes[offset] & 0xC0 != 0x80 {
        return Ok(());
    }
    let message = format!("offset {} does not land on character boundary", offset);
    Err(crate::vm::errors::simple_exception(
        "IndexError",
        &message,
        position,
    ))
}
