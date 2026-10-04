//! The elements of an Array as C reads and writes them through `RARRAY_PTR`
//! and `RARRAY_LEN`, and Arrays C builds. C is handed a copy of the elements'
//! VALUEs that metorex holds until the same Array is asked for again with
//! different elements. What C writes there is carried back into the Array
//! whenever control leaves C.

use super::calls::{call, class_name_of};
use super::handles::{QNIL, Value, objects_from, to_object, to_value};
use super::{called_from, raise};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;

/// The VALUEs handed to C for one Array, and the VALUEs they were when C
/// was last handed them or last left, which tell a write of C's apart.
#[derive(Default)]
struct Buffer {
    values: Vec<Value>,
    handed: Vec<Value>,
}

thread_local! {
    static BUFFERS: RefCell<HashMap<Value, Buffer>> = RefCell::new(HashMap::new());
}

/// Carries what C wrote into each buffer back into its Array.
pub(super) fn carry_element_writes_back() {
    let written: Vec<(Value, Vec<Value>)> = BUFFERS.with(|buffers| {
        buffers
            .borrow_mut()
            .iter_mut()
            .filter(|(_, buffer)| buffer.values != buffer.handed)
            .map(|(array, buffer)| {
                buffer.handed.clone_from(&buffer.values);
                (*array, buffer.values.clone())
            })
            .collect()
    });
    for (array, values) in written {
        if let Object::Array(elements) = to_object(array) {
            *elements.borrow_mut() = values.into_iter().map(to_object).collect();
        }
    }
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
pub extern "C-unwind" fn rb_rarray_ptr(array: Value) -> *mut Value {
    carry_element_writes_back();
    let values = element_values(array);
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let held = buffers.entry(array).or_default();
        if held.handed != values {
            held.values.clone_from(&values);
            held.handed = values;
        }
        held.values.as_mut_ptr()
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rarray_len(array: Value) -> i64 {
    carry_element_writes_back();
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
        .unwrap_or(QNIL)
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

/// An empty Array, refusing a negative capacity. Metorex sizes an Array as
/// it grows, so the capacity is not otherwise used.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_new_capa(capacity: i64) -> Value {
    if capacity < 0 {
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            "negative array size (or size too big)",
            called_from(),
        ));
    }
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

/// Answers `array` after calling `name` on it, for the C functions that
/// change an Array in place and answer it.
fn changed(array: Value, name: &str, arguments: Vec<Object>) -> Value {
    call(to_object(array), name, arguments);
    array
}

/// Answers what `name` called on `array` answers.
fn answered(array: Value, name: &str, arguments: Vec<Object>) -> Value {
    to_value(&call(to_object(array), name, arguments))
}

/// The Array `Kernel#Array` converts `object` to.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_Array(object: Value) -> Value {
    let kernel = super::calls::top_level_module("Kernel");
    to_value(&call(kernel, "Array", vec![to_object(object)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_aref(count: i32, values: *const Value, array: Value) -> Value {
    answered(array, "[]", objects_from(i64::from(count), values))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_cat(array: Value, values: *const Value, count: i64) -> Value {
    changed(array, "push", objects_from(count, values))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_clear(array: Value) -> Value {
    changed(array, "clear", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_concat(array: Value, other: Value) -> Value {
    changed(array, "concat", vec![to_object(other)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_delete(array: Value, element: Value) -> Value {
    answered(array, "delete", vec![to_object(element)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_delete_at(array: Value, index: i64) -> Value {
    answered(array, "delete_at", vec![Object::Int(index)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_freeze(array: Value) -> Value {
    answered(array, "freeze", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_includes(array: Value, element: Value) -> Value {
    answered(array, "include?", vec![to_object(element)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_join(array: Value, separator: Value) -> Value {
    answered(array, "join", vec![to_object(separator)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_plus(array: Value, other: Value) -> Value {
    answered(array, "+", vec![to_object(other)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_reverse(array: Value) -> Value {
    changed(array, "reverse!", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_rotate(array: Value, count: i64) -> Value {
    changed(array, "rotate!", vec![Object::Int(count)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_shift(array: Value) -> Value {
    answered(array, "shift", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_sort(array: Value) -> Value {
    answered(array, "sort", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_sort_bang(array: Value) -> Value {
    changed(array, "sort!", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_to_s(array: Value) -> Value {
    answered(array, "to_s", Vec::new())
}

/// The `length` elements from `start`, fewer when the Array ends first, or
/// nil when `start` is past the end or either number is negative.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_subseq(array: Value, start: i64, length: i64) -> Value {
    let values = element_values(array);
    if start < 0 || length < 0 || start as usize > values.len() {
        return QNIL;
    }
    let start = start as usize;
    let end = values.len().min(start.saturating_add(length as usize));
    to_value(&Object::array(
        values[start..end].iter().copied().map(to_object).collect(),
    ))
}

/// `object` itself when it is an Array, the Array its `to_ary` answers, or
/// else a new Array holding it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_to_ary(object: Value) -> Value {
    let held = to_object(object);
    if matches!(held, Object::Array(_)) {
        return object;
    }
    let elements = array_of(held.clone()).unwrap_or_else(|| vec![held]);
    to_value(&Object::array(elements))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_assoc_new(first: Value, second: Value) -> Value {
    to_value(&Object::array(vec![to_object(first), to_object(second)]))
}

/// Sets each of the `count` VALUEs at `values` to nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mem_clear(values: *mut Value, count: i64) {
    for index in 0..count.max(0) as usize {
        // SAFETY: C hands a pointer to at least `count` VALUEs.
        unsafe { *values.add(index) = QNIL };
    }
}
