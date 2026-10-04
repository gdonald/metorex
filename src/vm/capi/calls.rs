//! Calling into Ruby from C: methods by name, Marshal, and finalizers.

use super::handles::{Value, objects_from, to_object, to_value};
use super::methods::CFunction;
use super::symbols::symbol_name;
use super::{interpreter, or_raise, raise};
use crate::object::Object;

/// Calls `name` on `receiver` the way Ruby code would, from the line that
/// called into C. `Kernel#binding`
/// needs a Ruby method to describe the caller of, and a C function is not
/// one, so it is refused.
pub(super) fn call(receiver: Object, name: &str, arguments: Vec<Object>) -> Object {
    let machine = interpreter();
    let position = super::called_from();
    if name == "binding" && machine.lookup_method(&receiver, name).is_none() {
        raise(crate::vm::errors::simple_exception(
            "RuntimeError",
            "Cannot create Binding object for non-Ruby caller",
            position,
        ));
    }
    let answered = or_raise(machine.send_to_object(receiver, name, arguments, position));
    super::strings::carry_changes_in();
    answered
}

/// Whether `object` responds to `name`, asking it with `respond_to?` as
/// MRI's `rb_check_funcall` does.
pub(super) fn answers(object: &Object, name: &str) -> bool {
    let arguments = vec![Object::symbol(name), Object::Bool(true)];
    call(object.clone(), "respond_to?", arguments).is_truthy()
}

