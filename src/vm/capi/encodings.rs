//! The `rb_encoding` a C extension holds for each encoding: a struct metorex
//! makes once per encoding and keeps for the life of the program, naming the
//! encoding and the Encoding object it stands for.

use super::calls::{call, class_name_of, top_level_module};
use super::handles::{Value, to_object, to_value};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CString, c_char};

/// What C reads through an `rb_encoding *`.
#[repr(C)]
pub struct RbEncoding {
    name: *const c_char,
    encoding: Value,
}

thread_local! {
    static ENCODINGS: RefCell<HashMap<String, &'static RbEncoding>> = RefCell::new(HashMap::new());
}

/// The `rb_encoding` for the Encoding object `encoding`.
pub(super) fn encoding_struct(encoding: Object) -> *const RbEncoding {
    let name = call(encoding.clone(), "name", Vec::new()).to_string();
    ENCODINGS.with(|held| {
        let mut held = held.borrow_mut();
        let made = held.entry(name.clone()).or_insert_with(|| {
            let text = CString::new(name).unwrap_or_default();
            Box::leak(Box::new(RbEncoding {
                name: text.into_raw(),
                encoding: to_value(&encoding),
            }))
        });
        *made as *const RbEncoding
    })
}

/// The Encoding object an `rb_encoding *` from C stands for, or nil for a
/// NULL one.
pub(super) fn encoding_object(encoding: *const RbEncoding) -> Object {
    if encoding.is_null() {
        return Object::Nil;
    }
    // SAFETY: C hands back a pointer `encoding_struct` answered, which is
    // never freed.
    to_object(unsafe { (*encoding).encoding })
}

/// Whether `object` is an instance of the core class `name` or a subclass.
fn is_a_core(object: &Object, name: &str) -> bool {
    call(object.clone(), "is_a?", vec![top_level_module(name)]).is_truthy()
}

/// The `rb_encoding` of a String, Symbol, Regexp or IO, or of an Encoding
/// itself, as MRI's `rb_enc_get_index` reads it by the object's type, or
/// NULL for an object of any other type.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_get(object: Value) -> *const RbEncoding {
    let object = to_object(object);
    let encoding = match &object {
        Object::String(_) | Object::Symbol(_) | Object::Regex(_, _) => {
            call(object, "encoding", Vec::new())
        }
        _ if class_name_of(object.clone()) == "Encoding" => object,
        _ if is_a_core(&object, "String") || is_a_core(&object, "Regexp") => {
            call(object, "encoding", Vec::new())
        }
        _ if is_a_core(&object, "IO") => {
            match call(object.clone(), "internal_encoding", Vec::new()) {
                Object::Nil => call(object, "external_encoding", Vec::new()),
                inner => inner,
            }
        }
        _ => Object::Nil,
    };
    match encoding {
        Object::Nil => std::ptr::null(),
        encoding => encoding_struct(encoding),
    }
}

