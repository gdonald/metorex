//! Regexp instance methods and the MatchData a match answers with.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

/// The global holding the most recent match, which `$~` and `Regexp.last_match`
/// both read.
pub(crate) const LAST_MATCH: &str = "~";

/// The suffix a repeated group name takes so the pattern compiles, since the
/// engine wants every name to be its own.
const RENAMED_SUFFIX: &str = "__mx_dup";

/// The name a group was written with, which a repeated one carries under a
/// suffix so the pattern could compile.
pub(crate) fn original_group_name(name: &str) -> &str {
    match name.find(RENAMED_SUFFIX) {
        Some(index) => &name[..index],
        None => name,
    }
}

/// Ruby does not read the position past a closing newline as the start of a
/// line, while the regex crate does. A zero-width match there that goes away
/// once that newline is written as an ordinary letter was standing on `^`
/// alone, so Ruby would not have matched at all.
pub(crate) fn line_start_past_the_end(
    compiled: &crate::regexp::Pattern,
    subject: &str,
    start: usize,
    end: usize,
) -> bool {
    if start != end || end != subject.len() || !subject.ends_with('\n') {
        return false;
    }
    let mut probe = subject[..subject.len() - 1].to_string();
    probe.push('x');
    !compiled.is_match_at(&probe, start)
}

/// The two halves of a pattern around a `\G`, which stands for where the
/// previous match ended. None when the pattern carries none.
pub(crate) fn previous_match_split(pattern: &str) -> Option<(String, String)> {
    let letters: Vec<char> = pattern.chars().collect();
    let mut at = 0;
    while at + 1 < letters.len() {
        if letters[at] == '\\' {
            if letters[at + 1] == 'G' {
                let before: String = letters[..at].iter().collect();
                let after: String = letters[at + 2..].iter().collect();
                return Some((before, after));
            }
            at += 2;
            continue;
        }
        at += 1;
    }
    None
}

/// Read a pattern, answering what it could not be read as.
pub(crate) fn read_pattern(pattern: &str, flags: &str) -> Result<crate::regexp::Pattern, String> {
    let written = crate::regexp::Flags {
        folded: flags.contains('i'),
        dot_reads_newline: flags.contains('m'),
        extended: flags.contains('x'),
        ..crate::regexp::Flags::default()
    };
    crate::regexp::Pattern::compile(pattern, written).map_err(|trouble| trouble.0)
}

/// Compile a pattern, applying the flags the literal carried.
pub(crate) fn compile(pattern: &str, flags: &str) -> Option<crate::regexp::Pattern> {
    let written = crate::regexp::Flags {
        folded: flags.contains('i'),
        dot_reads_newline: flags.contains('m'),
        extended: flags.contains('x'),
        ..crate::regexp::Flags::default()
    };
    crate::regexp::Pattern::compile(pattern, written).ok()
}

/// The characters a `=~` operand searches, which a Symbol supplies from its
/// name the way Ruby's does.
pub(crate) fn subject_text(object: &Object) -> Option<String> {
    match object {
        Object::String(text) => Some(match_text(text)),
        Object::Symbol(text) => Some(text.as_str().to_string()),
        // An instance of a String subclass carries its characters in an
        // instance variable, and matches the same as a plain String.
        other => match super::string_subclass_value(other) {
            Some(Object::String(text)) => Some(match_text(&text)),
            _ => None,
        },
    }
}

/// The text a pattern searches in a string. A string in EUC-JP or the
/// Shift_JIS family is searched through its bytes, one character to each,
/// which is how a match reads the runs its characters take.
pub(crate) fn match_text(string_value: &crate::object::StringValue) -> String {
    let named = string_value.encoding_name();
    if (named == "EUC-JP" || super::string_methods::spells_shift_jis(&named))
        && !string_value.holds_bytes()
    {
        return super::string_methods::binary_bytes(string_value)
            .iter()
            .map(|byte| *byte as char)
            .collect();
    }
    string_value.as_str().to_string()
}

