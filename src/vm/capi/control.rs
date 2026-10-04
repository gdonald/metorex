//! Running C functions that may raise: `rb_protect`, `rb_rescue`,
//! `rb_rescue2` and `rb_ensure` catching what Ruby raised through them,
//! `rb_jump_tag` raising it again, `catch` and `throw`, and the rest of the
//! Kernel functions C calls for its own control flow.

use super::calls::{call, kernel_function, top_level_module};
use super::exceptions::rb_syserr_new;
use super::handles::{QNIL, Value, to_object, to_value};
use super::{ERROR_INFO, called_from, called_with_block};
use super::{interpreter, raise, running_method};
use crate::error::MetorexError;
use crate::object::Object;
use std::cell::RefCell;
use std::ffi::{CStr, c_char};

/// The state `rb_protect` reports for an exception, MRI's `TAG_RAISE`.
const RAISED: i32 = 6;
/// The state `rb_protect` reports for a `throw`, MRI's `TAG_THROW`.
const THROWN: i32 = 7;
/// The state `rb_protect` reports for a `break` and the other jumps out of a
/// block, MRI's `TAG_BREAK`.
const BROKEN_OUT: i32 = 2;

thread_local! {
    /// What the last `rb_protect` caught, which `rb_jump_tag` raises again.
    static CAUGHT: RefCell<Option<MetorexError>> = const { RefCell::new(None) };
    /// The function and object of each `rb_exec_recursive` running now.
    static RECURSING: RefCell<Vec<(usize, Value)>> = const { RefCell::new(Vec::new()) };
}

/// Runs `run`, answering the error it raised through C instead of letting
/// it unwind further.
pub(super) fn caught<T>(run: impl FnOnce() -> T) -> Result<T, MetorexError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).map_err(super::raised_error)
}

/// The Ruby exception `error` carries, or None for a jump out of a block or
/// a `throw`, which no `rescue` takes.
fn exception_in(error: &MetorexError) -> Option<Object> {
    match crate::vm::begin_rescue::as_rescuable(error.clone()) {
        MetorexError::UncaughtException { exception, .. } => Some(exception),
        _ => None,
    }
}

fn is_a(object: &Object, module: Object) -> bool {
    call(object.clone(), "is_a?", vec![module]).is_truthy()
}

/// Runs `run` with `exception` standing as `$!` and as what `rb_errinfo`
/// answers, putting both back however it ends.
fn handling<T>(exception: &Object, run: impl FnOnce() -> T) -> T {
    let machine = interpreter();
    let standing = machine.globals().get("!").unwrap_or(Object::Nil);
    let held_info = ERROR_INFO.with(|held| held.replace(exception.clone()));
    machine.set_current_exception(exception.clone());
    let answered = caught(run);
    interpreter().restore_current_exception(standing);
    ERROR_INFO.with(|held| held.replace(held_info));
    answered.unwrap_or_else(|error| raise(error))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_protect(
    function: extern "C-unwind" fn(Value) -> Value,
    data: Value,
    state: *mut i32,
) -> Value {
    let (answered, tag) = match caught(|| function(data)) {
        Ok(answered) => (answered, 0),
        Err(error) => {
            let tag = match (&error, exception_in(&error)) {
                (_, Some(exception)) => {
                    ERROR_INFO.with(|held| held.replace(exception));
                    RAISED
                }
                (MetorexError::Throw { .. }, None) => THROWN,
                (_, None) => BROKEN_OUT,
            };
            CAUGHT.with(|held| held.replace(Some(error)));
            (QNIL, tag)
        }
    };
    if !state.is_null() {
        // SAFETY: C hands over a pointer to an int, or NULL.
        unsafe { *state = tag };
    }
    answered
}

/// Raises again what the last `rb_protect` caught.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_jump_tag(_state: i32) -> ! {
    let error = CAUGHT
        .with(|held| held.borrow_mut().take())
        .unwrap_or_else(|| {
            crate::vm::errors::simple_exception(
                "RuntimeError",
                "rb_jump_tag called with nothing rb_protect caught",
                called_from(),
            )
        });
    ERROR_INFO.with(|held| held.replace(Object::Nil));
    raise(error)
}

