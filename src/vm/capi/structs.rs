//! Structs and Data classes from C: defining one, making and reading its
//! instances, and filling in one that was allocated.

use super::calls::{call, class_name_of, top_level_module};
use super::exports::text;
use super::handles::{QNIL, Value, to_object, to_value};
use super::{called_from, interpreter, or_raise, raise};
use crate::object::Object;
use std::ffi::c_char;

fn error(class: &str, message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        class,
        &message,
        called_from(),
    ))
}

/// The member names C handed over as `count` C strings, as Symbols,
/// refusing one named twice.
fn member_symbols(count: i32, names: *const *const c_char) -> Vec<Object> {
    let names = if count > 0 {
        // SAFETY: the header gathers `count` member names into an array.
        unsafe { std::slice::from_raw_parts(names, count as usize) }
    } else {
        &[]
    };
    let mut seen: Vec<String> = Vec::new();
    for name in names {
        let name = text(*name);
        if seen.contains(&name) {
            error("ArgumentError", format!("duplicate member: {}", name));
        }
        seen.push(name);
    }
    seen.into_iter().map(Object::symbol).collect()
}

fn members_of(object: Object) -> Vec<Object> {
    match call(object, "members", Vec::new()) {
        Object::Array(members) => members.borrow().clone(),
        _ => Vec::new(),
    }
}

/// A Struct class with the members named, a constant under Struct when it
/// has a name.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_struct_define(
    name: *const c_char,
    count: i32,
    names: *const *const c_char,
) -> Value {
    let mut arguments = member_symbols(count, names);
    if !name.is_null() {
        arguments.insert(0, Object::string(text(name)));
    }
    to_value(&call(top_level_module("Struct"), "new", arguments))
}

/// A Struct class bound to `name` under `outer`. A Struct class already
/// bound there is answered as it is, as `rb_define_class_id_under` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_struct_define_under(
    outer: Value,
    name: *const c_char,
    count: i32,
    names: *const *const c_char,
) -> Value {
    let members = member_symbols(count, names);
    let name = text(name);
    let outer = to_object(outer);
    let defined = call(
        outer.clone(),
        "const_defined?",
        vec![Object::symbol(name.clone()), Object::Bool(false)],
    );
    if defined.is_truthy() {
        let structs = to_value(&top_level_module("Struct"));
        let class = or_raise(interpreter().define_class_from_c(outer, &name, structs));
        return to_value(&Object::Class(class));
    }
    let made = call(top_level_module("Struct"), "new", members);
    call(outer, "const_set", vec![Object::symbol(name), made.clone()]);
    to_value(&made)
}

/// A Data class with the members named, a subclass of `superclass`, or of
/// Data when it is 0.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_data_define(
    superclass: Value,
    count: i32,
    names: *const *const c_char,
) -> Value {
    let superclass = if superclass == 0 {
        top_level_module("Data")
    } else {
        to_object(superclass)
    };
    if !matches!(superclass, Object::Class(_)) {
        error(
            "TypeError",
            format!(
                "wrong argument type {} (expected Class)",
                class_name_of(superclass)
            ),
        );
    }
    let members = member_symbols(count, names);
    to_value(&call(superclass, "define", members))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_new_instance(
    count: i32,
    values: *const Value,
    class: Value,
) -> Value {
    let arguments = super::handles::objects_from(i64::from(count), values);
    to_value(&call(to_object(class), "new", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_struct_s_members(class: Value) -> Value {
    to_value(&call(to_object(class), "members", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_struct_members(instance: Value) -> Value {
    to_value(&call(to_object(instance), "members", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_struct_size(instance: Value) -> Value {
    to_value(&Object::Int(members_of(to_object(instance)).len() as i64))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_struct_aref(instance: Value, key: Value) -> Value {
    to_value(&call(to_object(instance), "[]", vec![to_object(key)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_struct_aset(instance: Value, key: Value, value: Value) -> Value {
    let arguments = vec![to_object(key), to_object(value)];
    to_value(&call(to_object(instance), "[]=", arguments))
}

/// The member named by the ID `name`, refusing a name the struct has no
/// member for.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_struct_getmember(instance: Value, name: Value) -> Value {
    let instance = to_object(instance);
    let member = to_object(name);
    if !members_of(instance.clone()).contains(&member) {
        let named = super::symbols::symbol_name(name);
        error("NameError", format!("'{}' is not a struct member", named));
    }
    to_value(&call(instance, "[]", vec![member]))
}

/// Fills in the members of an allocated Struct or Data instance from the
/// Array `values`, the missing ones as nil. A Data instance is frozen once
/// it is filled in.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_struct_initialize(instance: Value, values: Value) -> Value {
    let instance_object = to_object(instance);
    let values = match to_object(values) {
        Object::Array(held) => held.borrow().clone(),
        _ => Vec::new(),
    };
    let is_data = call(
        instance_object.clone(),
        "is_a?",
        vec![top_level_module("Data")],
    )
    .is_truthy();
    if !is_data {
        let mut arguments = vec![Object::symbol("initialize")];
        arguments.extend(values);
        call(instance_object, "__send__", arguments);
        return QNIL;
    }
    if call(instance_object.clone(), "frozen?", Vec::new()).is_truthy() {
        super::exceptions::rb_error_frozen_object(instance);
    }
    let members = members_of(instance_object.clone());
    if values.len() > members.len() {
        error("ArgumentError", "struct size differs".to_string());
    }
    for (index, member) in members.iter().enumerate() {
        let named = match member {
            Object::Symbol(name) => name.as_str().to_string(),
            other => other.to_string(),
        };
        let variable = Object::symbol(format!("@{}", named));
        let value = values.get(index).cloned().unwrap_or(Object::Nil);
        call(
            instance_object.clone(),
            "instance_variable_set",
            vec![variable, value],
        );
    }
    call(instance_object, "freeze", Vec::new());
    QNIL
}
