// The methods ENV answers differently from any other hash.

use super::*;

impl VirtualMachine {
    /// What ENV answers for a name the operating system keeps, or `None`
    /// where ENV answers the same as any other hash would.
    pub(crate) fn call_environment_hash_method(
        &mut self,
        receiver: &Object,
        dict_rc: &Rc<RefCell<IndexMap<String, Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if self.dict_is_environment(dict_rc) {
            match method_name {
                "to_s" => return Ok(Some(Object::string("ENV"))),
                "rehash" => return Ok(Some(Object::Nil)),
                // A copy of ENV would stop tracking the environment, so Ruby
                // refuses and points at the hash it will make instead. The
                // keywords are read first, so a bad one is reported as such.
                "dup" | "clone" => {
                    if let Some(Object::Dict(entries)) = arguments.first() {
                        for (name, value) in entries.borrow().iter() {
                            let name = name.trim_start_matches(':');
                            if name.starts_with("__MX_") {
                                continue;
                            }
                            let refused =
                                name != "freeze" || !matches!(value, Object::Bool(_) | Object::Nil);
                            if refused {
                                let message = if name == "freeze" {
                                    "unexpected value for freeze: Integer".to_string()
                                } else {
                                    format!("unknown keyword: :{}", name)
                                };
                                return Err(crate::vm::errors::simple_exception(
                                    "ArgumentError",
                                    &message,
                                    position,
                                ));
                            }
                        }
                    }
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!(
                            "Cannot {method_name} ENV, use ENV.to_h to get a copy of ENV as a hash"
                        ),
                        position,
                    ));
                }
                // `to_h` and `to_hash` hand back a hash of their own, so
                // changing it leaves the environment alone.
                "to_h" | "to_hash" if self.pending_block.is_none() => {
                    let copied = dict_rc.borrow().clone();
                    return Ok(Some(Object::Dict(Rc::new(RefCell::new(copied)))));
                }
                // Every key the environment is asked about names a variable,
                // so anything that is not a String is refused outright. Only
                // the keys are checked: `fetch` takes a default after them.
                "fetch" | "values_at" => {
                    let keys = if method_name == "fetch" {
                        arguments.get(..1).unwrap_or(&[])
                    } else {
                        arguments
                    };
                    for argument in keys {
                        if !matches!(argument, Object::String(_))
                            && !self.responds_to(argument, "to_str")
                        {
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                &format!(
                                    "no implicit conversion of {} into String",
                                    self.builtins().class_of(argument).name()
                                ),
                                position,
                            ));
                        }
                    }
                }
                // What the environment is looked up by names a String, so
                // anything that reads as one is asked for its text. A name
                // that reads as nothing is refused, while a value that reads
                // as nothing simply matches nothing.
                name if ENVIRONMENT_TEXT_ARGUMENT.contains(&name)
                    && arguments.len() == 1
                    && !matches!(arguments[0], Object::String(_)) =>
                {
                    if self.responds_to(&arguments[0], "to_str") {
                        let named =
                            self.send_to_object(arguments[0].clone(), "to_str", vec![], position)?;
                        return self.call_hash_method(receiver, method_name, &[named], position);
                    }
                    if ENVIRONMENT_VALUE_ARGUMENT.contains(&name) {
                        return Ok(Some(Object::Nil));
                    }
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!(
                            "no implicit conversion of {} into String",
                            self.builtins().class_of(&arguments[0]).name()
                        ),
                        position,
                    ));
                }
                // A variable is named and valued in text, and a name that
                // holds an `=` or nothing at all names no variable.
                "[]=" | "store" if arguments.len() == 2 => {
                    let key = self.environment_text(&arguments[0], position)?;
                    // Setting a name to nil takes it away, and a name the
                    // environment could never hold simply has nothing to take.
                    if matches!(arguments[1], Object::Nil) {
                        let named = Object::string(key);
                        self.record_environment_change(dict_rc, &named, &Object::Nil);
                        return Ok(Some(Object::Nil));
                    }
                    if key.is_empty() || key.contains('=') {
                        let message = format!("Invalid argument - setenv({})", key);
                        return Err(crate::vm::errors::simple_exception(
                            "Errno::EINVAL",
                            &message,
                            position,
                        ));
                    }
                    let value = self.environment_text(&arguments[1], position)?;
                    self.store_variable(dict_rc, key, value);
                    // The assignment answers what it was handed, which is the
                    // same object when that was already text.
                    return Ok(Some(arguments[1].clone()));
                }
                // `slice` looks each name up as text and answers under the
                // objects it was handed.
                "slice" => {
                    let named = self.environment_reading_encoding();
                    let mut made: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
                    let mut keyed: Vec<(String, Object)> = Vec::new();
                    for argument in arguments {
                        let key = self.environment_text(argument, position)?;
                        let Some(value) = dict_rc.borrow().get(&key).cloned() else {
                            continue;
                        };
                        let Some(slot) = crate::vm::utils::object_to_dict_key(argument) else {
                            continue;
                        };
                        made.insert(slot.clone(), retagged(value, &named));
                        keyed.push((slot, argument.clone()));
                    }
                    if !keyed.is_empty() {
                        let mut objects: indexmap::IndexMap<String, Object> =
                            indexmap::IndexMap::new();
                        for (slot, held) in keyed {
                            objects.insert(slot, held);
                        }
                        made.insert(
                            "__MX_KEY_OBJECTS__".to_string(),
                            Object::Dict(Rc::new(RefCell::new(objects))),
                        );
                    }
                    return Ok(Some(Object::Dict(Rc::new(RefCell::new(made)))));
                }
                // What the environment answers is written in the encoding a
                // program asked for, and in the one the system uses otherwise.
                "[]" | "shift" if arguments.len() <= 1 => {
                    let named = self.environment_reading_encoding();
                    if method_name == "shift" {
                        let Some((key, value)) = self.hash_pairs(dict_rc).into_iter().next() else {
                            return Ok(Some(Object::Nil));
                        };
                        self.record_environment_change(dict_rc, &key, &Object::Nil);
                        return Ok(Some(Object::array(vec![
                            retagged(key, &named),
                            retagged(value, &named),
                        ])));
                    }
                    let key = self.environment_text(&arguments[0], position)?;
                    let held = dict_rc.borrow().get(&key).cloned();
                    return Ok(Some(match held {
                        // A value read out of the environment is frozen, so
                        // changing what was handed back cannot reach the
                        // environment behind it.
                        Some(value) => frozen_text(retagged(value, &named)),
                        None => Object::Nil,
                    }));
                }
                // Every name and value a whole hash brings is read as text
                // first, so a bad one stops the change before it starts.
                "replace" | "merge!" | "update" if arguments.len() == 1 => {
                    let Some(Object::Dict(given)) = arguments.first() else {
                        let message = format!(
                            "no implicit conversion of {} into Hash",
                            self.builtins()
                                .class_of(arguments.first().unwrap_or(&Object::Nil))
                                .name()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    };
                    let pairs = self.hash_pairs(given);
                    // `replace` reads every name and value before it changes
                    // anything, where `merge!` changes as it goes.
                    if method_name == "replace" {
                        let mut settled = Vec::new();
                        for (key, value) in &pairs {
                            let key = self.environment_text(key, position)?;
                            let value = self.environment_text(value, position)?;
                            self.refuse_bad_variable_name(&key, position)?;
                            settled.push((key, value));
                        }
                        for key in self.hash_pairs(dict_rc).into_iter().map(|(key, _)| key) {
                            self.record_environment_change(dict_rc, &key, &Object::Nil);
                        }
                        dict_rc.borrow_mut().clear();
                        for (key, value) in settled {
                            self.store_variable(dict_rc, key, value);
                        }
                        return Ok(Some(receiver.clone()));
                    }
                    let block = self.pending_block.take();
                    for (key, value) in pairs {
                        let key = self.environment_text(&key, position)?;
                        let mut value = self.environment_text(&value, position)?;
                        self.refuse_bad_variable_name(&key, position)?;
                        // A block settles a name both sides hold.
                        if let Some(Object::Block(block)) = &block
                            && let Some(held) = dict_rc.borrow().get(&key).cloned()
                        {
                            let given = vec![
                                Object::string(key.clone()),
                                held,
                                Object::string(value.clone()),
                            ];
                            let answered = self.execute_block_callable(block, given, position)?;
                            value = self.environment_text(&answered, position)?;
                        }
                        self.store_variable(dict_rc, key, value);
                    }
                    return Ok(Some(receiver.clone()));
                }
                // What the environment answers is written in the encoding
                // the program asked for, when it asked for one.
                "each" | "each_pair"
                    if arguments.is_empty()
                        && matches!(self.pending_block, Some(Object::Block(_))) =>
                {
                    let Some(Object::Block(block)) = self.pending_block.take() else {
                        return Ok(None);
                    };
                    let named = self.environment_reading_encoding();
                    for (key, value) in self.hash_pairs(dict_rc) {
                        let key = retagged(key, &named);
                        let value = retagged(value, &named);
                        let pair = Object::array(vec![key, value]);
                        match self.execute_block_with_control_flow(&block, vec![pair], position)? {
                            // `break` ends the walk and answers what it carried,
                            // which is what the call reports.
                            crate::vm::ControlFlow::Break { value, .. } => {
                                return Ok(Some(value));
                            }
                            crate::vm::ControlFlow::Return { value, position } => {
                                return Err(MetorexError::NonLocalReturn {
                                    value,
                                    location: position_to_location(position),
                                    home_frame: None,
                                });
                            }
                            _ => {}
                        }
                    }
                    return Ok(Some(receiver.clone()));
                }
                _ => {}
            }
        }
        Ok(None)
    }
}
