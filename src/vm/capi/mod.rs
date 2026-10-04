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
mod control;
mod data;

/// The class variable holding the address of the allocator C gave a class.
pub(crate) fn data_allocator_var() -> &'static str {
    data::ALLOCATOR_VAR
}
mod debug;
mod definitions;
mod digests;
mod encoded_text;
mod encodings;
mod enumerators;
mod exceptions;
mod exiting;
mod exports;
mod fibers;
mod files;
mod flags;
mod globals;
mod handles;
mod hashes;
mod integers;
mod io;
mod loading;
mod methods;
mod modules;
mod mutexes;
mod numbers;
mod numerics;
mod objects;
mod procs;
mod ranges;
mod regexps;
mod sets;
mod string_functions;
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
pub(crate) use objects::is_not_implemented;

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
    /// Whether that block was an existing Proc passed with `&` rather than
    /// a block written at the call.
    static BLOCK_FROM_AMPERSAND: Cell<bool> = const { Cell::new(false) };
    /// The exception `rb_errinfo` answers. Only C sets it, through
    /// `rb_set_errinfo`, `rb_protect`, `rb_rescue` and `rb_ensure`, so a
    /// Ruby `rescue` leaves it nil whatever `$!` holds.
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
    pub(crate) block_from_ampersand: bool,
    pub(crate) keywords_given: bool,
    pub(crate) method: Option<RunningMethod>,
}

impl Caller {
    /// A call into C made at `position` with no block, keywords or method.
    pub(crate) fn at(position: Position) -> Self {
        Caller {
            position,
            block: None,
            block_from_ampersand: false,
            keywords_given: false,
            method: None,
        }
    }
}

/// Whether a call into C is under way, somewhere below the code running now.
fn c_is_running() -> bool {
    !RUNNING_INTERPRETER.with(Cell::get).is_null()
}

/// Runs `wait`, which hands the turn to other threads, and puts back what
/// the C API knows about the call into C running now, since a thread that
/// ran meanwhile may be waiting inside a call into C of its own.
fn keeping_call_state<T>(wait: impl FnOnce() -> T) -> T {
    let interpreter = RUNNING_INTERPRETER.with(Cell::get);
    let position = CALLED_FROM.with(Cell::get);
    let block = CALLED_WITH_BLOCK.with(|held| held.borrow().clone());
    let from_ampersand = BLOCK_FROM_AMPERSAND.with(Cell::get);
    let keywords = KEYWORDS_GIVEN.with(Cell::get);
    let error = ERROR_INFO.with(|held| held.borrow().clone());
    let method = RUNNING_METHOD.with(|held| held.borrow().clone());
    let answered = wait();
    RUNNING_INTERPRETER.with(|held| held.set(interpreter));
    CALLED_FROM.with(|held| held.set(position));
    CALLED_WITH_BLOCK.with(|held| held.replace(block));
    BLOCK_FROM_AMPERSAND.with(|held| held.set(from_ampersand));
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
    let outer_ampersand =
        BLOCK_FROM_AMPERSAND.with(|held| held.replace(caller.block_from_ampersand));
    let outer_keywords = KEYWORDS_GIVEN.with(|held| held.replace(caller.keywords_given));
    let outer_method = RUNNING_METHOD.with(|held| held.replace(caller.method));
    let answered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(call));
    strings::carry_writes_back();
    arrays::carry_element_writes_back();
    flags::carry_flag_writes_back();
    CALLED_WITH_BLOCK.with(|held| held.replace(outer_block));
    BLOCK_FROM_AMPERSAND.with(|held| held.set(outer_ampersand));
    KEYWORDS_GIVEN.with(|held| held.set(outer_keywords));
    RUNNING_METHOD.with(|held| held.replace(outer_method));
    CALLED_FROM.with(|held| held.set(outer_position));
    RUNNING_INTERPRETER.with(|held| held.set(outer));
    answered.map_err(raised_error)
}

/// The error a C function raised, from the payload it unwound with. A
/// panic that is not a raised error goes on unwinding.
fn raised_error(payload: Box<dyn std::any::Any + Send>) -> MetorexError {
    if !payload.is::<RaisedThroughC>() {
        std::panic::resume_unwind(payload);
    }
    PENDING_ERROR
        .with(|pending| pending.borrow_mut().take())
        .expect("an error raised through C is held until it is caught")
}

/// The interpreter the C code running now was called from, with what C
/// wrote into strings, arrays and object flags carried back into them,
/// since Ruby code run from here may read them.
fn interpreter() -> &'static mut VirtualMachine {
    strings::carry_writes_back();
    arrays::carry_element_writes_back();
    flags::carry_flag_writes_back();
    running_interpreter()
}

/// The interpreter the C code running now was called from, as it stands.
fn running_interpreter() -> &'static mut VirtualMachine {
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

/// Whether the block handed to the C method running now was an existing
/// Proc passed with `&`.
fn block_from_ampersand() -> bool {
    BLOCK_FROM_AMPERSAND.with(Cell::get)
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
