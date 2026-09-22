// Running a block against the receiver, and the walks a method hands back.

use super::*;

impl VirtualMachine {
    /// Running a block against the receiver, and the walks a method hands back.
    pub(crate) fn call_object_evaluating_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // Ruby's `!~` is `!(self =~ other)`. An object with no `=~` of
            // its own has no `!~` either, so this raises rather than
            // answering true.
            "!~" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error("!~", 1, arguments.len(), position));
                }
                if let Some((class, method)) = self.lookup_method(receiver, "=~")
                    && !method.is_undefined
                {
                    let matched = self.invoke_method(
                        class,
                        method,
                        receiver.clone(),
                        vec![arguments[0].clone()],
                        position,
                    )?;
                    return Ok(Some(Object::Bool(!matched.is_truthy())));
                }
                if matches!(
                    (matchable_text(receiver), matchable_text(&arguments[0])),
                    (Some(MatchSide::Pattern(_, _)), Some(MatchSide::Text(_)))
                        | (Some(MatchSide::Text(_)), Some(MatchSide::Pattern(_, _)))
                ) {
                    let matched = self
                        .call_object_method(receiver, "=~", arguments, position)?
                        .unwrap_or(Object::Nil);
                    return Ok(Some(Object::Bool(!matched.is_truthy())));
                }
                let cls = self.builtins().class_of(receiver);
                let msg = format!("undefined method '=~' for an instance of {}", cls.name());
                let exc = Object::exception("NoMethodError", msg.clone());
                if let Object::Exception(cell) = &exc {
                    cell.borrow_mut().name = Some("=~".to_string());
                }
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            // An instance of a `Module` subclass (`class Sub < Module; end;
            // Sub.new`) is itself a module and answers the module-body methods.
            // Metorex models it as an `Instance`, so back it with a cached
            // anonymous class that hosts any methods the body defines.
            "class_exec" | "module_exec" | "class_eval" | "module_eval"
                if self.instance_acts_as_module(receiver) =>
            {
                let inst = match receiver {
                    Object::Instance(i) => std::rc::Rc::clone(i),
                    _ => unreachable!(),
                };
                let existing = match inst.borrow().get_var("__module_body_class__") {
                    Some(Object::Class(c)) => Some(std::rc::Rc::clone(c)),
                    _ => None,
                };
                let backing = match existing {
                    Some(c) => c,
                    None => {
                        let c = std::rc::Rc::new(crate::class::Class::new("", None));
                        inst.borrow_mut().set_var(
                            "__module_body_class__".to_string(),
                            Object::Class(std::rc::Rc::clone(&c)),
                        );
                        c
                    }
                };
                if method_name == "class_exec" || method_name == "module_exec" {
                    let block = match self.pending_block.take() {
                        Some(Object::Block(b)) => b,
                        _ => return Err(local_jump_error(method_name, position)),
                    };
                    let result = self.class_exec_block(
                        &backing,
                        receiver.clone(),
                        &block,
                        arguments.to_vec(),
                        position,
                    )?;
                    return Ok(Some(result));
                }
                let result =
                    self.class_eval_with_args(&backing, receiver.clone(), arguments, position)?;
                Ok(Some(result))
            }
            // `then` / `yield_self` pass the receiver to the block and answer
            // the block's value; `tap` answers the receiver instead.
            // `to_enum(:method, *args)` wraps a method that yields, so the
            // caller can step through its values instead of taking a block.
            "to_enum" | "enum_for" => {
                let mut arguments = arguments.to_vec();
                let enumerated = if arguments.is_empty() {
                    "each".to_string()
                } else {
                    self.coerce_method_name(&arguments.remove(0), method_name, position)?
                };
                // `to_enum { 100 }` names the size with a block, which is
                // only run when the size is asked for.
                let sizing = self.pending_block.take();
                let held = self.build_enumerator(
                    receiver.clone(),
                    &enumerated,
                    arguments,
                    None,
                    position,
                )?;
                if let (Some(block @ Object::Block(_)), Object::Instance(instance)) =
                    (sizing, &held)
                {
                    instance.borrow_mut().set_var("size".to_string(), block);
                }
                Ok(Some(held))
            }
            // `tap` is written in Ruby, in the prelude, so it is not here.
            "then" | "yield_self" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    // `then` and `yield_self` hand back an Enumerator of size
                    // one when they are called with no block.
                    _ => {
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                vec![],
                                Some(1),
                                position,
                            )
                            .map(Some);
                    }
                };
                // A block that declares no parameter is called with none:
                // Ruby's non-lambda blocks drop what they did not ask for.
                let block_arguments = if block.binding_parameters().is_empty() {
                    Vec::new()
                } else {
                    vec![receiver.clone()]
                };
                block.call(self, block_arguments, position).map(Some)
            }
            "instance_exec" | "instance_eval" => {
                let block = self.pending_block.take().or_else(|| {
                    if !arguments.is_empty()
                        && let Object::Block(_) = &arguments[0]
                    {
                        Some(arguments[0].clone())
                    } else {
                        None
                    }
                });
                let positional: Vec<Object> = arguments
                    .iter()
                    .filter(|argument| !matches!(argument, Object::Block(_)))
                    .cloned()
                    .collect();
                if let Some(Object::Block(body)) = block {
                    // Ruby refuses a singleton method on an immediate, and a
                    // `def` in the body would be exactly that.
                    if matches!(
                        receiver,
                        Object::Int(_) | Object::BigInt(_) | Object::Float(_) | Object::Symbol(_)
                    ) && body_defines_a_method(&body.body)
                    {
                        let message = "can't define singleton".to_string();
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("TypeError", message.clone()),
                            location: position_to_location(position),
                            message,
                        });
                    }
                    // `instance_eval` yields the receiver and takes no other
                    // arguments; `instance_exec` passes its own along.
                    if method_name == "instance_eval" {
                        if !positional.is_empty() {
                            return Err(crate::vm::errors::argument_count_error(
                                crate::vm::errors::Arity::Exact(0),
                                positional.len(),
                                position,
                            ));
                        }
                        let result = self.execute_block_with_receiver(
                            &body,
                            receiver.clone(),
                            vec![receiver.clone()],
                            position,
                        )?;
                        return Ok(Some(result));
                    }
                    // The block runs inside a method of BasicObject's, which
                    // is the name a backtrace gives the place it was called
                    // from. The frame carries no method of its own, so
                    // `__method__` in the body still names the one around it.
                    self.call_stack_push(
                        crate::vm::CallFrame::new(
                            format!("BasicObject#{}", method_name),
                            Some(format!("{}:{}", position.line, position.column)),
                        )
                        .nested_in_a_block(0)
                        .with_source_file(self.current_source_file.clone()),
                    );
                    let result = self.execute_block_with_receiver(
                        &body,
                        receiver.clone(),
                        positional,
                        position,
                    );
                    self.call_stack_pop();
                    return result.map(Some);
                }
                // The String form runs source in the receiver's context. A
                // second argument names the file the code is counted as being
                // written in, which `__FILE__` answers and which a
                // `require_relative` written there resolves against.
                if method_name == "instance_eval" {
                    if positional.is_empty() || positional.len() > 3 {
                        return Err(crate::vm::errors::argument_count_error(
                            crate::vm::errors::Arity::Range(1, 3),
                            positional.len(),
                            position,
                        ));
                    }
                    let source = self.coerce_name_argument(&positional[0], position)?;
                    // The file and the line are taken the way any other
                    // String and Integer argument is.
                    let named = match positional.get(1) {
                        None | Some(Object::Nil) => None,
                        Some(Object::String(text)) => Some(text.as_str().to_string()),
                        Some(held) => {
                            if !self.responds_to(held, "to_str") {
                                let message = format!(
                                    "no implicit conversion of {} into String",
                                    self.conversion_name(held)
                                );
                                return Err(crate::vm::errors::simple_exception(
                                    "TypeError",
                                    &message,
                                    position,
                                ));
                            }
                            Some(self.coerce_name_argument(held, position)?)
                        }
                    };
                    let lineno: i64 = match positional.get(2) {
                        None | Some(Object::Nil) => 1,
                        Some(Object::Int(held)) => *held,
                        Some(held) => {
                            let counted = self.coerce_integer_argument(held, position)?;
                            counted.try_into().unwrap_or(1)
                        }
                    };
                    return self
                        .evaluate_source_named_with_receiver(
                            &source,
                            receiver.clone(),
                            named,
                            lineno,
                            position,
                        )
                        .map(Some);
                }
                // `instance_exec` yields, so without a block Ruby reports the
                // same LocalJumpError any bare `yield` would.
                let message = "no block given (yield)".to_string();
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("LocalJumpError", message.clone()),
                    location: position_to_location(position),
                    message,
                })
            }
            _ => Ok(None),
        }
    }
}
