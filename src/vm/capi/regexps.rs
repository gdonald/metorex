//! Regexps from C: making one, matching with it, reading a match, and the
//! last match `$~` holds.

use super::calls::{call, top_level_module};
use super::handles::{QNIL, Value, to_object, to_value};
use super::interpreter;
use crate::object::{Object, StringValue};
use crate::vm::native_methods::regexp_methods::LAST_MATCH;
use std::rc::Rc;

/// What the `length` bytes at `source` spell: text where they are UTF-8,
/// and one character per byte where they are not.
fn source_string(source: *const u8, length: i64) -> Object {
    let bytes = if length > 0 {
        // SAFETY: C hands over `length` bytes at `source`.
        unsafe { std::slice::from_raw_parts(source, length as usize) }.to_vec()
    } else {
        Vec::new()
    };
    match String::from_utf8(bytes) {
        Ok(text) => Object::string(text),
        Err(error) => {
            let text = error
                .into_bytes()
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>();
            Object::String(Rc::new(StringValue::from_bytes(text)))
        }
    }
}

fn regexp_class() -> Object {
    top_level_module("Regexp")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_reg_new(source: *const u8, length: i64, options: i32) -> Value {
    let arguments = vec![
        source_string(source, length),
        Object::Int(i64::from(options)),
    ];
    to_value(&call(regexp_class(), "new", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_reg_regcomp(source: Value) -> Value {
    to_value(&call(regexp_class(), "new", vec![to_object(source)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_reg_options(regexp: Value) -> i32 {
    match call(to_object(regexp), "options", Vec::new()) {
        Object::Int(options) => options as i32,
        _ => 0,
    }
}

/// Where `regexp` first matches `string`, in characters, or nil, setting
/// `$~` as `=~` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_reg_match(regexp: Value, string: Value) -> Value {
    to_value(&call(to_object(regexp), "=~", vec![to_object(string)]))
}

/// The text group `nth` of `match` captured, counted from the end when
/// negative, or nil for no match, a group past the end, or one that took
/// no part.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_reg_nth_match(nth: i32, matched: Value) -> Value {
    if matched == QNIL {
        return QNIL;
    }
    let matched = to_object(matched);
    let Object::Int(group_count) = call(matched.clone(), "size", Vec::new()) else {
        return QNIL;
    };
    let mut nth = i64::from(nth);
    if nth >= group_count {
        return QNIL;
    }
    if nth < 0 {
        nth += group_count;
        if nth <= 0 {
            return QNIL;
        }
    }
    to_value(&call(matched, "[]", vec![Object::Int(nth)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_backref_get() -> Value {
    to_value(
        &interpreter()
            .globals()
            .get(LAST_MATCH)
            .unwrap_or(Object::Nil),
    )
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_backref_set(matched: Value) {
    interpreter()
        .globals_mut()
        .set(LAST_MATCH.to_string(), to_object(matched));
}

/// Compares `length` bytes of two buffers with ASCII letters folded to
/// lower case, answering the difference at the first byte that differs.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_memcicmp(first: *const u8, second: *const u8, length: i64) -> i32 {
    if length <= 0 {
        return 0;
    }
    // SAFETY: C hands over two buffers of at least `length` bytes.
    let (first, second) = unsafe {
        (
            std::slice::from_raw_parts(first, length as usize),
            std::slice::from_raw_parts(second, length as usize),
        )
    };
    first
        .iter()
        .zip(second)
        .map(|(left, right)| {
            i32::from(left.to_ascii_lowercase()) - i32::from(right.to_ascii_lowercase())
        })
        .find(|difference| *difference != 0)
        .unwrap_or(0)
}
