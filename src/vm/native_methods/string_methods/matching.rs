// Matching a pattern against a string.

use super::*;

impl VirtualMachine {
    /// Matching a pattern against a string.
    pub(crate) fn call_string_match_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
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
                if let Object::Regex(written, _) = &arguments[0] {
                    self.prepare_match_subject(written, &flags, receiver, position)?;
                }
                let subject = crate::vm::native_methods::regexp_methods::match_text(string_value);
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
                if let Object::Regex(written, _) = &arguments[0] {
                    self.prepare_match_subject(written, &flags, receiver, position)?;
                }
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
                let matched = crate::vm::native_methods::compile(&pattern, &flags)
                    .map(|compiled| compiled.find_at(&subject.as_str(), start).is_some())
                    .unwrap_or(false);
                Ok(Some(Object::Bool(matched)))
            }
            _ => Ok(None),
        }
    }
}
