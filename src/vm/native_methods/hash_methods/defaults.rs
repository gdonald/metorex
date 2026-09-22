// What a hash answers for a key it has no entry for.

use super::*;

impl VirtualMachine {
    /// What a hash answers for a key it has no entry for.
    pub(crate) fn call_hash_default_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "default_proc" => Ok(Some(
                dict_rc
                    .borrow()
                    .get(DEFAULT_PROC_KEY)
                    .cloned()
                    .unwrap_or(Object::Nil),
            )),
            "default_proc=" => {
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::Nil => {
                        dict_rc.borrow_mut().shift_remove(DEFAULT_PROC_KEY);
                    }
                    other => {
                        // Anything that spells itself out as a callable may
                        // be the default, and one that names its arguments
                        // strictly must take the hash and the key.
                        let held = match other {
                            Object::Block(_) | Object::Method(_) => other.clone(),
                            _ if self.responds_to(other, "to_proc") => {
                                self.send_to_object(other.clone(), "to_proc", vec![], position)?
                            }
                            _ => {
                                return Err(crate::vm::errors::simple_exception(
                                    "TypeError",
                                    &format!(
                                        "wrong default_proc type {} (expected Proc)",
                                        self.builtins().class_of(other).ruby_name()
                                    ),
                                    position,
                                ));
                            }
                        };
                        if let Object::Block(block) = &held
                            && block.is_lambda
                        {
                            let counted =
                                crate::vm::native_methods::method_object_methods::block_arity(
                                    block,
                                );
                            if counted != 2 {
                                return Err(crate::vm::errors::simple_exception(
                                    "TypeError",
                                    "default_proc takes two arguments (2 for 1)",
                                    position,
                                ));
                            }
                        }
                        let mut dict = dict_rc.borrow_mut();
                        // A hash answers with a default value or a default
                        // block, never both.
                        dict.shift_remove(DEFAULT_VALUE_KEY);
                        dict.insert(DEFAULT_PROC_KEY.to_string(), held);
                    }
                }
                Ok(Some(arguments[0].clone()))
            }
            "length" | "size" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let dict = dict_rc.borrow();
                let count = dict.keys().filter(|k| !is_internal_key(k)).count();
                Ok(Some(Object::Int(count as i64)))
            }
            "[]" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A subclass that writes its own `default` decides what a
                // missing key reads as, which is what Ruby asks before it
                // falls back to the default value or block.
                let found = self.hash_find_key(dict_rc, &arguments[0], position)?;
                if found.is_none()
                    && let Some((owner, method)) = self.lookup_method(receiver, "default")
                    && !method.is_undefined
                {
                    return self
                        .invoke_method(
                            owner,
                            method,
                            receiver.clone(),
                            vec![arguments[0].clone()],
                            position,
                        )
                        .map(Some);
                }
                Ok(Some(self.evaluate_index_operation(
                    receiver.clone(),
                    arguments[0].clone(),
                    position,
                )?))
            }
            _ => Ok(None),
        }
    }
}
