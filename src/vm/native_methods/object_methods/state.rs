// Freezing, naming, and the interpreter-level readings an object answers.

use super::*;

impl VirtualMachine {
    /// Freezing, naming, and the interpreter-level readings an object answers.
    pub(crate) fn call_object_state_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "itself" => {
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
            "frozen?" => Ok(Some(Object::Bool(self.object_is_frozen(receiver)))),
            "freeze" => {
                self.freeze_object(receiver);
                Ok(Some(receiver.clone()))
            }
            "to_sym" => match receiver {
                Object::Symbol(_) => Ok(Some(receiver.clone())),
                Object::String(s) => self.interned_symbol(s, position).map(Some),
                _ => Ok(None),
            },
            // Kernel's conversion functions are private instance methods on
            // every object, which is how `obj.send(:Integer, "10")` reaches
            // them.
            name if crate::vm::native_methods::kernel_conversion::is_kernel_conversion(name) => {
                self.call_kernel_conversion(name, arguments, position)
            }
            // `abort`, `exit`, and `exit!` are private instance methods on
            // Kernel, so every object reaches them. A class can make one
            // public, which is how a spec calls it with a receiver.
            "abort" | "exit" | "exit!" | "fork" | "system" | "spawn" | "exec" | "`" => self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some),
            // `send(:block_given?)` reports on the frame that sent it, the
            // same as the bare form.
            "block_given?" if arguments.is_empty() => Ok(Some(Object::Bool(matches!(
                self.environment().get("block_given?"),
                Some(Object::Bool(true))
            )))),
            // A TracePoint tells the interpreter when it is switched on or
            // off, and asks whether a handler is running.
            "__register__" => {
                self.register_tracepoint(receiver);
                Ok(Some(Object::Nil))
            }
            "__tracing__" => Ok(Some(Object::Bool(self.is_tracing()))),
            // The lines the statements of a method or a block body start on,
            // which is where a trace aimed at it can report `:line` events.
            "__code_lines__" if arguments.len() == 1 => {
                let body = match &arguments[0] {
                    Object::Method(method) => method.body.clone(),
                    Object::Block(block) => block.body.clone(),
                    _ => Vec::new(),
                };
                Ok(Some(Object::array(
                    body.iter()
                        .map(|statement| Object::Int(statement.position().line as i64))
                        .collect(),
                )))
            }
            // Run a block with tracing switched off, so an event the block
            // causes reaches the tracepoints again. `TracePoint.allow_reentry`
            // is what asks for this.
            "__reentrant__" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return Err(crate::vm::errors::simple_exception(
                        "LocalJumpError",
                        "no block given (yield)",
                        position,
                    ));
                };
                let held = self.take_tracing();
                let answered = self.execute_block_callable(&block, vec![], position);
                self.restore_tracing(held);
                answered.map(Some)
            }
            // ARGF#gets reads a line from the input stream.
            "gets"
                if arguments.is_empty()
                    && matches!(receiver, Object::Instance(inst) if inst.borrow().class.name() == "ARGF.class") =>
            {
                self.read_line_from_stdin(position).map(Some)
            }
            _ => Ok(None),
        }
    }
}
