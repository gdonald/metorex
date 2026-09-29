// Building an Array, Hash or Set from a class method.

use super::*;

impl VirtualMachine {
    /// `Array[]`, `Hash[]`, `Set[]` and the `new` each of them answers.
    pub(crate) fn call_collection_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // Array.new — `new(size)`, `new(size, default)`, `new(size) { |i| ... }`.
        // Without arguments, returns an empty array.
        // `Set[1, 2, 3]` builds a set from the arguments directly.
        if method_name == "[]" && class_rc.name() == "Set" {
            let elements = Object::array(arguments.to_vec());
            return self
                .send_to_object(
                    Object::Class(Rc::clone(class_rc)),
                    "new",
                    vec![elements],
                    position,
                )
                .map(Answered);
        }
        // `Hash[...]` builds a Hash from a single Hash, from an array of
        // pairs, or from an even number of key and value arguments.
        // `Hash.ruby2_keywords_hash` marks a copy of a hash as one that was
        // gathered from keyword arguments, and the predicate reports it.
        if matches!(method_name, "ruby2_keywords_hash" | "ruby2_keywords_hash?")
            && crate::vm::method_invocation::descends_from(class_rc, "Hash")
        {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let Some(Object::Dict(pairs)) = crate::vm::native_methods::as_dict(&arguments[0])
            else {
                let message = format!(
                    "wrong argument type {} (expected Hash)",
                    self.builtins().class_of(&arguments[0]).name()
                );
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    position,
                ));
            };
            if method_name == "ruby2_keywords_hash?" {
                return Ok(Answered(Object::Bool(pairs.borrow().contains_key(
                    crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY,
                ))));
            }
            let copy = self.call_object_method(&arguments[0], "dup", &[], position)?;
            let made = copy.unwrap_or_else(|| arguments[0].clone());
            if let Some(Object::Dict(copied)) = crate::vm::native_methods::as_dict(&made) {
                // The copy is a Hash of its own, not the keywords the call
                // handed over.
                copied
                    .borrow_mut()
                    .shift_remove(crate::vm::param_binding::KWARGS_MARKER);
                copied.borrow_mut().insert(
                    crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY.to_string(),
                    Object::Bool(true),
                );
            }
            return Ok(Answered(made));
        }
        if method_name == "[]" && crate::vm::method_invocation::descends_from(class_rc, "Hash") {
            let mut entries: Vec<(Object, Object)> = Vec::new();
            match arguments {
                [] => {}
                [Object::Dict(source)] => {
                    for (key, value) in source.borrow().iter() {
                        if key.starts_with("__MX_") {
                            continue;
                        }
                        entries.push((Object::string(key.clone()), value.clone()));
                    }
                    let mut built = indexmap::IndexMap::new();
                    for (key, value) in entries {
                        let Object::String(rendered) = key else {
                            continue;
                        };
                        built.insert(rendered.as_str().to_string(), value);
                    }
                    return Ok(Answered(hash_of_class(class_rc, built)));
                }
                [Object::Array(rows)] => {
                    let held = rows.borrow().clone();
                    for (at, row) in held.iter().enumerate() {
                        // Every element names a pair, so anything that is not
                        // one, and any pair with the wrong number of parts,
                        // is refused where it stands.
                        let Object::Array(pair) = row else {
                            let message = format!(
                                "wrong element type {} at {} (expected array)",
                                match row {
                                    Object::Nil => "nil".to_string(),
                                    other => self.builtins().class_of(other).ruby_name(),
                                },
                                at
                            );
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &message,
                                position,
                            ));
                        };
                        let pair = pair.borrow();
                        if pair.is_empty() || pair.len() > 2 {
                            let message =
                                format!("invalid number of elements ({} for 1..2)", pair.len());
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &message,
                                position,
                            ));
                        }
                        entries.push((
                            pair.first().cloned().unwrap_or(Object::Nil),
                            pair.get(1).cloned().unwrap_or(Object::Nil),
                        ));
                    }
                }
                // An instance of a Hash subclass is read by the entries it
                // holds, whatever `to_hash` it was given.
                [single] if crate::vm::native_methods::hash_subclass_value(single).is_some() => {
                    let held = crate::vm::native_methods::hash_subclass_value(single)
                        .expect("a hash subclass carries its entries");
                    return self
                        .call_class_methods(class_rc, method_name, &[held], position)
                        .map(nested_answer);
                }
                // A single argument that reads as a hash, or as an array of
                // pairs, is read as one before anything else is tried.
                [single]
                    if !matches!(single, Object::Dict(_) | Object::Array(_))
                        && (self.responds_to(single, "to_hash")
                            || self.responds_to(single, "to_ary")) =>
                {
                    let named = if self.responds_to(single, "to_hash") {
                        "to_hash"
                    } else {
                        "to_ary"
                    };
                    let read = self.send_to_object(single.clone(), named, vec![], position)?;
                    return self
                        .call_class_methods(class_rc, method_name, &[read], position)
                        .map(nested_answer);
                }
                values if values.len().is_multiple_of(2) => {
                    for pair in values.chunks(2) {
                        entries.push((pair[0].clone(), pair[1].clone()));
                    }
                }
                _ => {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "odd number of arguments for Hash",
                        position,
                    ));
                }
            }
            let mut built = indexmap::IndexMap::new();
            for (key, value) in entries {
                let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                built.insert(rendered, value);
            }
            return Ok(Answered(hash_of_class(class_rc, built)));
        }
        // `Array[1, 2, 3]` and the same form on a subclass build a value from
        // the arguments directly, without running `initialize`.
        if method_name == "[]" && crate::vm::method_invocation::descends_from(class_rc, "Array") {
            let elements = arguments.to_vec();
            if class_rc.name() == "Array" {
                return Ok(Answered(Object::array(elements)));
            }
            let instance = crate::object::Instance::new(Rc::clone(class_rc));
            instance.borrow_mut().set_var(
                crate::vm::native_methods::ARRAY_SUBCLASS_VAR.to_string(),
                Object::array(elements),
            );
            return Ok(Answered(Object::Instance(instance)));
        }
        if method_name == "new" && class_rc.name() == "Array" {
            let elements = self.build_array_elements(arguments, position)?;
            return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                elements,
            )))));
        }
        if method_name == "new" && class_rc.name() == "Set" {
            use crate::object::ObjectHash;
            let mut set: indexmap::IndexSet<ObjectHash> = indexmap::IndexSet::new();
            if arguments.len() == 1 {
                // An Array is taken as it is, and anything else that walks is
                // asked for one.
                let items = match &arguments[0] {
                    Object::Array(elements) => elements.borrow().clone(),
                    Object::Nil => Vec::new(),
                    // Ruby walks the seed with `each_entry`, falling back to
                    // `each`, and refuses anything that answers neither.
                    other => {
                        let walked = if self.responds_to(other, "each_entry") {
                            "each_entry"
                        } else if self.responds_to(other, "each") {
                            "each"
                        } else {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "value must be enumerable",
                                position,
                            ));
                        };
                        let walk = self.send_to_object(
                            other.clone(),
                            "to_enum",
                            vec![Object::symbol(walked.to_string())],
                            position,
                        )?;
                        match self.send_to_object(walk, "to_a", vec![], position)? {
                            Object::Array(elements) => elements.borrow().clone(),
                            _ => Vec::new(),
                        }
                    }
                };
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                for item in items {
                    // `Set.new(list) { |item| ... }` stores what the block
                    // answers rather than the item itself.
                    let item = match &block {
                        Some(block) => self.execute_block_callable(block, vec![item], position)?,
                        None => item,
                    };
                    if let Some(hash) = ObjectHash::from_object(&item) {
                        set.insert(hash);
                    } else {
                        return Err(MetorexError::runtime_error(
                            format!("Cannot add {} to set (not hashable)", item.type_name()),
                            position_to_location(position),
                        ));
                    }
                }
            } else if arguments.len() > 1 {
                return Err(MetorexError::runtime_error(
                    format!("Set.new expects 0-1 arguments, got {}", arguments.len()),
                    position_to_location(position),
                ));
            }
            return Ok(Answered(Object::Set(Rc::new(std::cell::RefCell::new(set)))));
        }
        Ok(Unclaimed)
    }
}
