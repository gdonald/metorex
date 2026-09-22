// Emptying a hash, and the copies that keep only some of its entries.

use super::*;

impl VirtualMachine {
    /// Emptying a hash, and the copies that keep only some of its entries.
    pub(crate) fn call_hash_rebuilding_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "clear" => {
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
                let kept: Vec<(String, Object)> = dict_rc
                    .borrow()
                    .iter()
                    .filter(|(key, _)| is_internal_key(key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                let mut dict = dict_rc.borrow_mut();
                dict.clear();
                for (key, value) in kept {
                    dict.insert(key, value);
                }
                drop(dict);
                Ok(Some(receiver.clone()))
            }
            // `compact` drops the pairs whose value is nil, and `compact!`
            // does it in place, answering nil when there was none to drop.
            "compact!" => {
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
                let pairs = self.hash_pairs(dict_rc);
                let dropped: Vec<String> = pairs
                    .iter()
                    .filter(|(_, value)| matches!(value, Object::Nil))
                    .map(|(key, _)| crate::vm::utils::object_to_dict_key(key).unwrap_or_default())
                    .collect();
                if dropped.is_empty() {
                    return Ok(Some(Object::Nil));
                }
                let mut dict = dict_rc.borrow_mut();
                for key in dropped {
                    dict.shift_remove(&key);
                }
                drop(dict);
                Ok(Some(receiver.clone()))
            }
            "compact" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut kept = indexmap::IndexMap::new();
                // `compact` keeps the default and the default proc, which the
                // other derived hashes leave behind.
                for sentinel in [DEFAULT_VALUE_KEY, DEFAULT_PROC_KEY] {
                    if let Some(value) = dict_rc.borrow().get(sentinel) {
                        kept.insert(sentinel.to_string(), value.clone());
                    }
                }
                for (key, value) in self.hash_pairs(dict_rc) {
                    if matches!(value, Object::Nil) {
                        continue;
                    }
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&key) {
                        remember_key_object(&mut kept, &rendered, &key);
                    }
                    kept.insert(rendered, value);
                }
                Ok(Some(carry_identity(dict_rc, kept)))
            }
            // `except` drops the named keys, and `slice` keeps only them.
            "except" | "slice" => {
                let mut named = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    named.push(crate::vm::utils::object_to_dict_key(argument).unwrap_or_default());
                }
                let pairs = self.hash_pairs(dict_rc);
                let mut kept = indexmap::IndexMap::new();
                // `slice` answers the pairs in the order they were asked for,
                // while `except` keeps the hash's own order.
                if method_name == "slice" {
                    for wanted in &named {
                        let Some((key, value)) = pairs.iter().find(|(key, _)| {
                            &crate::vm::utils::object_to_dict_key(key).unwrap_or_default() == wanted
                        }) else {
                            continue;
                        };
                        if !crate::vm::utils::is_primitive_key(key) {
                            remember_key_object(&mut kept, wanted, key);
                        }
                        kept.insert(wanted.clone(), value.clone());
                    }
                    return Ok(Some(carry_identity(dict_rc, kept)));
                }
                for (key, value) in pairs {
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    if named.contains(&rendered) {
                        continue;
                    }
                    if !crate::vm::utils::is_primitive_key(&key) {
                        remember_key_object(&mut kept, &rendered, &key);
                    }
                    kept.insert(rendered, value);
                }
                Ok(Some(carry_identity(dict_rc, kept)))
            }
            // `values_at` answers the values the named keys hold, and
            // `fetch_values` raises for a key the hash has no entry for.
            "values_at" | "fetch_values" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut picked = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    let rendered =
                        crate::vm::utils::object_to_dict_key(argument).unwrap_or_default();
                    let found = dict_rc.borrow().get(&rendered).cloned();
                    match found {
                        Some(value) => picked.push(value),
                        None if method_name == "values_at" => picked.push(Object::Nil),
                        None => match &block {
                            Some(block) => {
                                let block = Rc::clone(block);
                                picked.push(self.execute_block_callable(
                                    &block,
                                    vec![argument.clone()],
                                    position,
                                )?);
                            }
                            None => {
                                let message = format!(
                                    "key not found: {}",
                                    crate::vm::native_methods::array_methods::inspect_element(
                                        argument
                                    )
                                );
                                let error = crate::vm::errors::simple_exception(
                                    "KeyError", &message, position,
                                );
                                return Err(self.with_key_error_details(error, receiver, argument));
                            }
                        },
                    }
                }
                Ok(Some(Object::array(picked)))
            }
            _ => Ok(None),
        }
    }
}
