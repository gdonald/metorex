// Walking the values a range covers, and how many there are.

use super::*;

impl VirtualMachine {
    /// Walking the values a range covers, and how many there are.
    pub(crate) fn call_range_walking_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Range {
            start,
            end,
            exclusive,
            ..
        } = receiver
        else {
            return Ok(None);
        };
        match method_name {
            // block answers zero for.
            "bsearch" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let bounds = SearchBounds::of(start, end, *exclusive, position)?;
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                self.binary_search(&bounds, &block, position).map(Some)
            }
            "each" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let pending = self.pending_block.take();
                let block = match pending {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    // Without a block the walk is handed back, which is what
                    // `(1..3).each` answers. It counts as many values as the
                    // range holds, where the range can say.
                    None => {
                        let counted =
                            match self.send_to_object(receiver.clone(), "size", vec![], position) {
                                Ok(Object::Int(count)) => Some(count),
                                _ => None,
                            };
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                Vec::new(),
                                counted,
                                position,
                            )
                            .map(Some);
                    }
                };

                // A range that counts up from an Integer with no end in sight
                // is walked one value at a time, since there is no list of
                // them to collect. The block stops it with `break`.
                if let Some(first) = endless_int_start(start, end) {
                    let first = &first;
                    let mut current = *first;
                    loop {
                        match self.execute_block_with_control_flow(
                            &block,
                            vec![Object::Int(current)],
                            position,
                        )? {
                            crate::vm::ControlFlow::Next
                            | crate::vm::ControlFlow::Value(_)
                            | crate::vm::ControlFlow::Redo { .. }
                            | crate::vm::ControlFlow::Retry { .. }
                            | crate::vm::ControlFlow::Continue { .. } => {}
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
                        current += 1;
                    }
                }

                // A String range with no end follows `succ` for as long as
                // the block keeps asking, the same way the Integer one counts.
                if let Some(first) = endless_string_start(start, end) {
                    let mut current = first;
                    loop {
                        let value = Object::string(current.clone());
                        match self.execute_block_with_control_flow(&block, vec![value], position)? {
                            crate::vm::ControlFlow::Next
                            | crate::vm::ControlFlow::Value(_)
                            | crate::vm::ControlFlow::Redo { .. }
                            | crate::vm::ControlFlow::Retry { .. }
                            | crate::vm::ControlFlow::Continue { .. } => {}
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
                        let stepped = self.send_to_object(
                            Object::string(current.clone()),
                            "succ",
                            vec![],
                            position,
                        )?;
                        match stepped {
                            Object::String(text) => current = text.as_str().to_string(),
                            _ => return Ok(Some(receiver.clone())),
                        }
                    }
                }

                // The values are walked the same way `to_a` collects them,
                // so a String range follows `succ` too.
                let elements = self.range_elements(start, end, *exclusive, position)?;
                for element in elements {
                    match self.execute_block_with_control_flow(&block, vec![element], position)? {
                        crate::vm::ControlFlow::Next
                        | crate::vm::ControlFlow::Value(_)
                        | crate::vm::ControlFlow::Redo { .. }
                        | crate::vm::ControlFlow::Retry { .. }
                        | crate::vm::ControlFlow::Continue { .. } => continue,
                        // `break` ends the walk and answers what it carried,
                        // which is what the call reports.
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
            "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                Ok(Some(Object::array(elements)))
            }
            // `size` counts a numeric range without walking it, and answers
            // Infinity when it has no end.
            "size" | "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // Only a range that steps from an Integer has a size. A start
                // that is not a number at all has none to report, while a
                // numeric one that cannot be stepped from is an error.
                if !matches!(
                    start.as_ref(),
                    Object::Int(_) | Object::BigInt(_) | Object::Float(_) | Object::Nil
                ) {
                    // A start that has a successor can be walked, it just has
                    // no size to count. One that has none cannot be walked at
                    // all.
                    if matches!(start.as_ref(), Object::String(_) | Object::Symbol(_))
                        || self.responds_to(start, "succ")
                    {
                        return Ok(Some(Object::Nil));
                    }
                    let message = format!(
                        "can't iterate from {}",
                        self.builtins().class_of(start).ruby_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                if !matches!(start.as_ref(), Object::Int(_) | Object::BigInt(_)) {
                    let message = format!(
                        "can't iterate from {}",
                        self.builtins().class_of(start).ruby_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                let infinite = matches!(end.as_ref(), Object::Nil)
                    || matches!(end.as_ref(), Object::Float(value) if value.is_infinite());
                if infinite {
                    return Ok(Some(Object::Float(f64::INFINITY)));
                }
                let (Some(first), Some(last)) = (numeric_floor(start), numeric_ceiling(end)) else {
                    return Ok(Some(Object::Nil));
                };
                let last = if *exclusive && end_is_whole(end) {
                    last - 1
                } else {
                    last
                };
                Ok(Some(Object::Int((last - first + 1).max(0))))
            }
            // A numeric range answers by comparing the ends, and any other
            // walks its values, which is how a String range refuses one that
            // falls between its endpoints but is not among them.
            "member?" | "include?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Only a range of names is walked; every other kind answers
                // by comparing against the ends, which is what an object with
                // no `succ` of its own needs.
                // A range of numbers answers by comparing against its ends,
                // which is what lets a value coerce itself into the
                // comparison rather than being looked for one step at a time.
                let counts = matches!(
                    start.as_ref(),
                    Object::Int(_) | Object::Float(_) | Object::BigInt(_)
                );
                let walks = !counts
                    && (matches!(start.as_ref(), Object::String(_) | Object::Symbol(_))
                        || self.responds_to(start, "succ"));
                if !walks {
                    let covered =
                        self.range_covers(start, end, *exclusive, &arguments[0], position)?;
                    return Ok(Some(Object::Bool(covered)));
                }
                // A range of names the same length as the value reads like an
                // odometer: each place must stand between the two ends.
                if let (Object::String(low), Object::String(high), Object::String(held)) =
                    (start.as_ref(), end.as_ref(), &arguments[0])
                    && let Some(answer) =
                        place_by_place(&low.as_str(), &high.as_str(), &held.as_str(), *exclusive)
                {
                    return Ok(Some(Object::Bool(answer)));
                }
                let elements = self.range_elements(start, end, *exclusive, position)?;
                for element in elements {
                    if self.elements_equal(&element, &arguments[0], position)? {
                        return Ok(Some(Object::Bool(true)));
                    }
                }
                Ok(Some(Object::Bool(false)))
            }
            _ => Ok(None),
        }
    }
}
