//! Sets from C: making one, asking about and changing its elements, and
//! walking them with a C function.

use super::calls::{call, top_level_module};
use super::handles::{Value, to_object, to_value};
use crate::object::Object;

/// What a walk's function answers to go on, stop, or drop the element.
const ST_STOP: i32 = 1;
const ST_DELETE: i32 = 2;

fn new_set() -> Value {
    to_value(&call(top_level_module("Set"), "new", Vec::new()))
}

/// Whether a `?` method of the set answered something other than nil.
fn answered(set: Value, name: &str, element: Value) -> bool {
    !matches!(
        call(to_object(set), name, vec![to_object(element)]),
        Object::Nil | Object::Bool(false)
    )
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_new() -> Value {
    new_set()
}

/// A new Set. Metorex sizes a Set as it grows, so the capacity is not used.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_new_capa(_capacity: usize) -> Value {
    new_set()
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_lookup(set: Value, element: Value) -> bool {
    answered(set, "include?", element)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_add(set: Value, element: Value) -> bool {
    answered(set, "add?", element)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_delete(set: Value, element: Value) -> bool {
    answered(set, "delete?", element)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_clear(set: Value) -> Value {
    to_value(&call(to_object(set), "clear", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_size(set: Value) -> usize {
    match call(to_object(set), "size", Vec::new()) {
        Object::Int(count) => count as usize,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_set_foreach(
    set: Value,
    function: extern "C-unwind" fn(Value, Value) -> i32,
    data: Value,
) {
    let Object::Array(elements) = call(to_object(set), "to_a", Vec::new()) else {
        return;
    };
    let elements = elements.borrow().clone();
    for element in elements {
        match function(to_value(&element), data) {
            ST_STOP => return,
            ST_DELETE => {
                call(to_object(set), "delete", vec![element]);
            }
            _ => {}
        }
    }
}
