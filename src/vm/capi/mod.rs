//! The C extension API: the `rb_*` functions an extension built against the
//! headers under `include/` calls, and the loading of such an extension.
//!
//! The functions are exported from the binary and reach the interpreter
//! through the pointer `enter` keeps for the duration of each call into C.
//! An error raised inside one of them unwinds through the C frames between
//! it and the call into C that is waiting for it.

mod arguments;
mod arrays;
mod bignums;
mod blocks;
mod calls;
mod classes;
mod coercion;
mod collection;
mod data;
mod definitions;
mod encodings;
mod enumerators;
mod exceptions;
mod exports;
mod fibers;
mod files;
mod globals;
mod handles;
mod hashes;
mod integers;
mod loading;
mod methods;
mod modules;
mod mutexes;
mod numbers;
mod numerics;
mod procs;
mod ranges;
mod regexps;
mod sets;
mod strings;
mod structs;
mod subclasses;
mod symbols;
mod tables;
mod threads;
mod times;
mod tracepoints;
mod utilities;

pub(crate) use data::wrapped_size;
pub(crate) use globals::{HookedGlobal, read_hooked_global, write_hooked_global};
pub(crate) use methods::CFunction;

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use std::cell::{Cell, RefCell};

/// Where the headers a C extension is compiled against live.
pub(crate) const HEADER_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/include");

thread_local! {
    static RUNNING_INTERPRETER: Cell<*mut VirtualMachine> = const { Cell::new(std::ptr::null_mut()) };
    static PENDING_ERROR: RefCell<Option<MetorexError>> = const { RefCell::new(None) };
    /// Where in Ruby code the innermost call into C now running was made.
    static CALLED_FROM: Cell<Position> = Cell::new(Position::default());
    /// The block handed to the innermost C method now running, if any.
    static CALLED_WITH_BLOCK: RefCell<Option<Object>> = const { RefCell::new(None) };
    /// The exception `rb_errinfo` answers for the C method running now,
    /// which starts as nil whatever `$!` holds in the Ruby code that called it.
    static ERROR_INFO: RefCell<Object> = const { RefCell::new(Object::Nil) };
    /// Whether the C method running now was handed keywords.
    static KEYWORDS_GIVEN: Cell<bool> = const { Cell::new(false) };
    /// The method the C function running now was called as, which
    /// `rb_call_super` continues past.
    static RUNNING_METHOD: RefCell<Option<RunningMethod>> = const { RefCell::new(None) };
}

/// A method backed by a C function, as it was called: on which object,
/// under which name, and found in which class or module.
#[derive(Clone)]
pub(crate) struct RunningMethod {
    pub(crate) receiver: Object,
    pub(crate) name: String,
    pub(crate) owner: std::rc::Rc<crate::class::Class>,
}

/// What a C function unwinds with when it raises. The error itself waits in
/// `PENDING_ERROR`, since what it holds cannot cross threads.
struct RaisedThroughC;

/// How Ruby code called into C: from where, with which block, and whether
/// the last argument was passed as keywords.
pub(crate) struct Caller {
    pub(crate) position: Position,
    pub(crate) block: Option<Object>,
    pub(crate) keywords_given: bool,
    pub(crate) method: Option<RunningMethod>,
}

/// Runs `wait`, which hands the turn to other threads, and puts back what
/// the C API knows about the call into C running now, since a thread that
/// ran meanwhile may be waiting inside a call into C of its own.
fn keeping_call_state<T>(wait: impl FnOnce() -> T) -> T {
    let interpreter = RUNNING_INTERPRETER.with(Cell::get);
    let position = CALLED_FROM.with(Cell::get);
    let block = CALLED_WITH_BLOCK.with(|held| held.borrow().clone());
    let keywords = KEYWORDS_GIVEN.with(Cell::get);
    let error = ERROR_INFO.with(|held| held.borrow().clone());
    let method = RUNNING_METHOD.with(|held| held.borrow().clone());
    let answered = wait();
    RUNNING_INTERPRETER.with(|held| held.set(interpreter));
    CALLED_FROM.with(|held| held.set(position));
    CALLED_WITH_BLOCK.with(|held| held.replace(block));
    KEYWORDS_GIVEN.with(|held| held.set(keywords));
    ERROR_INFO.with(|held| held.replace(error));
    RUNNING_METHOD.with(|held| held.replace(method));
    answered
}

