// The work behind `sub`, `gsub` and a walk over lines.

use super::*;

impl VirtualMachine {
    /// `sub` and `gsub`: the string with the first match, or every match,
    /// replaced by what the argument or the block answers.
    pub(crate) fn substitute(
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
        let (anchored, source) =
            match crate::vm::native_methods::regexp_methods::previous_match_split(&pattern) {
                Some((before, after)) if before.is_empty() => (true, after),
                _ => (false, pattern.clone()),
            };
        let Some(compiled) = crate::vm::native_methods::regexp_methods::compile(&source, &flags)
        else {
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
            if crate::vm::native_methods::regexp_methods::line_start_past_the_end(
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
            self.globals_mut().set(
                crate::vm::native_methods::regexp_methods::LAST_MATCH,
                Object::Nil,
            );
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
    pub(crate) fn walk_lines(
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
