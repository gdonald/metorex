//! Mutexes from C: making one, locking and unlocking it, sleeping on it, and
//! running a C function while holding it.

use super::calls::{call, top_level_module};
use super::handles::{QNIL, Value, to_object, to_value};
use crate::object::Object;

fn send(mutex: Value, name: &str, arguments: Vec<Object>) -> Value {
    to_value(&call(to_object(mutex), name, arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mutex_new() -> Value {
    to_value(&call(top_level_module("Mutex"), "new", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mutex_locked_p(mutex: Value) -> Value {
    send(mutex, "locked?", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mutex_trylock(mutex: Value) -> Value {
    send(mutex, "try_lock", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mutex_lock(mutex: Value) -> Value {
    send(mutex, "lock", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mutex_unlock(mutex: Value) -> Value {
    send(mutex, "unlock", Vec::new())
}

/// Sleeps on the mutex for `timeout` seconds, or until woken when it is nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mutex_sleep(mutex: Value, timeout: Value) -> Value {
    let arguments = if timeout == QNIL {
        Vec::new()
    } else {
        vec![to_object(timeout)]
    };
    send(mutex, "sleep", arguments)
}

/// Runs `function` with `data` while holding the mutex, and unlocks it
/// however the function ends. What the function takes and answers is
/// handed through untouched, so it need not be a VALUE.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mutex_synchronize(
    mutex: Value,
    function: extern "C-unwind" fn(Value) -> Value,
    data: Value,
) -> Value {
    rb_mutex_lock(mutex);
    let answered = std::panic::catch_unwind(|| function(data));
    rb_mutex_unlock(mutex);
    answered.unwrap_or_else(|payload| std::panic::resume_unwind(payload))
}
