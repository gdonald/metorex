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

/// Rewrite a pattern so no two groups share a name, answering the new source
/// and, in group order, the name each group was written with. Ruby allows a
/// name to repeat and reports the farthest match under it.
pub(crate) fn uniquify_group_names(pattern: &str) -> (String, Vec<String>) {
    let mut rewritten = String::with_capacity(pattern.len());
    let mut names = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut chars = pattern.char_indices().peekable();
    while let Some((index, character)) = chars.next() {
        // An escaped character never opens a group.
        if character == '\\' {
            rewritten.push(character);
            if let Some((_, escaped)) = chars.next() {
                rewritten.push(escaped);
            }
            continue;
        }
        let opens_named_group = character == '('
            && pattern[index..].starts_with("(?<")
            && !pattern[index..].starts_with("(?<=")
            && !pattern[index..].starts_with("(?<!");
        if !opens_named_group {
            rewritten.push(character);
            continue;
        }
        let Some(close) = pattern[index + 3..].find('>') else {
            rewritten.push(character);
            continue;
        };
        let name = &pattern[index + 3..index + 3 + close];
        let count = seen.entry(name.to_string()).or_insert(0);
        *count += 1;
        let unique = if *count == 1 {
            name.to_string()
        } else {
            format!("{}{}{}", name, RENAMED_SUFFIX, count)
        };
        rewritten.push_str(&format!("(?<{}>", unique));
        names.push(name.to_string());
        // `close` counts bytes, and the walk steps by characters, so a
        // multi-byte name has to be measured in characters here.
        let consumed = pattern[index + 1..index + 3 + close + 1].chars().count();
        for _ in 0..consumed {
            chars.next();
        }
    }
    (rewritten, names)
}

/// The name a group was written with, which a repeated one carries under a
/// suffix so the pattern could compile.
pub(crate) fn original_group_name(name: &str) -> &str {
    match name.find(RENAMED_SUFFIX) {
        Some(index) => &name[..index],
        None => name,
    }
}

/// Compile a pattern, applying the flags the literal carried.
pub(crate) fn compile(pattern: &str, flags: &str) -> Option<regex::Regex> {
    let mut prefix = String::new();
    if flags.contains('i') {
        prefix.push('i');
    }
    if flags.contains('m') {
        prefix.push('s');
    }
    if flags.contains('x') {
        prefix.push('x');
    }
    let (pattern, _) = uniquify_group_names(pattern);
    let source = if prefix.is_empty() {
        pattern
    } else {
        format!("(?{}){}", prefix, pattern)
    };
    regex::Regex::new(&source).ok()
}

/// The characters a `=~` operand searches, which a Symbol supplies from its
/// name the way Ruby's does.
pub(crate) fn subject_text(object: &Object) -> Option<String> {
    match object {
        Object::String(text) | Object::Symbol(text) => Some(text.as_str().to_string()),
        // An instance of a String subclass carries its characters in an
        // instance variable, and matches the same as a plain String.
        other => match super::string_subclass_value(other) {
            Some(Object::String(text)) => Some(text.as_str().to_string()),
            _ => None,
        },
    }
}

impl VirtualMachine {
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
        let Some(found) = compiled.captures_at(text, start) else {
            self.globals_mut().set(LAST_MATCH, Object::Nil);
            return Ok(None);
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
                    begins.push(Object::Int(char_offset(text, group.start())));
                    ends.push(Object::Int(char_offset(text, group.end())));
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
            Some(named) => Object::String(Rc::new(crate::object::StringValue::with_encoding(
                text.to_string(),
                named.clone(),
            ))),
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

    /// Regexp instance methods. `Object::Regex` is a primitive rather than an
    /// instance, so its methods are dispatched from here.
    pub(crate) fn call_regexp_method(
        &mut self,
        pattern: &str,
        flags: &str,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "source" => Ok(Some(Object::string(pattern.to_string()))),
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
                            self.builtins().class_of(&arguments[0]).ruby_name()
                        ),
                        position,
                    ));
                };
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
                let found = self.regexp_match_data(pattern, flags, &text, start, position)?;
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
                let matched = compile(pattern, flags)
                    .map(|compiled| compiled.is_match(&text))
                    .unwrap_or(false);
                Ok(Some(Object::Bool(matched)))
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
                match self.regexp_match_data(pattern, flags, &text, 0, position)? {
                    Some(data) => self
                        .send_to_object(data, "begin", vec![Object::Int(0)], position)
                        .map(Some),
                    None => Ok(Some(Object::Nil)),
                }
            }
            "to_s" => Ok(Some(Object::string(to_source_string(pattern, flags)))),
            "inspect" => Ok(Some(Object::string(format!("/{}/{}", pattern, flags)))),
            "hash" => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&(pattern, comparable_flags(flags)), &mut hasher);
                Ok(Some(Object::Int(std::hash::Hasher::finish(&hasher) as i64)))
            }
            "==" | "eql?" => {
                let same = matches!(arguments.first(), Some(Object::Regex(other, other_flags))
                    if *other.as_str() == *pattern
                        && comparable_flags(other_flags) == comparable_flags(flags));
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

/// The byte offset `index` counted in characters, which is what a Ruby offset
/// into a String means.
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
    let mut on = String::new();
    for flag in ['m', 'i', 'x'] {
        if flags.contains(flag) {
            on.push(flag);
        }
    }
    let mut off = String::new();
    for flag in ['m', 'i', 'x'] {
        if !flags.contains(flag) {
            off.push(flag);
        }
    }
    if off.is_empty() {
        format!("(?{}:{})", on, pattern)
    } else {
        format!("(?{}-{}:{})", on, off, pattern)
    }
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
}

/// The match part `$1`, `` $` ``, `$'`, or `$&` names, if any.
pub(crate) fn capture_reference(name: &str) -> Option<MatchPart> {
    match name {
        "`" => Some(MatchPart::Before),
        "'" => Some(MatchPart::After),
        "&" => Some(MatchPart::Group(0)),
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
