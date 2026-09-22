// Where a thread stands, and what it is waiting for.

use super::*;

impl VirtualMachine {
    /// Where the code running now stands, one line per frame, innermost
    /// first. This is what the running thread answers for its own backtrace.
    pub(crate) fn own_backtrace_lines(&mut self, position: Position) -> Vec<String> {
        let here = self
            .current_source_file
            .clone()
            .or_else(|| {
                self.current_file
                    .as_ref()
                    .map(|path| path.display().to_string())
            })
            .unwrap_or_default();
        let frames: Vec<_> = self.call_stack().iter().rev().cloned().collect();
        let innermost = frames
            .first()
            .map(|frame| frame.name().to_string())
            .unwrap_or_else(|| "<main>".to_string());
        let mut lines = vec![format!("{}:{}:in '{}'", here, position.line, innermost)];
        for (index, frame) in frames.iter().enumerate() {
            let line = frame_line(frame);
            let path = frame.source_file().unwrap_or(&here);
            let label = frames
                .get(index + 1)
                .map(|caller| caller.name().to_string())
                .unwrap_or_else(|| "<main>".to_string());
            lines.push(format!("{}:{}:in '{}'", path, line, label));
        }
        lines
    }

    /// Where a thread stands: one line per frame it is inside, innermost
    /// first, the way a backtrace reads. A thread that has not started yet
    /// has no frames of its own.
    pub(crate) fn thread_backtrace_lines(&self, thread: &Object) -> Option<Vec<String>> {
        let Object::Instance(instance) = thread else {
            return None;
        };
        let handle = match instance.borrow().get_var("__thread_fiber") {
            Some(Object::Int(handle)) => *handle as usize,
            _ => return None,
        };
        let written_in = match instance.borrow().get_var("__thread_source") {
            Some(Object::String(file)) => file.as_str().to_string(),
            _ => String::new(),
        };
        let waiting_in = match instance.borrow().get_var("__thread_in") {
            Some(Object::String(named)) => Some(named.as_str().to_string()),
            _ => None,
        };
        let held = self.fibers.get(handle)?.held.as_ref()?;
        let frames: Vec<_> = held.call_stack.iter().rev().collect();
        let mut lines = Vec::with_capacity(frames.len() + 1);
        // The innermost entry is where the thread stands right now, which is
        // the name of the wait it is parked in when it is parked in one.
        let written_line = match instance.borrow().get_var("__thread_line") {
            Some(Object::Int(line)) => *line as usize,
            _ => 0,
        };
        let (here, at, named) = match frames.first() {
            Some(innermost) => (
                innermost.source_file().unwrap_or(&written_in).to_string(),
                frame_line(innermost),
                innermost.name().to_string(),
            ),
            None => (written_in.clone(), written_line, "<block>".to_string()),
        };
        let standing_in = waiting_in.unwrap_or_else(|| named.clone());
        lines.push(format!("{}:{}:in '{}'", here, at, standing_in));
        // A thread's own block is a place it is inside even when nothing it
        // called has a frame of its own.
        if frames.is_empty() && standing_in != named {
            lines.push(format!("{}:{}:in '{}'", here, at, named));
        }
        for (index, frame) in frames.iter().enumerate() {
            let line = frame_line(frame);
            let path = frame.source_file().unwrap_or(&written_in);
            let label = frames
                .get(index + 1)
                .map(|caller| caller.name().to_string())
                .unwrap_or_else(|| "<main>".to_string());
            lines.push(format!("{}:{}:in '{}'", path, line, label));
        }
        Some(lines)
    }

    /// Stop the thread running now when something has told it to stop. A
    /// thread is stopped where it can be rather than where it stands, so a
    /// lock it was taking is taken before this unwinds it.
    /// Whether the thread running now has been handed an exception that has
    /// not been raised yet.
    pub(crate) fn thread_holds_an_interrupt(&mut self) -> bool {
        let Object::Instance(running) = self.running_thread() else {
            return false;
        };
        let held = running.borrow().get_var("__thread_raise").cloned();
        matches!(held, Some(Object::Array(values)) if !values.borrow().is_empty())
    }

