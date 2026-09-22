// Walking the elements, and the arrays a walk builds.

use super::*;

impl VirtualMachine {
    /// Walking the elements, and the arrays a walk builds.
    pub(crate) fn call_array_iteration_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "each" => {
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
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                };
                let mut break_value: Option<Object> = None;
                // Read one element at a time rather than holding a borrow, so
                // the block may append to the array it is walking, which Ruby
                // allows and the walk then visits.
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let args = vec![element];
                    match self.execute_block_with_control_flow(&block, args, position)? {
                        crate::vm::ControlFlow::Next
                        | crate::vm::ControlFlow::Value(_)
                        | crate::vm::ControlFlow::Redo { .. }
                        | crate::vm::ControlFlow::Retry { .. }
                        | crate::vm::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        // `break <value>` from inside the block is what
                        // Ruby returns from `each` — capture the value
                        // and stop iterating. (Bare `break` carries Nil.)
                        crate::vm::ControlFlow::Break { value, .. } => {
                            break_value = Some(value);
                            break;
                        }
                        crate::vm::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: crate::vm::utils::position_to_location(position),
                                home_frame: block.home_frame,
                            });
                        }
                        crate::vm::ControlFlow::Exception {
                            exception,
                            position,
                        } => {
                            return Err(MetorexError::UncaughtException {
                                exception: exception.clone(),
                                location: crate::vm::utils::position_to_location(position),
                                message: crate::vm::utils::format_exception(&exception),
                            });
                        }
                    }
                }
                Ok(Some(break_value.unwrap_or_else(|| receiver.clone())))
            }
            "map" | "collect" => {
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
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                };
                let mut results = Vec::new();
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let value = self.execute_block_callable(&block, vec![element], position)?;
                    results.push(value);
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            "select" | "filter" | "reject" => {
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
                        return self
                            .make_enumerator(receiver, method_name, arguments, position)
                            .map(Some);
                    }
                };
                let reject = method_name == "reject";
                let mut results = Vec::new();
                let mut index = 0;
                while let Some(element) = element_at(array_rc, index) {
                    index += 1;
                    let value =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
                    let is_truthy = !matches!(value, Object::Bool(false) | Object::Nil);
                    if is_truthy != reject {
                        results.push(element);
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            // `grep(pattern)` keeps the elements the pattern matches under
            // `===`, passing each through the block when one is given.
            "grep" | "grep_v" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => Some(b),
                    _ => None,
                };
                let inverted = method_name == "grep_v";
                let pattern = arguments[0].clone();
                let elements = array_rc.borrow().clone();
                // Without a block the walk leaves the last match where it
                // found it, so a `$~` set before the call still reads the
                // same afterwards.
                let saved_match = self
                    .globals()
                    .get(crate::vm::native_methods::LAST_MATCH)
                    .unwrap_or(Object::Nil);
                let mut results = Vec::new();
                for element in elements {
                    let matched = self.evaluate_binary_operation(
                        &crate::ast::BinaryOp::CaseEqual,
                        pattern.clone(),
                        element.clone(),
                        position,
                    )?;
                    if matched.is_truthy() == inverted {
                        continue;
                    }
                    results.push(match &block {
                        Some(block) => {
                            self.execute_block_callable(block, vec![element], position)?
                        }
                        None => element,
                    });
                }
                if block.is_none() {
                    self.globals_mut()
                        .set(crate::vm::native_methods::LAST_MATCH, saved_match);
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
            }
            // The first element the block accepts. When nothing matches, the
            // ifnone argument is called for the answer, and a missing one
            // makes it nil.
            "find" | "detect" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let ifnone = match arguments.first() {
                    Some(Object::Nil) | None => None,
                    Some(other) => Some(other.clone()),
                };
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
                            format!("{} requires a block", method_name),
                            position_to_location(position),
                        ));
                    }
                };
                let elements = array_rc.borrow().clone();
                for element in elements {
                    let value =
                        self.execute_block_callable(&block, vec![element.clone()], position)?;
                    if !matches!(value, Object::Bool(false) | Object::Nil) {
                        return Ok(Some(element));
                    }
                }
                match ifnone {
                    Some(ifnone) => self
                        .send_to_object(ifnone, "call", vec![], position)
                        .map(Some),
                    None => Ok(Some(Object::Nil)),
                }
            }
            _ => Ok(None),
        }
    }
}
