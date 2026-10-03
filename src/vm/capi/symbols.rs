//! IDs, which are the VALUE of the Symbol they name. Symbols share one
//! handle per name, so two IDs for the same name compare equal in C.

use super::calls::{call, class_name_of};
use super::encodings::{RbEncoding, encoding_object};
use super::handles::{Value, to_object, to_value};
use super::raise;
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CString, c_char};

thread_local! {
    /// The C strings `rb_id2name` has answered, kept for as long as the
    /// program runs since C may hold on to any of them.
    static NAMES: RefCell<HashMap<Value, CString>> = RefCell::new(HashMap::new());
}

/// The name of the Symbol an ID or a Symbol VALUE stands for, refusing a
/// VALUE that is no Symbol.
pub(super) fn symbol_name(value: Value) -> String {
    match to_object(value) {
        Object::Symbol(name) => name.as_str().to_string(),
        other => raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!(
                "wrong argument type {} (expected symbol)",
                class_name_of(other)
            ),
            super::called_from(),
        )),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_intern(name: *const c_char) -> Value {
    to_value(&Object::symbol(super::exports::text(name)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_id2sym(id: Value) -> Value {
    id
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_sym2id(symbol: Value) -> Value {
    symbol_name(symbol);
    symbol
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_id2name(id: Value) -> *const c_char {
    if id == 0 {
        return std::ptr::null();
    }
    let name = symbol_name(id);
    NAMES.with(|names| {
        names
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| CString::new(name).unwrap_or_default())
            .as_ptr()
    })
}

/// A String holding the `length` bytes at `text`, tagged with `encoding`.
fn tagged_string(text: *const u8, length: i64, encoding: *const RbEncoding) -> Object {
    let made = to_object(super::strings::rb_str_new(text, length));
    call(made, "force_encoding", vec![encoding_object(encoding)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_intern3(
    text: *const u8,
    length: i64,
    encoding: *const RbEncoding,
) -> Value {
    to_value(&call(
        tagged_string(text, length, encoding),
        "to_sym",
        Vec::new(),
    ))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_intern2(text: *const u8, length: i64) -> Value {
    rb_intern3(text, length, super::encodings::rb_usascii_encoding())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_intern_str(string: Value) -> Value {
    to_value(&call(to_object(string), "to_sym", Vec::new()))
}

/// The Symbol for the bytes given when one has already been made, or nil,
/// without making one.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_symbol_cstr(
    text: *const u8,
    length: i64,
    encoding: *const RbEncoding,
) -> Value {
    let name = tagged_string(text, length, encoding).to_string();
    if !crate::symbol_registry::contains(&name) {
        return super::handles::QNIL;
    }
    to_value(&Object::symbol(name))
}

/// The name of an ID as a String in the symbol's encoding, or `Qfalse` for
/// the ID 0.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_id2str(id: Value) -> Value {
    if id == 0 {
        return super::handles::QFALSE;
    }
    symbol_name(id);
    to_value(&call(to_object(id), "to_s", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_sym2str(symbol: Value) -> Value {
    rb_id2str(symbol)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_symbol_p(value: Value) -> i32 {
    matches!(to_object(value), Object::Symbol(_)) as i32
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_is_const_id(id: Value) -> i32 {
    crate::vm::native_methods::is_valid_constant_name(&symbol_name(id)) as i32
}

/// Whether the name is an instance variable's, `@` and an identifier.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_is_instance_id(id: Value) -> i32 {
    let name = symbol_name(id);
    match name.strip_prefix('@') {
        Some(rest) => !rest.starts_with('@') && is_identifier(rest),
        None => false,
    }
    .into()
}

/// Whether the name is a class variable's, `@@` and an identifier.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_is_class_id(id: Value) -> i32 {
    symbol_name(id)
        .strip_prefix("@@")
        .is_some_and(is_identifier)
        .into()
}

/// Whether `name` is a plain identifier: a letter or underscore, or any
/// character past ASCII, then any of those or digits.
fn is_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    let starts = characters
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic() || !first.is_ascii());
    starts && characters.all(|held| held == '_' || held.is_alphanumeric() || !held.is_ascii())
}

/// A Symbol for a Symbol, a String, or what `to_str` answers, as
/// `rb_to_symbol` makes one.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_to_symbol(value: Value) -> Value {
    let object = to_object(value);
    if matches!(object, Object::Symbol(_)) {
        return value;
    }
    match super::strings::check_string(object.clone()) {
        Some(string) => to_value(&call(string, "to_sym", Vec::new())),
        None => {
            let inspected = call(object, "inspect", Vec::new()).to_string();
            raise(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("{} is not a symbol nor a string", inspected),
                super::called_from(),
            ))
        }
    }
}
