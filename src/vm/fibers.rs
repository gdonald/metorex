//! Fibers: a block that runs on a stack of its own and can suspend part-way
//! through, handing a value back to whoever resumed it.
//!
//! Every fiber runs on the one operating-system thread the interpreter runs
//! on. Only one of them holds the interpreter at a time, since `resume` and
//! `suspend` hand it over and wait, so the reference-counted objects the
//! interpreter is built from are never touched from two places at once.

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
        let running = Coroutine::new(move |yielder, first: ResumeInput| {
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
    pub(crate) fn fiber_resume(
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
        let taken = wanted.unwrap_or_else(|| FiberContext {
            environment: crate::environment::Environment::new(),
            call_stack: Vec::new(),
            def_scope_stack: Vec::new(),
            method_nesting_stack: Vec::new(),
        });
        FiberContext {
            environment: std::mem::replace(&mut self.environment, taken.environment),
            call_stack: std::mem::replace(&mut self.call_stack, taken.call_stack),
            def_scope_stack: std::mem::replace(&mut self.def_scope_stack, taken.def_scope_stack),
            method_nesting_stack: std::mem::replace(
                &mut self.method_nesting_stack,
                taken.method_nesting_stack,
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
