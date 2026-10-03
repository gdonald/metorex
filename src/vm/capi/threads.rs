//! Threads from C: making one around a C function, the thread running now
//! and its locals, waking and waiting, and running a C function on an
//! operating system thread of its own while the other threads take turns.

use super::calls::{call, top_level_module};
use super::handles::{Value, to_object, to_value};
use super::methods::{CFunction, THREAD_FUNCTION_ARITY};
use super::symbols::symbol_name;
use super::{called_from, interpreter, keeping_call_state, on_interpreter_thread, raise};
use crate::object::{Method, Object};
use std::ffi::c_void;
use std::os::unix::thread::JoinHandleExt;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

fn thread_class() -> Object {
    top_level_module("Thread")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_current() -> Value {
    to_value(&call(thread_class(), "current", Vec::new()))
}

/// Whether the thread running now is the only one alive.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_alone() -> i32 {
    match call(thread_class(), "list", Vec::new()) {
        Object::Array(threads) => (threads.borrow().len() == 1) as i32,
        _ => 1,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_local_aref(thread: Value, name: Value) -> Value {
    let arguments = vec![Object::symbol(symbol_name(name))];
    to_value(&call(to_object(thread), "[]", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_local_aset(thread: Value, name: Value, value: Value) -> Value {
    let arguments = vec![Object::symbol(symbol_name(name)), to_object(value)];
    call(to_object(thread), "[]=", arguments);
    value
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_wakeup(thread: Value) -> Value {
    to_value(&call(to_object(thread), "wakeup", Vec::new()))
}

/// A thread whose body calls `function` with `data` and answers what it
/// answers.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_create(function: *const (), data: *mut c_void) -> Value {
    let mut method = Method::new("call".to_string(), vec!["values".to_string()], Vec::new());
    method.variadic_param = Some((0, "values".to_string()));
    method.c_function = Some(CFunction {
        address: function as usize,
        arity: THREAD_FUNCTION_ARITY,
        data: data as usize,
    });
    let target = Object::Method(Rc::new(method));
    let block = crate::vm::program::block_for_method(target, called_from());
    interpreter().pending_block = Some(block);
    to_value(&call(thread_class(), "new", Vec::new()))
}

/// Whether this operating system thread is one the interpreter runs.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn ruby_native_thread_p() -> i32 {
    on_interpreter_thread() as i32
}

/// Whether this operating system thread holds the interpreter, which a
/// function `rb_thread_call_without_gvl` runs does not.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn ruby_thread_has_gvl_p() -> i32 {
    on_interpreter_thread() as i32
}

/// `struct timeval`, as the platform lays it out.
#[repr(C)]
pub struct Timeval {
    seconds: libc::time_t,
    microseconds: libc::suseconds_t,
}

/// Waits for the length `interval` gives, with the other threads taking
/// turns meanwhile.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_wait_for(interval: Timeval) {
    let length = std::time::Duration::from_secs(interval.seconds.max(0) as u64)
        + std::time::Duration::from_micros(interval.microseconds.max(0) as u64);
    let deadline = std::time::Instant::now() + length;
    let position = called_from();
    while std::time::Instant::now() < deadline {
        keeping_call_state(|| interpreter().wait_for_other_threads(position));
        if let Err(stopped) = interpreter().raise_if_thread_killed(position) {
            raise(stopped);
        }
    }
}

/// What `RUBY_UBF_IO` and `RUBY_UBF_PROCESS` stand for: interrupt the
/// operating system thread running the function, so a system call it waits
/// in ends with EINTR.
const INTERRUPT_THE_CALL: usize = usize::MAX;

/// The signal that interrupts a function `rb_thread_call_without_gvl` runs,
/// which is the one MRI sends its own blocked threads.
const INTERRUPTING_SIGNAL: libc::c_int = libc::SIGVTALRM;

extern "C" fn interrupted(_signal: libc::c_int) {}

/// Catches the interrupting signal with a handler that does nothing and
/// does not restart the call it arrives in, so that call ends with EINTR.
fn catch_interrupting_signal() {
    static CAUGHT: std::sync::Once = std::sync::Once::new();
    CAUGHT.call_once(|| {
        // SAFETY: a zeroed sigaction with an empty mask, no flags and a
        // handler that touches nothing is a valid one to install.
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = interrupted as extern "C" fn(libc::c_int) as usize;
            libc::sigemptyset(&mut action.sa_mask);
            libc::sigaction(INTERRUPTING_SIGNAL, &action, std::ptr::null_mut());
        }
    });
}

/// Runs `function` with `data` on an operating system thread of its own,
/// with the thread calling it asleep and the others taking turns until it
/// returns. Waking the thread, a signal arriving while it is the main one,
/// or killing it runs `unblock` with `unblock_data` to make `function`
/// return.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_thread_call_without_gvl(
    function: *const (),
    data: *mut c_void,
    unblock: *const (),
    unblock_data: *mut c_void,
) -> *mut c_void {
    let interrupts_the_call = unblock as usize == INTERRUPT_THE_CALL;
    if interrupts_the_call {
        catch_interrupting_signal();
    }
    let finished = Arc::new(AtomicBool::new(false));
    let answer = Arc::new(AtomicUsize::new(0));
    let (address, argument) = (function as usize, data as usize);
    let worker = {
        let finished = Arc::clone(&finished);
        let answer = Arc::clone(&answer);
        std::thread::spawn(move || {
            // SAFETY: the headers declare the function as taking a pointer
            // and answering one.
            let function: extern "C" fn(*mut c_void) -> *mut c_void =
                unsafe { std::mem::transmute(address as *const ()) };
            answer.store(function(argument as *mut c_void) as usize, Ordering::SeqCst);
            finished.store(true, Ordering::SeqCst);
        })
    };
    let worker_thread = worker.as_pthread_t();
    let unblock_now = || {
        if interrupts_the_call {
            // SAFETY: the worker has not been joined, so its thread exists.
            unsafe { libc::pthread_kill(worker_thread, INTERRUPTING_SIGNAL) };
        } else if !unblock.is_null() {
            // SAFETY: the headers declare the unblock function as taking
            // the pointer handed with it.
            let unblock: extern "C" fn(*mut c_void) = unsafe { std::mem::transmute(unblock) };
            unblock(unblock_data);
        }
    };
    let stopped = wait_until(&finished, interrupts_the_call, unblock_now);
    let _ = worker.join();
    if let Some(stopped) = stopped {
        raise(stopped);
    }
    answer.load(Ordering::SeqCst) as *mut c_void
}

