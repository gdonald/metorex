// Whether the block holds for the elements, and the values an array
// converts into.

use super::*;

impl VirtualMachine {
    /// Whether the block holds for the elements, and the values an array
    /// converts into.
    pub(crate) fn call_array_predicate_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "any?" | "all?" | "none?" | "one?" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // With a pattern argument each element is tested with
                // `pattern === element` and any block is ignored.
                let pattern = arguments.first().cloned();
                // A pattern is what decides the answer, so a block handed in
                // alongside one is unused and Ruby says so.
                if pattern.is_some() {
                    self.warn_unused_block(position)?;
                }
                let block = self.pending_block.take();
                let truthy = |v: &Object| !matches!(v, Object::Bool(false) | Object::Nil);
                let mut any_true = false;
                let mut all_true = true;
                let mut true_count = 0usize;
                let mut was_empty = true;
                // The block is free to grow the array it is walking, so each
                // element is read by index rather than through a borrow held
                // across the call.
                let mut index = 0usize;
                loop {
                    let element = {
                        let array = array_rc.borrow();
                        was_empty = was_empty && array.is_empty();
                        match array.get(index) {
                            Some(element) => element.clone(),
                            None => break,
                        }
                    };
                    index += 1;
                    let element = &element;
                    let result = match (&pattern, &block) {
                        (Some(pattern), _) => self.evaluate_binary_operation(
                            &crate::ast::BinaryOp::CaseEqual,
                            pattern.clone(),
                            element.clone(),
                            position,
                        )?,
                        (None, Some(Object::Block(b))) => {
                            let args = vec![element.clone()];
                            self.execute_block_callable(b, args, position)?
                        }
                        _ => element.clone(),
                    };
                    if truthy(&result) {
                        any_true = true;
                        true_count += 1;
                    } else {
                        all_true = false;
                    }
                }
                let value = match method_name {
                    "any?" => any_true,
                    "all?" => was_empty || all_true,
                    "none?" => !any_true,
                    "one?" => true_count == 1,
                    _ => unreachable!(),
                };
                Ok(Some(Object::Bool(value)))
            }
            // `pack` writes the items out as the directives describe them.
            "pack" => {
                // `buffer:` names a String the result is written into, which
                // is the object the call answers.
                let mut arguments = arguments;
                let mut buffer = None;
                if let Some(Object::Dict(entries)) = arguments.last()
                    && let Some(named) = entries.borrow().get(":buffer").cloned()
                {
                    if !matches!(named, Object::String(_)) {
                        let holds = self.builtins().class_of(&named).ruby_name();
                        return Err(simple_exception(
                            "TypeError",
                            &format!("buffer must be String, not {holds}"),
                            position,
                        ));
                    }
                    buffer = Some(named);
                    arguments = &arguments[..arguments.len() - 1];
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let format = match &arguments[0] {
                    Object::String(format) => format.as_str().to_string(),
                    other if self.responds_to(other, "to_str") => {
                        match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                            Object::String(format) => format.as_str().to_string(),
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "String",
                                    other,
                                    position,
                                ));
                            }
                        }
                    }
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                let items = array_rc.borrow().clone();
                let packed = self.array_pack(&items, &format, position)?;
                let Some(buffer) = buffer else {
                    return Ok(Some(packed));
                };
                self.pack_into_buffer(buffer, &format, &packed, position)
                    .map(Some)
            }
            // `to_a` and `entries` answer the array itself, which is what
            // Ruby returns for an Array that is not a subclass instance.
            "to_a" | "entries" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(receiver.clone()))
            }
            // `to_set` collects the elements into a Set, passing each through
            // the block first when one is given.
            "to_set" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A class given as the first argument builds the set instead
                // of Set itself, which Ruby warns about.
                let set_class = match arguments.first() {
                    Some(named @ Object::Class(_)) => {
                        let file = self
                            .current_source_file
                            .clone()
                            .unwrap_or_else(|| "-".to_string());
                        let message = format!(
                            "{}:{}: warning: passing arguments to Enumerable#to_set is deprecated\n",
                            file, position.line
                        );
                        self.warn_through_warning_module(message, position)?;
                        Some(named.clone())
                    }
                    _ => None,
                };
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut collected = Vec::new();
                for element in array_rc.borrow().clone() {
                    collected.push(match &block {
                        None => element,
                        Some(block) => {
                            self.execute_block_callable(block, vec![element], position)?
                        }
                    });
                }
                let set_class = match set_class {
                    Some(named) => named,
                    None => match self.globals().get("Set") {
                        Some(found) => found,
                        None => return Ok(None),
                    },
                };
                self.send_to_object(set_class, "new", vec![Object::array(collected)], position)
                    .map(Some)
            }
            _ => Ok(None),
        }
    }
}