/// The encoding a `=~` operand's String is tagged with, which the pieces a
/// match cuts out of it carry too.
pub(crate) fn subject_encoding(object: &Object) -> Option<String> {
    match object {
        Object::String(text) => Some(text.encoding_name()),
        other => match super::string_subclass_value(other) {
            Some(Object::String(text)) => Some(text.encoding_name()),
            _ => None,
        },
    }
}

impl VirtualMachine {
    /// Check that a pattern can match the string it is handed, the way Ruby
    /// settles the encoding a match runs in: a string its encoding cannot
    /// read is refused, a pattern and a string whose encodings do not meet
    /// are refused, and a `/.../n` pattern matched against text past ASCII
    /// in another encoding is warned about.
    pub(crate) fn prepare_match_subject(
        &mut self,
        pattern: &Rc<String>,
        flags: &str,
        subject: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let text = match subject {
            Object::String(text) => Rc::clone(text),
            other => match super::string_subclass_value(other) {
                Some(Object::String(text)) => text,
                _ => return Ok(()),
            },
        };
        let string_encoding = text.encoding_name();
        if !super::string_methods::holds_valid_text(&text) {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!("invalid byte sequence in {string_encoding}"),
                position,
            ));
        }
        let pattern_encoding = self.pattern_encoding_name(pattern, flags);
        if pattern_encoding == string_encoding {
            return Ok(());
        }
        let ascii_compatible =
            super::class_methods::encoding_reads_alongside_ascii(&string_encoding);
        let only_ascii = ascii_compatible
            && super::string_methods::binary_bytes(&text)
                .iter()
                .all(u8::is_ascii);
        if only_ascii && pattern_encoding == "US-ASCII" {
            return Ok(());
        }
        let incompatible = || {
            crate::vm::errors::simple_exception(
                "Encoding::CompatibilityError",
                &format!(
                    "incompatible encoding regexp match ({pattern_encoding} regexp with {string_encoding} string)"
                ),
                position,
            )
        };
        if !ascii_compatible {
            return Err(incompatible());
        }
        if self.pattern_fixes_encoding(pattern, flags) {
            if !super::class_methods::encoding_reads_alongside_ascii(&pattern_encoding)
                || !only_ascii
            {
                return Err(incompatible());
            }
            return Ok(());
        }
        if flags.contains('n') && string_encoding != "ASCII-8BIT" && !only_ascii {
            let message = format!(
                "{}historical binary regexp match /.../n against {string_encoding} string\n",
                self.warning_prefix(0, position)
            );
            self.warn_through_warning_module(message, position)?;
        }
        Ok(())
    }

    /// Match `pattern` against `text` from `start`, building the MatchData and
    /// recording it as the last match. Answers None when nothing matched, and
    /// clears the last match in that case the way Ruby does.
    pub(crate) fn regexp_match_data(
        &mut self,
        pattern: &str,
        flags: &str,
        text: &str,
        start: usize,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        self.regexp_match_data_in(pattern, flags, text, start, None, position)
    }

    /// The same match, told which encoding the subject was tagged with so the
    /// pieces cut out of it carry the same tag.
    pub(crate) fn regexp_match_data_in(
        &mut self,
        pattern: &str,
        flags: &str,
        text: &str,
        start: usize,
        subject_encoding: Option<String>,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Some(compiled) = compile(pattern, flags) else {
            self.globals_mut().set(LAST_MATCH, Object::Nil);
            return Ok(None);
        };
        if start > text.len() {
            self.globals_mut().set(LAST_MATCH, Object::Nil);
            return Ok(None);
        }
        // A subject whose characters are runs of bytes is searched one
        // character at a time, and the offsets a match reports count those.
        let reading = subject_encoding
            .as_deref()
            .and_then(|named| multibyte_reading(text, named));
        let (searched, start) = match &reading {
            Some((wide, unit_starts)) => {
                let raw_start = text[..start].chars().count();
                let unit = unit_starts.partition_point(|held| *held < raw_start);
                let wide_start = wide
                    .char_indices()
                    .nth(unit)
                    .map_or(wide.len(), |(at, _)| at);
                (wide.as_str(), wide_start)
            }
            None => (text, start),
        };
        // A pattern written with a time limit, or one matched while the
        // class names a limit, is given up on once that long has passed.
        let limit = self.pattern_time_limit(pattern, flags);
        let taken = compiled.captures_within(searched, start, limit);
        let found = match taken {
            Ok(Some(found)) => found,
            Ok(None) => {
                self.globals_mut().set(LAST_MATCH, Object::Nil);
                return Ok(None);
            }
            Err(_) => {
                self.globals_mut().set(LAST_MATCH, Object::Nil);
                return Err(crate::vm::errors::simple_exception(
                    "Regexp::TimeoutError",
                    "regexp match timeout",
                    position,
                ));
            }
        };
        // A pattern that names any of its groups leaves the unnamed ones
        // uncaptured, which is what Ruby does once a name appears.
        let named: Vec<(usize, String)> = compiled
            .capture_names()
            .enumerate()
            .filter_map(|(index, name)| name.map(|name| (index, name.to_string())))
            .collect();
        let kept: Vec<usize> = if named.is_empty() {
            (0..found.len()).collect()
        } else {
            std::iter::once(0)
                .chain(named.iter().map(|(index, _)| *index))
                .collect()
        };
        let mut begins = Vec::with_capacity(kept.len());
        let mut ends = Vec::with_capacity(kept.len());
        for index in &kept {
            match found.get(*index) {
                Some(group) => {
                    begins.push(Object::Int(char_offset(searched, group.start())));
                    ends.push(Object::Int(char_offset(searched, group.end())));
                }
                None => {
                    begins.push(Object::Nil);
                    ends.push(Object::Nil);
                }
            }
        }
        // A name written more than once reports the farthest group under it
        // that matched, and the first one when none did.
        let mut names = indexmap::IndexMap::new();
        for (name_index, (group, name)) in named.iter().enumerate() {
            let slot = Object::Int(name_index as i64 + 1);
            let name = original_group_name(name).to_string();
            let matched = found.get(*group).is_some();
            match names.get(&name) {
                Some(_) if !matched => {}
                _ => {
                    names.insert(name, slot);
                }
            }
        }
        let Some(match_data_class) = self.globals().get("MatchData") else {
            return Ok(None);
        };
        let subject = match &subject_encoding {
            Some(named) => {
                let held =
                    crate::object::StringValue::with_encoding(text.to_string(), named.clone());
                if reading.is_some() {
                    held.mark_bytes();
                }
                Object::String(Rc::new(held))
            }
            None => Object::string(text.to_string()),
        };
        let arguments = vec![
            subject,
            Object::Regex(Rc::new(pattern.to_string()), Rc::new(flags.to_string())),
            Object::Array(Rc::new(RefCell::new(begins))),
            Object::Array(Rc::new(RefCell::new(ends))),
            Object::Dict(Rc::new(RefCell::new(names))),
        ];
        let data = self.send_to_object(match_data_class, "new", arguments, position)?;
        self.globals_mut().set(LAST_MATCH, data.clone());
        Ok(Some(data))
    }

    /// How long a match of this pattern may take. A pattern written with a
    /// limit of its own answers that, and one written without answers what
    /// the class names.
    fn pattern_time_limit(&mut self, pattern: &str, flags: &str) -> Option<std::time::Duration> {
        let _ = flags;
        if let Some(held) = self.pattern_timeouts.get(pattern).copied().flatten() {
            return Some(held);
        }
        match self.globals().get("__regexp_timeout__") {
            Some(Object::Float(seconds)) if seconds > 0.0 => {
                Some(std::time::Duration::from_secs_f64(seconds))
            }
            Some(Object::Int(seconds)) if seconds > 0 => {
                Some(std::time::Duration::from_secs(seconds as u64))
            }
            _ => None,
        }
    }

    /// Regexp instance methods. `Object::Regex` is a primitive rather than an
    /// instance, so its methods are dispatched from here.
    pub(crate) fn call_regexp_method(
        &mut self,
        pattern: &Rc<String>,
        flags: &str,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // The limit a pattern was built with, which is nil for one built
            // without one however long the class lets a match run.
            "timeout" => Ok(Some(
                match self
                    .pattern_timeouts
                    .get(pattern.as_str())
                    .copied()
                    .flatten()
                {
                    Some(held) => Object::Float(held.as_secs_f64()),
                    None => Object::Nil,
                },
            )),
            // A pattern spelled with nothing but ASCII reads the same in
            // every ASCII-compatible encoding, which Ruby tags US-ASCII.
            "source" => Ok(Some(Object::String(Rc::new(
                crate::object::StringValue::with_encoding(
                    pattern.to_string(),
                    if names_only_ascii(pattern) {
                        "US-ASCII"
                    } else {
                        "UTF-8"
                    },
                ),
            )))),
            "options" => {
                let mut options = 0;
                if flags.contains('i') {
                    options |= 1;
                }
                if flags.contains('x') {
                    options |= 2;
                }
                if flags.contains('m') {
                    options |= 4;
                }
                // A pattern written with an encoding after it matches in that
                // encoding whatever the text is tagged with, which is what
                // `FIXEDENCODING` says. `n` matches bytes instead.
                if flags.contains('u') || flags.contains('e') || flags.contains('s') {
                    options |= 16;
                }
                if flags.contains('n') {
                    options |= 32;
                }
                Ok(Some(Object::Int(options)))
            }
            "casefold?" => Ok(Some(Object::Bool(flags.contains('i')))),
            "names" => {
                let mut named: Vec<Object> = Vec::new();
                let mut seen = Vec::new();
                for (_, name) in named_groups(pattern, flags) {
                    if seen.contains(&name) {
                        continue;
                    }
                    named.push(Object::string(name.clone()));
                    seen.push(name);
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(named)))))
            }
            // Each name answers the numbers of the groups written with it, in
            // the order they appear.
            "named_captures" => {
                let mut captures: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
                for (slot, name) in named_groups(pattern, flags) {
                    let number = Object::Int(slot as i64);
                    match captures.get(&name) {
                        Some(Object::Array(existing)) => existing.borrow_mut().push(number),
                        _ => {
                            captures
                                .insert(name, Object::Array(Rc::new(RefCell::new(vec![number]))));
                        }
                    }
                }
                Ok(Some(Object::Dict(Rc::new(RefCell::new(captures)))))
            }
            "match" => {
                if arguments.is_empty() {
                    return Err(crate::vm::errors::method_argument_error(
                        "match", 1, 0, position,
                    ));
                }
                let Some(text) = subject_text(&arguments[0]) else {
                    self.globals_mut().set(LAST_MATCH, Object::Nil);
                    if matches!(arguments[0], Object::Nil) {
                        return Ok(Some(Object::Nil));
                    }
                    return Err(type_error(
                        format!(
                            "no implicit conversion of {} into String",
                            crate::vm::errors::conversion_subject(&arguments[0])
                        ),
                        position,
                    ));
                };
                self.prepare_match_subject(pattern, flags, &arguments[0], position)?;
                let start = match arguments.get(1) {
                    Some(Object::Int(offset)) => resolve_offset(&text, *offset),
                    _ => Some(0),
                };
                let Some(start) = start else {
                    self.globals_mut().set(LAST_MATCH, Object::Nil);
                    return Ok(Some(Object::Nil));
                };
                // `match` with a block hands the MatchData to it. The block is
                // taken before the walk, since building the MatchData is
                // itself a call and would consume it.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let found = self.regexp_match_data_in(
                    pattern,
                    flags,
                    &text,
                    start,
                    subject_encoding(&arguments[0]),
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
                if arguments.is_empty() {
                    return Err(crate::vm::errors::method_argument_error(
                        "match?", 1, 0, position,
                    ));
                }
                let Some(text) = subject_text(&arguments[0]) else {
                    return Ok(Some(Object::Bool(false)));
                };
                self.prepare_match_subject(pattern, flags, &arguments[0], position)?;
                let start = match arguments.get(1) {
                    Some(Object::Int(offset)) => resolve_offset(&text, *offset),
                    _ => Some(0),
                };
                let Some(start) = start else {
                    return Ok(Some(Object::Bool(false)));
                };
                let byte_start: usize = text.chars().take(start).map(char::len_utf8).sum();
                let matched = compile(pattern, flags)
                    .map(|compiled| compiled.is_match(&text[byte_start..]))
                    .unwrap_or(false);
                Ok(Some(Object::Bool(matched)))
            }
            // A pattern written out on the left of `=~` leaves its named
            // captures behind as local variables, which is what Ruby does
            // for that form alone.
            "__match_named__" => {
                let answer = self.call_regexp_method(pattern, flags, "=~", arguments, position)?;
                let names = group_names(pattern);
                let found = self.globals().get(LAST_MATCH).unwrap_or(Object::Nil);
                for name in names {
                    let value = match &found {
                        Object::Nil => Object::Nil,
                        held => self.send_to_object(
                            held.clone(),
                            "[]",
                            vec![Object::string(name.clone())],
                            position,
                        )?,
                    };
                    self.environment_mut().define(name, value);
                }
                Ok(answer)
            }
            // `~ /pattern/` matches against the line `$_` holds, which is
            // what the line-reading command-line switches leave there.
            "~" => {
                let line = self.globals().get("_").unwrap_or(Object::Nil);
                self.call_regexp_method(pattern, flags, "=~", &[line], position)
            }
            "=~" => {
                if arguments.is_empty() {
                    return Err(crate::vm::errors::method_argument_error(
                        "=~", 1, 0, position,
                    ));
                }
                let Some(text) = subject_text(&arguments[0]) else {
                    self.globals_mut().set(LAST_MATCH, Object::Nil);
                    return Ok(Some(Object::Nil));
                };
                self.prepare_match_subject(pattern, flags, &arguments[0], position)?;
                let encoding = subject_encoding(&arguments[0]);
                match self.regexp_match_data_in(pattern, flags, &text, 0, encoding, position)? {
                    Some(data) => self
                        .send_to_object(data, "begin", vec![Object::Int(0)], position)
                        .map(Some),
                    None => Ok(Some(Object::Nil)),
                }
            }
            "to_s" => Ok(Some(Object::string(to_source_string(pattern, flags)))),
            // A pattern written back out the way it was read: the source
            // with its slashes escaped, and the flags that change how it
            // matches, in the order Ruby writes them.
            "inspect" => {
                let named = self.pattern_encoding_name(pattern, flags);
                // A pattern in an encoding other than UTF-8 holds a character
                // to each of its bytes, and a byte past ASCII is written by
                // its value, a character of several bytes by them together.
                let bytes: Option<Vec<u8>> = (!matches!(named.as_str(), "UTF-8" | "US-ASCII"))
                    .then(|| pattern.chars().map(|held| held as u32).collect::<Vec<_>>())
                    .filter(|codes| codes.iter().all(|code| *code < 0x100))
                    .map(|codes| codes.into_iter().map(|code| code as u8).collect());
                let mut written = String::new();
                let mut escaped = false;
                if let Some(bytes) = bytes {
                    let mut at = 0;
                    while at < bytes.len() {
                        let byte = bytes[at];
                        if byte.is_ascii() {
                            if byte == b'/' && !escaped {
                                written.push('\\');
                            }
                            escaped = byte == b'\\' && !escaped;
                            written.push(byte as char);
                            at += 1;
                            continue;
                        }
                        escaped = false;
                        let width = multibyte_width(&named, &bytes[at..]);
                        if width > 1 {
                            written.push_str("\\x{");
                            for held in &bytes[at..at + width] {
                                written.push_str(&format!("{held:02X}"));
                            }
                            written.push('}');
                        } else {
                            written.push_str(&format!("\\x{byte:02X}"));
                        }
                        at += width.max(1);
                    }
                } else {
                    for held in pattern.chars() {
                        if held == '/' && !escaped {
                            written.push('\\');
                        }
                        escaped = held == '\\' && !escaped;
                        written.push(held);
                    }
                }
                let mut shown = String::new();
                for flag in ['m', 'i', 'x', 'n'] {
                    if flags.contains(flag) {
                        shown.push(flag);
                    }
                }
                Ok(Some(Object::string(format!("/{}/{}", written, shown))))
            }
            "hash" => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&(pattern, comparable_flags(flags)), &mut hasher);
                Ok(Some(Object::Int(std::hash::Hasher::finish(&hasher) as i64)))
            }
            "==" | "eql?" => {
                // An instance of a subclass compares by the pattern it holds.
                let other = arguments.first().map(|held| {
                    crate::vm::native_methods::regexp_subclass_value(held)
                        .unwrap_or_else(|| held.clone())
                });
                let same = matches!(other, Some(Object::Regex(other, other_flags))
                    if *other.as_str() == ***pattern
                        && comparable_flags(&other_flags) == comparable_flags(flags));
                Ok(Some(Object::Bool(same)))
            }
            "freeze" | "itself" => Ok(Some(Object::Regex(
                Rc::new(pattern.to_string()),
                Rc::new(flags.to_string()),
            ))),
            "frozen?" => Ok(Some(Object::Bool(true))),
            _ => Ok(None),
        }
    }
}

