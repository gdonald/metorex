//! The bytes of a String as C reads and writes them through `RSTRING_PTR`
//! and `RSTRING_LEN`. Metorex keeps a string's characters as Rust text, so C
//! is handed a buffer holding a copy of its bytes, ended with as many NUL
//! bytes as one code unit of its encoding takes, and with room beyond that
//! C can grow into. What C writes there is carried back into the string
//! whenever control leaves C, what Ruby code changes is carried into the
//! buffer when control comes back to C, and the buffer stays where it is
//! until the string outgrows it.

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

/// The memory handed to C for one string: its bytes, the terminator, and
/// the room beyond, which keeps what C wrote there until the string grows
/// over it. `handed` holds the string's bytes as they stood when C was last
/// handed the memory or last left, which tells a write of C's apart. A
/// string made over static C memory hands that memory back instead while
/// its bytes are unchanged.
struct Buffer {
    area: Vec<u8>,
    handed: Vec<u8>,
    external: Option<*mut c_char>,
}

/// How many NUL bytes end a string in `encoding`: one code unit of it.
fn terminator_length(string: &StringValue) -> usize {
    wide_encoding(&string.encoding_name()).map_or(1, |shape| shape.unit())
}

impl Buffer {
    fn holding(current: Vec<u8>, terminator: usize) -> Buffer {
        let mut area = current.clone();
        area.resize(current.len() + terminator, 0);
        Buffer {
            area,
            handed: current,
            external: None,
        }
    }

    /// Puts `current`, the bytes the string holds now, in the memory,
    /// growing it when it is too small and ending it with the terminator.
    fn take(&mut self, current: Vec<u8>, terminator: usize) {
        let needed = current.len() + terminator;
        if self.area.len() < needed {
            self.area.resize(needed, 0);
        }
        self.area[..current.len()].copy_from_slice(&current);
        self.area[current.len()..needed].fill(0);
        self.handed = current;
        self.external = None;
    }

    /// How many bytes the memory holds beyond the terminator's room.
    fn capacity(&self, terminator: usize) -> usize {
        self.area.len() - terminator
    }
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
            let length = buffer.handed.len();
            if buffer.external.is_some() || buffer.area[..length] == buffer.handed[..] {
                continue;
            }
            if let Object::String(string) = to_object(*value) {
                store_bytes(&string, &buffer.area[..length]);
            }
            buffer.handed.copy_from_slice(&buffer.area[..length]);
        }
    });
}

/// Carries what Ruby code changed in each string into its buffer, once it
/// has run and control is back with C.
pub(super) fn carry_changes_in() {
    BUFFERS.with(|buffers| {
        for (value, buffer) in buffers.borrow_mut().iter_mut() {
            if let Object::String(string) = to_object(*value) {
                let current = binary_bytes(&string);
                if current != buffer.handed {
                    buffer.take(current, terminator_length(&string));
                }
            }
        }
    });
}

/// Runs `with` on the buffer of `string`, making one holding its bytes as
/// they stand when it has none.
fn with_buffer<T>(string: Value, with: impl FnOnce(&mut Buffer, &StringValue) -> T) -> T {
    carry_writes_back();
    let held = string_from(string);
    let current = binary_bytes(&held);
    let terminator = terminator_length(&held);
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers
            .entry(string)
            .or_insert_with(|| Buffer::holding(current.clone(), terminator));
        if buffer.handed != current {
            buffer.take(current, terminator);
        }
        with(buffer, &held)
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rstring_ptr(string: Value) -> *mut c_char {
    with_buffer(string, |buffer, _| {
        buffer
            .external
            .unwrap_or(buffer.area.as_mut_ptr().cast::<c_char>())
    })
}

/// How many bytes the string can hold before its buffer has to grow.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_capacity(string: Value) -> usize {
    with_buffer(string, |buffer, held| {
        buffer.capacity(terminator_length(held))
    })
}

/// Makes room for `expand` more bytes after the string's own, leaving its
/// bytes as they are.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_modify_expand(string: Value, expand: i64) {
    rb_str_modify(string);
    if expand < 0 {
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            "negative expanding string size",
            super::called_from(),
        ));
    }
    with_buffer(string, |buffer, held| {
        let needed = buffer.handed.len() + expand as usize + terminator_length(held);
        if buffer.area.len() < needed {
            buffer.area.resize(needed, 0);
        }
        buffer.external = None;
    });
}

/// Refuses a change to a frozen string or one locked for C.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_modify(string: Value) {
    super::objects::rb_check_frozen(string);
    if string_from(string).is_borrowed() {
        raise(crate::vm::errors::simple_exception(
            "RuntimeError",
            "can't modify string; temporarily locked",
            super::called_from(),
        ));
    }
}

/// Makes the string the first `length` bytes of its buffer, which may hold
/// bytes C wrote past the end it had, and ends it with the terminator.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_set_len(string: Value, length: i64) {
    rb_str_modify(string);
    with_buffer(string, |buffer, held| {
        let terminator = terminator_length(held);
        let capacity = buffer.capacity(terminator) as i64;
        if length > capacity || length < 0 {
            raise(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!("probable buffer overflow: {} for {}", length, capacity),
                super::called_from(),
            ));
        }
        let length = length as usize;
        buffer.area[length..length + terminator].fill(0);
        let kept = buffer.area[..length].to_vec();
        store_bytes(held, &kept);
        buffer.handed = kept;
        buffer.external = None;
    });
}

/// Makes the string `length` bytes long, growing its buffer first when it
/// is too small, and keeping the bytes the buffer holds.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_resize(string: Value, length: i64) -> Value {
    if length < 0 {
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            "negative string size (or size too big)",
            super::called_from(),
        ));
    }
    let capacity = rb_str_capacity(string) as i64;
    if length > capacity {
        rb_str_modify_expand(
            string,
            length - binary_bytes(&string_from(string)).len() as i64,
        );
    }
    rb_str_set_len(string, length);
    string
}

/// A new String over the `length` bytes of static C memory at `text`,
/// whose `RSTRING_PTR` is that memory while its bytes are unchanged.
pub(super) fn static_string(text: *const c_char, length: i64, encoding: &str) -> Value {
    let made = rb_str_new(text.cast::<u8>(), length);
    let held = string_from(made);
    held.set_encoding(encoding.to_string());
    let bytes = binary_bytes(&held);
    held.clear_bytes();
    store_bytes(&held, &bytes);
    with_buffer(made, |buffer, _| {
        buffer.external = Some(text as *mut c_char)
    });
    made
}

/// A new String of `capacity` bytes' room and none of its own, tagged
/// ASCII-8BIT.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_buf_new(capacity: i64) -> Value {
    let made = rb_str_new(std::ptr::null(), 0);
    with_buffer(made, |buffer, _| {
        buffer.area.resize(capacity.max(0) as usize + 1, 0);
    });
    made
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
    rb_str_new(text.cast::<u8>(), length as i64)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_append(string: Value, added: Value) -> Value {
    let added = string_value(to_object(added));
    call(to_object(string), "<<", vec![added]);
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
