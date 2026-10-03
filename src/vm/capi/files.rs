//! Paths and files from C: `FilePathValue`, `rb_file_open` and
//! `rb_file_open_str`.

use super::calls::{answers, call};
use super::exports::text;
use super::handles::{Value, to_object, to_value};
use super::interpreter;
use super::strings::string_value;
use crate::object::Object;
use std::ffi::c_char;

/// The String a path argument stands for: a String itself, or what the
/// object answers to `to_path`, or the object, converted through `to_str`.
fn path_from(object: Object) -> Object {
    if let Object::String(_) = object {
        return object;
    }
    if answers(&object, "to_path") {
        return string_value(call(object, "to_path", Vec::new()));
    }
    string_value(object)
}

fn open(path: Object, mode: *const c_char) -> Value {
    let file = interpreter()
        .globals()
        .get("File")
        .expect("File is defined before any extension loads");
    let mode = Object::string(text(mode));
    to_value(&call(file, "open", vec![path, mode]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_get_path(object: Value) -> Value {
    to_value(&path_from(to_object(object)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_file_open(name: *const c_char, mode: *const c_char) -> Value {
    open(Object::string(text(name)), mode)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_file_open_str(name: Value, mode: *const c_char) -> Value {
    open(path_from(to_object(name)), mode)
}
