// The fiber a thread runs on, and the turns threads take.

use super::*;

impl VirtualMachine {
    /// The fiber a thread's body runs on, made the first time the thread is
    /// given a turn. A thread runs a step at a time so another can run while
    /// it waits, which is what lets a server written in one answer a client
    /// written in another.
    pub(crate) fn thread_fiber_handle(&mut self, thread: &Object) -> Option<usize> {
        let Object::Instance(instance) = thread else {
            return None;
        };
        if let Some(Object::Int(handle)) = instance.borrow().get_var("__thread_fiber") {
            return Some(*handle as usize);
        }
        let block = instance.borrow().get_var("__thread_block").cloned();
        let Some(Object::Block(block)) = block else {
            return None;
        };
        // The fiber belongs to the thread it runs, so `Fiber.current` inside
        // the body does not read as the thread that made it.
        let owner = std::mem::replace(&mut self.thread_current_stack, vec![thread.clone()]);
        // A thread's body carries a copy of what the fiber making it keeps,
        // the way a fiber made there would.
        let carried = match self.fiber_storage_if_held(self.fiber_current_handle()) {
            Some(Object::Dict(held)) => {
                let copied = held.borrow().clone();
                Some(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                    copied,
                ))))
            }
            other => other,
        };
        let handle = self.fiber_create(block, true, carried);
        self.thread_current_stack = owner;
        self.thread_body_fibers.push(handle);
        // The body runs on a Fiber of its own, which is the thread's root
        // one and what `Fiber.current` answers inside it.
        if let Some(Object::Class(fiber_class)) = self.globals().get("Fiber") {
            let made = crate::object::Instance::new(std::rc::Rc::clone(&fiber_class));
            made.borrow_mut()
                .set_var("__fiber__".to_string(), Object::Int(handle as i64));
            let held = Object::Instance(made);
            self.fibers[handle].object = Some(held.clone());
            instance
                .borrow_mut()
                .set_var("__thread_fiber_object".to_string(), held);
        }
        instance
            .borrow_mut()
            .set_var("__thread_fiber".to_string(), Object::Int(handle as i64));
        Some(handle)
    }

    /// What `Thread.new` was handed, which the body's block is given the
    /// first time the thread runs. They are handed over once, so a body that
    /// suspends and resumes is not handed them again.
    pub(crate) fn thread_start_arguments(&mut self, thread: &Object) -> Vec<Object> {
        let Object::Instance(instance) = thread else {
            return Vec::new();
        };
        let held = instance.borrow().get_var("__thread_args").cloned();
        let Some(Object::Array(values)) = held else {
            return Vec::new();
        };
        instance
            .borrow_mut()
            .set_var("__thread_args".to_string(), Object::Nil);
        values.borrow().clone()
    }

    /// Give every thread that has not finished one turn, answering whether
    /// any of them ran. A thread that finishes records the value its block
    /// answered and leaves the queue.
    /// End every thread still running when the program does. Each one
    /// unwinds where it waits, so the `ensure` clauses the fiber it is on
    /// sits inside run. A fiber it left suspended stays that way, and its
    /// own `ensure` clauses do not run.
    pub(crate) fn end_live_threads(&mut self) {
        let position = Position::new(0, 0, 0);
        for thread in self.pending_threads.clone() {
            let Object::Instance(held) = thread else {
                continue;
            };
            // A thread waiting inside a fiber of its own unwinds that fiber,
            // which is the one whose `ensure` clauses are its to run.
            let waiting_on = match held.borrow().get_var("__thread_on_fiber") {
                Some(Object::Int(handle)) => Some(*handle as usize),
                _ => None,
            };
            let root = match held.borrow().get_var("__thread_fiber") {
                Some(Object::Int(handle)) => Some(*handle as usize),
                _ => None,
            };
            {
                let mut held = held.borrow_mut();
                held.set_var("__thread_killed".to_string(), Object::Bool(true));
                held.set_var("__thread_waiting".to_string(), Object::Bool(false));
            }
            // A thread waiting inside a fiber of its own says so, and that
            // fiber is stopped where it waits when the thread next has a
            // turn, so the `ensure` clauses it sits inside run.
            if let Some(handle) = waiting_on
                && Some(handle) != root
                && let Some(state) = self.fibers.get_mut(handle)
                && !state.finished
            {
                state.killing = true;
            }
        }
        let waited_enough = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !self.pending_threads.is_empty() {
            if std::time::Instant::now() >= waited_enough {
                return;
            }
            if !self.step_pending_threads(position) {
                return;
            }
        }
    }

    pub(crate) fn step_pending_threads(&mut self, position: Position) -> bool {
        if self.pending_threads.is_empty() || self.stepping_threads {
            return false;
        }
        self.stepping_threads = true;
        let waiting = self.pending_threads.clone();
        let mut ran = false;
        for thread in waiting {
            let Some(handle) = self.thread_fiber_handle(&thread) else {
                continue;
            };
            if self.fibers.get(handle).is_some_and(|state| state.finished) {
                continue;
            }
            // A fiber part-way through resuming another cannot be resumed,
            // which is what a thread waiting on another looks like from here.
            if handle == self.fiber_current_handle()
                || self.fiber_frames.iter().any(|frame| frame.handle == handle)
            {
                continue;
            }
            ran = true;
            let carried = self.thread_fiber_object(&thread, handle);
            let started_with = self.thread_start_arguments(&thread);
            self.locks_this_turn = 0;
            self.thread_current_stack.push(thread.clone());
            let stepped = self.fiber_resume(handle, carried, started_with, position);
            self.thread_current_stack.pop();
            match stepped {
                Ok(FiberStep::Finished(value)) => self.finish_thread(&thread, value),
                Ok(FiberStep::Failed(trouble)) => {
                    let died_of = match &trouble {
                        MetorexError::UncaughtException { exception, .. } => exception.clone(),
                        other => Object::string(format!("{other}")),
                    };
                    self.report_thread_death(&thread, &died_of, position);
                    self.finish_thread(&thread, Object::Nil);
                    if let Object::Instance(instance) = &thread {
                        instance
                            .borrow_mut()
                            .set_var("__thread_error".to_string(), died_of.clone());
                    }
                    // A thread asked to take the program down with it hands
                    // what it died of to whoever is waiting, and a thread
                    // ending the program does so whether it was asked to or
                    // not.
                    if ends_the_program(&died_of)
                        || self.thread_aborts_the_program(&thread, position)
                    {
                        self.thread_abort = Some(died_of);
                    }
                }
                Ok(FiberStep::Suspended(_)) => {}
                Err(_) => {
                    self.finish_thread(&thread, Object::Nil);
                }
            }
        }
        self.stepping_threads = false;
        ran
    }

    /// Record what a thread answered and take it off the waiting list.
    pub(crate) fn finish_thread(&mut self, thread: &Object, value: Object) {
        if let Object::Instance(instance) = thread {
            instance
                .borrow_mut()
                .set_var("__thread_value".to_string(), value);
        }
        self.pending_threads.retain(|held| {
            !matches!((held, thread), (Object::Instance(a), Object::Instance(b))
                if std::rc::Rc::ptr_eq(a, b))
        });
        self.release_locks_held_by(thread);
        // A fiber a scheduler holds until the thread ends is let go. The
        // thread is over, so there is nowhere to raise what that raises.
        let _ = self.scheduler_unblock_all(thread, Position::new(0, 0, 0));
    }

    /// Let go of every lock a thread still holds. Ruby releases a thread's
    /// mutexes when it ends, so a lock its body never unlocked stops blocking
    /// whoever waits for it next.
    pub(crate) fn release_locks_held_by(&mut self, thread: &Object) {
        for mutex in &self.taken_mutexes {
            let holder = mutex.borrow().get_var("__mutex_thread").cloned();
            let held = matches!(
                mutex.borrow().get_var("__mutex_locked"),
                Some(Object::Bool(true))
            );
            if held
                && matches!((holder, thread), (Some(Object::Instance(a)), Object::Instance(b))
                    if std::rc::Rc::ptr_eq(&a, b))
            {
                mutex
                    .borrow_mut()
                    .set_var("__mutex_locked".to_string(), Object::Bool(false));
            }
        }
    }

    /// Hand control over while waiting for something outside the program. A
    /// thread's body suspends so whoever gave it a turn carries on, and the
    /// main program gives every waiting thread a turn instead.
    /// Hand the turn over without saying the thread is waiting on anything,
    /// which is what `Thread.pass` does: the thread is ready to run again as
    /// soon as it is given another turn.
    pub(crate) fn pass_to_other_threads(&mut self, position: Position) -> Result<(), MetorexError> {
        if self.running_a_thread_body() {
            self.fiber_suspend(Object::Nil, position)?;
            // Handing the turn over is not a place the thread waits on
            // anything, so an interrupt held until the next blocking call
            // stays held here.
            return self.deliver_thread_interrupts(false, position);
        }
        self.step_pending_threads(position);
        Ok(())
    }

    /// Hand the turn over from the middle of a thread's own code when other
    /// threads are waiting for one. Code of the interpreter's own written in
    /// Ruby runs to its end first, as a method written in C does in Ruby.
    pub(crate) fn share_the_turn(&mut self, position: Position) -> Result<(), MetorexError> {
        let in_the_program = !self
            .current_source_file
            .as_deref()
            .is_some_and(|file| file.starts_with(crate::vm::INTERNAL_FILE_PREFIX));
        let threads_waiting = self.running_a_thread_body()
            || (self.fiber_current_handle() == ROOT_FIBER && !self.pending_threads.is_empty());
        if in_the_program && threads_waiting {
            self.pass_to_other_threads(position)?;
        }
        Ok(())
    }

    /// Wait until something wakes the thread, which is what a `sleep` with
    /// no length asks for. Every other waiting thread gets a turn in the
    /// meantime, and the wait ends when one of them wakes this one.
    pub(crate) fn sleep_until_woken(&mut self, position: Position) -> Result<(), MetorexError> {
        self.sleep_until_woken_within(std::time::Duration::from_secs(2), position)
    }

    /// Wait to be woken, giving up after `limit` so a wake that never comes
    /// ends the program rather than holding it forever.
    pub(crate) fn sleep_until_woken_within(
        &mut self,
        limit: std::time::Duration,
        position: Position,
    ) -> Result<(), MetorexError> {
        if !self.running_a_thread_body() {
            // A fiber of a thread's own that waits holds the whole thread up,
            // so the wait is handed out to whoever resumed the fiber.
            if self.fiber_current_handle() != ROOT_FIBER
                && self
                    .fiber_frames
                    .iter()
                    .any(|frame| self.thread_body_fibers.contains(&frame.handle))
            {
                // Which fiber of the thread's own is waiting here, so a
                // program that ends while it waits unwinds that one.
                let on_fiber = self.fiber_current_handle();
                if let Some(Object::Instance(held)) = self.thread_current_stack.last().cloned() {
                    held.borrow_mut().set_var(
                        "__thread_on_fiber".to_string(),
                        Object::Int(on_fiber as i64),
                    );
                }
                self.blocking_in_fiber = true;
                self.fiber_suspend(Object::Nil, position)?;
                return self.raise_if_thread_killed(position);
            }
            self.step_pending_threads(position);
            return self.raise_if_thread_killed(position);
        }
        let Some(Object::Instance(instance)) = self.thread_current_stack.last().cloned() else {
            self.fiber_suspend(Object::Nil, position)?;
            return Ok(());
        };
        // Which fiber of the thread's own is waiting here, so a program that
        // ends while it waits unwinds that one rather than another the thread
        // left suspended earlier.
        let on_fiber = self.fiber_current_handle();
        {
            let mut held = instance.borrow_mut();
            held.set_var("__thread_in".to_string(), Object::string("sleep"));
            held.set_var(
                "__thread_on_fiber".to_string(),
                Object::Int(on_fiber as i64),
            );
        }
        let deadline = std::time::Instant::now() + limit;
        let mut outcome = Ok(());
        loop {
            instance
                .borrow_mut()
                .set_var("__thread_waiting".to_string(), Object::Bool(true));
            if let Err(stopped) = self.fiber_suspend(Object::Nil, position) {
                outcome = Err(stopped);
                break;
            }
            let asleep = matches!(
                instance.borrow().get_var("__thread_waiting"),
                Some(Object::Bool(true))
            );
            if !asleep || std::time::Instant::now() >= deadline {
                break;
            }
        }
        instance
            .borrow_mut()
            .set_var("__thread_waiting".to_string(), Object::Bool(false));
        instance
            .borrow_mut()
            .set_var("__thread_in".to_string(), Object::Nil);
        outcome?;
        self.raise_if_thread_killed(position)
    }
}
