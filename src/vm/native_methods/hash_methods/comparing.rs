// Whether two hashes hold the same entries, and the hash an inversion
// makes.

use super::*;

impl VirtualMachine {
    /// Whether two hashes hold the same entries, and the hash an inversion
    /// makes.
    pub(crate) fn call_hash_comparing_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "invert" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut inverted = indexmap::IndexMap::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let rendered = crate::vm::utils::object_to_dict_key(&value).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&value) {
                        remember_key_object(&mut inverted, &rendered, &value);
                    }
                    inverted.insert(rendered, key);
                }
                // The values become the keys, so the identity setting the old
                // keys were matched under does not carry over.
                Ok(Some(Object::Dict(Rc::new(RefCell::new(inverted)))))
            }
            // Two hashes are equal when they hold the same entries, and
            // each key is looked up in the other hash the way any key is,
            // through `#hash` and `#eql?`. `eql?` compares the values the
            // same strict way; `==` asks them `==`.
            "==" | "eql?" if arguments.len() == 1 => {
                let other = match &arguments[0] {
                    held @ Object::Dict(_) => held.clone(),
                    held => match crate::vm::native_methods::hash_subclass_value(held) {
                        Some(backing @ Object::Dict(_)) => backing,
                        _ => return Ok(Some(Object::Bool(false))),
                    },
                };
                let Object::Dict(other_rc) = &other else {
                    return Ok(Some(Object::Bool(false)));
                };
                if Rc::ptr_eq(dict_rc, other_rc) {
                    return Ok(Some(Object::Bool(true)));
                }
                let pair = (Rc::as_ptr(dict_rc) as usize, Rc::as_ptr(other_rc) as usize);
                if self.hash_comparisons.contains(&pair) {
                    return Ok(Some(Object::Bool(true)));
                }
                self.hash_comparisons.push(pair);
                let answer = self.hash_entries_match(dict_rc, other_rc, method_name, position);
                self.hash_comparisons.pop();
                answer.map(|same| Some(Object::Bool(same)))
            }
            // `to_h` without a block answers the hash itself, and `to_hash`
            // always does.
            "to_hash" => Ok(Some(receiver.clone())),
            // Ruby recomputes every key's place, so a key whose `#hash`
            // changed since it was stored is found again and two keys that
            // have become equal collapse into the entry stored first.
            "rehash" => {
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
                let by_identity = dict_rc.borrow().contains_key(BY_IDENTITY_KEY);
                let held: Vec<(Object, Object)> = {
                    let dict = dict_rc.borrow();
                    dict.iter()
                        .filter(|(slot, _)| !is_internal_key(slot))
                        .map(|(slot, value)| (reconstruct_key(&dict, slot), value.clone()))
                        .collect()
                };
                let carried: Vec<(String, Object)> = {
                    let dict = dict_rc.borrow();
                    dict.iter()
                        .filter(|(slot, _)| is_internal_key(slot) && *slot != KEY_OBJECTS_KEY)
                        .map(|(slot, value)| (slot.clone(), value.clone()))
                        .collect()
                };
                let mut rebuilt = indexmap::IndexMap::new();
                for (key, value) in held {
                    let slot = self.dict_slot_in(&rebuilt, &key, by_identity, position)?;
                    if !rebuilt.contains_key(&slot) {
                        remember_key_object(&mut rebuilt, &slot, &key);
                    }
                    rebuilt.insert(slot, value);
                }
                for (slot, value) in carried {
                    rebuilt.insert(slot, value);
                }
                *dict_rc.borrow_mut() = rebuilt;
                Ok(Some(receiver.clone()))
            }
            "store" => {
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
            _ => Ok(None),
        }
    }
}
