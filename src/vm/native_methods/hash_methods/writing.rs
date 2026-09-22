// Writing entries in, and what a hash reads back as.

use super::*;

impl VirtualMachine {
    /// Writing entries in, and what a hash reads back as.
    pub(crate) fn call_hash_writing_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // Each key and value renders through its own `inspect`, so an
            // object that defines one is shown the way it asks to be. A hash
            // that reaches itself prints `{...}` rather than recursing.
            "inspect" | "to_s" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let address = Rc::as_ptr(dict_rc) as usize;
                if crate::object::rendering_in_progress(address) {
                    return Ok(Some(Object::string("{...}")));
                }
                crate::object::begin_rendering(address);
                let mut parts = Vec::new();
                // A hash with nothing in it spells the same in every
                // encoding, and one with something in it is written in the
                // encoding the first key was rendered in.
                let mut writing = "US-ASCII".to_string();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let rendered = self.render_pair(&key, &value, position);
                    match rendered {
                        Ok((rendered, held)) => {
                            if parts.is_empty() {
                                writing = held;
                            }
                            parts.push(rendered);
                        }
                        Err(error) => {
                            crate::object::end_rendering();
                            return Err(error);
                        }
                    }
                }
                crate::object::end_rendering();
                let written = format!("{{{}}}", parts.join(", "));
                // Written in an encoding that spells a character with bytes of
                // its own, the answer stands for those bytes.
                let by_the_byte = !written.is_ascii()
                    && !matches!(writing.as_str(), "UTF-8" | "US-ASCII" | "UTF8-MAC");
                let made = crate::object::StringValue::with_encoding(written, writing);
                if by_the_byte {
                    made.mark_bytes();
                }
                Ok(Some(Object::String(Rc::new(made))))
            }
            // `[]=` writes one entry, the way `hash[key] = value` does.
            "[]=" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                self.hash_store(dict_rc, &arguments[0], arguments[1].clone(), position)?;
                self.record_environment_change(dict_rc, &arguments[0], &arguments[1]);
                Ok(Some(arguments[1].clone()))
            }
            // `to_h` answers the hash itself, or what the block makes of
            // each pair.
            "to_h" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return Ok(Some(receiver.clone()));
                };
                let mut built = indexmap::IndexMap::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let produced =
                        self.execute_block_callable(&block, vec![key, value], position)?;
                    let (new_key, new_value) = self.pair_from_block_result(produced, position)?;
                    let rendered =
                        crate::vm::utils::object_to_dict_key(&new_key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&new_key) {
                        remember_key_object(&mut built, &rendered, &new_key);
                    }
                    built.insert(rendered, new_value);
                }
                Ok(Some(Object::Dict(Rc::new(RefCell::new(built)))))
            }
            // `replace` takes on the entries of another hash, which it asks
            // for with `to_hash`.
            "replace" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let other = match &arguments[0] {
                    Object::Dict(_) => arguments[0].clone(),
                    // An instance of a Hash subclass is read by the entries it
                    // holds, whatever `to_hash` it was given.
                    other if crate::vm::native_methods::hash_subclass_value(other).is_some() => {
                        crate::vm::native_methods::hash_subclass_value(other)
                            .expect("a hash subclass carries its entries")
                    }
                    other if self.responds_to(other, "to_hash") => {
                        self.send_to_object(other.clone(), "to_hash", vec![], position)?
                    }
                    other => {
                        let message = format!(
                            "no implicit conversion of {} into Hash",
                            self.builtins().class_of(other).ruby_name()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    }
                };
                let Object::Dict(other_rc) = &other else {
                    return Ok(Some(receiver.clone()));
                };
                // A Hash written as `replace(c: -1)` arrives carrying the
                // parser's keyword marker, which is not one of its entries.
                let mut incoming = other_rc.borrow().clone();
                incoming.shift_remove(crate::vm::param_binding::KWARGS_MARKER);
                *dict_rc.borrow_mut() = incoming;
                Ok(Some(receiver.clone()))
            }
            // The bang forms of the transforms change the receiver.
            "transform_values!" => {
                if self.object_is_frozen(receiver)
                    && matches!(self.pending_block, Some(Object::Block(_)))
                {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let transforming_values = method_name == "transform_values!";
                let mut built = indexmap::IndexMap::new();
                // A `break` out of the block keeps what was transformed so
                // far and leaves the rest of the entries as they were.
                let mut broke_with = None;
                let pairs = self.hash_pairs(dict_rc);
                let mut walked = pairs.iter();
                for (key, value) in walked.by_ref() {
                    let (key, value) = (key.clone(), value.clone());
                    let subject = if transforming_values {
                        value.clone()
                    } else {
                        key.clone()
                    };
                    let answered =
                        match self.execute_block_callable(&block, vec![subject], position) {
                            Ok(answered) => answered,
                            Err(MetorexError::BlockBreak {
                                value: broke_value, ..
                            }) => {
                                broke_with = Some(broke_value);
                                keep_pair(&mut built, key, value);
                                break;
                            }
                            Err(error) => return Err(error),
                        };
                    let (new_key, new_value) = if transforming_values {
                        (key, answered)
                    } else {
                        (answered, value)
                    };
                    keep_pair(&mut built, new_key, new_value);
                }
                if broke_with.is_some() {
                    for (key, value) in walked {
                        keep_pair(&mut built, key.clone(), value.clone());
                    }
                }
                let sentinels: Vec<(String, Object)> = dict_rc
                    .borrow()
                    .iter()
                    .filter(|(key, _)| is_internal_key(key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                let mut dict = dict_rc.borrow_mut();
                dict.clear();
                for (key, value) in sentinels.into_iter().chain(built) {
                    dict.insert(key, value);
                }
                drop(dict);
                Ok(Some(broke_with.unwrap_or_else(|| receiver.clone())))
            }
            _ => Ok(None),
        }
    }
}
