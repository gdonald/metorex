//! The garbage collector as C sees it. Every object handed to C stays in the
//! handle table for the life of the program, so registering an address or
//! an object to keep alive asks for nothing more. Turning collection on and
//! off, running it and reading what it last did go to the GC module.

use super::calls::{call, top_level_module};
use super::handles::{Value, to_object, to_value};
use crate::object::Object;

fn garbage_collector(name: &str, arguments: Vec<Object>) -> Value {
    to_value(&call(top_level_module("GC"), name, arguments))
}

/// Keeps what C stores at `address` alive, which the handle table already
/// does for every object C holds.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_register_address(_address: *mut Value) {}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_unregister_address(_address: *mut Value) {}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_global_variable(_address: *mut Value) {}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_register_mark_object(_object: Value) {}

/// Notes memory C allocated outside Ruby objects. Metorex does not weigh
/// collection by memory, so the count is not kept.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_adjust_memory_usage(_difference: isize) {}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_enable() -> Value {
    garbage_collector("enable", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_disable() -> Value {
    garbage_collector("disable", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc() {
    garbage_collector("start", Vec::new());
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_start() -> Value {
    garbage_collector("start", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_count() -> usize {
    match to_object(garbage_collector("count", Vec::new())) {
        Object::Int(count) => count as usize,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_latest_gc_info(key_or_hash: Value) -> Value {
    garbage_collector("latest_gc_info", vec![to_object(key_or_hash)])
}
