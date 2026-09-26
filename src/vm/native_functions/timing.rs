// Waiting, and the clock a timeout runs against.

use super::*;

impl VirtualMachine {
    /// Metorex does not hold the program up: a sleep answers at once.
    /// A sleep inside `Timeout.timeout` that would run past the limit
    /// is the one thing it reports on, since that is what the block
    /// was given a limit for.
    /// `sleep`, run in a frame of its own so what interrupts the wait, such
    /// as an exception another thread hands over, names it the way Ruby does.
    pub(crate) fn sleep_for(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let frame = crate::vm::CallFrame::method(
            "Kernel#sleep".to_string(),
            Some(format!("{}:{}", position.line, position.column)),
            "sleep".to_string(),
            "sleep".to_string(),
        )
        .with_source_file(self.current_source_file.clone());
        self.with_call_frame(frame, |vm| vm.sleep_waiting(arguments, position))
    }

    fn sleep_waiting(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A fiber that is not blocking hands its waiting to the
        // scheduler in place rather than waiting itself.
        if let Some(scheduler) = self.fiber_scheduler.clone() {
            let running = self.fiber_current_handle();
            if !self.fiber_is_blocking(running) {
                let given: Vec<Object> = arguments
                    .iter()
                    .filter(|held| !matches!(held, Object::Nil))
                    .cloned()
                    .collect();
                self.send_to_object(scheduler, "kernel_sleep", given, position)?;
                return Ok(Object::Int(0));
            }
        }
        let wanted = match arguments.first() {
            None | Some(Object::Nil) => None,
            // A length may be named by anything that says how to
            // divide itself, which is what a Rational-like value of
            // the program's own does.
            Some(held @ Object::Instance(_)) if self.responds_to(held, "divmod") => {
                let parts =
                    self.send_to_object(held.clone(), "divmod", vec![Object::Int(1)], position)?;
                let Object::Array(held) = parts else {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "can't convert into time interval",
                        position,
                    ));
                };
                let whole = held.borrow().first().cloned().unwrap_or(Object::Int(0));
                let rest = held.borrow().get(1).cloned().unwrap_or(Object::Int(0));
                Some(
                    self.float_value_of(&whole, position)?
                        + self.float_value_of(&rest, position)?,
                )
            }
            Some(held) => Some(self.float_value_of(held, position)?),
        };
        // A length is a wait, so there is no waiting backwards.
        if wanted.is_some_and(|seconds| seconds < 0.0) {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "time interval must not be negative",
                position,
            ));
        }
        if let Some((deadline, class, message)) = self.timeout_limits.last().cloned() {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if wanted.is_none_or(|seconds| seconds > left.as_secs_f64()) {
                self.call_native_function("raise", vec![class, message], position)?;
            }
        }
        // Sleeping hands the turn over, so whatever else the program
        // has to run gets one while this waits. With no length at all
        // the wait lasts until something wakes the thread.
        if wanted.is_none() {
            self.sleep_until_woken(position)?;
        } else {
            self.wait_for_other_threads(position);
            self.raise_if_thread_killed(position)?;
        }
        Ok(Object::Int(wanted.unwrap_or(0.0) as i64))
    }

    /// `Timeout.timeout` opens a limit around the block it runs, and
    /// closes it however the block ends.
    pub(crate) fn open_timeout(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let seconds = self.float_value_of(&arguments[0], position)?;
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs_f64(seconds.max(0.0));
        self.timeout_limits
            .push((deadline, arguments[1].clone(), arguments[2].clone()));
        Ok(Object::Nil)
    }
}
