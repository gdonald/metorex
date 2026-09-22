// Keeping or dropping elements in place, and reading several positions
// at once.

use super::*;

impl VirtualMachine {
    /// Keeping or dropping elements in place, and reading several positions
    /// at once.
    pub(crate) fn call_array_filtering_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `delete_if` and `keep_if` filter in place, and answer the array
            // itself so a chain carries on from it.
            "delete_if" | "keep_if" => {
                // Without a block there is nothing to filter by, so the array
                // is not touched and an Enumerator comes back even from a
                // frozen one.
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                // The survivors are moved forward as the walk goes and the
                // array is cut to length at the end, so it keeps its size
                // while the block runs and holds what was already decided if
                // the block raises.
                let mut kept = 0;
                let mut index = 0;
                let mut outcome = Ok(());
                while index < array_rc.borrow().len() {
                    let element = array_rc.borrow()[index].clone();
                    match self.execute_block_callable(&block, vec![element.clone()], position) {
                        Ok(answer) => {
                            let remove = if method_name == "delete_if" {
                                answer.is_truthy()
                            } else {
                                !answer.is_truthy()
                            };
                            if !remove {
                                array_rc.borrow_mut()[kept] = element;
                                kept += 1;
                            }
                        }
                        Err(error) => {
                            // The element that raised stays, which is where the
                            // walk stopped.
                            let remaining: Vec<Object> = array_rc.borrow()[index..].to_vec();
                            let mut array = array_rc.borrow_mut();
                            array.truncate(kept);
                            array.extend(remaining);
                            outcome = Err(error);
                            break;
                        }
                    }
                    index += 1;
                }
                outcome?;
                array_rc.borrow_mut().truncate(kept);
                Ok(Some(receiver.clone()))
            }
            // `each_index` walks the positions rather than the elements.
            "each_index" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                // The length is read again each step, so an array that grows
                // while it is walked is walked to its new end.
                let mut index = 0;
                while index < array_rc.borrow().len() {
                    self.execute_block_callable(&block, vec![Object::Int(index as i64)], position)?;
                    index += 1;
                }
                Ok(Some(receiver.clone()))
            }
            "reverse_each" => {
                let elements = array_rc.borrow().clone();
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                // Walking backwards reads the position each step, so an array
                // that grows while it is walked keeps its remaining elements
                // lined up with the positions still to come.
                let _ = elements;
                let mut index = array_rc.borrow().len();
                while index > 0 {
                    index -= 1;
                    let element = match array_rc.borrow().get(index) {
                        Some(element) => element.clone(),
                        None => continue,
                    };
                    self.execute_block_callable(&block, vec![element], position)?;
                }
                Ok(Some(receiver.clone()))
            }
            // `values_at` reads several positions at once, answering nil for
            // one the array does not reach.
            // `values_at` takes indexes and Ranges, answering nil for a
            // position the array has no element at.
            "values_at" => {
                let array = array_rc.borrow().clone();
                let length = array.len() as i64;
                let mut picked = Vec::new();
                for argument in arguments {
                    if let Object::Range {
                        start,
                        end,
                        exclusive,
                        ..
                    } = argument
                    {
                        let first = match start.as_ref() {
                            Object::Nil => 0,
                            bound => self.index_from(bound, length, position)?,
                        };
                        // A start before the front names nothing. A start past
                        // the end still fills nil for every position the range
                        // covers, which is what Ruby answers.
                        if first < 0 {
                            continue;
                        }
                        let last = match end.as_ref() {
                            Object::Nil => length - 1,
                            bound => {
                                let resolved = self.index_from(bound, length, position)?;
                                if *exclusive { resolved - 1 } else { resolved }
                            }
                        };
                        for index in first..=last.max(first - 1) {
                            picked.push(array.get(index as usize).cloned().unwrap_or(Object::Nil));
                        }
                        continue;
                    }
                    let index = self.index_from(argument, length, position)?;
                    picked.push(
                        usize::try_from(index)
                            .ok()
                            .and_then(|index| array.get(index).cloned())
                            .unwrap_or(Object::Nil),
                    );
                }
                Ok(Some(Object::array(picked)))
            }
            // `intersect?` asks whether the two arrays share an element.
            "intersect?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = self.coerce_to_array(&arguments[0], position)?;
                // Ruby matches on `hash` and `eql?`, so an object that
                // answers `eql?` decides for itself.
                let elements = array_rc.borrow().clone();
                let mut shared = false;
                'outer: for element in &elements {
                    for candidate in &other {
                        // The same object is shared whatever its `eql?` says,
                        // which is what Ruby's hash lookup amounts to.
                        if identical(element, candidate)
                            || self.values_eql(element, candidate, position)?
                        {
                            shared = true;
                            break 'outer;
                        }
                    }
                }
                Ok(Some(Object::Bool(shared)))
            }
            _ => Ok(None),
        }
    }
}
