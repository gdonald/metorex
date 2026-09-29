// Counting from one number to another, and the exact quotient of two.

use super::*;

impl VirtualMachine {
    /// Counting from one number to another, and the exact quotient of two.
    pub(crate) fn call_int_walk_method(
        &mut self,
        receiver: &Object,
        n: &i64,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "times" => {
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
                    // Without a block, Ruby returns an Enumerator. We
                    // approximate by returning the integer range as an Array
                    // so chained calls like `n.times.map { ... }` work.
                    None => {
                        let nums: Vec<Object> = (0..*n).map(Object::Int).collect();
                        return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(nums)))));
                    }
                };
                for i in 0..*n {
                    let args = vec![Object::Int(i)];
                    match self.execute_block_with_control_flow(&block, args, position)? {
                        crate::vm::ControlFlow::Next
                        | crate::vm::ControlFlow::Value(_)
                        | crate::vm::ControlFlow::Redo { .. }
                        | crate::vm::ControlFlow::Retry { .. }
                        | crate::vm::ControlFlow::Continue { .. } => {
                            continue;
                        }
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
                            return Err(MetorexError::runtime_error(
                                format!(
                                    "Uncaught exception: {}",
                                    crate::vm::utils::format_exception(&exception)
                                ),
                                crate::vm::utils::position_to_location(position),
                            ));
                        }
                    }
                }
                Ok(Some(Object::Int(*n)))
            }
            // `upto(limit)` / `downto(limit)` — yield each integer from the
            // receiver to `limit` inclusive, answering the receiver. Without
            // a block they answer the sequence as an Array, matching how
            // `times` stands in for an Enumerator here.
            // `quo(other)` — exact division, so Integer / Integer answers a
            // Rational rather than truncating.
            "quo" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Dividing by a Float answers a Float, and dividing by a
                // number too large for a machine word answers one too.
                if matches!(arguments[0], Object::Float(_) | Object::BigInt(_)) {
                    return self
                        .send_to_object(
                            Object::Float(*n as f64),
                            "/",
                            vec![arguments[0].clone()],
                            position,
                        )
                        .map(Some);
                }
                let Some(Object::Int(divisor)) = arguments.first() else {
                    return Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        &arguments[0],
                        position,
                    ));
                };
                if *divisor == 0 {
                    return Err(crate::vm::errors::divide_by_zero_error(position));
                }
                let (mut numerator, mut denominator) = (*n, *divisor);
                if denominator < 0 {
                    numerator = -numerator;
                    denominator = -denominator;
                }
                let divisor = greatest_common_divisor(numerator.abs(), denominator);
                let Some(Object::Class(rational_class)) = self.globals().get("Rational") else {
                    return Ok(None);
                };
                let instance = crate::object::Instance::new(rational_class);
                instance
                    .borrow_mut()
                    .set_var("numerator".to_string(), Object::Int(numerator / divisor));
                instance.borrow_mut().set_var(
                    "denominator".to_string(),
                    Object::Int(denominator / divisor),
                );
                Ok(Some(Object::Instance(instance)))
            }
            "upto" | "downto" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // A Float endpoint counts up to the whole number inside it, so
                // `1.upto(3.7)` stops at 3 and `5.downto(2.3)` stops at 3.
                // Without a block the walk is handed back as an Enumerator
                // whatever the endpoint is, and only asking it for a size
                // reports that it cannot count to one it does not understand.
                let numeric_limit = matches!(&arguments[0], Object::Int(_) | Object::Float(_));
                if !numeric_limit && self.pending_block.is_none() {
                    let message = format!(
                        "comparison of Integer with {} failed",
                        crate::vm::native_methods::array_methods::inspect_element(&arguments[0])
                    );
                    let walk = self.build_enumerator(
                        receiver.clone(),
                        method_name,
                        arguments.to_vec(),
                        None,
                        position,
                    )?;
                    return self
                        .send_to_object(
                            walk,
                            "__refuse_size__",
                            vec![Object::string(message)],
                            position,
                        )
                        .map(Some);
                }
                // An endless limit walks on without stopping, so the walk
                // ends only when what reads it stops asking.
                let endless = matches!(
                    &arguments[0],
                    Object::Float(limit)
                        if limit.is_infinite()
                            && (limit.is_sign_positive() == (method_name == "upto"))
                );
                let limit = match &arguments[0] {
                    Object::Int(limit) => *limit,
                    Object::Float(_) if endless => {
                        if method_name == "upto" {
                            i64::MAX
                        } else {
                            i64::MIN
                        }
                    }
                    Object::Float(limit) if limit.is_finite() => {
                        let whole = match method_name {
                            "upto" => limit.floor(),
                            _ => limit.ceil(),
                        };
                        whole as i64
                    }
                    other => {
                        let message = format!(
                            "comparison of Integer with {} failed",
                            crate::vm::native_methods::array_methods::inspect_element(other)
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &message,
                            position,
                        ));
                    }
                };
                let limit = &limit;
                let sequence: Box<dyn Iterator<Item = i64>> = if method_name == "upto" {
                    Box::new(*n..=*limit)
                } else {
                    Box::new((*limit..=*n).rev())
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
                        // Without a block it answers an Enumerator over the
                        // sequence, which is what Ruby hands back.
                        if endless {
                            return self
                                .build_enumerator_of_size(
                                    receiver.clone(),
                                    method_name,
                                    arguments.to_vec(),
                                    Object::Float(f64::INFINITY),
                                    position,
                                )
                                .map(Some);
                        }
                        let counted = if method_name == "upto" {
                            (*limit - *n + 1).max(0)
                        } else {
                            (*n - *limit + 1).max(0)
                        };
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                arguments.to_vec(),
                                Some(counted),
                                position,
                            )
                            .map(Some);
                    }
                };
                for value in sequence {
                    let args = vec![Object::Int(value)];
                    match self.execute_block_with_control_flow(&block, args, position)? {
                        crate::vm::ControlFlow::Next
                        | crate::vm::ControlFlow::Value(_)
                        | crate::vm::ControlFlow::Redo { .. }
                        | crate::vm::ControlFlow::Retry { .. }
                        | crate::vm::ControlFlow::Continue { .. } => {
                            continue;
                        }
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
                            return Err(MetorexError::runtime_error(
                                format!(
                                    "Uncaught exception: {}",
                                    crate::vm::utils::format_exception(&exception)
                                ),
                                crate::vm::utils::position_to_location(position),
                            ));
                        }
                    }
                }
                Ok(Some(Object::Int(*n)))
            }
            _ => Ok(None),
        }
    }
}
