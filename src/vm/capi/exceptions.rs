//! Exceptions from C: making one, raising one, the exception `rb_errinfo`
//! holds, and the FrozenError and SystemCallError helpers.

use super::calls::{call, class_name_of, top_level_module};
use super::handles::{QNIL, Value, objects_from, to_object, to_value};
use super::strings::check_string;
use super::{ERROR_INFO, called_from, raise};
use crate::object::Object;
use std::cell::RefCell;
use std::ffi::{CStr, c_char};

thread_local! {
    /// The objects `rb_error_frozen_object` is inspecting now, so an
    /// `inspect` that reports the same object again is cut short.
    static INSPECTING: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

fn type_error(message: &str) -> ! {
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        message,
        called_from(),
    ))
}

/// Raises `exception` as Ruby's `raise` would.
fn raise_object(exception: Object) -> ! {
    call(top_level_module("Kernel"), "raise", vec![exception]);
    unreachable!("Kernel#raise always raises")
}

fn new_exception(class: Value, message: Object) -> Value {
    to_value(&call(to_object(class), "new", vec![message]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_errinfo() -> Value {
    ERROR_INFO.with(|held| to_value(&held.borrow()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_errinfo(exception: Value) {
    let exception = to_object(exception);
    let is_exception = matches!(
        call(
            exception.clone(),
            "is_a?",
            vec![top_level_module("Exception")]
        ),
        Object::Bool(true)
    );
    if !matches!(exception, Object::Nil) && !is_exception {
        type_error("assigning non-exception to $!");
    }
    ERROR_INFO.with(|held| *held.borrow_mut() = exception);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_exc_new(class: Value, text: *const u8, length: i64) -> Value {
    let bytes = if length > 0 {
        // SAFETY: C hands over `length` bytes at `text`.
        unsafe { std::slice::from_raw_parts(text, length as usize) }
    } else {
        &[]
    };
    let message = Object::string(String::from_utf8_lossy(bytes).into_owned());
    new_exception(class, message)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_exc_new_cstr(class: Value, text: *const c_char) -> Value {
    // SAFETY: C hands over a NUL-terminated string.
    let message = unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned();
    new_exception(class, Object::string(message))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_exc_new_str(class: Value, message: Value) -> Value {
    let message = super::strings::string_value(to_object(message));
    new_exception(class, message)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_exc_raise(exception: Value) -> ! {
    raise_object(to_object(exception))
}

/// What `inspect` says about `object`, or ` ...` when it is already being
/// inspected for a FrozenError further out.
fn frozen_inspection(object: Value) -> String {
    if INSPECTING.with(|held| held.borrow().contains(&object)) {
        return " ...".to_string();
    }
    INSPECTING.with(|held| held.borrow_mut().push(object));
    let inspected =
        std::panic::catch_unwind(|| call(to_object(object), "inspect", Vec::new()).to_string());
    INSPECTING.with(|held| held.borrow_mut().pop());
    inspected.unwrap_or_else(|payload| std::panic::resume_unwind(payload))
}

/// Raises the FrozenError Ruby raises for a change to `object`, whether or
/// not it is frozen.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_error_frozen_object(object: Value) -> ! {
    let mut message = format!(
        "can't modify frozen {}: {}",
        class_name_of(to_object(object)),
        frozen_inspection(object)
    );
    if let Object::String(string) = to_object(object)
        && let Some(written_at) = string.created_at()
    {
        message.push_str(&format!(", created at {}", written_at));
    }
    let mut keywords = indexmap::IndexMap::new();
    keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
    keywords.insert(":receiver".to_string(), to_object(object));
    let arguments = vec![
        Object::string(message),
        Object::Dict(std::rc::Rc::new(RefCell::new(keywords))),
    ];
    raise_object(call(top_level_module("FrozenError"), "new", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_syserr_new(number: i32, message: *const c_char) -> Value {
    let message = if message.is_null() {
        QNIL
    } else {
        // SAFETY: C hands over a NUL-terminated string or NULL.
        let text = unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned();
        to_value(&Object::string(text))
    };
    rb_syserr_new_str(number, message)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_syserr_new_str(number: i32, message: Value) -> Value {
    let arguments = vec![to_object(message), Object::Int(i64::from(number))];
    to_value(&call(top_level_module("SystemCallError"), "new", arguments))
}

/// The exception `raise` would make of its arguments, without raising it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_make_exception(count: i32, values: *const Value) -> Value {
    let arguments = objects_from(i64::from(count), values);
    if arguments.is_empty() {
        return QNIL;
    }
    if arguments.len() > 3 {
        super::arguments::rb_error_arity(count, 0, 3);
    }
    let first = arguments[0].clone();
    if arguments.len() == 1
        && !matches!(first, Object::Nil)
        && let Some(message) = check_string(first.clone())
    {
        return to_value(&call(
            top_level_module("RuntimeError"),
            "new",
            vec![message],
        ));
    }
    if !super::calls::answers(&first, "exception") {
        type_error("exception class/object expected");
    }
    let made = call(
        first,
        "exception",
        arguments[1..arguments.len().min(2)].to_vec(),
    );
    let is_exception = matches!(
        call(made.clone(), "is_a?", vec![top_level_module("Exception")]),
        Object::Bool(true)
    );
    if !is_exception {
        type_error("exception object expected");
    }
    if let Some(backtrace) = arguments.get(2) {
        call(made.clone(), "set_backtrace", vec![backtrace.clone()]);
    }
    to_value(&made)
}
