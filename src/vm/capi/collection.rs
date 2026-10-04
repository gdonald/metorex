//! The garbage collector as C sees it. Every object handed to C stays in the
//! handle table, except a Data object that nothing reaches when a collection
//! runs: not Ruby, not a mark function, and not an address or object C asked
//! to keep alive. Turning collection on and off, running it and reading what
//! it last did go to the GC module.

use super::calls::{call, top_level_module};
use super::handles::{Value, to_object, to_value};
use crate::object::Object;
use std::cell::RefCell;

fn garbage_collector(name: &str, arguments: Vec<Object>) -> Value {
    to_value(&call(top_level_module("GC"), name, arguments))
}

thread_local! {
    /// The addresses C asked to have the VALUE stored there kept alive.
    static REGISTERED_ADDRESSES: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    /// The objects C asked to have kept alive for good.
    static PINNED: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
    /// What the mark functions running now have marked, while a collection
    /// is under way.
    static MARKED: RefCell<Option<Vec<Value>>> = const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_register_address(address: *mut Value) {
    REGISTERED_ADDRESSES.with(|held| held.borrow_mut().push(address as usize));
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_unregister_address(address: *mut Value) {
    REGISTERED_ADDRESSES.with(|held| held.borrow_mut().retain(|kept| *kept != address as usize));
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_global_variable(address: *mut Value) {
    rb_gc_register_address(address);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_register_mark_object(object: Value) {
    PINNED.with(|held| held.borrow_mut().push(object));
}

/// The VALUEs C asked to have kept alive: the ones stored at the addresses
/// it registered, and the objects it pinned.
pub(super) fn roots() -> Vec<Value> {
    let mut found = PINNED.with(|held| held.borrow().clone());
    REGISTERED_ADDRESSES.with(|held| {
        for address in held.borrow().iter() {
            // SAFETY: C registered the address of a VALUE that outlives the
            // registration.
            found.push(unsafe { *(*address as *const Value) });
        }
    });
    found
}

/// Run `marking` with marks recorded, answering what it marked.
pub(super) fn marks_made_by(marking: impl FnOnce()) -> Vec<Value> {
    let outer = MARKED.with(|held| held.replace(Some(Vec::new())));
    marking();
    MARKED.with(|held| held.replace(outer)).unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_mark(object: Value) {
    MARKED.with(|held| {
        if let Some(marked) = held.borrow_mut().as_mut() {
            marked.push(object);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_mark_movable(object: Value) {
    rb_gc_mark(object);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_mark_maybe(object: Value) {
    rb_gc_mark(object);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_mark_locations(start: *const Value, end: *const Value) {
    let mut cursor = start;
    while cursor < end {
        // SAFETY: C hands over the VALUEs from `start` up to `end`.
        rb_gc_mark(unsafe { *cursor });
        cursor = cursor.wrapping_add(1);
    }
}

/// Where an object moved to, which is where it was, since objects do not
/// move.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gc_location(object: Value) -> Value {
    object
}

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