    /// Whether a thread was asked to take the program down with it when it
    /// dies of an exception.
    pub(crate) fn thread_aborts_the_program(
        &mut self,
        thread: &Object,
        position: Position,
    ) -> bool {
        matches!(
            self.send_to_object(thread.clone(), "abort_on_exception", vec![], position),
            Ok(Object::Bool(true))
        )
    }

    pub(crate) fn raise_if_thread_killed(
        &mut self,
        position: Position,
    ) -> Result<(), MetorexError> {
        self.deliver_thread_interrupts(true, position)
    }

    /// Raise whatever is waiting for the thread running now: what another
    /// thread died of, an exception it was handed, or the word that it is
    /// being stopped. `blocking` says whether this is a place the thread
    /// waits on something, which is what `Thread.handle_interrupt` tells
    /// apart from a place it merely hands the turn over.
    pub(crate) fn deliver_thread_interrupts(
        &mut self,
        blocking: bool,
        position: Position,
    ) -> Result<(), MetorexError> {
        if let Some(handed) = self.thread_abort.take() {
            self.call_native_function("raise", vec![handed], position)?;
        }
        // An exception handed to the thread is raised here, where the thread
        // can be stopped, rather than wherever it happened to be. Whether it
        // is raised now is what the masks the thread set decide, which is
        // asked of Thread itself.
        if self.thread_holds_an_interrupt() {
            let named = self.globals().get("Thread").unwrap_or(Object::Nil);
            if !matches!(named, Object::Nil) {
                self.send_to_object(
                    named,
                    "__run_pending_interrupt__",
                    vec![Object::Bool(blocking)],
                    position,
                )?;
            }
        }
        let Some(Object::Instance(running)) = self.thread_current_stack.last().cloned() else {
            return Ok(());
        };
        if !matches!(
            running.borrow().get_var("__thread_killed"),
            Some(Object::Bool(true))
        ) {
            return Ok(());
        }
        // Stopping a thread happens once, and it is the thread's own body
        // that takes the word rather than a fiber running on it: a fiber
        // unwinds first, and the body unwinds when control comes back to it.
        // An `ensure` clause the unwinding runs may wait on something of its
        // own, and that wait is not interrupted again.
        if self.running_a_thread_body() {
            running
                .borrow_mut()
                .set_var("__thread_killed".to_string(), Object::Bool(false));
            running
                .borrow_mut()
                .set_var("__thread_dying".to_string(), Object::Bool(true));
        }
        Err(MetorexError::UncaughtException {
            exception: Object::exception(FIBER_KILLED, "thread killed".to_string()),
            location: crate::vm::utils::position_to_location(position),
            message: "thread killed".to_string(),
        })
    }

