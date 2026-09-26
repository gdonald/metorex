// Making a fiber, handing control to one, and suspending it again.

use super::*;

impl VirtualMachine {
    /// Make a fiber that runs `block` when it is first resumed. The number it
    /// answers names the fiber in the interpreter's own table.
    pub(crate) fn fiber_create(
        &mut self,
        block: std::rc::Rc<BlockStatement>,
        blocking: bool,
        storage: Option<Object>,
    ) -> usize {
        let machine: *mut VirtualMachine = self;
        let handle = self.fibers.len();
        let owner = self.thread_current_stack.last().cloned();
        let opened = match (&block.source_file, block.opened_at) {
            (Some(file), Some(line)) => Some((file.clone(), line)),
            _ => None,
        };
        // A fiber runs the interpreter, which nests a frame per call, so it
        // is given room to work rather than the small default a coroutine
        // would otherwise take.
        let stack = DefaultStack::new(FIBER_STACK_BYTES).expect("fiber stack");
        let running = Coroutine::with_stack(stack, move |yielder, first: ResumeInput| {
            // The interpreter is where it was when `resume` handed over, and
            // it stays there until this body suspends or runs out.
            let vm = unsafe { &mut *machine };
            vm.fibers[handle].yielder = yielder as *const _;
            // The fiber's body is a block, and a backtrace names it as one:
            // by the scope it was written in and how many blocks deep.
            let frame = match block.defining_method.clone() {
                Some((callee, defined)) => crate::vm::CallFrame::method(
                    crate::callable::Callable::name(&*block).to_string(),
                    None,
                    callee,
                    defined,
                ),
                None => crate::vm::CallFrame::boundary(
                    crate::callable::Callable::name(&*block).to_string(),
                ),
            }
            .nested_in_a_block(block.written_depth.unwrap_or(1))
            .written_in_scope(block.written_in.clone())
            .with_source_file(block.source_file.clone());
            vm.call_stack_push(frame);
            let answered = vm.execute_block_body(&block, first);
            vm.call_stack_pop();
            // A `break` or a `return` written in a fiber's block has nothing
            // to jump out to, which Ruby reports where the fiber was entered.
            match answered {
                Err(MetorexError::BlockBreak { .. }) => Err(crate::vm::errors::simple_exception(
                    "LocalJumpError",
                    "break from proc-closure",
                    Position::new(0, 0, 0),
                )),
                other => other,
            }
        });
        self.fibers.push(FiberState {
            running: Some(running),
            finished: false,
            yielder: std::ptr::null(),
            held: None,
            object: None,
            opened_at: opened,
            killing: false,
            blocking,
            storage,
            owner,
            raising: None,
        });
        handle
    }

