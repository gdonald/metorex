// Cutting a string into lines and taking pieces out of it.

use super::*;

impl VirtualMachine {
    /// Cutting a string into lines and taking pieces out of it.
    pub(crate) fn call_string_slicing_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
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
                        match crate::vm::native_methods::regexp_methods::compile(pattern, flags) {
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
                let chars = character_units(string_value);
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
                    let sliced: String = chars[start_idx..end_idx].concat();
                    // A piece is written in the whole string's encoding.
                    let piece = crate::object::StringValue::with_encoding(
                        sliced,
                        string_value.encoding_name(),
                    );
                    if string_value.holds_bytes() {
                        piece.mark_bytes();
                    }
                    Ok(Some(Object::String(Rc::new(piece))))
                }
            }
            _ => Ok(None),
        }
    }
}