fn named_encoding(name: &str) -> *const RbEncoding {
    let encoding = call(
        top_level_module("Encoding"),
        "find",
        vec![Object::string(name)],
    );
    encoding_struct(encoding)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_utf8_encoding() -> *const RbEncoding {
    named_encoding("UTF-8")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_usascii_encoding() -> *const RbEncoding {
    named_encoding("US-ASCII")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ascii8bit_encoding() -> *const RbEncoding {
    named_encoding("ASCII-8BIT")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_from_encoding(encoding: *const RbEncoding) -> Value {
    to_value(&encoding_object(encoding))
}

/// The `rb_encoding` for an Encoding, or for the encoding a name names.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_to_encoding(encoding: Value) -> *const RbEncoding {
    let held = to_object(encoding);
    if class_name_of(held.clone()) == "Encoding" {
        return encoding_struct(held);
    }
    encoding_struct(call(top_level_module("Encoding"), "find", vec![held]))
}

/// Every encoding in the order `Encoding.list` gives them, which is the
/// order their indexes count.
fn listed_encodings() -> Vec<Object> {
    super::arrays::array_of(call(top_level_module("Encoding"), "list", Vec::new()))
        .unwrap_or_default()
}

/// The index of the Encoding object `encoding`, or -1 for one not listed.
fn index_of(encoding: &Object) -> i32 {
    let name = call(encoding.clone(), "name", Vec::new()).to_string();
    listed_encodings()
        .iter()
        .position(|held| call(held.clone(), "name", Vec::new()).to_string() == name)
        .map_or(-1, |index| index as i32)
}

/// The Encoding a name finds, or None for a name no encoding has.
fn found_encoding(name: &str) -> Option<Object> {
    let arguments = vec![Object::string(name)];
    super::control::caught(|| call(top_level_module("Encoding"), "find", arguments)).ok()
}

fn text(pointer: *const c_char) -> String {
    // SAFETY: C hands over a NUL-terminated string.
    unsafe { std::ffi::CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned()
}

fn struct_or_null(encoding: Option<Object>) -> *const RbEncoding {
    encoding.map_or(std::ptr::null(), encoding_struct)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_find(name: *const c_char) -> *const RbEncoding {
    struct_or_null(found_encoding(&text(name)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_find_index(name: *const c_char) -> i32 {
    found_encoding(&text(name)).map_or(-1, |found| index_of(&found))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_from_index(index: i32) -> *const RbEncoding {
    let listed = listed_encodings();
    struct_or_null(
        usize::try_from(index)
            .ok()
            .and_then(|index| listed.get(index).cloned()),
    )
}

/// The index of `encoding`, or 0 for NULL, as MRI answers.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_to_index(encoding: *const RbEncoding) -> i32 {
    if encoding.is_null() {
        return 0;
    }
    index_of(&encoding_object(encoding))
}

/// The index of an Encoding, or of the encoding a name finds, or -1 for a
/// name no encoding has.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_to_encoding_index(encoding: Value) -> i32 {
    let held = to_object(encoding);
    if class_name_of(held.clone()) == "Encoding" {
        return index_of(&held);
    }
    let name = super::strings::string_value(held).to_string();
    found_encoding(&name).map_or(-1, |found| index_of(&found))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ascii8bit_encindex() -> i32 {
    index_of(&encoding_object(rb_ascii8bit_encoding()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_utf8_encindex() -> i32 {
    index_of(&encoding_object(rb_utf8_encoding()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_usascii_encindex() -> i32 {
    index_of(&encoding_object(rb_usascii_encoding()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_locale_encoding() -> *const RbEncoding {
    named_encoding("locale")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_filesystem_encoding() -> *const RbEncoding {
    named_encoding("filesystem")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_locale_encindex() -> i32 {
    index_of(&encoding_object(rb_locale_encoding()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_filesystem_encindex() -> i32 {
    index_of(&encoding_object(rb_filesystem_encoding()))
}

fn default_encoding(name: &str) -> *const RbEncoding {
    match call(top_level_module("Encoding"), name, Vec::new()) {
        Object::Nil => std::ptr::null(),
        encoding => encoding_struct(encoding),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_default_internal_encoding() -> *const RbEncoding {
    default_encoding("default_internal")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_default_external_encoding() -> *const RbEncoding {
    default_encoding("default_external")
}

/// Makes `alias` a name for the encoding `original` names, answering that
/// encoding's index, or -1 when `original` names none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_alias(alias: *const c_char, original: *const c_char) -> i32 {
    let Some(found) = found_encoding(&text(original)) else {
        return -1;
    };
    let display = call(found.clone(), "name", Vec::new()).to_string();
    crate::vm::native_methods::class_methods::add_encoding_alias(&text(alias), &display);
    index_of(&found)
}

/// The most encodings a program can have, as MRI counts them.
const ENCODING_LIMIT: usize = 256;

/// Defines a dummy encoding named `name`, answering its index.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_dummy_encoding(name: *const c_char) -> i32 {
    let name = text(name);
    let listed = listed_encodings();
    if listed.len() >= ENCODING_LIMIT {
        super::raise(crate::vm::errors::simple_exception(
            "EncodingError",
            &format!("too many encoding (> {})", ENCODING_LIMIT),
            super::called_from(),
        ));
    }
    if found_encoding(&name).is_some() {
        super::raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            &format!("encoding {} is already registered", name),
            super::called_from(),
        ));
    }
    let encoding_class = super::exports::class_from(to_value(&top_level_module("Encoding")));
    let made = crate::class::Class::new(&name, Some(encoding_class));
    crate::vm::native_methods::class_methods::add_dummy_encoding(&name);
    let interpreter = super::interpreter();
    if let Some(Object::Array(held)) = interpreter.globals().get("__Encoding_list") {
        held.borrow_mut().push(Object::Class(made));
    }
    listed.len() as i32
}
