// The scheduler a thread hands the waits of its non-blocking fibers to.

use super::*;
use crate::vm::ractors::moved_address;

/// The methods a scheduler has to answer before a thread will take it.
const SCHEDULER_METHODS: [&str; 4] = ["block", "unblock", "kernel_sleep", "io_wait"];

impl VirtualMachine {
    /// The scheduler the running thread was given.
    pub(crate) fn thread_scheduler(&mut self) -> Option<Object> {
        let thread = moved_address(&self.running_thread()).unwrap_or(0);
        self.thread_schedulers.get(&thread).cloned()
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
        if !matches!(scheduler, Object::Nil) {
            let thread = moved_address(&self.running_thread()).unwrap_or(0);
            self.thread_schedulers.insert(thread, scheduler);
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
        let thread = moved_address(&self.running_thread()).unwrap_or(0);
        self.thread_schedulers.remove(&thread);
        Ok(())
    }

    /// Hand the running fiber to `scheduler` to hold until `blocker` lets it
    /// go, noting it against the blocker so the one letting go knows to tell
    /// the scheduler.
    pub(crate) fn scheduler_block(
        &mut self,
        scheduler: Object,
        blocker: &Object,
        timeout: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let fiber = self.fiber_current();
        self.scheduler_waiting
            .entry(moved_address(blocker).unwrap_or(0))
            .or_default()
            .push((fiber, scheduler.clone()));
        self.send_to_object(scheduler, "block", vec![blocker.clone(), timeout], position)
    }

    /// Tell the scheduler holding the first fiber waiting on `blocker` that
    /// the blocker let it go, when one is waiting.
    pub(crate) fn scheduler_unblock(
        &mut self,
        blocker: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let key = moved_address(blocker).unwrap_or(0);
        let Some((fiber, scheduler)) = self
            .scheduler_waiting
            .get_mut(&key)
            .filter(|waiting| !waiting.is_empty())
            .map(|waiting| waiting.remove(0))
        else {
            return Ok(());
        };
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
        let key = moved_address(blocker).unwrap_or(0);
        let waiting = self.scheduler_waiting.remove(&key).unwrap_or_default();
        for (fiber, scheduler) in waiting {
            self.send_to_object(scheduler, "unblock", vec![blocker.clone(), fiber], position)?;
        }
        Ok(())
    }
}
