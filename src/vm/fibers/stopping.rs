// Stopping a fiber where it stands, and what it keeps.

use super::*;

impl VirtualMachine {
    /// Stop a fiber where it stands, unwinding whatever it was part-way
    /// through so its `ensure` blocks run. A fiber that has already run out
    /// is left alone.
    pub(crate) fn fiber_kill(&mut self, handle: usize, position: Position) {
        if handle == ROOT_FIBER {
            return;
        }
        let Some(state) = self.fibers.get_mut(handle) else {
            return;
        };
        if state.finished {
            return;
        }
        // A fiber holding the interpreter is stopped when control comes back
        // to it rather than where it stands, since the stack it is on is the
        // one running.
        if self.fiber_frames.iter().any(|frame| frame.handle == handle) {
            self.fibers[handle].killing = true;
            return;
        }
        let state = &mut self.fibers[handle];
        // A fiber that has not started yet has nothing to unwind.
        if state.held.is_none() {
            state.finished = true;
            state.running = None;
            return;
        }
        // A fiber part-way through is resumed once more with the word that it
        // is being stopped. The suspend it is parked in reports that as an
        // exception, so the `ensure` blocks it is inside run the way they
        // would had the body raised.
        state.killing = true;
        let object = state.object.clone().unwrap_or(Object::Nil);
        let _ = self.fiber_step(handle, object, Vec::new(), Handoff::Resume, position);
        if let Some(state) = self.fibers.get_mut(handle) {
            state.killing = false;
            state.finished = true;
            state.running = None;
            state.held = None;
        }
    }

    /// The handle of the fiber holding the interpreter.
    pub(crate) fn fiber_current_handle(&self) -> usize {
        self.fiber_frames
            .last()
            .map(|frame| frame.handle)
            .unwrap_or(ROOT_FIBER)
    }

    /// Raise an exception inside a fiber, where it stands. The fiber running
    /// now raises it at once, and one part-way through resuming another
    /// passes it on to that one.
    pub(crate) fn fiber_raise(
        &mut self,
        handle: usize,
        fiber: Object,
        raised: Object,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        if let Some(resumed) = self.fiber_resumed_by(handle) {
            let object = self.fiber_object(resumed);
            return self.fiber_raise(resumed, object, raised, position);
        }
        let current = self.fiber_current_handle();
        let refused = match self.fibers.get(handle) {
            _ if handle == current => None,
            None => None,
            Some(state) if state.finished => Some("attempt to resume a terminated fiber"),
            Some(state) if state.held.is_none() => Some("cannot raise exception on unborn fiber"),
            Some(_) => {
                self.fiber_check_thread(handle, position)?;
                let state = &mut self.fibers[handle];
                state.raising = Some(raised.clone());
                // A fiber waiting in `Fiber.yield` is resumed, and one that
                // handed control on by a transfer is transferred to.
                return if state.yielding {
                    self.fiber_resume(handle, fiber, Vec::new(), position)
                } else {
                    self.fiber_transfer(handle, fiber, Vec::new(), position)
                };
            }
        };
        if let Some(message) = refused {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                message,
                position,
            ));
        }
        let message = match &raised {
            Object::Exception(details) => details.borrow().message.clone(),
            other => format!("{other}"),
        };
        Err(MetorexError::UncaughtException {
            exception: raised,
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }

    /// Whether a fiber blocks when it waits rather than handing control to a
    /// scheduler. The fiber a program starts on always blocks.
    pub(crate) fn fiber_is_blocking(&self, handle: usize) -> bool {
        if handle == ROOT_FIBER {
            return true;
        }
        self.fibers.get(handle).is_some_and(|state| state.blocking)
    }

    /// The names a fiber keeps, made the first time one is written.
    pub(crate) fn fiber_storage(&mut self, handle: usize) -> Object {
        let held = if handle == ROOT_FIBER {
            self.root_storage.clone()
        } else {
            self.fibers
                .get(handle)
                .and_then(|state| state.storage.clone())
        };
        if let Some(found) = held {
            return found;
        }
        let made = Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
            indexmap::IndexMap::new(),
        )));
        self.fiber_set_storage(handle, Some(made.clone()));
        made
    }

    /// What a fiber keeps, without making one where it holds none.
    pub(crate) fn fiber_storage_if_held(&self, handle: usize) -> Option<Object> {
        if handle == ROOT_FIBER {
            return self.root_storage.clone();
        }
        self.fibers
            .get(handle)
            .and_then(|state| state.storage.clone())
    }

    /// Put a set of names in place for a fiber, or clear what it held.
    pub(crate) fn fiber_set_storage(&mut self, handle: usize, wanted: Option<Object>) {
        if handle == ROOT_FIBER {
            self.root_storage = wanted;
            return;
        }
        if let Some(state) = self.fibers.get_mut(handle) {
            state.storage = wanted;
        }
    }

    /// Say whether a fiber blocks, handing back what it said before.
    pub(crate) fn fiber_set_blocking(&mut self, handle: usize, wanted: bool) -> bool {
        let Some(state) = self.fibers.get_mut(handle) else {
            return true;
        };
        std::mem::replace(&mut state.blocking, wanted)
    }

    /// Whether a fiber has not yet run out.
    pub(crate) fn fiber_is_alive(&self, handle: usize) -> bool {
        if handle == ROOT_FIBER {
            return true;
        }
        self.fibers.get(handle).is_some_and(FiberState::is_alive)
    }

    /// The fiber holding the interpreter. At the top level that is the fiber
    /// the program started on, which is made the first time it is asked for.
    pub(crate) fn fiber_current(&mut self) -> Object {
        if let Some(frame) = self.fiber_frames.last() {
            return frame.fiber.clone();
        }
        if let Some(held) = &self.root_fiber {
            return held.clone();
        }
        let Some(Object::Class(fiber_class)) = self.globals().get("Fiber") else {
            return Object::Nil;
        };
        let made = crate::object::Instance::new(fiber_class);
        made.borrow_mut()
            .set_var("__fiber__".to_string(), Object::Int(ROOT_FIBER as i64));
        let held = Object::Instance(made);
        self.root_fiber = Some(held.clone());
        held
    }
}

/// The line a frame was entered from, which its location records after the
/// file it sits in.
pub(crate) fn frame_line(frame: &crate::vm::CallFrame) -> usize {
    frame
        .location()
        .and_then(|location| {
            location
                .rsplit(':')
                .nth(1)
                .and_then(|line| line.parse::<usize>().ok())
        })
        .unwrap_or(0)
}

/// Whether what a thread died of ends the whole program rather than just
/// that thread, which is what `exit` raises.
pub(crate) fn ends_the_program(died_of: &Object) -> bool {
    let Object::Exception(details) = died_of else {
        return false;
    };
    matches!(
        details.borrow().exception_type.as_str(),
        "SystemExit" | "Interrupt" | "SignalException" | "NoMemoryError" | "SystemStackError"
    )
}

/// Whether an error is the one a stopped fiber unwinds with.
pub(crate) fn fiber_was_killed(trouble: &MetorexError) -> bool {
    let MetorexError::UncaughtException { exception, .. } = trouble else {
        return false;
    };
    let Object::Exception(details) = exception else {
        return false;
    };
    details.borrow().exception_type == FIBER_KILLED
}
