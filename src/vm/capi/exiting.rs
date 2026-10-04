//! C functions run when the program ends: `rb_set_end_proc` alongside the
//! `at_exit` blocks, and `ruby_vm_at_exit` once the interpreter is done.

use super::calls::kernel_function;
use super::handles::{QNIL, Value};
use super::{called_from, interpreter};
use std::cell::RefCell;
use std::sync::{Mutex, Once};

thread_local! {
    /// Each function `rb_set_end_proc` was handed, with its data.
    static END_PROCS: RefCell<Vec<(usize, Value)>> = const { RefCell::new(Vec::new()) };
}

/// Each function `ruby_vm_at_exit` was handed. The C library runs them from
/// `exit`, after the interpreter's own threads are done with.
static VM_EXIT_HOOKS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
static VM_EXIT_REGISTERED: Once = Once::new();

unsafe extern "C" {
    fn atexit(callback: extern "C" fn()) -> i32;
}

/// The block function behind an end proc, handed the end proc's place in
/// `END_PROCS` as a Fixnum.
extern "C-unwind" fn run_end_proc(
    _yielded: Value,
    place: Value,
    _count: i32,
    _values: *const Value,
    _block: Value,
) -> Value {
    let (function, data) = END_PROCS.with(|held| held.borrow()[place >> 1]);
    // SAFETY: `rb_set_end_proc` was handed a function taking one VALUE.
    let function: extern "C-unwind" fn(Value) = unsafe { std::mem::transmute(function) };
    function(data);
    QNIL
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_end_proc(function: *const (), data: Value) {
    let place = END_PROCS.with(|held| {
        let mut held = held.borrow_mut();
        held.push((function as usize, data));
        held.len() - 1
    });
    let method = super::fibers::block_function_method(run_end_proc as *const (), (place << 1) | 1);
    let block = crate::vm::program::block_for_method(method, called_from());
    interpreter().pending_block = Some(block);
    kernel_function("at_exit", Vec::new());
}

extern "C" fn run_vm_exit_hooks() {
    let hooks = VM_EXIT_HOOKS
        .lock()
        .map(|held| held.clone())
        .unwrap_or_default();
    for hook in hooks.into_iter().rev() {
        // SAFETY: `ruby_vm_at_exit` was handed a function taking a VM pointer.
        let hook: extern "C" fn(*mut std::ffi::c_void) = unsafe { std::mem::transmute(hook) };
        hook(std::ptr::null_mut());
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn ruby_vm_at_exit(function: *const ()) {
    if let Ok(mut hooks) = VM_EXIT_HOOKS.lock() {
        hooks.push(function as usize);
    }
    VM_EXIT_REGISTERED.call_once(|| {
        // SAFETY: atexit takes a function of no arguments that stays valid
        // for the life of the process.
        unsafe { atexit(run_vm_exit_hooks) };
    });
}