/// Runs `body`, and `rescue` with the exception when it raises one that
/// `rescues` says to take, answering nil for a NULL `rescue`.
fn rescuing(
    body: extern "C-unwind" fn(Value) -> Value,
    data: Value,
    rescue: Option<extern "C-unwind" fn(Value, Value) -> Value>,
    rescue_data: Value,
    rescues: impl Fn(&Object) -> bool,
) -> Value {
    let error = match caught(|| body(data)) {
        Ok(answered) => return answered,
        Err(error) => error,
    };
    let Some(exception) = exception_in(&error).filter(|held| rescues(held)) else {
        raise(error)
    };
    let Some(rescue) = rescue else {
        return QNIL;
    };
    handling(&exception, || rescue(rescue_data, to_value(&exception)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rescue(
    body: extern "C-unwind" fn(Value) -> Value,
    data: Value,
    rescue: Option<extern "C-unwind" fn(Value, Value) -> Value>,
    rescue_data: Value,
) -> Value {
    let standard_error = top_level_module("StandardError");
    rescuing(body, data, rescue, rescue_data, |exception| {
        is_a(exception, standard_error.clone())
    })
}

/// `rb_rescue2`, with the classes it takes gathered from its arguments.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_rescue2(
    body: extern "C-unwind" fn(Value) -> Value,
    data: Value,
    rescue: Option<extern "C-unwind" fn(Value, Value) -> Value>,
    rescue_data: Value,
    count: i32,
    classes: *const Value,
) -> Value {
    let classes = super::handles::objects_from(i64::from(count), classes);
    rescuing(body, data, rescue, rescue_data, |exception| {
        classes.iter().any(|class| {
            if !matches!(class, Object::Class(_) | Object::Module(_)) {
                raise(crate::vm::errors::simple_exception(
                    "TypeError",
                    "class or module required",
                    called_from(),
                ));
            }
            is_a(exception, class.clone())
        })
    })
}

/// Runs `body`, then `ensure` however `body` ended, with the exception
/// `body` raised standing as `$!` while `ensure` runs.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ensure(
    body: extern "C-unwind" fn(Value) -> Value,
    data: Value,
    ensure: extern "C-unwind" fn(Value) -> Value,
    ensure_data: Value,
) -> Value {
    let answered = caught(|| body(data));
    match answered.as_ref().err().and_then(exception_in) {
        Some(exception) => handling(&exception, || ensure(ensure_data)),
        None => ensure(ensure_data),
    };
    answered.unwrap_or_else(|error| raise(error))
}

fn symbol_of(tag: *const c_char) -> Value {
    // SAFETY: C hands over a NUL-terminated string.
    let name = unsafe { CStr::from_ptr(tag) }.to_string_lossy();
    to_value(&Object::symbol(name.as_ref()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_catch_obj(tag: Value, function: *const (), data: Value) -> Value {
    let method = super::fibers::block_function_method(function, data);
    let block = crate::vm::program::block_for_method(method, called_from());
    interpreter().pending_block = Some(block);
    to_value(&kernel_function("catch", vec![to_object(tag)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_catch(tag: *const c_char, function: *const (), data: Value) -> Value {
    rb_catch_obj(symbol_of(tag), function, data)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_throw_obj(tag: Value, value: Value) -> ! {
    kernel_function("throw", vec![to_object(tag), to_object(value)]);
    unreachable!("Kernel#throw always unwinds")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_throw(tag: *const c_char, value: Value) -> ! {
    rb_throw_obj(symbol_of(tag), value)
}

/// Runs `source` as Ruby code where the C method was called from, so it
/// sees the locals there.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_eval_string(source: *const c_char) -> Value {
    // SAFETY: C hands over a NUL-terminated string.
    let source = unsafe { CStr::from_ptr(source) }
        .to_string_lossy()
        .into_owned();
    to_value(&kernel_function("eval", vec![Object::string(source)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_eval_string_protect(source: *const c_char, state: *mut i32) -> Value {
    extern "C-unwind" fn evaluate(source: Value) -> Value {
        rb_eval_string(source as *const c_char)
    }
    rb_protect(evaluate, source as Value, state)
}

/// Calls `function` with `object` and `data`, telling it through its last
/// argument whether a call for the same function and object is already
/// running further out.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_exec_recursive(
    function: extern "C-unwind" fn(Value, Value, i32) -> Value,
    object: Value,
    data: Value,
) -> Value {
    let key = (function as usize, object);
    if RECURSING.with(|held| held.borrow().contains(&key)) {
        return function(object, data, 1);
    }
    RECURSING.with(|held| held.borrow_mut().push(key));
    let answered = caught(|| function(object, data, 0));
    RECURSING.with(|held| held.borrow_mut().pop());
    answered.unwrap_or_else(|error| raise(error))
}

/// Raises LocalJumpError unless the C method running now was given a block.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_need_block() {
    if called_with_block().is_none() {
        raise(crate::vm::errors::simple_exception(
            "LocalJumpError",
            "no block given",
            called_from(),
        ));
    }
}

/// The block the C method running now was called with, as a lambda. A
/// Proc passed with `&` is answered as it is.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_block_lambda() -> Value {
    let block = super::utilities::rb_block_proc();
    match to_object(block) {
        Object::Block(held) if !held.is_lambda && !super::block_from_ampersand() => {
            let mut as_lambda = (*held).clone();
            as_lambda.is_lambda = true;
            to_value(&Object::Block(std::rc::Rc::new(as_lambda)))
        }
        _ => block,
    }
}

/// The name the C method running now was called by, as an ID, or 0
/// outside a C method.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_frame_this_func() -> Value {
    running_method().map_or(0, |method| to_value(&Object::symbol(&method.name)))
}

/// Raises the SystemCallError for `errno` as it stands.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_sys_fail(message: *const c_char) -> ! {
    let number = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    rb_syserr_fail(number, message)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_syserr_fail(number: i32, message: *const c_char) -> ! {
    super::exceptions::rb_exc_raise(rb_syserr_new(number, message))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_syserr_fail_str(number: i32, message: Value) -> ! {
    super::exceptions::rb_exc_raise(super::exceptions::rb_syserr_new_str(number, message))
}
