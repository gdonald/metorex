// Where a needle sits inside a string, counted in bytes.

use super::*;

impl VirtualMachine {
    /// Where a needle sits in a string, counted in bytes. `byterindex` reads
    /// from the offset backwards, and `byteindex` from the offset forwards.
    pub(crate) fn byte_index_of(
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
    pub(crate) fn string_needle(
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
pub(crate) fn pattern_of(value: &Object) -> Option<(String, String)> {
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
