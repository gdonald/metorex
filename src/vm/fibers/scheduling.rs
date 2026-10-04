// The scheduler a thread hands the waits of its non-blocking fibers to.

use super::*;

/// The methods a scheduler has to answer before a thread will take it.
const SCHEDULER_METHODS: [&str; 4] = ["block", "unblock", "kernel_sleep", "io_wait"];

/// Where a thread keeps the scheduler it was given.
const SCHEDULER_VAR: &str = "__fiber_scheduler";

/// Where a Mutex, Queue or Thread keeps the fibers a scheduler is holding
/// until it is ready for them, each with the scheduler holding it.
const WAITING_FIBERS_VAR: &str = "__scheduler_waiting_fibers";

impl VirtualMachine {
    /// The scheduler the running thread was given.
    pub(crate) fn thread_scheduler(&mut self) -> Option<Object> {
        let Object::Instance(thread) = self.running_thread() else {
            return None;
        };
        match thread.borrow().get_var(SCHEDULER_VAR) {
            None | Some(Object::Nil) => None,
            Some(held) => Some(held.clone()),
        }
    }

    /// The scheduler a wait of the running fiber goes to: the thread's, while
    /// the fiber is not a blocking one.
    pub(crate) fn current_scheduler(&mut self) -> Option<Object> {
        let running = self.fiber_current_handle();
        if self.fiber_is_blocking(running) {
            return None;
        }
        self.thread_scheduler()
    }

    /// Give the running thread `scheduler`, or take its scheduler away with
    /// nil. The scheduler it had is closed first, which runs whatever that
    /// one still holds.
    pub(crate) fn set_thread_scheduler(
        &mut self,
        scheduler: Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        if !matches!(scheduler, Object::Nil) {
            for wanted in SCHEDULER_METHODS {
                if !self.responds_to(&scheduler, wanted) {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &format!("Scheduler must implement #{wanted}"),
                        position,
                    ));
                }
            }
            if !self.responds_to(&scheduler, "fiber_interrupt") {
                let text = format!(
                    "{}Scheduler should implement #fiber_interrupt\n",
                    self.warning_prefix(0, position)
                );
                self.warn_through_warning_module(text, position)?;
            }
        }
        self.close_thread_scheduler(position)?;
        if let Object::Instance(thread) = self.running_thread() {
            thread
                .borrow_mut()
                .set_var(SCHEDULER_VAR.to_string(), scheduler);
        }
        Ok(())
    }

    /// Close the running thread's scheduler and take it away, as a thread
    /// does when it is given another one or comes to its end.
    pub(crate) fn close_thread_scheduler(
        &mut self,
        position: Position,
    ) -> Result<(), MetorexError> {
        let Some(scheduler) = self.thread_scheduler() else {
            return Ok(());
        };
        if self.responds_to(&scheduler, "close") {
            self.send_to_object(scheduler, "close", Vec::new(), position)?;
        }
        if let Object::Instance(thread) = self.running_thread() {
            thread
                .borrow_mut()
                .set_var(SCHEDULER_VAR.to_string(), Object::Nil);
        }
        Ok(())
    }

    /// Hand the running fiber to `scheduler` to hold until `blocker` lets it
    /// go, noting it on the blocker so the one letting go knows to tell the
    /// scheduler.
    pub(crate) fn scheduler_block(
        &mut self,
        scheduler: Object,
        blocker: &Object,
        timeout: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let fiber = Object::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![
            self.fiber_current(),
            scheduler.clone(),
        ])));
        if let Object::Instance(held) = blocker {
            let waiting = match held.borrow().get_var(WAITING_FIBERS_VAR) {
                Some(Object::Array(waiting)) => Some(std::rc::Rc::clone(waiting)),
                _ => None,
            };
            match waiting {
                Some(waiting) => waiting.borrow_mut().push(fiber),
                None => held.borrow_mut().set_var(
                    WAITING_FIBERS_VAR.to_string(),
                    Object::Array(std::rc::Rc::new(std::cell::RefCell::new(vec![fiber]))),
                ),
            }
        }
        self.send_to_object(scheduler, "block", vec![blocker.clone(), timeout], position)
    }

    /// Tell the scheduler holding the first fiber waiting on `blocker` that
    /// the blocker let it go, when one is waiting.
    pub(crate) fn scheduler_unblock(
        &mut self,
        blocker: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let Object::Instance(held) = blocker else {
            return Ok(());
        };
        let waiting = match held.borrow().get_var(WAITING_FIBERS_VAR) {
            Some(Object::Array(waiting)) => std::rc::Rc::clone(waiting),
            _ => return Ok(()),
        };
        if waiting.borrow().is_empty() {
            return Ok(());
        }
        let Object::Array(entry) = waiting.borrow_mut().remove(0) else {
            return Ok(());
        };
        let fiber = entry.borrow()[0].clone();
        let scheduler = entry.borrow()[1].clone();
        self.send_to_object(scheduler, "unblock", vec![blocker.clone(), fiber], position)?;
        Ok(())
    }

    /// Tell the schedulers holding every fiber waiting on `blocker` that it
    /// let them go.
    pub(crate) fn scheduler_unblock_all(
        &mut self,
        blocker: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        while self.fibers_waiting_on(blocker) > 0 {
            self.scheduler_unblock(blocker, position)?;
        }
        Ok(())
    }

    fn fibers_waiting_on(&self, blocker: &Object) -> usize {
        let Object::Instance(held) = blocker else {
            return 0;
        };
        match held.borrow().get_var(WAITING_FIBERS_VAR) {
            Some(Object::Array(waiting)) => waiting.borrow().len(),
            _ => 0,
        }
    }
}
