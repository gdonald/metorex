// Putting the elements in order.

use super::*;

impl VirtualMachine {
    /// Putting the elements in order.
    pub(crate) fn call_array_ordering_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "size" => {
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
            "unshift" | "prepend" => {
                // Ruby: unshift(*items) prepends all items in order, accepts 0+ args
                let mut arr = array_rc.borrow_mut();
                for (i, item) in arguments.iter().enumerate() {
                    arr.insert(i, item.clone());
                }
                drop(arr);
                Ok(Some(receiver.clone()))
            }
            "sort" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // A block decides the order, answering negative, zero, or
                // positive the way `<=>` does.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let elements = array_rc.borrow().clone();
                let sorted = self.sort_elements(elements, block, position)?;
                Ok(Some(Object::array(sorted)))
            }
            "sort_by" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    None => {
                        return Err(MetorexError::runtime_error(
                            "sort_by requires a block",
                            position_to_location(position),
                        ));
                    }
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                };
                let mut keyed: Vec<(Object, Object)> = Vec::new();
                for element in array_rc.borrow().iter() {
                    let key =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
                    keyed.push((key, element.clone()));
                }
                let sorted = self.sort_by_keys(keyed, position)?;
                Ok(Some(Object::Array(Rc::new(RefCell::new(sorted)))))
            }
            "reverse" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut reversed = array_rc.borrow().clone();
                reversed.reverse();
                Ok(Some(Object::Array(Rc::new(RefCell::new(reversed)))))
            }
            _ => Ok(None),
        }
    }
}
