// Folding the elements into one value, and pairing them with another
// array.

use super::*;

impl VirtualMachine {
    /// Folding the elements into one value, and pairing them with another
    /// array.
    pub(crate) fn call_array_folding_method(
        &mut self,
        _receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "partition" => {
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
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            "partition requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                let array = array_rc.borrow();
                let mut truthy = Vec::new();
                let mut falsy = Vec::new();
                for element in array.iter() {
                    let args = vec![element.clone()];
                    let value = self.execute_block_callable(&block, args, position)?;
                    if !matches!(value, Object::Bool(false) | Object::Nil) {
                        truthy.push(element.clone());
                    } else {
                        falsy.push(element.clone());
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(vec![
                    Object::Array(Rc::new(RefCell::new(truthy))),
                    Object::Array(Rc::new(RefCell::new(falsy))),
                ])))))
            }
            "inject" | "reduce" => {
                if arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                // The last argument names an operator when there are two of
                // them, or when there is one and no block. The rest is the
                // starting value.
                let names_operator =
                    arguments.len() == 2 || (arguments.len() == 1 && block.is_none());
                let operator = match arguments.last().filter(|_| names_operator) {
                    Some(name) => Some(self.coerce_method_name(name, method_name, position)?),
                    None => None,
                };
                if operator.is_some() && arguments.len() == 2 && block.is_some() {
                    let warning =
                        format!("{}given block not used", self.warning_prefix(0, position));
                    self.emit_warning_to_stderr(&warning, position);
                }
                if operator.is_none() && block.is_none() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let seeded = arguments.len() == 2 || (arguments.len() == 1 && operator.is_none());
                let mut accumulator = if seeded {
                    Some(arguments[0].clone())
                } else {
                    array_rc.borrow().first().cloned()
                };
                // The walk reads one element at a time, so a block that grows
                // the array reaches what it added.
                let mut index = if seeded { 0 } else { 1 };
                loop {
                    let Some(element) = array_rc.borrow().get(index).cloned() else {
                        break;
                    };
                    index += 1;
                    let carried = accumulator.clone().unwrap_or(Object::Nil);
                    accumulator = Some(match (&operator, &block) {
                        (Some(operator), _) => {
                            self.send_to_object(carried, operator, vec![element], position)?
                        }
                        (None, Some(block)) => {
                            self.execute_block_callable(block, vec![carried, element], position)?
                        }
                        (None, None) => unreachable!("a block or an operator was required"),
                    });
                }
                Ok(Some(accumulator.unwrap_or(Object::Nil)))
            }
            "zip" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let held = array_rc.borrow().clone();
                let wanted = held.len();
                // Each argument is put through `to_ary`, and one that answers
                // none is walked with `each`, taking only as many values as
                // this array holds so an endless walk still ends.
                let mut columns = Vec::new();
                for argument in arguments {
                    columns.push(self.zip_column(argument, wanted, position)?);
                }
                let mut results = Vec::new();
                for (index, element) in held.iter().enumerate() {
                    let mut tuple = vec![element.clone()];
                    for column in &columns {
                        tuple.push(column.get(index).cloned().unwrap_or(Object::Nil));
                    }
                    results.push(Object::Array(Rc::new(RefCell::new(tuple))));
                }
                // With a block the rows are handed over one at a time and
                // nothing is answered.
                if let Some(Object::Block(block)) = self.pending_block.take() {
                    for row in results {
                        self.execute_block_callable(&block, vec![row], position)?;
                    }
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            "transpose" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();

                if array.is_empty() {
                    return Ok(Some(Object::Array(Rc::new(RefCell::new(Vec::new())))));
                }

                let elements = array.clone();
                drop(array);
                // Each row goes through `to_ary`, so an object standing for an
                // array transposes the way one does.
                let mut row_arrays = Vec::new();
                for element in &elements {
                    row_arrays.push(self.coerce_to_array(element, position)?);
                }

                // Every row must be the same length, which Ruby reports as an
                // IndexError naming the two it found.
                let max_cols = row_arrays.first().map(|row| row.len()).unwrap_or(0);
                if let Some(odd) = row_arrays.iter().find(|row| row.len() != max_cols) {
                    let message = format!(
                        "element size differs ({} should be {})",
                        odd.len(),
                        max_cols
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "IndexError",
                        &message,
                        position,
                    ));
                }

                let mut transposed = Vec::new();
                for col_idx in 0..max_cols {
                    let mut new_row = Vec::new();
                    for row in &row_arrays {
                        if col_idx < row.len() {
                            new_row.push(row[col_idx].clone());
                        } else {
                            new_row.push(Object::Nil);
                        }
                    }
                    transposed.push(Object::Array(Rc::new(RefCell::new(new_row))));
                }

                Ok(Some(Object::Array(Rc::new(RefCell::new(transposed)))))
            }
            _ => Ok(None),
        }
    }
}