/// The name of the class `object` is an instance of, which a TypeError
/// names.
pub(super) fn class_name_of(object: Object) -> String {
    call(object, "class", Vec::new()).to_string()
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_funcallv(
    receiver: Value,
    name: Value,
    count: i32,
    arguments: *const Value,
) -> Value {
    let arguments = objects_from(i64::from(count), arguments);
    to_value(&call(to_object(receiver), &symbol_name(name), arguments))
}

thread_local! {
    /// For each `rb_block_call` running now, innermost last, its block
    /// function and data and the block of the C method that called it,
    /// which `rb_yield` in that block function yields to.
    static BLOCK_CALLS: std::cell::RefCell<Vec<(CFunction, Option<Object>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Takes the innermost `rb_block_call` off `BLOCK_CALLS` when it ends,
/// returning or raising.
struct BlockCallEnded;

impl Drop for BlockCallEnded {
    fn drop(&mut self) {
        BLOCK_CALLS.with(|calls| calls.borrow_mut().pop());
    }
}

/// The block `function` yields to when an `rb_block_call` running now
/// was handed it, or None.
pub(super) fn block_of_block_call(function: CFunction) -> Option<Object> {
    BLOCK_CALLS.with(|calls| {
        calls
            .borrow()
            .iter()
            .rev()
            .find(|(held, _)| *held == function)
            .and_then(|(_, block)| block.clone())
    })
}

/// Calls `name` on `receiver` with a block that runs the C `function`,
/// handed `data`, for each value yielded.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_block_call(
    receiver: Value,
    name: Value,
    count: i32,
    arguments: *const Value,
    function: *const (),
    data: Value,
) -> Value {
    if function.is_null() {
        interpreter().pending_block = super::called_with_block();
        return rb_funcallv(receiver, name, count, arguments);
    }
    let c_function = super::fibers::block_function(function, data);
    let method = super::fibers::method_for_block_function(c_function);
    BLOCK_CALLS.with(|calls| {
        calls
            .borrow_mut()
            .push((c_function, super::called_with_block()))
    });
    let _ended = BlockCallEnded;
    let block = crate::vm::program::block_for_method(method, super::called_from());
    interpreter().pending_block = Some(block);
    rb_funcallv(receiver, name, count, arguments)
}

/// Calls the Kernel function `name`, as Ruby code calling it without a
/// receiver where the call into C was made would.
pub(super) fn kernel_function(name: &str, arguments: Vec<Object>) -> Object {
    let answered =
        or_raise(interpreter().call_native_function(name, arguments, super::called_from()));
    super::strings::carry_changes_in();
    answered
}

/// The arguments C handed over, the last marked as the keyword Hash when
/// `keywords` says so. An empty keyword Hash passes no keywords at all.
fn arguments_with_keywords(count: i32, arguments: *const Value, keywords: i32) -> Vec<Object> {
    let mut arguments = objects_from(i64::from(count), arguments);
    if keywords != super::procs::PASS_KEYWORDS {
        return arguments;
    }
    if matches!(arguments.last(), Some(Object::Dict(pairs)) if pairs.borrow().is_empty()) {
        arguments.pop();
    } else {
        super::procs::pass_last_as_keywords(&mut arguments);
    }
    arguments
}

/// Calls `name` the way `public_send` does, refusing a private or
/// protected method, with `block` unless it is nil.
fn public_call(receiver: Value, name: Value, arguments: Vec<Object>, block: Value) -> Value {
    if block != super::handles::QNIL {
        interpreter().pending_block = Some(to_object(block));
    }
    let mut sent = vec![Object::symbol(symbol_name(name))];
    sent.extend(arguments);
    to_value(&call(to_object(receiver), "public_send", sent))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_funcallv_kw(
    receiver: Value,
    name: Value,
    count: i32,
    arguments: *const Value,
    keywords: i32,
) -> Value {
    let arguments = arguments_with_keywords(count, arguments, keywords);
    to_value(&call(to_object(receiver), &symbol_name(name), arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_funcallv_public(
    receiver: Value,
    name: Value,
    count: i32,
    arguments: *const Value,
) -> Value {
    let arguments = objects_from(i64::from(count), arguments);
    public_call(receiver, name, arguments, super::handles::QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_funcall_with_block(
    receiver: Value,
    name: Value,
    count: i32,
    arguments: *const Value,
    block: Value,
) -> Value {
    let arguments = objects_from(i64::from(count), arguments);
    public_call(receiver, name, arguments, block)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_funcall_with_block_kw(
    receiver: Value,
    name: Value,
    count: i32,
    arguments: *const Value,
    block: Value,
    keywords: i32,
) -> Value {
    let arguments = arguments_with_keywords(count, arguments, keywords);
    public_call(receiver, name, arguments, block)
}

/// Calls `name` when `receiver` responds to it, asking with `respond_to?`,
/// or answers `Qundef`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_funcall(
    receiver: Value,
    name: Value,
    count: i32,
    arguments: *const Value,
) -> Value {
    if !answers(&to_object(receiver), &symbol_name(name)) {
        return super::handles::QUNDEF;
    }
    rb_funcallv(receiver, name, count, arguments)
}

/// What `Kernel#format` answers for the format and arguments at `values`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_f_sprintf(count: i32, values: *const Value) -> Value {
    let arguments = objects_from(i64::from(count), values);
    to_value(&kernel_function("sprintf", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_format(count: i32, values: *const Value, format: Value) -> Value {
    let mut arguments = vec![to_object(format)];
    arguments.extend(objects_from(i64::from(count), values));
    to_value(&kernel_function("sprintf", arguments))
}

/// The backtrace of the call into C running now, as `caller(0)` gives it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_make_backtrace() -> Value {
    to_value(&kernel_function("caller", vec![Object::Int(0)]))
}

/// Warns as `rb_warn` does when `Warning[]` says the category is on, and
/// says nothing while `$VERBOSE` is nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_category_warn(category: i32, message: Value) {
    if matches!(
        interpreter().globals().get("VERBOSE"),
        None | Some(Object::Nil)
    ) {
        return;
    }
    let named = match category {
        1 => Some("deprecated"),
        2 => Some("experimental"),
        3 => Some("performance"),
        _ => None,
    };
    let enabled = named.is_none_or(|name| {
        call(
            top_level_module("Warning"),
            "[]",
            vec![Object::symbol(name)],
        )
        .is_truthy()
    });
    if enabled {
        rb_warn_message(message, 0);
    }
}

/// A module defined at the top level before any extension loads.
pub(super) fn top_level_module(name: &str) -> Object {
    interpreter()
        .globals()
        .get(name)
        .unwrap_or_else(|| panic!("{} is defined before any extension loads", name))
}

fn marshal() -> Object {
    top_level_module("Marshal")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_marshal_dump(object: Value, port: Value) -> Value {
    let mut arguments = vec![to_object(object)];
    if port != super::handles::QNIL {
        arguments.push(to_object(port));
    }
    to_value(&call(marshal(), "dump", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_marshal_load(port: Value) -> Value {
    to_value(&call(marshal(), "load", vec![to_object(port)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_finalizer(object: Value, finalizer: Value) -> Value {
    let arguments = vec![to_object(object), to_object(finalizer)];
    call(
        top_level_module("ObjectSpace"),
        "define_finalizer",
        arguments,
    );
    finalizer
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_undefine_finalizer(object: Value) -> Value {
    let arguments = vec![to_object(object)];
    call(
        top_level_module("ObjectSpace"),
        "undefine_finalizer",
        arguments,
    );
    object
}

/// The String `object` writes itself as, through `to_s`, or the default
/// `#<Class>` form when `to_s` answers something else.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_as_string(object: Value) -> Value {
    let held = to_object(object);
    if matches!(held, Object::String(_)) {
        return object;
    }
    let written = call(held.clone(), "to_s", Vec::new());
    if matches!(written, Object::String(_)) {
        return to_value(&written);
    }
    let default = call(
        top_level_module("Kernel"),
        "instance_method",
        vec![Object::symbol("to_s")],
    );
    to_value(&call(default, "bind_call", vec![held]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_inspect(object: Value) -> Value {
    to_value(&call(to_object(object), "inspect", Vec::new()))
}

/// Writes `message` as a warning from the line that called into C, when
/// `$VERBOSE` is not nil, or only when it is true for `verbose_only`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_warn_message(message: Value, verbose_only: i32) {
    let machine = interpreter();
    if verbose_only != 0 && !matches!(machine.globals().get("VERBOSE"), Some(Object::Bool(true))) {
        return;
    }
    let position = super::called_from();
    let text = format!(
        "{}{}\n",
        machine.warning_prefix(0, position),
        to_object(message)
    );
    or_raise(machine.warn_through_warning_module(text, position));
}
