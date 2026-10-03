//! Calling into Ruby from C: methods by name, Marshal, and finalizers.

use super::handles::{Value, objects_from, to_object, to_value};
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
    or_raise(machine.send_to_object(receiver, name, arguments, position))
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
