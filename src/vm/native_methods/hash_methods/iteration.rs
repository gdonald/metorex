// Walking the entries, and the values a walk builds.

use super::*;

impl VirtualMachine {
    /// Walking the entries, and the values a walk builds.
    pub(crate) fn call_hash_iteration_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `each_pair` is Ruby's alias for `each`.
            // `map` yields each pair and collects what the block answers,
            // which is an Array rather than a Hash.
            "map" | "collect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return Err(MetorexError::runtime_error(
                        format!("{} requires a block", method_name),
                        position_to_location(position),
                    ));
                };
                let dict = dict_rc.borrow();
                let entries: Vec<(Object, Object)> = dict
                    .iter()
                    .filter(|(key, _)| !is_internal_key(key))
                    .map(|(key, value)| (reconstruct_key(&dict, key), value.clone()))
                    .collect();
                drop(dict);
                // A block written for one value reads the pair as an array,
                // which is the single value a Hash yields. One written for
                // two reads the key and the value apart.
                let takes_pair_apart = block
                    .binding_parameters()
                    .iter()
                    .filter(|named| !named.starts_with('&'))
                    .count()
                    >= 2;
                let mut mapped = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    let given = if takes_pair_apart {
                        vec![key, value]
                    } else {
                        vec![Object::array(vec![key, value])]
                    };
                    mapped.push(self.execute_block_callable(&block, given, position)?);
                }
                Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    mapped,
                )))))
            }
            "each" | "each_pair" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                };
                let entries = self.hash_pairs(dict_rc);
                for (key, value) in entries {
                    // Ruby yields one `[key, value]` array, which a block of
                    // two parameters spreads across them.
                    let args = vec![Object::array(vec![key, value])];
                    match self.execute_block_with_control_flow(&block, args, position)? {
                        crate::vm::ControlFlow::Next
                        | crate::vm::ControlFlow::Value(_)
                        | crate::vm::ControlFlow::Redo { .. }
                        | crate::vm::ControlFlow::Retry { .. }
                        | crate::vm::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        // `break` ends the walk and answers what it carried,
                        // which is what the call reports.
                        crate::vm::ControlFlow::Break { value, .. } => {
                            return Ok(Some(value));
                        }
                        crate::vm::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: crate::vm::utils::position_to_location(position),
                                home_frame: block.home_frame,
                            });
                        }
                        crate::vm::ControlFlow::Exception {
                            exception,
                            position,
                        } => {
                            return Err(MetorexError::UncaughtException {
                                exception: exception.clone(),
                                location: crate::vm::utils::position_to_location(position),
                                message: crate::vm::utils::format_exception(&exception),
                            });
                        }
                    }
                }
                Ok(Some(receiver.clone()))
            }
            // The size and emptiness of the entries, leaving the sentinels
            // the hash keeps for its default and its key objects out.
            "empty?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let empty = dict_rc.borrow().keys().all(|key| is_internal_key(key));
                Ok(Some(Object::Bool(empty)))
            }
            // `has_value?` compares with `==`, which is what Ruby uses for
            // values as against the `eql?` it uses for keys.
            "has_value?" | "value?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                for value in self.hash_values(dict_rc) {
                    if self.elements_equal(&value, &arguments[0], position)? {
                        return Ok(Some(Object::Bool(true)));
                    }
                }
                Ok(Some(Object::Bool(false)))
            }
            // `key(value)` answers the first key holding that value.
            "key" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                for (key, value) in self.hash_pairs(dict_rc) {
                    if self.elements_equal(&value, &arguments[0], position)? {
                        return Ok(Some(key));
                    }
                }
                Ok(Some(Object::Nil))
            }
            "each_key" | "each_value" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                // A name the environment holds is read in the encoding the
                // locale names, the way its values are.
                let named = match self.dict_is_environment(dict_rc) {
                    true => self.environment_reading_encoding(),
                    false => None,
                };
                for (key, value) in self.hash_pairs(dict_rc) {
                    let yielded = if method_name == "each_key" {
                        retagged(key, &named)
                    } else {
                        value
                    };
                    self.execute_block_callable(&block, vec![yielded], position)?;
                }
                Ok(Some(receiver.clone()))
            }
            _ => Ok(None),
        }
    }
}
