//! Fibers: a block that runs on a stack of its own and can suspend part-way
//! through, handing a value back to whoever resumed it.
//!
//! Every fiber runs on the one operating-system thread the interpreter runs
//! on. Only one of them holds the interpreter at a time, since `resume` and
//! `suspend` hand it over and wait, so the reference-counted objects the
//! interpreter is built from are never touched from two places at once.

use corosensei::stack::DefaultStack;
use corosensei::{Coroutine, CoroutineResult, Yielder};

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{BlockStatement, Object};
use crate::vm::VirtualMachine;

/// The handle the fiber a program starts on carries. It runs no block of its
/// own, so it is never resumed and never runs out.
pub(crate) const ROOT_FIBER: usize = usize::MAX;

/// The exception a fiber being stopped is unwound with. It is swallowed where
/// the fiber's body ends, so a program never sees it.
pub(crate) const FIBER_KILLED: &str = "__FiberKilled__";

/// How much room a fiber's own stack has. The interpreter nests a frame per
/// call, so a fiber running ordinary Ruby needs far more than a coroutine's
/// small default.
const FIBER_STACK_BYTES: usize = 32 * 1024 * 1024;

/// What a fiber hands back when it stops: a value it suspended with, or the
/// value its block answered.
pub(crate) enum FiberStep {
    Suspended(SuspendOutput),
    Finished(Object),
    Failed(MetorexError),
}

/// What `resume` hands into a fiber: the arguments it was called with.
type ResumeInput = Vec<Object>;
/// What a fiber hands out when it suspends: a value for whoever resumed it,
/// or a request to hand control to another fiber.
pub(crate) enum SuspendOutput {
    Yielded(Object),
    TransferTo {
        handle: usize,
        fiber: Object,
        given: Vec<Object>,
    },
}
/// What a fiber's block answers when it runs out.
type FiberOutput = Result<Object, MetorexError>;

/// The interpreter state that belongs to whichever fiber is running: the
/// scopes its names live in, and the frames a backtrace reads. A fiber that
/// suspends part-way through leaves these half-built, so they travel with it
/// rather than staying on the interpreter.
pub(crate) struct FiberContext {
    environment: crate::environment::Environment,
    call_stack: Vec<crate::vm::CallFrame>,
    def_scope_stack: Vec<std::rc::Rc<crate::class::Class>>,
    method_nesting_stack: Vec<Vec<std::rc::Rc<crate::class::Class>>>,
    /// The invocation a `return` written here belongs to. It travels with the
    /// fiber, so a method the interrupted side is part-way through still
    /// returns to itself once control comes back to it.
    current_method_frame: Option<u64>,
    lexical_home_frame: Option<Option<u64>>,
}

/// One fiber the program made, and the coroutine it runs on.
pub(crate) struct FiberState {
    running: Option<Coroutine<ResumeInput, SuspendOutput, FiberOutput>>,
    /// Whether the block has run out. A fiber that has is dead and cannot be
    /// resumed again.
    finished: bool,
    /// The handle the body suspends through, written once when the body first
    /// runs. It stands for a place on the fiber's own stack, which stays put
    /// for as long as the fiber is alive.
    yielder: *const Yielder<ResumeInput, SuspendOutput>,
    /// What the fiber had built when it last suspended, put back when it is
    /// resumed again.
    held: Option<FiberContext>,
    /// The Fiber object standing for this one, remembered so a chain can go
    /// back to it without the caller naming it again.
    object: Option<Object>,
    /// Where the block was written, which a fiber names when it reports
    /// itself.
    opened_at: Option<(String, usize)>,
    /// Whether the fiber is being stopped, which is what the suspend it is
    /// parked in reports rather than handing a value back.
    killing: bool,
    /// Whether the fiber was asked to block rather than hand control to a
    /// scheduler when it waits on something.
    blocking: bool,
    /// The names a fiber keeps for itself, which `Fiber[]` reads and writes.
    /// A fiber made without any inherits what the one making it held.
    storage: Option<Object>,
    /// The thread the fiber was made on. Only that thread may run it.
    owner: Option<Object>,
    /// What the fiber is to raise when it next holds the interpreter, which
    /// is how `Fiber#raise` reaches into a suspended one.
    raising: Option<Object>,
}

impl FiberState {
    pub(crate) fn is_alive(&self) -> bool {
        !self.finished
    }
}

