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
            .owned_by(block.defining_owner.clone())
            .with_source_file(block.source_file.clone());
            vm.call_stack_push(frame);
            // A thread's body is where a trace hears the thread start and end.
            let body_of_a_thread = vm.thread_body_fibers.contains(&handle);
            let started = if body_of_a_thread {
                vm.fire_event("thread_begin", Position::new(0, 0, 0), Vec::new())
            } else {
                Ok(())
            };
            let answered = started.and_then(|()| vm.execute_block_body(&block, first));
            // A thread closes its scheduler as it ends, which runs whatever
            // the scheduler still holds, on this thread.
            let answered = match answered {
                Ok(value) if body_of_a_thread => vm
                    .close_thread_scheduler(Position::new(0, 0, 0))
                    .map(|()| value),
                other => other,
            };
            let answered = match answered {
                Ok(value) if body_of_a_thread => vm
                    .fire_event("thread_end", Position::new(0, 0, 0), Vec::new())
                    .map(|()| value),
                other => other,
            };
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
            yielding: false,
            host: None,
            relaying: None,
            relay_request: None,
        });
        handle
    }

    /// Hand control to a fiber, with the arguments `resume` was called with.
    pub(crate) fn fiber_resume(
        &mut self,
        handle: usize,
        fiber: Object,
        given: Vec<Object>,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        self.fiber_run(handle, fiber, given, Handoff::Resume, position)
    }

    /// Run a fiber, keeping the last match and the last line read to itself.
    /// `$~`, the numbered globals reading it, and `$_` belong to the fiber
    /// that set them, so a thread starts with none and what it sets is not
    /// seen outside.
    pub(crate) fn fiber_run(
        &mut self,
        handle: usize,
        fiber: Object,
        given: Vec<Object>,
        handoff: Handoff,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        let held = self
            .globals()
            .get(crate::vm::native_methods::regexp_methods::LAST_MATCH)
            .unwrap_or(Object::Nil);
        let held_line = self.globals().get("_").unwrap_or(Object::Nil);
        let (carried, carried_line) = self
            .fiber_last_match_and_line
            .remove(&handle)
            .unwrap_or((Object::Nil, Object::Nil));
        self.globals_mut().set(
            crate::vm::native_methods::regexp_methods::LAST_MATCH,
            carried,
        );
        self.globals_mut().set_variable("_", carried_line);
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
        let stepped = self.fiber_resume_within(handle, fiber, given, handoff, position);
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
        let left_line = self.globals().get("_").unwrap_or(Object::Nil);
        self.fiber_last_match_and_line
            .insert(handle, (left, left_line));
        self.globals_mut()
            .set(crate::vm::native_methods::regexp_methods::LAST_MATCH, held);
        self.globals_mut().set_variable("_", held_line);
        stepped
    }

    /// Run a chain of fibers from this stack. A fiber that transfers control
    /// on rather than yielding is followed here, so the chain runs on this
    /// stack rather than nesting.
    pub(crate) fn fiber_resume_within(
        &mut self,
        handle: usize,
        fiber: Object,
        given: Vec<Object>,
        handoff: Handoff,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        let entry = handle;
        let owner = self.fiber_current_handle();
        let mut running_handle = handle;
        let mut running_fiber = fiber;
        let mut running_given = given;
        loop {
            let stepped = self.fiber_step(
                running_handle,
                running_fiber.clone(),
                running_given,
                handoff,
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
                        // An exception handed to the thread is raised by the
                        // fiber where it waits, so a `rescue` there sees it.
                        let handed_one = self.thread_holds_an_interrupt();
                        if !handed_one && let Err(stopping) = self.raise_if_thread_killed(position)
                        {
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
                                let _ = self.fiber_step(
                                    running_handle,
                                    carried,
                                    Vec::new(),
                                    handoff,
                                    position,
                                );
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
                // Control handed back to the fiber this chain runs from answers
                // the `transfer` it started the chain with.
                FiberStep::Suspended(SuspendOutput::TransferTo {
                    handle: wanted,
                    given: carried,
                    ..
                }) if wanted == owner => {
                    return Ok(FiberStep::Suspended(SuspendOutput::Yielded(passing_value(
                        carried,
                    ))));
                }
                // The fiber a program starts on is below the one this chain
                // runs on, which hands the transfer down and waits there,
                // part-way through resuming the chain, to be reached again.
                FiberStep::Suspended(SuspendOutput::TransferTo {
                    handle: ROOT_FIBER,
                    fiber: named,
                    given: carried,
                }) => {
                    self.fibers[owner].relaying = Some(entry);
                    let relayed = self.fiber_suspend_with(
                        SuspendOutput::TransferTo {
                            handle: ROOT_FIBER,
                            fiber: named,
                            given: carried,
                        },
                        position,
                    );
                    self.fibers[owner].relaying = None;
                    let back = relayed?;
                    let entry_fiber = self.fiber_object(entry);
                    (running_handle, running_fiber, running_given) = self.fibers[owner]
                        .relay_request
                        .take()
                        .unwrap_or((entry, entry_fiber, back));
                }
                FiberStep::Suspended(SuspendOutput::TransferTo {
                    handle: wanted,
                    fiber: named,
                    given: carried,
                }) => {
                    running_handle = wanted;
                    running_fiber = named;
                    running_given = carried;
                }
                // A fiber reached by transfer that runs out hands what it
                // answered to the fiber the chain was resumed into, or to the
                // fiber a program starts on when that one transferred.
                FiberStep::Finished(value) => {
                    if running_handle == entry
                        || handoff == Handoff::Transfer
                        || !self.fiber_is_alive(entry)
                    {
                        return Ok(FiberStep::Finished(value));
                    }
                    running_handle = entry;
                    running_fiber = self.fiber_object(entry);
                    running_given = vec![value];
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
        handoff: Handoff,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        self.fiber_check_thread(handle, position)?;
        // A fiber resumed from inside one that is now relaying runs on that
        // one's stack, so that one is resumed and told to run it.
        let current = self.fiber_current_handle();
        if let Some(host) = self.fibers.get(handle).and_then(|state| state.host)
            && host != current
            && self
                .fibers
                .get(host)
                .is_some_and(|state| state.relaying.is_some())
        {
            let carrier = self.fiber_object(host);
            self.fibers[host].relay_request = Some((handle, fiber, given));
            return self.fiber_step(host, carrier, Vec::new(), handoff, position);
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
        if handle == ROOT_FIBER
            || self.fiber_frames.iter().any(|frame| frame.handle == handle)
            || self
                .fibers
                .get(handle)
                .is_some_and(|state| state.relaying.is_some() && state.relay_request.is_none())
        {
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
        state.host = Some(current);
        state.yielding = false;
        // The interpreter's scopes and frames belong to whoever is running,
        // so the resumer's are set aside and the fiber's own put in place.
        let held = self.fibers[handle].held.take();
        let resumer = self.swap_context(held);
        self.fiber_frames.push(FiberFrame {
            handle,
            fiber,
            handoff,
        });
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
        // Handing control to the fiber that holds it already leaves it there.
        if handle == self.fiber_current_handle() {
            return Ok(FiberStep::Suspended(SuspendOutput::Yielded(passing_value(
                given,
            ))));
        }
        // What stops a transfer is raised here, in the fiber asking for it,
        // before control leaves it.
        self.fiber_check_thread(handle, position)?;
        let refused = if self.fiber_resumed_by(handle).is_some() {
            Some("attempt to transfer to a resuming fiber")
        } else if self.fibers.get(handle).is_some_and(|state| state.yielding) {
            Some("attempt to transfer to a yielding fiber")
        } else if !self.fiber_is_alive(handle) {
            Some("dead fiber called")
        } else {
            None
        };
        if let Some(message) = refused {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                message,
                position,
            ));
        }
        if self.fiber_frames.is_empty() {
            return self.fiber_run(handle, fiber, given, Handoff::Transfer, position);
        }
        let back = self.fiber_suspend_with(
            SuspendOutput::TransferTo {
                handle,
                fiber,
                given,
            },
            position,
        )?;
        Ok(FiberStep::Suspended(SuspendOutput::Yielded(passing_value(
            back,
        ))))
    }

    /// A fiber belongs to the thread it was made on, and no other thread may
    /// run it. The fiber a program starts on belongs to the main thread.
    pub(crate) fn fiber_check_thread(
        &self,
        handle: usize,
        position: Position,
    ) -> Result<(), MetorexError> {
        let made_on = match self.fibers.get(handle) {
            Some(state) => state.owner.as_ref(),
            None => None,
        };
        let running_on = self.thread_current_stack.last();
        let same = match (made_on, running_on) {
            (None, None) => true,
            (Some(Object::Instance(made)), Some(Object::Instance(running))) => {
                std::rc::Rc::ptr_eq(made, running)
            }
            _ => false,
        };
        if same {
            return Ok(());
        }
        Err(crate::vm::errors::simple_exception(
            "FiberError",
            "fiber called across threads",
            position,
        ))
    }

    /// The fiber a fiber is part-way through resuming, if it is: the next one
    /// up the stack, the one it relays for, or for the fiber a program starts
    /// on, the one it resumed at the bottom of the stack.
    pub(crate) fn fiber_resumed_by(&self, handle: usize) -> Option<usize> {
        if handle == ROOT_FIBER {
            return self
                .fiber_frames
                .first()
                .filter(|frame| frame.handoff == Handoff::Resume)
                .map(|frame| frame.handle);
        }
        if let Some(at) = self
            .fiber_frames
            .iter()
            .position(|frame| frame.handle == handle)
        {
            return self.fiber_frames.get(at + 1).map(|frame| frame.handle);
        }
        self.fibers.get(handle).and_then(|state| state.relaying)
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
                class_var_home: Vec::new(),
                class_var_cref_stack: Vec::new(),
                method_nesting_stack: Vec::new(),
                current_method_frame: Some(crate::vm::core::TOP_LEVEL_FRAME),
                lexical_home_frame: None,
                attached_block_flags: Vec::new(),
                running_block_breaks: Vec::new(),
            }
        });
        FiberContext {
            environment: std::mem::replace(&mut self.environment, taken.environment),
            call_stack: std::mem::replace(&mut self.call_stack, taken.call_stack),
            def_scope_stack: std::mem::replace(&mut self.def_scope_stack, taken.def_scope_stack),
            class_var_home: std::mem::replace(&mut self.class_var_home, taken.class_var_home),
            class_var_cref_stack: std::mem::replace(
                &mut self.class_var_cref_stack,
                taken.class_var_cref_stack,
            ),
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
            attached_block_flags: std::mem::replace(
                &mut self.attached_block_flags,
                taken.attached_block_flags,
            ),
            running_block_breaks: std::mem::replace(
                &mut self.running_block_breaks,
                taken.running_block_breaks,
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