    /// Hand control to a fiber, with the arguments `resume` was called with.
    /// A fiber that transfers control on rather than yielding is followed
    /// here, so the chain runs on this stack rather than nesting.
    /// Run a fiber, keeping the last match to itself. `$~` and the numbered
    /// globals reading it belong to the fiber that set them, so a thread
    /// starts with none and a match it makes is not seen outside.
    pub(crate) fn fiber_resume(
        &mut self,
        handle: usize,
        fiber: Object,
        given: Vec<Object>,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        let held = self
            .globals()
            .get(crate::vm::native_methods::regexp_methods::LAST_MATCH)
            .unwrap_or(Object::Nil);
        let carried = self
            .fiber_last_matches
            .remove(&handle)
            .unwrap_or(Object::Nil);
        self.globals_mut().set(
            crate::vm::native_methods::regexp_methods::LAST_MATCH,
            carried,
        );
        // `$!` and `$@` belong to the fiber handling the exception, so a
        // fiber sees its own and leaves the one it was resumed from alone.
        let outer_error = (
            self.globals().get("!").unwrap_or(Object::Nil),
            self.globals().get("@").unwrap_or(Object::Nil),
        );
        let (own_error, own_trace) = self
            .fiber_errors
            .remove(&handle)
            .unwrap_or((Object::Nil, Object::Nil));
        self.globals_mut().set_variable("!", own_error);
        self.globals_mut().set_variable("@", own_trace);
        // The file the running code was written in belongs to the fiber
        // running it, so the resumer reads its own again afterwards.
        let outer_file = self.current_source_file.clone();
        if let Some(own_file) = self.fiber_source_files.remove(&handle) {
            self.current_source_file = own_file;
        }
        let stepped = self.fiber_resume_within(handle, fiber, given, position);
        let left_file = std::mem::replace(&mut self.current_source_file, outer_file);
        self.fiber_source_files.insert(handle, left_file);
        let left_error = (
            self.globals().get("!").unwrap_or(Object::Nil),
            self.globals().get("@").unwrap_or(Object::Nil),
        );
        self.fiber_errors.insert(handle, left_error);
        self.globals_mut().set_variable("!", outer_error.0);
        self.globals_mut().set_variable("@", outer_error.1);
        let left = self
            .globals()
            .get(crate::vm::native_methods::regexp_methods::LAST_MATCH)
            .unwrap_or(Object::Nil);
        self.fiber_last_matches.insert(handle, left);
        self.globals_mut()
            .set(crate::vm::native_methods::regexp_methods::LAST_MATCH, held);
        stepped
    }

    pub(crate) fn fiber_resume_within(
        &mut self,
        handle: usize,
        fiber: Object,
        given: Vec<Object>,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        let entry = handle;
        let mut running_handle = handle;
        let mut running_fiber = fiber;
        let mut running_given = given;
        loop {
            let stepped = self.fiber_step(
                running_handle,
                running_fiber.clone(),
                running_given,
                position,
            )?;
            match stepped {
                FiberStep::Suspended(SuspendOutput::Yielded(value)) => {
                    // A fiber waiting on something holds up the thread it
                    // runs on, so the thread waits with it and hands the
                    // fiber control back once there is a reason to.
                    if self.blocking_in_fiber {
                        self.blocking_in_fiber = false;
                        self.wait_for_other_threads(position);
                        if let Err(stopping) = self.raise_if_thread_killed(position) {
                            // The fiber the thread is waiting inside unwinds
                            // first, so the `ensure` clauses it sits inside
                            // run before the thread itself is gone.
                            if self
                                .fibers
                                .get(running_handle)
                                .is_some_and(|state| !state.finished)
                            {
                                self.fibers[running_handle].killing = true;
                                let carried = self.fiber_object(running_handle);
                                let _ =
                                    self.fiber_step(running_handle, carried, Vec::new(), position);
                            }
                            return Err(stopping);
                        }
                        running_fiber = self.fiber_object(running_handle);
                        running_given = Vec::new();
                        continue;
                    }
                    return Ok(FiberStep::Suspended(SuspendOutput::Yielded(value)));
                }
                // A transfer names the fiber to run next, and the one that
                // asked for it stays suspended until something resumes it.
                FiberStep::Suspended(SuspendOutput::TransferTo {
                    handle: wanted,
                    fiber: named,
                    given: carried,
                }) => {
                    running_handle = wanted;
                    running_fiber = named;
                    running_given = carried;
                }
                // The fiber the chain started from is what control goes back
                // to when a fiber reached by transfer runs out.
                FiberStep::Finished(value) => {
                    if running_handle == entry || !self.fiber_is_alive(entry) {
                        return Ok(FiberStep::Finished(value));
                    }
                    running_handle = entry;
                    running_fiber = self.fiber_object(entry);
                    running_given = Vec::new();
                }
                FiberStep::Failed(trouble) => return Ok(FiberStep::Failed(trouble)),
            }
        }
    }