    pub(crate) fn wait_for_other_threads(&mut self, position: Position) {
        if self.running_a_thread_body() {
            // A thread that hands control over while it waits is asleep for
            // as long as the wait lasts, which is what `status` reports.
            let running = self.thread_current_stack.last().cloned();
            if let Some(Object::Instance(instance)) = &running {
                instance
                    .borrow_mut()
                    .set_var("__thread_waiting".to_string(), Object::Bool(true));
            }
            let _ = self.fiber_suspend(Object::Nil, position);
            if let Some(Object::Instance(instance)) = &running {
                instance
                    .borrow_mut()
                    .set_var("__thread_waiting".to_string(), Object::Bool(false));
            }
            return;
        }
        // The thread handing control over is asleep for as long as the wait
        // lasts, whichever thread it is, which is what `status` reports.
        let running = self.running_thread();
        if let Object::Instance(instance) = &running {
            instance
                .borrow_mut()
                .set_var("__thread_waiting".to_string(), Object::Bool(true));
        }
        let stepped = self.step_pending_threads(position);
        if let Object::Instance(instance) = &running {
            instance
                .borrow_mut()
                .set_var("__thread_waiting".to_string(), Object::Bool(false));
        }
        if !stepped {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    /// Run a thread until its block runs out, answering what the block
    /// answered. Every other waiting thread gets a turn in between, so a
    /// thread waiting on something one of them will do still finishes.
    /// Run a thread until it runs out or `limit` passes, answering None when
    /// the limit came first and the thread is still going.
    pub(crate) fn run_thread_within(
        &mut self,
        thread: &Object,
        limit: Option<std::time::Duration>,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Some(handle) = self.thread_fiber_handle(thread) else {
            return Ok(Some(Object::Nil));
        };
        let deadline = limit.map(|held| std::time::Instant::now() + held);
        let mut turns = 0;
        loop {
            if self.fibers.get(handle).is_some_and(|state| state.finished) {
                break;
            }
            // A thread cannot wait on itself, and one part-way through
            // running another is already on its way.
            if handle == self.fiber_current_handle()
                || self.fiber_frames.iter().any(|frame| frame.handle == handle)
            {
                break;
            }
            // A limit of no time at all still gives the thread one turn,
            // which is what makes `join(0)` report whether it is over.
            if let Some(deadline) = deadline
                && turns > 0
                && std::time::Instant::now() >= deadline
            {
                return Ok(None);
            }
            turns += 1;
            let carried = self.thread_fiber_object(thread, handle);
            let started_with = self.thread_start_arguments(thread);
            self.thread_current_stack.push(thread.clone());
            let stepped = self.fiber_resume(handle, carried, started_with, position);
            self.thread_current_stack.pop();
            match stepped {
                Ok(FiberStep::Finished(value)) => {
                    self.finish_thread(thread, value.clone());
                    return Ok(Some(value));
                }
                Ok(FiberStep::Failed(trouble)) => {
                    let died_of = match &trouble {
                        MetorexError::UncaughtException { exception, .. } => exception.clone(),
                        other => Object::string(format!("{other}")),
                    };
                    self.report_thread_death(thread, &died_of, position);
                    self.finish_thread(thread, Object::Nil);
                    if let Object::Instance(instance) = thread {
                        instance
                            .borrow_mut()
                            .set_var("__thread_error".to_string(), died_of);
                    }
                    return Err(trouble);
                }
                Ok(FiberStep::Suspended(_)) => {
                    // The thread is waiting on something, so everything else
                    // waiting gets a turn before it is tried again. A thread
                    // waiting here is itself asleep while it waits.
                    self.wait_for_other_threads(position);
                    self.raise_if_thread_killed(position)?;
                }
                Err(trouble) => {
                    self.finish_thread(thread, Object::Nil);
                    return Err(trouble);
                }
            }
        }
        let held = match thread {
            Object::Instance(instance) => instance.borrow().get_var("__thread_value").cloned(),
            _ => None,
        };
        Ok(Some(held.unwrap_or(Object::Nil)))
    }

    /// Say on stderr what a thread died of, which a thread does for itself
    /// while it is still running.
    pub(crate) fn report_thread_death(
        &mut self,
        thread: &Object,
        died_of: &Object,
        position: Position,
    ) {
        if !matches!(died_of, Object::Exception(_)) {
            return;
        }
        let _ = self.send_to_object(
            thread.clone(),
            "__report_terminated__",
            vec![died_of.clone()],
            position,
        );
    }

    /// The Fiber object a thread's body runs on, which is the thread's root
    /// one.
    pub(crate) fn thread_fiber_object(&mut self, thread: &Object, handle: usize) -> Object {
        if let Object::Instance(instance) = thread
            && let Some(held) = instance.borrow().get_var("__thread_fiber_object")
        {
            return held.clone();
        }
        self.fibers
            .get(handle)
            .and_then(|state| state.object.clone())
            .unwrap_or(Object::Nil)
    }

    /// Whether what is running now is a thread's own body, which is what
    /// decides whether waiting hands control back to whoever gave it a turn
    /// or gives the waiting threads one.
    /// Whether handing control over would let something else run: another
    /// thread waiting for a turn, or the one that made this one.
    pub(crate) fn other_threads_are_waiting(&self) -> bool {
        self.running_a_thread_body() || !self.pending_threads.is_empty()
    }

    pub(crate) fn running_a_thread_body(&self) -> bool {
        self.thread_body_fibers
            .contains(&self.fiber_current_handle())
    }
}
