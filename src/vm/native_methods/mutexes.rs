// The methods a Mutex answers.

use super::*;

/// How many locks a thread takes in one turn before it hands the turn over.
const LOCKS_PER_TURN: usize = 100;

impl VirtualMachine {
    /// Mutex instance methods. Metorex runs a thread's block on the thread
    /// that made it, so a lock never has to wait: it records who holds it and
    /// refuses a second taking, which is what the visible behavior rests on.
    pub(crate) fn call_mutex_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        _arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Instance(inst) = receiver else {
            return Ok(None);
        };
        let inst = Rc::clone(inst);
        let held = matches!(
            inst.borrow().get_var("__mutex_locked"),
            Some(Object::Bool(true))
        );
        match method_name {
            "synchronize" => {
                let block = self.pending_block.take();
                let Some(Object::Block(body)) = block else {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "Mutex#synchronize requires a block",
                        position,
                    ));
                };
                self.call_mutex_method(receiver, "lock", &[], position)?;
                let answer = self.execute_block_body(&body, vec![]);
                self.call_mutex_method(receiver, "unlock", &[], position)?;
                Ok(Some(answer?))
            }
            "lock" => {
                if held && self.mutex_is_owned(&inst) {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "deadlock; recursive locking",
                        position,
                    ));
                }
                // Another fiber of this thread holding the lock cannot let it
                // go while this one waits, since nothing else of the thread
                // runs until this one does.
                if held && self.fiber_scheduler.is_none() && self.mutex_held_by_this_thread(&inst) {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "deadlock; lock already owned by another fiber belonging to the same thread",
                        position,
                    ));
                }
                if !self.thread_current_stack.is_empty() {
                    self.locks_this_turn += 1;
                    if self.locks_this_turn >= LOCKS_PER_TURN {
                        self.locks_this_turn = 0;
                        self.hand_over_turn(position)?;
                    }
                }
                // A lock something else holds is waited for, so whatever else
                // the program has to run gets a turn until it is let go.
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
                while matches!(
                    inst.borrow().get_var("__mutex_locked"),
                    Some(Object::Bool(true))
                ) && std::time::Instant::now() < deadline
                {
                    self.wait_for_other_threads(position);
                }
                self.mark_mutex_held(&inst);
                // A thread told to stop while it waited takes the lock first,
                // so whatever unwinds next still holds it.
                self.raise_if_thread_killed(position)?;
                Ok(Some(receiver.clone()))
            }
            "try_lock" => {
                if held {
                    return Ok(Some(Object::Bool(false)));
                }
                self.mark_mutex_held(&inst);
                Ok(Some(Object::Bool(true)))
            }
            "unlock" => {
                if !held {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "Attempt to unlock a mutex which is not locked",
                        position,
                    ));
                }
                if !self.mutex_is_owned(&inst) {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "Attempt to unlock a mutex which is locked by another thread",
                        position,
                    ));
                }
                inst.borrow_mut()
                    .set_var("__mutex_locked".to_string(), Object::Bool(false));
                Ok(Some(receiver.clone()))
            }
            // `Mutex#sleep` lets the lock go, waits to be woken, and takes
            // the lock again, which is what a condition variable waits on.
            "sleep" => {
                let wanted = match _arguments.first() {
                    None | Some(Object::Nil) => None,
                    Some(given) => Some(self.float_value_of(given, position)?),
                };
                if wanted.is_some_and(|seconds| seconds < 0.0) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "time interval must not be negative",
                        position,
                    ));
                }
                if !held || !self.mutex_is_owned(&inst) {
                    return Err(crate::vm::errors::simple_exception(
                        "ThreadError",
                        "Attempt to unlock a mutex which is not locked",
                        position,
                    ));
                }
                let started = std::time::Instant::now();
                self.call_mutex_method(receiver, "unlock", &[], position)?;
                // The lock is taken again before anything that interrupted
                // the sleep is raised, so an `ensure` around the sleep sees
                // the thread still holding it.
                let stopped = match wanted {
                    None => self.sleep_until_woken(position),
                    Some(_) => {
                        self.wait_for_other_threads(position);
                        Ok(())
                    }
                };
                self.call_mutex_method(receiver, "lock", &[], position)?;
                if let Some(handed) = self.exception_handed_to_thread() {
                    self.call_native_function("raise", handed, position)?;
                }
                self.raise_if_thread_killed(position)?;
                stopped?;
                Ok(Some(Object::Int(started.elapsed().as_secs() as i64)))
            }
            "locked?" => Ok(Some(Object::Bool(held))),
            "owned?" => Ok(Some(Object::Bool(held && self.mutex_is_owned(&inst)))),
            _ => Ok(None),
        }
    }
}
