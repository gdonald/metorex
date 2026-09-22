// The elements at either end, and removing one by position or value.

use super::*;

impl VirtualMachine {
    /// The elements at either end, and removing one by position or value.
    pub(crate) fn call_array_edge_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "empty?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(array_rc.borrow().is_empty())))
            }
            // `first` and `last` answer one element, or the first or last
            // `count` of them when given a count.
            "first" | "last" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = array_rc.borrow().clone();
                let Some(argument) = arguments.first() else {
                    let element = match method_name {
                        "first" => elements.first(),
                        _ => elements.last(),
                    };
                    return Ok(Some(element.cloned().unwrap_or(Object::Nil)));
                };
                let count = self.coerce_integer_argument(argument, position)?;
                // A count too large for a machine word is a RangeError, which
                // is what Ruby raises before it looks at the array at all.
                let Ok(count) = i64::try_from(&count) else {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "bignum too big to convert into `long'",
                        position,
                    ));
                };
                if count < 0 {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative array size",
                        position,
                    ));
                }
                let count = (count as usize).min(elements.len());
                let taken = match method_name {
                    "first" => elements[..count].to_vec(),
                    _ => elements[elements.len() - count..].to_vec(),
                };
                Ok(Some(Object::array(taken)))
            }
            // `take(n)` answers the first n elements, and `drop(n)` the rest.
            // A negative count raises, as Ruby does.
            "take" | "drop" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // The count is read the way `Integer()` reads one, so an
                // object carrying `to_int` counts.
                let count = self.coerce_integer_argument(&arguments[0], position)?;
                let count = &i64::try_from(&count).unwrap_or(i64::MAX);
                if *count < 0 {
                    let message = format!("attempt to {} negative size", method_name);
                    let exception = Object::exception("ArgumentError", message.clone());
                    return Err(MetorexError::UncaughtException {
                        exception,
                        location: position_to_location(position),
                        message,
                    });
                }
                let borrowed = array_rc.borrow();
                let count = (*count as usize).min(borrowed.len());
                let taken = if method_name == "take" {
                    borrowed[..count].to_vec()
                } else {
                    borrowed[count..].to_vec()
                };
                Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(taken)))))
            }
            // `rindex` scans from the end, answering the last position that
            // matches rather than the first.
            "rindex" => {
                if arguments.len() == 1 {
                    self.warn_unused_block(position)?;
                    let elements = array_rc.borrow().clone();
                    for index in (0..elements.len()).rev() {
                        if self.elements_equal(&elements[index], &arguments[0], position)? {
                            return Ok(Some(Object::Int(index as i64)));
                        }
                    }
                    return Ok(Some(Object::Nil));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                // Walking backwards reads the position each step, so an array
                // the block shortens is still walked safely.
                let mut index = array_rc.borrow().len();
                while index > 0 {
                    index -= 1;
                    let Some(element) = array_rc.borrow().get(index).cloned() else {
                        continue;
                    };
                    let answer = self.execute_block_callable(&block, vec![element], position)?;
                    if answer.is_truthy() {
                        return Ok(Some(Object::Int(index as i64)));
                    }
                }
                Ok(Some(Object::Nil))
            }
            // `concat` appends every element of each array it is given.
            "concat" => {
                let mut added = Vec::new();
                for argument in arguments {
                    let other = self.coerce_to_array(argument, position)?;
                    added.extend(other);
                }
                array_rc.borrow_mut().extend(added);
                Ok(Some(receiver.clone()))
            }
            // `delete_at` removes the element at one index and answers it.
            // `slice!` cuts the part a subscript names out of the array and
            // answers it, leaving the rest in place.
            "slice!" => {
                let size = array_rc.borrow().len();
                let Some((from, width, one)) = self.slice_span(arguments, size, position)? else {
                    return Ok(Some(Object::Nil));
                };
                let taken: Vec<Object> = array_rc.borrow_mut().drain(from..from + width).collect();
                Ok(Some(if one {
                    taken.into_iter().next().unwrap_or(Object::Nil)
                } else {
                    Object::array(taken)
                }))
            }
            "delete_at" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let index = self.coerce_integer_argument(&arguments[0], position)?;
                let index = Object::integer(index);
                let length = array_rc.borrow().len() as i64;
                let Some(index) = normalize_index(&index, length) else {
                    return Ok(Some(Object::Nil));
                };
                Ok(Some(array_rc.borrow_mut().remove(index)))
            }
            _ => Ok(None),
        }
    }
}