/// Whether this operating system thread is the one the interpreter runs
/// on, inside a call into C.
fn on_interpreter_thread() -> bool {
    !RUNNING_INTERPRETER.with(Cell::get).is_null()
}

/// Runs `call`, made the way `caller` says, with the interpreter reachable
/// from the exported functions, and turns an error raised inside them into
/// the `Err` it was.
pub(crate) fn enter<T>(
    machine: &mut VirtualMachine,
    caller: Caller,
    call: impl FnOnce() -> T,
) -> Result<T, MetorexError> {
    let outer = RUNNING_INTERPRETER.with(|held| held.replace(machine as *mut VirtualMachine));
    let outer_position = CALLED_FROM.with(|held| held.replace(caller.position));
    let outer_block = CALLED_WITH_BLOCK.with(|held| held.replace(caller.block));
    let outer_keywords = KEYWORDS_GIVEN.with(|held| held.replace(caller.keywords_given));
    let outer_error = ERROR_INFO.with(|held| held.replace(Object::Nil));
    let outer_method = RUNNING_METHOD.with(|held| held.replace(caller.method));
    let answered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(call));
    strings::carry_writes_back();
    CALLED_WITH_BLOCK.with(|held| held.replace(outer_block));
    KEYWORDS_GIVEN.with(|held| held.set(outer_keywords));
    ERROR_INFO.with(|held| held.replace(outer_error));
    RUNNING_METHOD.with(|held| held.replace(outer_method));
    CALLED_FROM.with(|held| held.set(outer_position));
    RUNNING_INTERPRETER.with(|held| held.set(outer));
    match answered {
        Ok(value) => Ok(value),
        Err(payload) if payload.is::<RaisedThroughC>() => Err(PENDING_ERROR
            .with(|pending| pending.borrow_mut().take())
            .expect("an error raised through C is held until it is caught")),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// The interpreter the C code running now was called from, with what C
/// wrote into strings carried back into them, since Ruby code run from here
/// may read them.
fn interpreter() -> &'static mut VirtualMachine {
    strings::carry_writes_back();
    let running = RUNNING_INTERPRETER.with(Cell::get);
    assert!(
        !running.is_null(),
        "a C API function was called outside a call into C"
    );
    // SAFETY: `enter` set the pointer from a live `&mut VirtualMachine` whose
    // caller does not touch it until the C call it is waiting on returns.
    unsafe { &mut *running }
}

/// Where in Ruby code the call into C running now was made.
fn called_from() -> Position {
    CALLED_FROM.with(Cell::get)
}

/// Whether the C method running now was handed keywords.
fn keywords_given() -> bool {
    KEYWORDS_GIVEN.with(Cell::get)
}

/// The method the C function running now was called as, if any.
fn running_method() -> Option<RunningMethod> {
    RUNNING_METHOD.with(|held| held.borrow().clone())
}

/// The block handed to the C method running now.
fn called_with_block() -> Option<Object> {
    CALLED_WITH_BLOCK.with(|held| held.borrow().clone())
}

/// Ends the C function running now with `error`, unwinding to the `enter`
/// that called into C.
fn raise(error: MetorexError) -> ! {
    PENDING_ERROR.with(|pending| *pending.borrow_mut() = Some(error));
    std::panic::resume_unwind(Box::new(RaisedThroughC))
}

/// The value of `answered`, or the error it holds raised through C.
fn or_raise<T>(answered: Result<T, MetorexError>) -> T {
    answered.unwrap_or_else(|error| raise(error))
}