/// How many bytes the character at the start of `bytes` takes in `named`,
/// for the Japanese encodings that spell a character in several.
fn multibyte_width(named: &str, bytes: &[u8]) -> usize {
    let decoded = if named == "EUC-JP" {
        crate::vm::native_methods::euc_jp_table::euc_jp_character(bytes)
    } else if crate::vm::native_methods::string_methods::spells_shift_jis(named) {
        crate::vm::native_methods::shift_jis_table::shift_jis_character(bytes)
    } else {
        None
    };
    decoded.map_or(1, |(_, width)| width)
}

/// The names a pattern gives its capture groups, in the order they are
/// written.
fn group_names(pattern: &str) -> Vec<String> {
    let letters: Vec<char> = pattern.chars().collect();
    let mut names = Vec::new();
    let mut at = 0;
    while at < letters.len() {
        if letters[at] == '\\' {
            at += 2;
            continue;
        }
        if letters[at] == '('
            && at + 2 < letters.len()
            && letters[at + 1] == '?'
            && letters[at + 2] == '<'
            && !matches!(letters.get(at + 3), Some('=') | Some('!'))
        {
            let mut held = String::new();
            let mut index = at + 3;
            while index < letters.len() && letters[index] != '>' {
                held.push(letters[index]);
                index += 1;
            }
            names.push(held);
            at = index + 1;
            continue;
        }
        at += 1;
    }
    names
}

