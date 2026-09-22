// How long an array is, what it reads back as, and putting elements
// on either end of it.

use super::*;

impl VirtualMachine {
    /// How long an array is, what it reads back as, and putting elements
    /// on either end of it.
    pub(crate) fn call_array_basic_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(array_rc.borrow().len() as i64)))
            }
            "inspect" | "to_s" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = array_rc.borrow().clone();
                let address = Rc::as_ptr(array_rc) as usize;
                // An array that reaches itself prints `[...]` rather than
                // recursing forever, which is what Ruby shows.
                if crate::object::rendering_in_progress(address) {
                    return Ok(Some(Object::string("[...]")));
                }
                crate::object::begin_rendering(address);
                // Each element renders through its own `inspect`, so an
                // object that defines one is shown the way it asks to be.
                let mut parts = Vec::with_capacity(elements.len());
                // An array with nothing in it spells the same in every
                // encoding, and one with something in it is written in the
                // encoding the first element was rendered in.
                let mut writing = "US-ASCII".to_string();
                for element in &elements {
                    let rendered = self.inspected_object(element, position);
                    match rendered {
                        Ok(rendered) => {
                            if parts.is_empty()
                                && let Object::String(text) = &rendered
                            {
                                writing = text.encoding_name();
                            }
                            parts.push(match &rendered {
                                Object::String(text) => text.to_string(),
                                other => format!("{}", other),
                            });
                        }
                        Err(error) => {
                            crate::object::end_rendering();
                            return Err(error);
                        }
                    }
                }
                crate::object::end_rendering();
                let made = crate::object::StringValue::with_encoding(
                    format!("[{}]", parts.join(", ")),
                    writing,
                );
                Ok(Some(Object::String(Rc::new(made))))
            }
            "clear" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                array_rc.borrow_mut().clear();
                Ok(Some(receiver.clone()))
            }
            // `ary.send :initialize, ...` builds the elements the way
            // `Array.new` does and puts them in place of what is there.
            "initialize" => {
                // A `break` out of the block leaves the array holding what
                // the block had answered up to then, so the elements are put
                // in place whether the run finished or not.
                let mut elements = Vec::new();
                let outcome = self.collect_array_elements(arguments, position, &mut elements);
                if outcome.is_ok() || !elements.is_empty() {
                    let mut held = array_rc.borrow_mut();
                    held.clear();
                    held.extend(elements);
                }
                outcome?;
                Ok(Some(receiver.clone()))
            }
            "replace" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = self.coerce_to_array(&arguments[0], position)?;
                let mut arr = array_rc.borrow_mut();
                arr.clear();
                arr.extend(other);
                drop(arr);
                Ok(Some(receiver.clone()))
            }
            // Ruby asks each element whether it is `==` to the object, and
            // runs the block when nothing matched.
            "delete" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let elements = array_rc.borrow().clone();
                let mut kept = Vec::new();
                let mut found = false;
                for element in elements {
                    if self.elements_equal(&element, &arguments[0], position)? {
                        found = true;
                        continue;
                    }
                    kept.push(element);
                }
                // A frozen array refuses the removal, but only when there is
                // one to make.
                if found {
                    if self.object_is_frozen(receiver) {
                        return Err(self.frozen_modification_error(receiver, position));
                    }
                    *array_rc.borrow_mut() = kept;
                    return Ok(Some(arguments[0].clone()));
                }
                match block {
                    Some(block) => self
                        .execute_block_callable(&block, vec![arguments[0].clone()], position)
                        .map(Some),
                    None => Ok(Some(Object::Nil)),
                }
            }
            "push" | "append" | "<<" => {
                if arguments.is_empty() {
                    return Ok(Some(receiver.clone()));
                }
                let mut arr = array_rc.borrow_mut();
                for arg in arguments {
                    arr.push(arg.clone());
                }
                drop(arr);
                Ok(Some(receiver.clone()))
            }
            // `pop` and `shift` take an optional count, and then answer an
            // array of what they removed rather than a single element.
            "pop" | "shift" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let from_front = method_name == "shift";
                let Some(count) = arguments.first() else {
                    let mut array = array_rc.borrow_mut();
                    if array.is_empty() {
                        return Ok(Some(Object::Nil));
                    }
                    return Ok(Some(if from_front {
                        array.remove(0)
                    } else {
                        array.pop().unwrap_or(Object::Nil)
                    }));
                };
                let wanted: i64 = self
                    .coerce_integer_argument(count, position)?
                    .try_into()
                    .unwrap_or(i64::MAX);
                if wanted < 0 {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative array size",
                        position,
                    ));
                }
                let mut array = array_rc.borrow_mut();
                let taken = (wanted as usize).min(array.len());
                let kept = array.len() - taken;
                let removed: Vec<Object> = if from_front {
                    array.drain(..taken).collect()
                } else {
                    array.split_off(kept)
                };
                Ok(Some(Object::array(removed)))
            }
            _ => Ok(None),
        }
    }
}
