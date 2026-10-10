// Searching the entries, flattening them out, and comparing one hash
// against another as a set.

use super::*;

impl VirtualMachine {
    /// Searching the entries, flattening them out, and comparing one hash
    /// against another as a set.
    pub(crate) fn call_hash_query_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `assoc(key)` and `rassoc(value)` answer the matching pair.
            "assoc" | "rassoc" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let matching_key = method_name == "assoc";
                for (key, value) in self.hash_pairs(dict_rc) {
                    let candidate = if matching_key { &key } else { &value };
                    if self.elements_equal(candidate, &arguments[0], position)? {
                        return Ok(Some(Object::array(vec![key, value])));
                    }
                }
                Ok(Some(Object::Nil))
            }
            // `flatten` answers the pairs one after another, flattening a
            // level deeper when asked.
            "flatten" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let depth: i64 = match arguments.first() {
                    None => 1,
                    Some(Object::Int(level)) => *level,
                    Some(other) => self
                        .coerce_integer_argument(other, position)?
                        .try_into()
                        .unwrap_or(1),
                };
                let mut rows = Vec::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    rows.push(Object::array(vec![key, value]));
                }
                let flat = Object::array(rows);
                self.send_to_object(flat, "flatten", vec![Object::Int(depth)], position)
                    .map(Some)
            }
            // `sort` orders the pairs the way an array of them sorts.
            "sort" => {
                let mut rows = Vec::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    rows.push(Object::array(vec![key, value]));
                }
                let pairs = Object::array(rows);
                if let Some(block) = self.pending_block.take() {
                    self.pending_block = Some(block);
                }
                self.send_to_object(pairs, "sort", vec![], position)
                    .map(Some)
            }
            // `shift` removes the first pair and answers it.
            "shift" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                // The first entry is removed from the slot it sits in, which
                // for a key placed by its `#hash` is not how the key renders.
                let first = {
                    let dict = dict_rc.borrow();
                    dict.iter()
                        .find(|(slot, _)| !is_internal_key(slot))
                        .map(|(slot, value)| {
                            (slot.clone(), reconstruct_key(&dict, slot), value.clone())
                        })
                };
                let Some((slot, key, value)) = first else {
                    return Ok(Some(Object::Nil));
                };
                let mut dict = dict_rc.borrow_mut();
                dict.shift_remove(&slot);
                forget_key_object(&mut dict, &slot);
                Ok(Some(Object::array(vec![key, value])))
            }
            // `deconstruct_keys` answers the hash itself, whatever keys the
            // pattern asked for.
            "deconstruct_keys" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(receiver.clone()))
            }
            // `any?` answers whether the block holds for a pair, or whether
            // the hash holds anything at all.
            "any?" | "none?" | "all?" | "one?" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                // A pattern argument is matched with `===` against the
                // `[key, value]` pair, and it wins over a block.
                let pattern = arguments.first().cloned();
                if pattern.is_some() && block.is_some() {
                    self.emit_warning_to_stderr("warning: given block not used", position);
                }
                let pairs = self.hash_pairs(dict_rc);
                if pattern.is_none() && block.is_none() {
                    let any = !pairs.is_empty();
                    return Ok(Some(Object::Bool(match method_name {
                        "any?" => any,
                        "none?" => !any,
                        "one?" => pairs.len() == 1,
                        _ => true,
                    })));
                }
                let mut answers = Vec::with_capacity(pairs.len());
                for (key, value) in pairs {
                    let verdict = match &pattern {
                        Some(pattern) => self.evaluate_binary_operation(
                            &crate::ast::BinaryOp::CaseEqual,
                            pattern.clone(),
                            Object::array(vec![key, value]),
                            position,
                        )?,
                        None => {
                            let block = block.as_ref().expect("a block or a pattern was given");
                            self.execute_block_callable(block, vec![key, value], position)?
                        }
                    };
                    answers.push(verdict.is_truthy());
                }
                Ok(Some(Object::Bool(match method_name {
                    "any?" => answers.iter().any(|held| *held),
                    "none?" => !answers.iter().any(|held| *held),
                    "one?" => answers.iter().filter(|held| **held).count() == 1,
                    _ => answers.iter().all(|held| *held),
                })))
            }
            // `<`, `<=`, `>`, and `>=` compare hashes by containment.
            "<" | "<=" | ">" | ">=" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // An operand that is not a Hash is asked for one, which is
                // what `to_hash` answers.
                let other = match &arguments[0] {
                    Object::Dict(_) => arguments[0].clone(),
                    other if self.responds_to(other, "to_hash") => {
                        self.send_to_object(other.clone(), "to_hash", vec![], position)?
                    }
                    other => {
                        let message = format!(
                            "no implicit conversion of {} into Hash",
                            crate::vm::errors::conversion_subject(other)
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    }
                };
                let Object::Dict(other_rc) = &other else {
                    let message = format!(
                        "no implicit conversion of {} into Hash",
                        crate::vm::errors::conversion_subject(&arguments[0])
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &message,
                        position,
                    ));
                };
                let ours = self.hash_pairs(dict_rc);
                let theirs = self.hash_pairs(other_rc);
                let (smaller, larger) = match method_name {
                    "<" | "<=" => (&ours, &theirs),
                    _ => (&theirs, &ours),
                };
                let mut contained = true;
                for (key, value) in smaller {
                    let rendered = crate::vm::utils::object_to_dict_key(key).unwrap_or_default();
                    let found = larger
                        .iter()
                        .find(|(other_key, _)| {
                            crate::vm::utils::object_to_dict_key(other_key).unwrap_or_default()
                                == rendered
                        })
                        .map(|(_, other_value)| other_value.clone());
                    match found {
                        Some(other_value) => {
                            if !self.elements_equal(value, &other_value, position)? {
                                contained = false;
                                break;
                            }
                        }
                        None => {
                            contained = false;
                            break;
                        }
                    }
                }
                let strict = matches!(method_name, "<" | ">");
                let same_size = ours.len() == theirs.len();
                Ok(Some(Object::Bool(contained && !(strict && same_size))))
            }
            _ => Ok(None),
        }
    }
}