/// The byte offset `index` counted in characters, which is what a Ruby offset
/// into a String means.
/// Where the supplementary private-use plane starts.
const PRIVATE_USE_PLANE: u32 = 0xf0000;

/// A subject held as bytes in EUC-JP or the Shift_JIS family, read with one
/// character standing for each run of bytes a character takes: the one the
/// run decodes to, or a private-use character when the tables name none.
/// Alongside it, where each run starts among the bytes. None for any other
/// subject, whose characters are already its own.
fn multibyte_reading(text: &str, encoding: &str) -> Option<(String, Vec<usize>)> {
    let euc_jp = encoding == "EUC-JP";
    if !(euc_jp || super::string_methods::spells_shift_jis(encoding))
        || text.chars().any(|held| u32::from(held) > 0xff)
    {
        return None;
    }
    let bytes: Vec<u8> = text.chars().map(|held| held as u8).collect();
    let groups = if euc_jp {
        super::string_methods::euc_jp_characters(&bytes)
    } else {
        super::string_methods::shift_jis_characters(&bytes)
    };
    let mut wide = String::with_capacity(bytes.len());
    let mut unit_starts = Vec::with_capacity(groups.len());
    let mut at = 0;
    for group in &groups {
        unit_starts.push(at);
        at += group.len();
        let decoded = if euc_jp {
            super::euc_jp_table::euc_jp_character(group)
        } else {
            super::shift_jis_table::shift_jis_character(group)
        };
        let character = match (group.as_slice(), decoded) {
            ([single], _) => *single as char,
            (_, Some((held, used))) if used == group.len() => held,
            // The last two bytes name the private-use character, which is
            // enough to tell two runs apart where the tables name neither.
            _ => {
                let low_bytes = group
                    .iter()
                    .fold(0_u32, |code, byte| (code << 8) | u32::from(*byte))
                    & 0xffff;
                char::from_u32(PRIVATE_USE_PLANE + low_bytes).unwrap_or(char::REPLACEMENT_CHARACTER)
            }
        };
        wide.push(character);
    }
    Some((wide, unit_starts))
}

