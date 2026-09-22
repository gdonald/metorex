// The smallest, largest and how many, and the sets two arrays make.

use super::*;

impl VirtualMachine {
    /// The smallest, largest and how many, and the sets two arrays make.
    pub(crate) fn call_array_aggregate_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `count` answers how many elements there are, how many equal the
            // argument, or how many the block answers for.
            "count" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Some(wanted) = arguments.first() {
                    self.warn_unused_block(position)?;
                    let elements = array_rc.borrow().clone();
                    let mut counted = 0;
                    for element in &elements {
                        if self.elements_equal(element, wanted, position)? {
                            counted += 1;
                        }
                    }
                    return Ok(Some(Object::Int(counted)));
                }
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return Ok(Some(Object::Int(array_rc.borrow().len() as i64)));
                };
                let mut counted = 0;
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let verdict = self.execute_block_callable(&block, vec![element], position)?;
                    if verdict.is_truthy() {
                        counted += 1;
                    }
                }
                Ok(Some(Object::Int(counted)))
            }
            // `take_while` keeps the leading run the block answers for, and
            // `drop_while` answers what is left after it.
            "take_while" | "drop_while" => {
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
                let mut taken = Vec::new();
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    let verdict =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
                    if !verdict.is_truthy() {
                        break;
                    }
                    taken.push(element);
                    index += 1;
                }
                if method_name == "take_while" {
                    return Ok(Some(Object::array(taken)));
                }
                let rest = array_rc.borrow()[index.min(array_rc.borrow().len())..].to_vec();
                Ok(Some(Object::array(rest)))
            }
            // `union`, `intersection`, and `difference` fold the matching
            // operator over every argument, so each one is coerced and matched
            // the same way `|`, `&`, and `-` are.
            "union" | "intersection" | "difference" => {
                let operator = match method_name {
                    "union" => crate::ast::BinaryOp::BitwiseOr,
                    "intersection" => crate::ast::BinaryOp::BitwiseAnd,
                    _ => crate::ast::BinaryOp::Subtract,
                };
                // With no arguments each answers a copy, which for `union` and
                // `intersection` has its duplicates dropped.
                let mut folded = if arguments.is_empty() && method_name != "difference" {
                    let elements = array_rc.borrow().clone();
                    let mut unique = Vec::new();
                    for element in &elements {
                        if !self.contains_eql(&unique, element, position)? {
                            unique.push(element.clone());
                        }
                    }
                    Object::array(unique)
                } else {
                    Object::array(array_rc.borrow().clone())
                };
                for argument in arguments {
                    folded = self.evaluate_binary_operation(
                        &operator,
                        folded,
                        argument.clone(),
                        position,
                    )?;
                }
                Ok(Some(folded))
            }
            // `fetch(index)` raises when the index names no element, unless a
            // default value or a block says what to answer instead.
            "fetch" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // The block is set aside first, since coercing the index runs
                // `to_int` on an object of the program's own, which would
                // otherwise take the block for itself.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let index: i64 = self
                    .coerce_integer_argument(&arguments[0], position)?
                    .try_into()
                    .unwrap_or(i64::MAX);
                let length = array_rc.borrow().len() as i64;
                let resolved = if index < 0 { index + length } else { index };
                if resolved >= 0 && resolved < length {
                    if block.is_some() {
                        self.pending_block = None;
                        let file = self
                            .current_source_file
                            .clone()
                            .unwrap_or_else(|| "-".to_string());
                        let message = format!(
                            "{}:{}: warning: given block not used\n",
                            file, position.line
                        );
                        self.warn_through_warning_module(message, position)?;
                    }
                    return Ok(Some(array_rc.borrow()[resolved as usize].clone()));
                }
                // A block wins over a default value, and Ruby says so.
                if let Some(block) = block {
                    if arguments.len() == 2 {
                        let file = self
                            .current_source_file
                            .clone()
                            .unwrap_or_else(|| "-".to_string());
                        let message = format!(
                            "{}:{}: warning: block supersedes default value argument\n",
                            file, position.line
                        );
                        self.warn_through_warning_module(message, position)?;
                    }
                    return self
                        .execute_block_callable(&block, vec![arguments[0].clone()], position)
                        .map(Some);
                }
                if let Some(fallback) = arguments.get(1) {
                    return Ok(Some(fallback.clone()));
                }
                let message = format!(
                    "index {} outside of array bounds: {}...{}",
                    index, -length, length
                );
                Err(crate::vm::errors::simple_exception(
                    "IndexError",
                    &message,
                    position,
                ))
            }
            // `fetch_values(*indexes)` fetches each one, so a missing index
            // raises unless the block says what to answer for it.
            "fetch_values" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut fetched = Vec::with_capacity(arguments.len());
                for wanted in arguments {
                    let index: i64 = self
                        .coerce_integer_argument(wanted, position)?
                        .try_into()
                        .unwrap_or(i64::MAX);
                    let length = array_rc.borrow().len() as i64;
                    let resolved = if index < 0 { index + length } else { index };
                    if resolved >= 0 && resolved < length {
                        fetched.push(array_rc.borrow()[resolved as usize].clone());
                        continue;
                    }
                    let Some(block) = &block else {
                        let message = format!(
                            "index {} outside of array bounds: {}...{}",
                            index, -length, length
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "IndexError",
                            &message,
                            position,
                        ));
                    };
                    let block = Rc::clone(block);
                    fetched.push(self.execute_block_callable(
                        &block,
                        vec![wanted.clone()],
                        position,
                    )?);
                }
                Ok(Some(Object::array(fetched)))
            }
            _ => Ok(None),
        }
    }
}
