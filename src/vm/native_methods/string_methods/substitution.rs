// Copying a string, and replacing what a pattern matches in it.

use super::*;

impl VirtualMachine {
    /// Copying a string, and replacing what a pattern matches in it.
    pub(crate) fn call_string_substitution_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
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
                let (anchored, source) =
                    match crate::vm::native_methods::regexp_methods::previous_match_split(&pattern)
                    {
                        Some((before, after)) if before.is_empty() => (true, after),
                        _ => (false, pattern.clone()),
                    };
                let Some(compiled) =
                    crate::vm::native_methods::regexp_methods::compile(&source, &flags)
                else {
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
                        self.globals_mut().set(
                            crate::vm::native_methods::regexp_methods::LAST_MATCH,
                            Object::Nil,
                        );
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
            _ => Ok(None),
        }
    }
}