fn char_offset(text: &str, index: usize) -> i64 {
    text[..index].chars().count() as i64
}

/// A `match` start offset, counting from the end when negative. None when it
/// falls outside the subject.
fn resolve_offset(text: &str, offset: i64) -> Option<usize> {
    let length = text.chars().count() as i64;
    let resolved = if offset < 0 { offset + length } else { offset };
    if resolved < 0 || resolved > length {
        return None;
    }
    Some(
        text.char_indices()
            .nth(resolved as usize)
            .map(|(index, _)| index)
            .unwrap_or(text.len()),
    )
}

/// Ruby renders a Regexp's `to_s` as `(?flags-offflags:source)`.
fn to_source_string(pattern: &str, flags: &str) -> String {
    // A source that is nothing but one option group folds into the wrapper,
    // so `/(?i:a)/` writes itself as `(?i-mx:a)` rather than nesting.
    let (pattern, group_on, group_off) = match sole_option_group(pattern) {
        Some(held) => held,
        None => (pattern.to_string(), String::new(), String::new()),
    };
    let pattern = pattern.as_str();
    let mut on = String::new();
    for flag in ['m', 'i', 'x'] {
        if group_off.contains(flag) {
            continue;
        }
        if flags.contains(flag) || group_on.contains(flag) {
            on.push(flag);
        }
    }
    let mut off = String::new();
    for flag in ['m', 'i', 'x'] {
        if !on.contains(flag) {
            off.push(flag);
        }
    }
    if off.is_empty() {
        format!("(?{}:{})", on, pattern)
    } else {
        format!("(?{}-{}:{})", on, off, pattern)
    }
}