/// The interpreter and the suspend handle a fiber's body needs, reached
/// through a raw pointer because the body runs on a stack of its own while
/// the interpreter stays where it was. Only the fiber holding the interpreter
/// reads either, so no two stacks reach them at once.
pub(crate) struct FiberFrame {
    handle: usize,
    fiber: Object,
}

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
            let answered = vm.execute_block_body(&block, first);
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
        let stepped = self.fiber_resume_within(handle, fiber, given, position);
        let left = self
            .globals()
            .get(crate::vm::native_methods::regexp_methods::LAST_MATCH)
            .unwrap_or(Object::Nil);
        self.fiber_last_matches.insert(handle, left);
        self.globals_mut()
            .set(crate::vm::native_methods::regexp_methods::LAST_MATCH, held);
        stepped
    }

    fn fiber_resume_within(
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
    fn fiber_step(
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
    fn fiber_object(&self, handle: usize) -> Object {
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
    fn swap_context(&mut self, wanted: Option<FiberContext>) -> FiberContext {
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
        let _ = self.fiber_step(handle, object, Vec::new(), position);
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

    /// Raise an exception inside a fiber, where it stands. A fiber that has
    /// not started yet is started and raises at its first statement.
    pub(crate) fn fiber_raise(
        &mut self,
        handle: usize,
        fiber: Object,
        raised: Object,
        position: Position,
    ) -> Result<FiberStep, MetorexError> {
        if handle == ROOT_FIBER {
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
        let Some(state) = self.fibers.get_mut(handle) else {
            return Err(crate::vm::errors::simple_exception(
                "FiberError",
                "attempt to resume a terminated fiber",
                position,
            ));
        };
        // A fiber that has never run has no suspend point to raise at, so it
        // is started and told to raise as soon as it hands control back.
        if state.held.is_none() {
            state.finished = true;
            state.running = None;
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
        state.raising = Some(raised);
        self.fiber_resume(handle, fiber, Vec::new(), position)
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
        let instance = crate::object::Instance::new(fiber_class);
        let made = std::rc::Rc::new(std::cell::RefCell::new(instance));
        made.borrow_mut()
            .set_var("__fiber__".to_string(), Object::Int(ROOT_FIBER as i64));
        let held = Object::Instance(made);
        self.root_fiber = Some(held.clone());
        held
    }
}

/// The line a frame was entered from, which its location records after the
/// file it sits in.
fn frame_line(frame: &crate::vm::CallFrame) -> usize {
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
fn ends_the_program(died_of: &Object) -> bool {
    let Object::Exception(details) = died_of else {
        return false;
    };
    matches!(
        details.borrow().exception_type.as_str(),
        "SystemExit" | "Interrupt" | "SignalException" | "NoMemoryError" | "SystemStackError"
    )
}

/// Whether an error is the one a stopped fiber unwinds with.
fn fiber_was_killed(trouble: &MetorexError) -> bool {
    let MetorexError::UncaughtException { exception, .. } = trouble else {
        return false;
    };
    let Object::Exception(details) = exception else {
        return false;
    };
    details.borrow().exception_type == FIBER_KILLED
}

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
            let mut made = crate::object::Instance::new(std::rc::Rc::clone(&fiber_class));
            made.set_var("__fiber__".to_string(), Object::Int(handle as i64));
            let held = Object::Instance(std::rc::Rc::new(std::cell::RefCell::new(made)));
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
    fn thread_start_arguments(&mut self, thread: &Object) -> Vec<Object> {
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
    }

    /// Let go of every lock a thread still holds. Ruby releases a thread's
    /// mutexes when it ends, so a lock its body never unlocked stops blocking
    /// whoever waits for it next.
    fn release_locks_held_by(&mut self, thread: &Object) {
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
    fn thread_holds_an_interrupt(&mut self) -> bool {
        let Object::Instance(running) = self.running_thread() else {
            return false;
        };
        let held = running.borrow().get_var("__thread_raise").cloned();
        matches!(held, Some(Object::Array(values)) if !values.borrow().is_empty())
    }

    /// Whether a thread was asked to take the program down with it when it
    /// dies of an exception.
    fn thread_aborts_the_program(&mut self, thread: &Object, position: Position) -> bool {
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
}

impl VirtualMachine {
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
}

impl VirtualMachine {
    /// Say on stderr what a thread died of, which a thread does for itself
    /// while it is still running.
    fn report_thread_death(&mut self, thread: &Object, died_of: &Object, position: Position) {
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
    fn thread_fiber_object(&mut self, thread: &Object, handle: usize) -> Object {
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
}

impl VirtualMachine {
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
