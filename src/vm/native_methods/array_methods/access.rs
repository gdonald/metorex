// Reading one element, a run of them, or one nested inside.

use super::*;

impl VirtualMachine {
    /// Reading one element, a run of them, or one nested inside.
    pub(crate) fn call_array_access_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "[]" | "slice" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let total = array_rc.borrow().len() as i64;
                if arguments.len() == 2 {
                    let start = self.machine_index(&arguments[0], total, position)?;
                    let span = self.machine_index(&arguments[1], 0, position)?;
                    let array = array_rc.borrow();
                    if start < 0 || start > total || span < 0 {
                        return Ok(Some(Object::Nil));
                    }
                    let end = start.saturating_add(span).min(total);
                    return Ok(Some(Object::array(
                        array[start as usize..end as usize].to_vec(),
                    )));
                }
                // An arithmetic sequence names a strided run of the array.
                if let Some(held) = self.sequence_slice(array_rc, &arguments[0], total, position)? {
                    return Ok(Some(held));
                }
                // A Range slices, and each bound goes through `to_int` too.
                if let Some(span) = crate::vm::native_methods::as_range(&arguments[0]) {
                    let (start, span) = self.range_bounds(&span, total, position)?;
                    if start < 0 || start > total {
                        return Ok(Some(Object::Nil));
                    }
                    let array = array_rc.borrow();
                    let end = start.saturating_add(span).min(total);
                    return Ok(Some(Object::array(
                        array[start as usize..end.max(start) as usize].to_vec(),
                    )));
                }
                let index = self.machine_index(&arguments[0], total, position)?;
                let array = array_rc.borrow();
                if index < 0 || index >= total {
                    return Ok(Some(Object::Nil));
                }
                Ok(Some(array[index as usize].clone()))
            }
            // `dig(index, *rest)` — index, then keep digging into the result.
            "dig" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let Object::Int(index) = &arguments[0] else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                let array = array_rc.borrow();
                let length = array.len() as i64;
                let resolved = if *index < 0 { index + length } else { *index };
                if resolved < 0 || resolved >= length {
                    return Ok(Some(Object::Nil));
                }
                let value = array[resolved as usize].clone();
                drop(array);
                if arguments.len() == 1 {
                    return Ok(Some(value));
                }
                if matches!(value, Object::Nil) {
                    return Ok(Some(Object::Nil));
                }
                self.dig_into(&value, &arguments[1..], position).map(Some)
            }
            // `each_with_index` yields the element and its position. Without
            // a block it answers an Enumerator, the way `each` does.
            "each_with_index" => {
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
                        .build_enumerator(receiver.clone(), method_name, vec![], None, position)
                        .map(Some);
                };
                let elements = array_rc.borrow().clone();
                for (index, element) in elements.iter().enumerate() {
                    let args = vec![element.clone(), Object::Int(index as i64)];
                    match self.execute_block_with_control_flow(&block, args, position)? {
                        crate::vm::ControlFlow::Next
                        | crate::vm::ControlFlow::Value(_)
                        | crate::vm::ControlFlow::Redo { .. }
                        | crate::vm::ControlFlow::Retry { .. }
                        | crate::vm::ControlFlow::Continue { .. } => continue,
                        crate::vm::ControlFlow::Break { value, .. } => {
                            return Ok(Some(value));
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
                Ok(Some(receiver.clone()))
            }
            _ => Ok(None),
        }
    }
}