/// Whether a pattern names nothing but ASCII. A `\\u` escape counts for the
/// character it names rather than for the letters that spell it.
fn names_only_ascii(pattern: &str) -> bool {
    if !pattern.is_ascii() {
        return false;
    }
    let letters: Vec<char> = pattern.chars().collect();
    let mut at = 0;
    while at + 1 < letters.len() {
        if letters[at] != '\\' || letters[at + 1] != 'u' {
            at += 1;
            continue;
        }
        let mut digits = String::new();
        let mut cursor = at + 2;
        if letters.get(cursor) == Some(&'{') {
            cursor += 1;
            while let Some(letter) = letters.get(cursor) {
                if *letter == '}' {
                    break;
                }
                digits.push(*letter);
                cursor += 1;
            }
        } else {
            while digits.len() < 4 && letters.get(cursor).is_some_and(char::is_ascii_hexdigit) {
                digits.push(letters[cursor]);
                cursor += 1;
            }
        }
        for named in digits.split_whitespace() {
            if u32::from_str_radix(named, 16).is_ok_and(|value| value > 0x7f) {
                return false;
            }
        }
        at = cursor.max(at + 2);
    }
    true
}

/// The body and options of a source that is one option group and nothing
/// else, as `(?on-off:body)`.
fn sole_option_group(pattern: &str) -> Option<(String, String, String)> {
    let rest = pattern.strip_prefix("(?")?;
    let body = rest.strip_suffix(')')?;
    let (options, inner) = body.split_once(':')?;
    if !options
        .chars()
        .all(|letter| matches!(letter, 'm' | 'i' | 'x' | '-'))
    {
        return None;
    }
    // The group must reach the end of the source on its own, which it does
    // only when nothing in its body closes it early.
    let mut depth = 1;
    let mut escaped = false;
    for letter in inner.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match letter {
            '\\' => escaped = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return None;
                }
            }
            _ => {}
        }
    }
    let (on, off) = match options.split_once('-') {
        Some((on, off)) => (on, off),
        None => (options, ""),
    };
    Some((inner.to_string(), on.to_string(), off.to_string()))
}