    /// Run one fiber until it suspends or runs out.
    pub(crate) fn fiber_step(
        &mut self,
        handle: usize,
        fiber: Object,
        given: Vec<Object>,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        // A fiber belongs to the thread it was made on, and no other thread
        // may run it.
        if let Some(state) = self.fibers.get(handle) {
            let running_on = self.thread_current_stack.last();
            let made_on = state.owner.as_ref();
            let same = match (made_on, running_on) {
                (None, None) => true,
                (Some(Object::Instance(made)), Some(Object::Instance(running))) => {
                    std::rc::Rc::ptr_eq(made, running)
                }
                _ => false,
            };
            if !same {
                return Err(crate::vm::errors::simple_exception(
                    "FiberError",
                    "fiber called across threads",
                    position,
                ));
            }
        }
        // A fiber cannot resume itself, and one part-way through resuming
        // another cannot be resumed either. The fiber a program starts on is
        // always one of the latter.
        if handle == self.fiber_current_handle() {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                "attempt to resume the current fiber",
                position,
            ));
        }
        if handle == ROOT_FIBER || self.fiber_frames.iter().any(|frame| frame.handle == handle) {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                "attempt to resume a resuming fiber",
                position,
            ));
        }
        let Some(state) = self.fibers.get_mut(handle) else {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                "attempt to resume a fiber that was never started",
                position,
            ));
        };
        if state.finished {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                "attempt to resume a terminated fiber",
                position,
            ));
        }
        let Some(mut running) = state.running.take() else {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                "attempt to resume the current fiber",
                position,
            ));
        };
        state.object = Some(fiber.clone());
        // The interpreter's scopes and frames belong to whoever is running,
        // so the resumer's are set aside and the fiber's own put in place.
        let held = self.fibers[handle].held.take();
        let resumer = self.swap_context(held);
        self.fiber_frames.push(FiberFrame { handle, fiber });
        let stepped = running.resume(given);
        self.fiber_frames.pop();
        let left = self.swap_context(Some(resumer));
        let state = &mut self.fibers[handle];
        match stepped {
            CoroutineResult::Yield(handed) => {
                state.held = Some(left);
                state.running = Some(running);
                // A fiber told to stop while it was running is stopped at the
                // first point it hands control back.
                if state.killing {
                    self.fiber_kill(handle, position);
                    return Ok(FiberStep::Finished(Object::Nil));
                }
                Ok(FiberStep::Suspended(handed))
            }
            CoroutineResult::Return(Ok(value)) => {
                state.finished = true;
                Ok(FiberStep::Finished(value))
            }
            CoroutineResult::Return(Err(trouble)) => {
                state.finished = true;
                // The word that a fiber is being stopped is not the program's
                // to see, so it ends the fiber quietly.
                if fiber_was_killed(&trouble) {
                    return Ok(FiberStep::Finished(Object::Nil));
                }
                Ok(FiberStep::Failed(trouble))
            }
        }
    }

    /// The Fiber object a handle names, for putting back on the frame stack
    /// when the chain returns to it.
    pub(crate) fn fiber_object(&self, handle: usize) -> Object {
        self.fibers
            .get(handle)
            .and_then(|state| state.object.clone())
            .unwrap_or(Object::Nil)
    }

    /// Hand control to another fiber. Inside a fiber that is a request the
    /// loop driving the chain acts on, so the one asking stays suspended
    /// rather than holding the other one's run on its own stack.
    pub(crate) fn fiber_transfer(
        &mut self,
        handle: usize,
        fiber: Object,
        given: Vec<Object>,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        if self.fiber_frames.is_empty() {
            return self.fiber_resume(handle, fiber, given, position);
        }
        let back = self.fiber_suspend_with(
            SuspendOutput::TransferTo {
                handle,
                fiber,
                given,
            },
            position,
        )?;
        Ok(FiberStep::Suspended(SuspendOutput::Yielded(
            back.into_iter().next().unwrap_or(Object::Nil),
        )))
    }

    /// Suspend the fiber that holds the interpreter, handing `value` to
    /// whoever resumed it. What comes back is the arguments of the resume
    /// that starts it again.
    pub(crate) fn fiber_suspend(
        &mut self,
        value: Object,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        self.fiber_suspend_with(SuspendOutput::Yielded(value), position)
    }

    /// Suspend the running fiber, handing the request out to the loop that
    /// resumed it.
    pub(crate) fn fiber_suspend_with(
        &mut self,
        handed: SuspendOutput,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let Some(frame) = self.fiber_frames.last() else {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                "can't yield from root fiber",
                position,
            ));
        };
        let frame_handle = frame.handle;
        let yielder = self.fibers[frame_handle].yielder;
        if yielder.is_null() {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                "can't yield from root fiber",
                position,
            ));
        }
        // The handle stays valid for as long as the body runs, and the body
        // is what is being suspended here.
        let back = unsafe { (*yielder).suspend(handed) };
        // A fiber told to raise does so where it was suspended, so the
        // `rescue` and `ensure` blocks it is inside run the way they would
        // had the exception been raised there.
        if let Some(raised) = self.fibers[frame_handle].raising.take() {
            let message = match &raised {
                Object::Exception(details) => details.borrow().message.clone(),
                other => format!("{other}"),
            };
            return Err(MetorexError::UncaughtException {
                exception: raised,
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        if self.fibers[frame_handle].killing {
            return Err(MetorexError::UncaughtException {
                exception: Object::exception(FIBER_KILLED, "fiber killed".to_string()),
                location: crate::vm::utils::position_to_location(position),
                message: "fiber killed".to_string(),
            });
        }
        Ok(back)
    }

    /// Put `wanted` in place as the running state, handing back what was
    /// there. A fiber starting fresh is given scopes of its own.
    pub(crate) fn swap_context(&mut self, wanted: Option<FiberContext>) -> FiberContext {
        let taken = wanted.unwrap_or_else(|| {
            // A fiber starts in a scope of its own, seeded the way the scope
            // the program started in was, so a Kernel function written in it
            // is reached by its bare name.
            let mut fresh = crate::environment::Environment::new();
            crate::vm::init::seed_environment_with_globals(&mut fresh, &self.globals);
            FiberContext {
                environment: fresh,
                call_stack: Vec::new(),
                def_scope_stack: Vec::new(),
                method_nesting_stack: Vec::new(),
                current_method_frame: Some(crate::vm::core::TOP_LEVEL_FRAME),
                lexical_home_frame: None,
            }
        });
        FiberContext {
            environment: std::mem::replace(&mut self.environment, taken.environment),
            call_stack: std::mem::replace(&mut self.call_stack, taken.call_stack),
            def_scope_stack: std::mem::replace(&mut self.def_scope_stack, taken.def_scope_stack),
            method_nesting_stack: std::mem::replace(
                &mut self.method_nesting_stack,
                taken.method_nesting_stack,
            ),
            current_method_frame: std::mem::replace(
                &mut self.current_method_frame,
                taken.current_method_frame,
            ),
            lexical_home_frame: std::mem::replace(
                &mut self.lexical_home_frame,
                taken.lexical_home_frame,
            ),
        }
    }

    /// How a fiber reports itself: `created` before it has run, `resumed`
    /// while it holds the interpreter, `suspended` once it has handed control
    /// back part-way through, and `terminated` when its block has run out.
    pub(crate) fn fiber_status(&self, handle: usize) -> &'static str {
        if self.fiber_frames.iter().any(|frame| frame.handle == handle) {
            return "resumed";
        }
        if handle == ROOT_FIBER {
            return "resumed";
        }
        let Some(state) = self.fibers.get(handle) else {
            return "terminated";
        };
        if state.finished {
            return "terminated";
        }
        if state.held.is_some() {
            "suspended"
        } else {
            "created"
        }
    }

    /// Where the block a fiber runs was written, which its own report names.
    pub(crate) fn fiber_source(&self, handle: usize) -> Option<(String, usize)> {
        let state = self.fibers.get(handle)?;
        let opened = state.opened_at.clone()?;
        Some(opened)
    }
}
