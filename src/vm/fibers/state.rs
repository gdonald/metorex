// What a fiber is made of, and the state that follows whichever one
// is running.

use super::*;

/// The handle the fiber a program starts on carries. It runs no block of its
/// own, so it is never resumed and never runs out.
pub(crate) const ROOT_FIBER: usize = usize::MAX;

/// The exception a fiber being stopped is unwound with. It is swallowed where
/// the fiber's body ends, so a program never sees it.
pub(crate) const FIBER_KILLED: &str = "__FiberKilled__";

/// How much room a fiber's own stack has. The interpreter nests a frame per
/// call, so a fiber running ordinary Ruby needs far more than a coroutine's
/// small default.
pub(crate) const FIBER_STACK_BYTES: usize = 32 * 1024 * 1024;

/// What a fiber hands back when it stops: a value it suspended with, or the
/// value its block answered.
pub(crate) enum FiberStep {
    Suspended(SuspendOutput),
    Finished(Object),
    Failed(MetorexError),
}

/// What `resume` hands into a fiber: the arguments it was called with.
pub(crate) type ResumeInput = Vec<Object>;
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
pub(crate) type FiberOutput = Result<Object, MetorexError>;

/// The interpreter state that belongs to whichever fiber is running: the
/// scopes its names live in, and the frames a backtrace reads. A fiber that
/// suspends part-way through leaves these half-built, so they travel with it
/// rather than staying on the interpreter.
pub(crate) struct FiberContext {
    pub(crate) environment: crate::environment::Environment,
    pub(crate) call_stack: Vec<crate::vm::CallFrame>,
    pub(crate) def_scope_stack: Vec<std::rc::Rc<crate::class::Class>>,
    pub(crate) method_nesting_stack: Vec<Vec<std::rc::Rc<crate::class::Class>>>,
    /// The invocation a `return` written here belongs to. It travels with the
    /// fiber, so a method the interrupted side is part-way through still
    /// returns to itself once control comes back to it.
    pub(crate) current_method_frame: Option<u64>,
    pub(crate) lexical_home_frame: Option<Option<u64>>,
}

/// One fiber the program made, and the coroutine it runs on.
pub(crate) struct FiberState {
    pub(crate) running: Option<Coroutine<ResumeInput, SuspendOutput, FiberOutput>>,
    /// Whether the block has run out. A fiber that has is dead and cannot be
    /// resumed again.
    pub(crate) finished: bool,
    /// The handle the body suspends through, written once when the body first
    /// runs. It stands for a place on the fiber's own stack, which stays put
    /// for as long as the fiber is alive.
    pub(crate) yielder: *const Yielder<ResumeInput, SuspendOutput>,
    /// What the fiber had built when it last suspended, put back when it is
    /// resumed again.
    pub(crate) held: Option<FiberContext>,
    /// The Fiber object standing for this one, remembered so a chain can go
    /// back to it without the caller naming it again.
    pub(crate) object: Option<Object>,
    /// Where the block was written, which a fiber names when it reports
    /// itself.
    pub(crate) opened_at: Option<(String, usize)>,
    /// Whether the fiber is being stopped, which is what the suspend it is
    /// parked in reports rather than handing a value back.
    pub(crate) killing: bool,
    /// Whether the fiber was asked to block rather than hand control to a
    /// scheduler when it waits on something.
    pub(crate) blocking: bool,
    /// The names a fiber keeps for itself, which `Fiber[]` reads and writes.
    /// A fiber made without any inherits what the one making it held.
    pub(crate) storage: Option<Object>,
    /// The thread the fiber was made on. Only that thread may run it.
    pub(crate) owner: Option<Object>,
    /// What the fiber is to raise when it next holds the interpreter, which
    /// is how `Fiber#raise` reaches into a suspended one.
    pub(crate) raising: Option<Object>,
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
    pub(crate) handle: usize,
    pub(crate) fiber: Object,
}