/// Hands the turn to the other threads until `finished` is set, reading as
/// asleep meanwhile, and calls `unblock` each time something asks the wait
/// to end. An unblock that interrupts the call is repeated until the call
/// returns, since the interrupt is lost when it arrives before the call
/// starts waiting. Answers the error that killed the thread, if one did.
fn wait_until(
    finished: &AtomicBool,
    repeat_unblock: bool,
    unblock: impl Fn(),
) -> Option<crate::error::MetorexError> {
    let position = called_from();
    let machine = interpreter();
    let on_main = !machine.running_a_thread_body();
    let thread = machine.running_thread();
    let Object::Instance(instance) = &thread else {
        unreachable!("a thread is an instance of Thread")
    };
    let mut asked_to_end = false;
    let mut stopped = None;
    let mut signals_seen = crate::vm::signals::signals_taken();
    while !finished.load(Ordering::SeqCst) {
        instance
            .borrow_mut()
            .set_var("__thread_waiting".to_string(), Object::Bool(true));
        keeping_call_state(|| {
            let machine = interpreter();
            if on_main {
                if !machine.step_pending_threads(position) {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            } else {
                let _ = machine.fiber_suspend(Object::Nil, position);
            }
        });
        let woken = !matches!(
            instance.borrow().get_var("__thread_waiting"),
            Some(Object::Bool(true))
        );
        let machine = interpreter();
        if on_main
            && crate::vm::signals::signal_pending()
            && let Err(error) = machine.deliver_pending_signals(position)
        {
            stopped.get_or_insert(error);
        }
        // A signal is the main thread's to take, wherever it was handled.
        let taken = crate::vm::signals::signals_taken();
        let signalled = on_main && taken != signals_seen;
        signals_seen = taken;
        let mut ask = woken || signalled;
        if stopped.is_none()
            && let Err(error) = machine.raise_if_thread_killed(position)
        {
            stopped = Some(error);
            ask = true;
        }
        if ask || (asked_to_end && repeat_unblock) {
            asked_to_end = true;
            unblock();
        }
    }
    instance
        .borrow_mut()
        .set_var("__thread_waiting".to_string(), Object::Bool(false));
    stopped
}
