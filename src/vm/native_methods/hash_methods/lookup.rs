// Whether a hash holds a key, and the entry it keeps for one.

use super::*;

impl VirtualMachine {
    /// Whether a hash holds a key, and the entry it keeps for one.
    pub(crate) fn call_hash_lookup_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "has_key?" | "key?" | "include?" | "member?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let found = self.hash_find_key(dict_rc, &arguments[0], position)?;
                Ok(Some(Object::Bool(found.is_some())))
            }
            // Two hashes holding the same pairs hash alike however they were
            // built, so the pairs are folded in a way the order cannot
            // change. A hash held inside another stands for its kind and its
            // count, which is what lets one that reaches itself hash at all.
            "hash" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let pairs: Vec<(Object, Object)> = {
                    let dict = dict_rc.borrow();
                    dict.iter()
                        .filter(|(key, _)| !is_internal_key(key))
                        .map(|(key, value)| (reconstruct_key(&dict, key), value.clone()))
                        .collect()
                };
                let mut total: i64 = pairs.len() as i64;
                for (key, value) in pairs {
                    let key_digest = self.shallow_digest(&key, position)?;
                    let value_digest = self.shallow_digest(&value, position)?;
                    total =
                        total.wrapping_add(key_digest.wrapping_mul(31).wrapping_add(value_digest));
                }
                Ok(Some(Object::Int(
                    crate::vm::native_methods::object_methods::hashing::seeded_hash(total),
                )))
            }
            "entries" | "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let dict = dict_rc.borrow();
                let entries: Vec<Object> = dict
                    .iter()
                    .filter(|(k, _)| !is_internal_key(k))
                    .map(|(k, v)| {
                        Object::Array(Rc::new(RefCell::new(vec![
                            reconstruct_key(&dict, k),
                            v.clone(),
                        ])))
                    })
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(entries)))))
            }
            "delete" => {
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
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let found = self.hash_find_key(dict_rc, &arguments[0], position)?;
                let Some(key_str) = found else {
                    // A key the hash has no entry for is handed to the block,
                    // which decides what `delete` answers.
                    return match block {
                        Some(block) => self
                            .execute_block_callable(&block, vec![arguments[0].clone()], position)
                            .map(Some),
                        None => Ok(Some(Object::Nil)),
                    };
                };
                let mut dict = dict_rc.borrow_mut();
                let removed = dict.shift_remove(&key_str).unwrap_or(Object::Nil);
                // Also remove from key objects sentinel if present
                forget_key_object(&mut dict, &key_str);
                Ok(Some(removed))
            }
            // The value and the block a hash answers with for a key it has no
            // entry for, and the writers that set them.
            "default" => {
                // `default(key)` runs the default proc for that key, while
                // `default` on its own answers the stored value.
                let proc = dict_rc.borrow().get(DEFAULT_PROC_KEY).cloned();
                if let (Some(key), Some(Object::Block(block))) = (arguments.first(), proc) {
                    return self
                        .execute_block_callable(
                            &block,
                            vec![receiver.clone(), key.clone()],
                            position,
                        )
                        .map(Some);
                }
                Ok(Some(
                    dict_rc
                        .borrow()
                        .get(DEFAULT_VALUE_KEY)
                        .cloned()
                        .unwrap_or(Object::Nil),
                ))
            }
            "default=" => {
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
                // Setting a default value clears the default proc, since a
                // hash answers with one or the other.
                let mut dict = dict_rc.borrow_mut();
                dict.shift_remove(DEFAULT_PROC_KEY);
                dict.insert(DEFAULT_VALUE_KEY.to_string(), arguments[0].clone());
                drop(dict);
                Ok(Some(arguments[0].clone()))
            }
            _ => Ok(None),
        }
    }
}