fn type_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("TypeError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

/// Which part of the last match a global names: a capture number, or the text
/// before or after the whole match.
pub(crate) enum MatchPart {
    Group(i64),
    Before,
    After,
    /// The last group that matched, which is what `$+` names.
    LastGroup,
}

/// The match part `$1`, `` $` ``, `$'`, or `$&` names, if any.
pub(crate) fn capture_reference(name: &str) -> Option<MatchPart> {
    match name {
        "`" => Some(MatchPart::Before),
        "'" => Some(MatchPart::After),
        "&" => Some(MatchPart::Group(0)),
        "+" => Some(MatchPart::LastGroup),
        _ => name
            .parse::<i64>()
            .ok()
            .filter(|number| *number > 0)
            .map(MatchPart::Group),
    }
}

impl VirtualMachine {
    /// Read one part of the last match, which is nil when nothing matched.
    pub(crate) fn last_match_part(
        &mut self,
        part: MatchPart,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(data) = self
            .globals()
            .get(LAST_MATCH)
            .filter(|found| !matches!(found, Object::Nil))
        else {
            return Ok(Object::Nil);
        };
        match part {
            MatchPart::Group(index) => {
                self.send_to_object(data, "[]", vec![Object::Int(index)], position)
            }
            MatchPart::Before => self.send_to_object(data, "pre_match", vec![], position),
            MatchPart::After => self.send_to_object(data, "post_match", vec![], position),
            // `$+` names the last group that matched, which is the last one
            // the match holds anything for.
            MatchPart::LastGroup => {
                let held = self.send_to_object(data, "captures", vec![], position)?;
                let Object::Array(groups) = held else {
                    return Ok(Object::Nil);
                };
                let found = groups
                    .borrow()
                    .iter()
                    .rev()
                    .find(|group| !matches!(group, Object::Nil))
                    .cloned();
                Ok(found.unwrap_or(Object::Nil))
            }
        }
    }
}

/// The flags two patterns are compared on. `/n` names an encoding rather than
/// a matching rule, so two patterns that differ only in it are the same.
pub(crate) fn comparable_flags(flags: &str) -> String {
    flags.chars().filter(|flag| *flag != 'n').collect()
}

/// The named groups a pattern declares, as the number each takes among the
/// named ones and the name it was written with.
fn named_groups(pattern: &str, flags: &str) -> Vec<(usize, String)> {
    let Some(compiled) = compile(pattern, flags) else {
        return Vec::new();
    };
    compiled
        .capture_names()
        .flatten()
        .enumerate()
        .map(|(slot, name)| (slot + 1, original_group_name(name).to_string()))
        .collect()
}
