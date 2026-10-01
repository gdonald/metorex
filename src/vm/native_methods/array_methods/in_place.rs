// The variants that write their answer back into the receiver.

use super::*;

impl VirtualMachine {
    /// The variants that write their answer back into the receiver.
    pub(crate) fn call_array_in_place_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // The in-place variants. Each writes the new elements back into
            // the receiver; the ones Ruby documents as answering nil when
            // nothing changed do so here too.
            "compact!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let kept: Vec<Object> = array_rc
                    .borrow()
                    .iter()
                    .filter(|element| !matches!(element, Object::Nil))
                    .cloned()
                    .collect();
                let changed = kept.len() != array_rc.borrow().len();
                *array_rc.borrow_mut() = kept;
                Ok(Some(if changed {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
            }
            "reverse!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                array_rc.borrow_mut().reverse();
                Ok(Some(receiver.clone()))
            }
            "sort!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let elements = array_rc.borrow().clone();
                let sorted = self.sort_elements(elements, block, position)?;
                *array_rc.borrow_mut() = sorted;
                Ok(Some(receiver.clone()))
            }
            "sort_by!" => {
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
                let mut keyed: Vec<(Object, Object)> = Vec::new();
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let key =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
                    keyed.push((key, element));
                }
                *array_rc.borrow_mut() = self.sort_by_keys(keyed, position)?;
                Ok(Some(receiver.clone()))
            }
            "map!" | "collect!" => {
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
                // Each element is replaced as the block answers for it, so an
                // array left behind by a break or an exception holds the
                // results so far and the originals after them.
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    let value = self.execute_block_callable(&block, vec![element], position)?;
                    array_rc.borrow_mut()[index] = value;
                    index += 1;
                }
                Ok(Some(receiver.clone()))
            }
            "reject!" | "select!" | "filter!" => {
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
                let rejecting = method_name == "reject!";
                let before = array_rc.borrow().len();
                let kept = self.filter_in_place(array_rc, &block, rejecting, position)?;
                Ok(Some(if kept != before {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
            }
            "uniq!" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let unique = self.unique_live_elements(array_rc, block, position)?;
                let changed = unique.len() != array_rc.borrow().len();
                *array_rc.borrow_mut() = unique;
                Ok(Some(if changed {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
            }
            "rotate" | "rotate!" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let by = match arguments.first() {
                    None => 1,
                    Some(Object::Int(count)) => *count,
                    Some(other) => {
                        let coerced = self.coerce_integer_argument(other, position)?;
                        coerced.try_into().unwrap_or(0)
                    }
                };
                let elements = array_rc.borrow().clone();
                let rotated = rotate_elements(&elements, by);
                if method_name == "rotate!" {
                    *array_rc.borrow_mut() = rotated;
                    return Ok(Some(receiver.clone()));
                }
                Ok(Some(Object::array(rotated)))
            }
            "shuffle" | "shuffle!" => {
                // A `random:` keyword names the object each index is drawn
                // from, which Ruby asks for `rand` rather than drawing itself.
                let mut source = None;
                let mut positional = arguments;
                if let Some(Object::Dict(entries)) = arguments.last()
                    && let Some(named) = entries.borrow().get(":random").cloned()
                {
                    source = Some(named);
                    positional = &arguments[..arguments.len() - 1];
                }
                if !positional.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        positional.len(),
                        position,
                    ));
                }
                let mut elements = array_rc.borrow().clone();
                for index in (1..elements.len()).rev() {
                    let swap =
                        self.random_upto(source.as_ref(), index as i64 + 1, position)? as usize;
                    elements.swap(index, swap);
                }
                if method_name == "shuffle!" {
                    *array_rc.borrow_mut() = elements;
                    return Ok(Some(receiver.clone()));
                }
                Ok(Some(Object::array(elements)))
            }
            "sample" => {
                let (count, source) = self.sample_arguments(method_name, arguments, position)?;
                let mut elements = array_rc.borrow().clone();
                let Some(count) = count else {
                    if elements.is_empty() {
                        return Ok(Some(Object::Nil));
                    }
                    let index =
                        self.random_upto(source.as_ref(), elements.len() as i64, position)?
                            as usize;
                    return Ok(Some(elements[index].clone()));
                };
                if count < 0 {
                    return Err(simple_exception(
                        "ArgumentError",
                        "negative sample number",
                        position,
                    ));
                }
                // Drawing without replacement is a partial Fisher-Yates: each
                // pick is swapped to the front, so the stretch already taken
                // is never drawn from again.
                let wanted = (count as usize).min(elements.len());
                for taken in 0..wanted {
                    let remaining = (elements.len() - taken) as i64;
                    let drawn =
                        taken + self.random_upto(source.as_ref(), remaining, position)? as usize;
                    elements.swap(taken, drawn);
                }
                elements.truncate(wanted);
                Ok(Some(Object::array(elements)))
            }
            _ => Ok(None),
        }
    }
}
