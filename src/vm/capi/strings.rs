//! The bytes of a String as C reads and writes them through `RSTRING_PTR`
//! and `RSTRING_LEN`. Metorex keeps a string's characters as Rust text, so C
//! is handed a buffer holding a copy of its bytes, ended with NUL. What C
//! writes there is carried back into the string whenever control leaves C,
//! and the buffer stays where it is until the string changes length.

use super::calls::{answers, call, class_name_of};
use super::handles::{Value, to_object, to_value};
use super::{interpreter, raise};
use crate::object::{Object, StringValue};
use crate::vm::native_methods::string_methods::{
    binary_bytes, bytes_are_characters, wide_encoding,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_char;
use std::rc::Rc;

/// The buffer handed to C for one string, and the bytes it held when C was
/// last handed it or last left, which tell a write of C's apart.
struct Buffer {
    bytes: Vec<u8>,
    handed: Vec<u8>,
}

thread_local! {
    static BUFFERS: RefCell<HashMap<Value, Buffer>> = RefCell::new(HashMap::new());
}

/// The String a VALUE C code handed over stands for.
pub(super) fn string_from(value: Value) -> Rc<StringValue> {
    match to_object(value) {
        Object::String(held) => held,
        other => raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!(
                "wrong argument type {} (expected String)",
                class_name_of(other)
            ),
            super::called_from(),
        )),
    }
}

/// The String `object` converts to through `to_str`, as `StringValue` does.
pub(super) fn string_value(object: Object) -> Object {
    if let Object::String(_) = object {
        return object;
    }
    if !answers(&object, "to_str") {
        raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!(
                "no implicit conversion of {} into String",
                interpreter().conversion_name(&object)
            ),
            super::called_from(),
        ));
    }
    let converted = call(object.clone(), "to_str", Vec::new());
    if let Object::String(_) = converted {
        return converted;
    }
    let class_name = class_name_of(object);
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        &format!(
            "can't convert {} to String ({}#to_str gives {})",
            class_name,
            class_name,
            class_name_of(converted)
        ),
        super::called_from(),
    ))
}

/// `object` as a String when it is one or converts through `to_str`, as
/// `rb_check_string_type` finds it, or None when it has no `to_str`.
pub(super) fn check_string(object: Object) -> Option<Object> {
    if matches!(object, Object::String(_)) || answers(&object, "to_str") {
        return Some(string_value(object));
    }
    None
}

/// Puts `bytes` in `string`, read the way the string's encoding reads them.
/// Bytes that spell no text are held one character per byte.
fn store_bytes(string: &StringValue, bytes: &[u8]) {
    let as_characters = || bytes.iter().map(|byte| *byte as char).collect::<String>();
    if bytes_are_characters(string) {
        string.replace_text(as_characters());
        return;
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => string.replace_text(text),
        Err(_) => {
            string.replace_text(as_characters());
            string.mark_bytes();
        }
    }
}

/// Carries what C wrote into each buffer back into its string.
pub(super) fn carry_writes_back() {
    BUFFERS.with(|buffers| {
        for (value, buffer) in buffers.borrow_mut().iter_mut() {
            if buffer.bytes == buffer.handed {
                continue;
            }
            if let Object::String(string) = to_object(*value) {
                store_bytes(&string, &buffer.bytes[..buffer.bytes.len() - 1]);
            }
            buffer.handed.clone_from(&buffer.bytes);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rstring_ptr(string: Value) -> *mut c_char {
    carry_writes_back();
    let mut current = binary_bytes(&string_from(string));
    current.push(0);
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry(string).or_insert_with(|| Buffer {
            bytes: current.clone(),
            handed: current.clone(),
        });
        if buffer.handed != current {
            if buffer.bytes.len() == current.len() {
                buffer.bytes.copy_from_slice(&current);
            } else {
                buffer.bytes.clone_from(&current);
            }
            buffer.handed = current;
        }
        buffer.bytes.as_mut_ptr() as *mut c_char
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rstring_len(string: Value) -> i64 {
    carry_writes_back();
    binary_bytes(&string_from(string)).len() as i64
}

/// Converts what `pointer` holds to a String through `to_str` and stores it
/// back, as `StringValue` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_string_value(pointer: *mut Value) -> Value {
    // SAFETY: `StringValue` hands over the address of a VALUE it can write.
    let held = unsafe { *pointer };
    let converted = to_value(&string_value(to_object(held)));
    // SAFETY: as above.
    unsafe { *pointer = converted };
    converted
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_string_value_ptr(pointer: *mut Value) -> *mut c_char {
    rb_rstring_ptr(rb_string_value(pointer))
}

/// The string's bytes for C to read as a NUL-terminated string, refusing one
/// that holds a NUL of its own. An encoding that spells a character in more
/// than one byte looks for a whole code unit of zeros.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_string_value_cstr(pointer: *mut Value) -> *mut c_char {
    let string = rb_string_value(pointer);
    let held = string_from(string);
    let bytes = binary_bytes(&held);
    let unit = wide_encoding(&held.encoding_name()).map_or(1, |shape| shape.unit());
    if bytes
        .chunks(unit)
        .any(|chunk| chunk.iter().all(|byte| *byte == 0))
    {
        let message = if unit > 1 {
            "string contains null char"
        } else {
            "string contains null byte"
        };
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            message,
            super::called_from(),
        ));
    }
    rb_rstring_ptr(string)
}

/// A String holding the `length` bytes at `text`, tagged ASCII-8BIT as
/// MRI's `rb_str_new` makes it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_new(text: *const u8, length: i64) -> Value {
    let bytes = if length > 0 && !text.is_null() {
        // SAFETY: C hands over `length` bytes at `text`.
        unsafe { std::slice::from_raw_parts(text, length as usize) }
    } else {
        &[]
    };
    let characters = bytes.iter().map(|byte| *byte as char).collect::<String>();
    to_value(&Object::String(Rc::new(StringValue::from_bytes(
        characters,
    ))))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_new_cstr(text: *const c_char) -> Value {
    // SAFETY: C hands over a NUL-terminated string.
    let length = unsafe { std::ffi::CStr::from_ptr(text) }.to_bytes().len();
    rb_str_new(text as *const u8, length as i64)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_append(string: Value, added: Value) -> Value {
    call(to_object(string), "<<", vec![to_object(added)]);
    string
}

/// Retags a formatted string with the encoding of a string written into it,
/// when that one is ASCII-compatible and not binary, as MRI's formatting
/// does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_adopt_encoding(string: Value, source: Value) {
    let made = string_from(string);
    let named = string_from(source).encoding_name();
    if matches!(named.as_str(), "ASCII-8BIT" | "BINARY") || wide_encoding(&named).is_some() {
        return;
    }
    let bytes = binary_bytes(&made);
    made.set_encoding(named);
    made.clear_bytes();
    store_bytes(&made, &bytes);
}
