//! Fibers from C: the current one, making one whose body is a C block
//! function, and resuming, yielding, transferring and raising.

use super::calls::{call, top_level_module};
use super::handles::{Value, objects_from, to_object, to_value};
use super::methods::{BLOCK_FUNCTION_ARITY, CFunction};
use super::{called_from, interpreter};
use crate::object::{Method, Object};
use crate::vm::program::block_for_method;
use std::rc::Rc;

fn fiber_class() -> Object {
    top_level_module("Fiber")
}

/// A Method that calls the block function at `function` with `data`, taking
/// any number of values.
pub(super) fn block_function_method(function: *const (), data: Value) -> Object {
    let mut method = Method::new("call".to_string(), vec!["values".to_string()], Vec::new());
    method.variadic_param = Some((0, "values".to_string()));
    method.c_function = Some(CFunction {
        address: function as usize,
        arity: BLOCK_FUNCTION_ARITY,
        data,
    });
    Object::Method(Rc::new(method))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fiber_new(function: *const (), data: Value) -> Value {
    let block = block_for_method(block_function_method(function, data), called_from());
    interpreter().pending_block = Some(block);
    to_value(&call(fiber_class(), "new", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fiber_current() -> Value {
    to_value(&call(fiber_class(), "current", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fiber_alive_p(fiber: Value) -> Value {
    to_value(&call(to_object(fiber), "alive?", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fiber_resume(fiber: Value, count: i32, values: *const Value) -> Value {
    let arguments = objects_from(i64::from(count), values);
    to_value(&call(to_object(fiber), "resume", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fiber_yield(count: i32, values: *const Value) -> Value {
    let arguments = objects_from(i64::from(count), values);
    to_value(&call(fiber_class(), "yield", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fiber_transfer(
    fiber: Value,
    count: i32,
    values: *const Value,
) -> Value {
    let arguments = objects_from(i64::from(count), values);
    to_value(&call(to_object(fiber), "transfer", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fiber_raise(fiber: Value, count: i32, values: *const Value) -> Value {
    let arguments = objects_from(i64::from(count), values);
    to_value(&call(to_object(fiber), "raise", arguments))
}
