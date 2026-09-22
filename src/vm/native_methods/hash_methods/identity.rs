// Placing keys by identity rather than by value, and the keys and
// values a hash holds.

use super::*;

impl VirtualMachine {
    /// Placing keys by identity rather than by value, and the keys and
    /// values a hash holds.
    pub(crate) fn call_hash_identity_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // No-op stub: metorex doesn't distinguish identity from equality
            // for hash keys, so `compare_by_identity` just returns the receiver.
            "compare_by_identity" => {
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
                dict_rc
                    .borrow_mut()
                    .insert(BY_IDENTITY_KEY.to_string(), Object::Bool(true));
                // Entries already held were placed by value, so they are put
                // back where their identity belongs.
                self.call_hash_method(receiver, "rehash", &[], position)?;
                Ok(Some(receiver.clone()))
            }
            "compare_by_identity?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(
                    dict_rc.borrow().contains_key(BY_IDENTITY_KEY),
                )))
            }
            "keys" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // A name the environment holds is read in the encoding the
                // locale names, the way its values are.
                let named = match self.dict_is_environment(dict_rc) {
                    true => self.environment_reading_encoding(),
                    false => None,
                };
                let dict = dict_rc.borrow();
                let keys: Vec<Object> = dict
                    .keys()
                    .filter(|k| !is_internal_key(k))
                    .map(|k| retagged(reconstruct_key(&dict, k), &named))
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(keys)))))
            }
            "values" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let dict = dict_rc.borrow();
                let values: Vec<Object> = dict
                    .iter()
                    .filter(|(k, _)| !is_internal_key(k))
                    .map(|(_, v)| v.clone())
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(values)))))
            }
            _ => Ok(None),
        }
    }
}
