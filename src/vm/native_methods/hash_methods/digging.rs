// Reading a key that may be missing, one nested inside, and the entries
// another hash brings.

use super::*;

impl VirtualMachine {
    /// Reading a key that may be missing, one nested inside, and the entries
    /// another hash brings.
    pub(crate) fn call_hash_digging_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `dig(key, *rest)` — fetch `key`, then keep digging into the
            // result. A missing key answers nil without visiting `rest`.
            "dig" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let Some(key_str) = crate::vm::utils::object_to_dict_key(&arguments[0]) else {
                    return Ok(Some(Object::Nil));
                };
                // A key the hash holds nothing for reads as the default,
                // since `dig` goes through `[]` the way Ruby's does.
                let found = dict_rc.borrow().get(&key_str).cloned();
                let value = match found {
                    Some(held) => held,
                    None => {
                        let default =
                            self.call_hash_method(receiver, "[]", &arguments[..1], position)?;
                        match default {
                            Some(held) => held,
                            None => return Ok(Some(Object::Nil)),
                        }
                    }
                };
                if arguments.len() == 1 {
                    return Ok(Some(value));
                }
                if matches!(value, Object::Nil) {
                    return Ok(Some(Object::Nil));
                }
                self.dig_into(&value, &arguments[1..], position).map(Some)
            }
            "get" | "fetch" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(1, 2),
                        arguments.len(),
                        position,
                    ));
                }
                // The block is set aside first, since looking the key up may
                // run code of the program's own that would otherwise take it.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let found = self.hash_find_key(dict_rc, &arguments[0], position)?;
                if let Some(key) = found {
                    let value = dict_rc.borrow().get(&key).cloned();
                    if let Some(value) = value {
                        if block.is_some() && arguments.len() == 2 {
                            self.warn_hash_block_supersedes(position)?;
                        }
                        return Ok(Some(value));
                    }
                }
                // A block wins over a default value, and Ruby says so.
                if let Some(block) = block {
                    if arguments.len() == 2 {
                        self.warn_hash_block_supersedes(position)?;
                    }
                    return self
                        .execute_block_callable(&block, vec![arguments[0].clone()], position)
                        .map(Some);
                }
                if arguments.len() == 2 {
                    return Ok(Some(arguments[1].clone()));
                }
                if method_name == "get" {
                    return Ok(Some(Object::Nil));
                }
                let rendered =
                    crate::vm::native_methods::array_methods::inspect_element(&arguments[0]);
                let message = format!("key not found: {}", rendered);
                let error = crate::vm::errors::simple_exception("KeyError", &message, position);
                Err(self.with_key_error_details(error, receiver, &arguments[0]))
            }
            // `merge` answers a new hash and `merge!`/`update` change the
            // receiver. Each argument is put through `to_hash`, and a block
            // decides what a key both hashes hold ends up with.
            "merge" | "merge!" | "update" => {
                let in_place = method_name != "merge";
                if in_place && self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut merged = dict_rc.borrow().clone();
                for argument in arguments {
                    let other = match argument {
                        Object::Dict(_) => argument.clone(),
                        // An instance of a Hash subclass is merged by the
                        // entries it holds, without `to_hash` being asked for.
                        other
                            if crate::vm::native_methods::hash_subclass_value(other).is_some() =>
                        {
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
                        continue;
                    };
                    let incoming: Vec<(String, Object)> = other_rc
                        .borrow()
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    for (key, value) in incoming {
                        if is_internal_key(&key) {
                            continue;
                        }
                        let settled = match (&block, merged.get(&key).cloned()) {
                            (Some(block), Some(existing)) => {
                                let key_object = reconstruct_key(&merged, &key);
                                self.execute_block_callable(
                                    block,
                                    vec![key_object, existing, value.clone()],
                                    position,
                                )?
                            }
                            _ => value.clone(),
                        };
                        merged.insert(key, settled);
                    }
                    // The key objects the other hash recorded travel with it.
                    if let Some(Object::Dict(other_keys)) = other_rc.borrow().get(KEY_OBJECTS_KEY) {
                        let mut ours = match merged.get(KEY_OBJECTS_KEY) {
                            Some(Object::Dict(existing)) => existing.borrow().clone(),
                            _ => indexmap::IndexMap::new(),
                        };
                        for (key, object) in other_keys.borrow().iter() {
                            ours.insert(key.clone(), object.clone());
                        }
                        merged.insert(
                            KEY_OBJECTS_KEY.to_string(),
                            Object::Dict(Rc::new(RefCell::new(ours))),
                        );
                    }
                }
                if !in_place {
                    return Ok(Some(carry_identity(dict_rc, merged)));
                }
                *dict_rc.borrow_mut() = merged;
                Ok(Some(receiver.clone()))
            }
            _ => Ok(None),
        }
    }
}
