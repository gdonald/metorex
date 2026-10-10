// Waiting, and the clock a timeout runs against.

use super::*;

impl VirtualMachine {
    /// Metorex does not hold the program up: a sleep answers at once.
    /// A sleep inside `Timeout.timeout` that would run past the limit
    /// is the one thing it reports on, since that is what the block
    /// was given a limit for.
    pub(crate) fn sleep_for(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A fiber that is not blocking hands its waiting to the
        // scheduler in place rather than waiting itself.
        if let Some(scheduler) = self.current_scheduler() {
            {
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
            Some(held @ (Object::Int(_) | Object::BigInt(_) | Object::Float(_))) => {
                Some(self.float_value_of(held, position)?)
            }
            // Anything else names no length of time.
            Some(other) => {
                let named = match other {
                    Object::Instance(instance) => instance.borrow().class.name().to_string(),
                    held => {
                        crate::vm::native_methods::define_method::ruby_class_name(held).to_string()
                    }
                };
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!("can't convert {named} into time interval"),
                    position,
                ));
            }
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
        let started = std::time::Instant::now();
        match wanted {
            None => self.sleep_until_woken(false, position)?,
            Some(seconds) => {
                self.sleep_for_length(std::time::Duration::from_secs_f64(seconds), position)?
            }
        }
        // A signal that arrived while the program waited is handled before
        // the wait answers.
        self.deliver_pending_signals(position)?;
        // The answer is how many whole seconds passed, which is less than
        // asked for when something woke the thread early.
        Ok(Object::Int(started.elapsed().as_secs_f64().round() as i64))
    }

    /// Wait until `length` passes or something wakes the thread, giving
    /// every other thread turns until then.
    fn sleep_for_length(
        &mut self,
        length: std::time::Duration,
        position: Position,
    ) -> Result<(), MetorexError> {
        if self.running_a_thread_body() {
            return self.sleep_until_woken_within(Some(length), false, position);
        }
        let deadline = std::time::Instant::now() + length;
        loop {
            self.wait_for_other_threads(position);
            self.raise_if_thread_killed(position)?;
            self.deliver_pending_signals(position)?;
            if std::time::Instant::now() >= deadline {
                return Ok(());
            }
        }
    }

    /// `Timeout.timeout` opens a limit around the block it runs, and
    /// closes it however the block ends.
    /// Raise the error of the open `Timeout.timeout` limit whose time has
    /// passed, the one that passed first, wherever the program stands. A
    /// limit raises once, however long its block takes to unwind.
    pub(crate) fn raise_expired_timeout(&mut self, position: Position) -> Result<(), MetorexError> {
        let now = std::time::Instant::now();
        let expired = self
            .timeout_limits
            .iter()
            .enumerate()
            .filter(|(_, (deadline, _, _))| *deadline <= now)
            .min_by_key(|(_, (deadline, _, _))| *deadline)
            .map(|(index, _)| index);
        let Some(index) = expired else {
            return Ok(());
        };
        let (_, class, message) = self.timeout_limits[index].clone();
        self.timeout_limits[index].0 = now + RAISED_LIMIT_WAIT;
        self.call_native_function("raise", vec![class, message], position)
            .map(|_| ())
    }

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

/// How far off a limit that has raised is put, so it does not raise again
/// while its block unwinds.
const RAISED_LIMIT_WAIT: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 365);
