// Finding an element, and the elements that appear only once.

use super::*;

impl VirtualMachine {
    /// Finding an element, and the elements that appear only once.
    pub(crate) fn call_array_searching_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `assoc` and `rassoc` look through an array of arrays, matching
            // the first element or the second.
            "assoc" | "rassoc" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // The row's own element is asked `==`, so an object of the
                // program's own answers, and the row itself comes back rather
                // than a copy of it.
                let slot = usize::from(method_name == "rassoc");
                let rows = array_rc.borrow().clone();
                for element in rows {
                    // A row that is not an Array is asked for one, and what it
                    // answers is what comes back when it matches.
                    let row = match &element {
                        Object::Array(_) => element.clone(),
                        other if self.responds_to(other, "to_ary") => {
                            self.send_to_object(other.clone(), "to_ary", vec![], position)?
                        }
                        _ => continue,
                    };
                    let Object::Array(pair) = &row else {
                        continue;
                    };
                    let candidate = pair.borrow().get(slot).cloned();
                    let Some(candidate) = candidate else {
                        continue;
                    };
                    if self.elements_equal(&candidate, &arguments[0], position)? {
                        return Ok(Some(row));
                    }
                }
                Ok(Some(Object::Nil))
            }
            // The array is read a position at a time rather than held open,
            // since the block or the `==` it runs may change it.
            "index" | "find_index" => {
                if arguments.len() == 1 {
                    self.warn_unused_block(position)?;
                    let elements = array_rc.borrow().clone();
                    for (index, element) in elements.iter().enumerate() {
                        if self.elements_equal(element, &arguments[0], position)? {
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
                let mut index = 0;
                while index < array_rc.borrow().len() {
                    let element = array_rc.borrow()[index].clone();
                    let answer = self.execute_block_callable(&block, vec![element], position)?;
                    if answer.is_truthy() {
                        return Ok(Some(Object::Int(index as i64)));
                    }
                    index += 1;
                }
                Ok(Some(Object::Nil))
            }
            // Ruby asks each element whether it is `==` to what it was given,
            // in order, so an object that defines `==` decides for itself.
            // An Array is already the array these ask for.
            "to_ary" | "deconstruct" => {
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
            "include?" | "contains?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = array_rc.borrow().clone();
                for element in elements {
                    if self.elements_equal(&element, &arguments[0], position)? {
                        return Ok(Some(Object::Bool(true)));
                    }
                }
                Ok(Some(Object::Bool(false)))
            }
            "uniq" => {
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
                Ok(Some(Object::array(unique)))
            }
            // `min`, `max`, and `minmax` order with `<=>`, or with the block
            // when one is given.
            "min" | "max" | "minmax" => {
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
                if elements.is_empty() {
                    return Ok(Some(match method_name {
                        "minmax" => Object::array(vec![Object::Nil, Object::Nil]),
                        _ => Object::Nil,
                    }));
                }
                let wants_min = matches!(method_name, "min" | "minmax");
                let wants_max = matches!(method_name, "max" | "minmax");
                let mut smallest = elements[0].clone();
                let mut largest = elements[0].clone();
                for element in &elements[1..] {
                    if wants_min && self.compare_elements(element, &smallest, &block, position)? < 0
                    {
                        smallest = element.clone();
                    }
                    if wants_max && self.compare_elements(element, &largest, &block, position)? > 0
                    {
                        largest = element.clone();
                    }
                }
                Ok(Some(match method_name {
                    "min" => smallest,
                    "max" => largest,
                    _ => Object::array(vec![smallest, largest]),
                }))
            }
            _ => Ok(None),
        }
    }
}
