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
        _ if call(object.clone(), "is_a?", vec![top_level_module("IO")]).is_truthy() => {
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
