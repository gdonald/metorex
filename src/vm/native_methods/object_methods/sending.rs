// Sending a message by name, and the blocks a call is handed.

use super::*;

impl VirtualMachine {
    /// Sending a message by name, and the blocks a call is handed.
    pub(crate) fn call_object_sending_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `a&.b` reaches here as a call on `a`. A nil receiver answers
            // nil without the named method running at all.
            crate::parser::SAFE_CALL => {
                if matches!(receiver, Object::Nil) {
                    self.pending_block.take();
                    return Ok(Some(Object::Nil));
                }
                self.call_object_method(receiver, "send", arguments, position)
            }
            "__send__" | "send" | "public_send" => {
                // A complaint about what the call was handed names the call
                // itself in the backtrace, the way Ruby's does.
                let named = crate::vm::CallFrame::method(
                    format!("Kernel#{}", method_name),
                    Some(format!("{}", position_to_location(position))),
                    method_name,
                    method_name,
                );
                if arguments.is_empty() {
                    let message = "no method name given".to_string();
                    return self.with_call_frame(named, |vm| {
                        let raised = vm.add_stack_trace_to_exception(
                            Object::exception("ArgumentError", message.clone()),
                            position,
                        );
                        Err(MetorexError::UncaughtException {
                            exception: raised,
                            location: position_to_location(position),
                            message,
                        })
                    });
                }
                let method = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => {
                        let message = format!(
                            "{} is not a symbol nor a string",
                            self.get_inspect_representation(other, position)?
                        );
                        return self.with_call_frame(named, |vm| {
                            let raised = vm.add_stack_trace_to_exception(
                                Object::exception("TypeError", message.clone()),
                                position,
                            );
                            Err(MetorexError::UncaughtException {
                                exception: raised,
                                location: position_to_location(position),
                                message,
                            })
                        });
                    }
                };
                let rest_args: Vec<Object> = arguments[1..].to_vec();
                // `public_send` reaches only what a caller outside the object
                // could have written, so a private or protected name is
                // refused rather than run.
                if method_name == "public_send" {
                    // The class the method was found on is the one that knows
                    // how it was declared, which for a singleton method is
                    // the object's own class.
                    let holder = match self.lookup_method(receiver, &method) {
                        Some((owner, _)) => owner,
                        None => self.builtins().class_of(receiver),
                    };
                    let hidden = if holder.is_method_private(&method) {
                        Some("private")
                    } else if holder.is_method_protected(&method) {
                        Some("protected")
                    } else {
                        None
                    };
                    if let Some(named) = hidden {
                        let message = format!(
                            "{} method '{}' called for an instance of {}",
                            named,
                            method,
                            self.builtins().class_of(receiver).inspect_name()
                        );
                        let exception = Object::exception("NoMethodError", message.clone());
                        if let Object::Exception(cell) = &exception {
                            cell.borrow_mut().name = Some(method.clone());
                        }
                        return Err(MetorexError::UncaughtException {
                            exception,
                            location: position_to_location(position),
                            message,
                        });
                    }
                }
                // A refinement in force where the call was written stands
                // ahead of what the receiver's class answers, for a name
                // reached this way just as for one written out.
                if let Some(refined) =
                    crate::vm::method_lookup::find_refinement(receiver, &method, self)
                {
                    let class = self.builtins().class_of(receiver);
                    return Ok(Some(self.invoke_method(
                        class,
                        refined,
                        receiver.clone(),
                        rest_args,
                        position,
                    )?));
                }
                // Prefer full lookup (walks singleton class + mixins) so mocked
                // or per-instance overrides take precedence over the class's
                // own method table.
                if let Some((resolved_class, m)) = self.lookup_method(receiver, &method)
                    && !m.is_undefined
                {
                    return Ok(Some(self.invoke_method(
                        resolved_class,
                        m,
                        receiver.clone(),
                        rest_args,
                        position,
                    )?));
                }
                let class = self.builtins().class_of(receiver);
                if let Some(result) = self.call_native_method(
                    class.as_ref(),
                    receiver,
                    &method,
                    &rest_args,
                    position,
                )? {
                    return Ok(Some(result));
                }
                if let Some(result) =
                    self.call_object_method(receiver, &method, &rest_args, position)?
                {
                    return Ok(Some(result));
                }
                // Every Kernel function is a private method of every object,
                // and `send` reaches a private method the way a bare call
                // does.
                if method_name != "public_send"
                    && crate::vm::native_methods::is_kernel_private_function(&method)
                {
                    return self
                        .call_native_function(&method, rest_args, position)
                        .map(Some);
                }
                // A class that answers what it was not asked for decides what
                // a name with no method behind it means.
                if let Some((owner, handler)) = self.lookup_method(receiver, "method_missing")
                    && !handler.is_undefined
                {
                    let mut relayed = vec![Object::symbol(method.clone())];
                    relayed.extend(rest_args);
                    return Ok(Some(self.invoke_method(
                        owner,
                        handler,
                        receiver.clone(),
                        relayed,
                        position,
                    )?));
                }
                let wording = self.receiver_wording_for(receiver, position);
                Err(crate::vm::errors::undefined_method_error_worded(
                    &method, receiver, &rest_args, wording, position,
                ))
            }
            // Kernel#lambda reached by dispatch (`send(:lambda) { }`) rather
            // than by a bare call. The block is already in `pending_block`.
            "warn" => self
                .kernel_warn_for(Some(receiver), arguments.to_vec(), position)
                .map(Some),
            "lambda" | "proc" | "raise" => self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some),
            // `using` is written on `main` alone, and Ruby permits it only at
            // the top level, which a class or module body is not.
            "using" if self.is_the_main_object(receiver) => {
                let inside_body = matches!(
                    self.environment().get("self"),
                    Some(Object::Class(_) | Object::Module(_))
                );
                if inside_body {
                    let message = "main.using is permitted only at toplevel".to_string();
                    return Err(crate::vm::errors::simple_exception(
                        "RuntimeError",
                        &message,
                        position,
                    ));
                }
                self.call_native_function("using", arguments.to_vec(), position)
                    .map(Some)
            }
            _ => Ok(None),
        }
    }
}
