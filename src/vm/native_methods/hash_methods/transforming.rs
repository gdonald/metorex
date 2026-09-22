// Rewriting the keys or the values, in place or into a copy.

use super::*;

impl VirtualMachine {
    /// Rewriting the keys or the values, in place or into a copy.
    pub(crate) fn call_hash_transforming_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `transform_keys` rebuilds the hash under new keys: a hash
            // argument names the replacement for the keys it holds, and a
            // block answers one for every key it does not.
            "transform_keys" | "transform_keys!" => {
                let in_place = method_name == "transform_keys!";
                let mapping = match arguments.first() {
                    None => None,
                    Some(Object::Dict(given)) => Some(Rc::clone(given)),
                    Some(other) => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &format!("no implicit conversion of {} into Hash", other.type_name()),
                            position,
                        ));
                    }
                };
                let has_block = matches!(self.pending_block, Some(Object::Block(_)));
                if in_place && (has_block || mapping.is_some()) && self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                if !has_block && mapping.is_none() {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let replacements: indexmap::IndexMap<String, Object> = match &mapping {
                    Some(given) => self
                        .hash_pairs(given)
                        .into_iter()
                        .map(|(key, value)| {
                            (
                                crate::vm::utils::object_to_dict_key(&key).unwrap_or_default(),
                                value,
                            )
                        })
                        .collect(),
                    None => indexmap::IndexMap::new(),
                };
                let mut built = indexmap::IndexMap::new();
                let mut broke_with = None;
                // The pair the block broke out of, plus the ones it never
                // reached. They keep the keys they already had.
                let mut left_as_written: Vec<(Object, Object)> = Vec::new();
                let pairs = self.hash_pairs(dict_rc);
                let mut walked = pairs.iter();
                for (key, value) in walked.by_ref() {
                    let (key, value) = (key.clone(), value.clone());
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    let new_key = match replacements.get(&rendered) {
                        Some(named) => named.clone(),
                        None => match &block {
                            Some(block) => {
                                match self.execute_block_callable(
                                    block,
                                    vec![key.clone()],
                                    position,
                                ) {
                                    Ok(answered) => answered,
                                    Err(MetorexError::BlockBreak {
                                        value: broke_value, ..
                                    }) => {
                                        broke_with = Some(broke_value);
                                        left_as_written.push((key, value));
                                        break;
                                    }
                                    Err(error) => return Err(error),
                                }
                            }
                            None => key.clone(),
                        },
                    };
                    keep_pair(&mut built, new_key, value);
                }
                // A `break` leaves the entries the block never reached under
                // the keys they already had, and what was transformed keeps
                // the key it was given.
                if broke_with.is_some() {
                    left_as_written.extend(walked.map(|(key, value)| (key.clone(), value.clone())));
                    for (key, value) in left_as_written {
                        let rendered =
                            crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                        if built.contains_key(&rendered) {
                            continue;
                        }
                        keep_pair(&mut built, key, value);
                    }
                }
                if !in_place {
                    // The copy is a plain Hash comparing keys by value,
                    // however the receiver compares its own.
                    return Ok(Some(Object::Dict(Rc::new(RefCell::new(built)))));
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
            // `transform_values` rebuilds the hash with what the block
            // answers for each value.
            "transform_values" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                let transforming_values = method_name == "transform_values";
                let mut built = indexmap::IndexMap::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let subject = if transforming_values {
                        value.clone()
                    } else {
                        key.clone()
                    };
                    let answered = self.execute_block_callable(&block, vec![subject], position)?;
                    let (new_key, new_value) = if transforming_values {
                        (key, answered)
                    } else {
                        (answered, value)
                    };
                    let rendered =
                        crate::vm::utils::object_to_dict_key(&new_key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&new_key) {
                        remember_key_object(&mut built, &rendered, &new_key);
                    }
                    built.insert(rendered, new_value);
                }
                Ok(Some(carry_identity(dict_rc, built)))
            }
            // `select` and `reject` answer a new hash, while `keep_if` and
            // `delete_if` change the receiver and answer it.
            "select" | "filter" | "reject" | "keep_if" | "delete_if" | "select!" | "filter!"
            | "reject!" => {
                let in_place = matches!(
                    method_name,
                    "keep_if" | "delete_if" | "select!" | "filter!" | "reject!"
                );
                // Without a block it answers an Enumerator first, which Ruby
                // allows even on a frozen hash.
                if in_place
                    && matches!(self.pending_block, Some(Object::Block(_)))
                    && self.object_is_frozen(receiver)
                {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                let rejecting = matches!(method_name, "reject" | "delete_if" | "reject!");
                let mut kept = indexmap::IndexMap::new();
                for (key, value) in self.hash_pairs(dict_rc) {
                    let verdict = self.execute_block_callable(
                        &block,
                        vec![key.clone(), value.clone()],
                        position,
                    )?;
                    if verdict.is_truthy() == rejecting {
                        continue;
                    }
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&key) {
                        remember_key_object(&mut kept, &rendered, &key);
                    }
                    kept.insert(rendered, value);
                }
                if !in_place {
                    return Ok(Some(carry_identity(dict_rc, kept)));
                }
                let before = dict_rc
                    .borrow()
                    .keys()
                    .filter(|key| !is_internal_key(key))
                    .count();
                let changed = kept.len() != before;
                let sentinels: Vec<(String, Object)> = dict_rc
                    .borrow()
                    .iter()
                    .filter(|(key, _)| is_internal_key(key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                let mut dict = dict_rc.borrow_mut();
                dict.clear();
                for (key, value) in sentinels.into_iter().chain(kept) {
                    dict.insert(key, value);
                }
                drop(dict);
                // The bang forms answer nil when nothing changed, while
                // `keep_if` and `delete_if` always answer the hash.
                let bang = matches!(method_name, "select!" | "filter!" | "reject!");
                Ok(Some(if bang && !changed {
                    Object::Nil
                } else {
                    receiver.clone()
                }))
            }
            _ => Ok(None),
        }
    }
}
