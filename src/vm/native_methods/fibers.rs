// The methods a Fiber answers.

use super::*;

impl VirtualMachine {
    /// The methods a fiber answers. The coroutine behind one lives in the
    /// interpreter, and the object carries the number naming it.
    pub(crate) fn call_fiber_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Instance(inst) = receiver else {
            return Ok(None);
        };
        let handle = match inst.borrow().get_var("__fiber__") {
            Some(Object::Int(number)) => *number as usize,
            _ => {
                return Err(crate::vm::errors::simple_exception(
                    "FiberError",
                    "uninitialized fiber",
                    position,
                ));
            }
        };
        match method_name {
            "resume" => {
                let stepped =
                    self.fiber_resume(handle, receiver.clone(), arguments.to_vec(), position)?;
                match stepped {
                    crate::vm::fibers::FiberStep::Suspended(handed) => Ok(Some(match handed {
                        crate::vm::fibers::SuspendOutput::Yielded(value) => value,
                        crate::vm::fibers::SuspendOutput::TransferTo { .. } => Object::Nil,
                    })),
                    crate::vm::fibers::FiberStep::Finished(value) => Ok(Some(value)),
                    crate::vm::fibers::FiberStep::Failed(trouble) => Err(trouble),
                }
            }
            "transfer" => {
                let stepped =
                    self.fiber_transfer(handle, receiver.clone(), arguments.to_vec(), position)?;
                match stepped {
                    crate::vm::fibers::FiberStep::Suspended(handed) => Ok(Some(match handed {
                        crate::vm::fibers::SuspendOutput::Yielded(value) => value,
                        crate::vm::fibers::SuspendOutput::TransferTo { .. } => Object::Nil,
                    })),
                    crate::vm::fibers::FiberStep::Finished(value) => Ok(Some(value)),
                    crate::vm::fibers::FiberStep::Failed(trouble) => Err(trouble),
                }
            }
            "kill" => {
                self.fiber_kill(handle, position);
                Ok(Some(Object::Nil))
            }
            "__raise__" => {
                let raised = self.build_raise_exception(arguments, position)?;
                let stepped = self.fiber_raise(handle, receiver.clone(), raised, position)?;
                match stepped {
                    crate::vm::fibers::FiberStep::Suspended(handed) => Ok(Some(match handed {
                        crate::vm::fibers::SuspendOutput::Yielded(value) => value,
                        crate::vm::fibers::SuspendOutput::TransferTo { .. } => Object::Nil,
                    })),
                    crate::vm::fibers::FiberStep::Finished(value) => Ok(Some(value)),
                    crate::vm::fibers::FiberStep::Failed(trouble) => Err(trouble),
                }
            }
            "blocking?" => Ok(Some(Object::Bool(self.fiber_is_blocking(handle)))),
            // The names a fiber keeps are its own, so only the fiber running
            // may read or replace them.
            "storage" => {
                if handle != self.fiber_current_handle() {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "Fiber storage can only be accessed from the Fiber it belongs to",
                        position,
                    ));
                }
                Ok(Some(self.fiber_storage(handle)))
            }
            "storage=" => {
                if arguments.len() != 1 {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match &arguments[0] {
                    Object::Nil => self.fiber_set_storage(handle, None),
                    held => {
                        self.check_fiber_storage(held, position)?;
                        self.fiber_set_storage(handle, Some(held.clone()));
                    }
                }
                Ok(Some(arguments[0].clone()))
            }
            "alive?" => Ok(Some(Object::Bool(self.fiber_is_alive(handle)))),
            "inspect" | "to_s" => {
                let address = Rc::as_ptr(inst) as usize;
                let status = self.fiber_status(handle);
                let named = self.builtins().class_of(receiver).name().to_string();
                let written = match self.fiber_source(handle) {
                    Some((file, line)) => {
                        format!("#<{named}:0x{address:016x} {file}:{line} ({status})>")
                    }
                    None => format!("#<{named}:0x{address:016x} ({status})>"),
                };
                Ok(Some(Object::string(written)))
            }
            _ => Ok(None),
        }
    }
}
