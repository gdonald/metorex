//! The elements of an Array as C reads them through `RARRAY_PTR` and
//! `RARRAY_LEN`, and Arrays C builds. C is handed a copy of the elements'
//! VALUEs that metorex holds until the same Array is asked for again with
//! different elements.

use super::calls::class_name_of;
use super::handles::{Value, objects_from, to_object, to_value};
use super::{called_from, raise};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static BUFFERS: RefCell<HashMap<Value, Vec<Value>>> = RefCell::new(HashMap::new());
}

/// The VALUEs of the elements of the Array a VALUE C code handed over
/// stands for.
fn element_values(array: Value) -> Vec<Value> {
    match to_object(array) {
        Object::Array(elements) => elements.borrow().iter().map(to_value).collect(),
        other => raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!(
                "wrong argument type {} (expected Array)",
                class_name_of(other)
            ),
            super::called_from(),
        )),
    }
}

/// The Array `object` converts to through `to_ary`, as `rb_check_array_type`
/// finds it, or None for an object that has no `to_ary`.
pub(super) fn array_of(object: Object) -> Option<Vec<Object>> {
    if let Object::Array(elements) = &object {
        return Some(elements.borrow().clone());
    }
    if !super::calls::answers(&object, "to_ary") {
        return None;
    }
    match super::calls::call(object.clone(), "to_ary", Vec::new()) {
        Object::Array(elements) => Some(elements.borrow().clone()),
        Object::Nil => None,
        converted => {
            let class_name = class_name_of(object);
            raise(crate::vm::errors::simple_exception(
                "TypeError",
                &format!(
                    "can't convert {} to Array ({}#to_ary gives {})",
                    class_name,
                    class_name,
                    class_name_of(converted)
                ),
                called_from(),
            ))
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rarray_ptr(array: Value) -> *const Value {
    let values = element_values(array);
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let held = buffers.entry(array).or_default();
        if *held != values {
            *held = values;
        }
        held.as_ptr()
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rarray_len(array: Value) -> i64 {
    element_values(array).len() as i64
}

/// The element at `offset`, counted from the end when negative, or nil past
/// either end.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_entry(array: Value, offset: i64) -> Value {
    let values = element_values(array);
    let index = if offset < 0 {
        offset + values.len() as i64
    } else {
        offset
    };
    usize::try_from(index)
        .ok()
        .and_then(|index| values.get(index).copied())
        .unwrap_or(super::handles::QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_new_from_values(count: i64, values: *const Value) -> Value {
    to_value(&Object::array(objects_from(count, values)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_new() -> Value {
    to_value(&Object::array(Vec::new()))
}

/// Appends `element`, refusing a frozen Array as `Array#push` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_push(array: Value, element: Value) -> Value {
    super::calls::call(to_object(array), "push", vec![to_object(element)]);
    array
}

/// Removes and answers the last element, or nil for an empty Array,
/// refusing a frozen Array as `Array#pop` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_pop(array: Value) -> Value {
    to_value(&super::calls::call(to_object(array), "pop", Vec::new()))
}

/// Stores `element` at `index`, counted from the end when negative, as
/// `Array#[]=` does with one index.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_store(array: Value, index: i64, element: Value) {
    let arguments = vec![Object::Int(index), to_object(element)];
    super::calls::call(to_object(array), "[]=", arguments);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_dup(array: Value) -> Value {
    to_value(&Object::array(
        element_values(array).into_iter().map(to_object).collect(),
    ))
}

/// An empty Array. Metorex sizes an Array as it grows, so the capacity is
/// not used.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_new_capa(_capacity: i64) -> Value {
    rb_ary_new()
}

/// Puts `element` at the front, refusing a frozen Array as `Array#unshift`
/// does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_unshift(array: Value, element: Value) -> Value {
    super::calls::call(to_object(array), "unshift", vec![to_object(element)]);
    array
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_dup(hash: Value) -> Value {
    to_value(&super::calls::call(to_object(hash), "dup", Vec::new()))
}
