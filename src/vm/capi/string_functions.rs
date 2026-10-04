//! The String functions C calls beyond reading and writing the bytes:
//! making strings in an encoding, appending, comparing, slicing,
//! converting between encodings, interning, and locking a string while C
//! holds its bytes.

use super::calls::{call, top_level_module};
use super::encodings::{
    RbEncoding, encoding_object, rb_default_external_encoding, rb_default_internal_encoding,
    rb_enc_get, rb_locale_encoding,
};
use super::handles::{QNIL, Value, to_object, to_value};
use super::strings::{
    rb_rstring_ptr, rb_str_new, rb_str_new_cstr, static_string, string_from, string_value,
};
use super::{called_from, raise};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::{CStr, c_char};

thread_local! {
    /// The objects made hidden from Ruby code, whose class C reads as 0.
    static HIDDEN: RefCell<HashSet<Value>> = RefCell::new(HashSet::new());
}

/// Whether `object` is hidden from Ruby code, as a string from
/// `rb_str_tmp_new` is until `rb_obj_reveal`.
pub(super) fn is_hidden(object: Value) -> bool {
    HIDDEN.with(|held| held.borrow().contains(&object))
}

fn error(class_name: &str, message: &str) -> ! {
    raise(crate::vm::errors::simple_exception(
        class_name,
        message,
        called_from(),
    ))
}

fn send(object: Value, name: &str, arguments: Vec<Object>) -> Value {
    to_value(&call(to_object(object), name, arguments))
}

fn integer(object: Value) -> i64 {
    super::numbers::rb_num2long(object)
}

fn cstr_length(text: *const c_char) -> i64 {
    // SAFETY: C hands over a NUL-terminated string.
    unsafe { CStr::from_ptr(text) }.to_bytes().len() as i64
}

/// A new String holding the bytes at `text`, tagged with `encoding`.
fn tagged(text: *const c_char, length: i64, encoding: Object) -> Value {
    let made = rb_str_new(text.cast::<u8>(), length);
    send(made, "force_encoding", vec![encoding]);
    made
}

fn named(name: &str) -> Object {
    call(
        top_level_module("Encoding"),
        "find",
        vec![Object::string(name)],
    )
}

/// The Encoding `encoding` stands for, or BINARY for NULL.
fn encoding_or_binary(encoding: *const RbEncoding) -> Object {
    if encoding.is_null() {
        named("ASCII-8BIT")
    } else {
        encoding_object(encoding)
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_String(object: Value) -> Value {
    to_value(&call(
        top_level_module("Kernel"),
        "String",
        vec![to_object(object)],
    ))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_to_str(object: Value) -> Value {
    to_value(&string_value(to_object(object)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_dup(string: Value) -> Value {
    send(string, "dup", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_new_shared(string: Value) -> Value {
    rb_str_dup(string)
}

/// `string` itself when it is frozen, or a frozen copy.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_new_frozen(string: Value) -> Value {
    if super::objects::rb_obj_frozen_p(string) == super::handles::QTRUE {
        return string;
    }
    send(rb_str_dup(string), "freeze", Vec::new())
}

/// A String of the class `string` is, holding the bytes at `text`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_new_with_class(
    string: Value,
    text: *const c_char,
    length: i64,
) -> Value {
    let klass = super::objects::rb_obj_class(string);
    send(
        klass,
        "new",
        vec![to_object(rb_str_new(text.cast::<u8>(), length))],
    )
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_buf_new_cstr(text: *const c_char) -> Value {
    rb_str_new_cstr(text)
}

/// A String of `length` NUL bytes, hidden from Ruby code until revealed.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_tmp_new(length: i64) -> Value {
    let zeros = vec![0_u8; length.max(0) as usize];
    let made = rb_str_new(zeros.as_ptr(), zeros.len() as i64);
    HIDDEN.with(|held| held.borrow_mut().insert(made));
    made
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_reveal(object: Value, _klass: Value) -> Value {
    HIDDEN.with(|held| held.borrow_mut().remove(&object));
    object
}

/// Drops the first `length` bytes of the string.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_drop_bytes(string: Value, length: i64) -> Value {
    super::strings::rb_str_modify(string);
    let size = integer(send(string, "bytesize", Vec::new()));
    let rest = send(
        string,
        "byteslice",
        vec![Object::Int(length), Object::Int(size)],
    );
    send(string, "replace", vec![to_object(rest)]);
    string
}

/// Metorex frees nothing a String holds while it is reachable.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_free(_string: Value) {}

/// Locks the string against changes while C holds its bytes.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_locktmp(string: Value) -> Value {
    super::objects::rb_check_frozen(string);
    let held = string_from(string);
    if held.is_borrowed() {
        error("RuntimeError", "temporal locking already locked string");
    }
    held.set_borrowed(true);
    string
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_unlocktmp(string: Value) -> Value {
    super::objects::rb_check_frozen(string);
    let held = string_from(string);
    if !held.is_borrowed() {
        error("RuntimeError", "temporal unlocking already unlocked string");
    }
    held.set_borrowed(false);
    string
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_usascii_str_new(text: *const c_char, length: i64) -> Value {
    tagged(text, length, named("US-ASCII"))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_usascii_str_new_cstr(text: *const c_char) -> Value {
    rb_usascii_str_new(text, cstr_length(text))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_usascii_str_new_static(text: *const c_char, length: i64) -> Value {
    static_string(text, length, "US-ASCII")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_utf8_str_new(text: *const c_char, length: i64) -> Value {
    tagged(text, length, named("UTF-8"))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_utf8_str_new_cstr(text: *const c_char) -> Value {
    rb_utf8_str_new(text, cstr_length(text))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_utf8_str_new_static(text: *const c_char, length: i64) -> Value {
    static_string(text, length, "UTF-8")
}

fn is_ascii_only(string: Value) -> bool {
    to_object(send(string, "ascii_only?", Vec::new())).is_truthy()
}

/// A String read from outside the program in `encoding`: BINARY when
/// `encoding` is US-ASCII and the bytes go beyond it, and converted to
/// `Encoding.default_internal` when that is set.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_external_str_new_with_enc(
    text: *const c_char,
    length: i64,
    encoding: *const RbEncoding,
) -> Value {
    let made = tagged(text, length, encoding_object(encoding));
    let name = to_object(send(made, "encoding", Vec::new())).to_string();
    if name == "US-ASCII" && !is_ascii_only(made) {
        send(made, "force_encoding", vec![named("ASCII-8BIT")]);
        return made;
    }
    let internal = rb_default_internal_encoding();
    if internal.is_null() {
        return made;
    }
    rb_str_conv_enc(made, encoding, internal)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_external_str_new(text: *const c_char, length: i64) -> Value {
    rb_external_str_new_with_enc(text, length, rb_default_external_encoding())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_external_str_new_cstr(text: *const c_char) -> Value {
    rb_external_str_new(text, cstr_length(text))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_locale_str_new(text: *const c_char, length: i64) -> Value {
    rb_external_str_new_with_enc(text, length, rb_locale_encoding())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_locale_str_new_cstr(text: *const c_char) -> Value {
    rb_locale_str_new(text, cstr_length(text))
}

/// The one frozen String `String#-@` answers for these bytes in
/// `encoding`, or in US-ASCII or BINARY by what the bytes hold when
/// `encoding` is None.
fn interned(text: *const c_char, length: i64, encoding: Option<Object>) -> Value {
    let made = rb_str_new(text.cast::<u8>(), length);
    let encoding = encoding.unwrap_or_else(|| {
        named(if is_ascii_only(made) {
            "US-ASCII"
        } else {
            "ASCII-8BIT"
        })
    });
    send(made, "force_encoding", vec![encoding]);
    send(made, "-@", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_interned_str(text: *const c_char, length: i64) -> Value {
    interned(text, length, None)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_interned_str_cstr(text: *const c_char) -> Value {
    rb_interned_str(text, cstr_length(text))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_interned_str(
    text: *const c_char,
    length: i64,
    encoding: *const RbEncoding,
) -> Value {
    interned(text, length, Some(encoding_or_binary(encoding)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_interned_str_cstr(
    text: *const c_char,
    encoding: *const RbEncoding,
) -> Value {
    rb_enc_interned_str(text, cstr_length(text), encoding)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_to_interned_str(string: Value) -> Value {
    send(string, "-@", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_plus(first: Value, second: Value) -> Value {
    send(first, "+", vec![to_object(second)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_times(string: Value, count: Value) -> Value {
    send(string, "*", vec![to_object(count)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_buf_append(string: Value, added: Value) -> Value {
    send(string, "<<", vec![to_object(added)]);
    string
}

/// Appends the bytes at `text`, read in `encoding`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_str_buf_cat(
    string: Value,
    text: *const c_char,
    length: i64,
    encoding: *const RbEncoding,
) -> Value {
    rb_str_buf_append(string, tagged(text, length, encoding_object(encoding)))
}

/// Appends the bytes at `text`, read in the string's own encoding.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_buf_cat(string: Value, text: *const c_char, length: i64) -> Value {
    rb_enc_str_buf_cat(string, text, length, rb_enc_get(string))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_cat(string: Value, text: *const c_char, length: i64) -> Value {
    rb_str_buf_cat(string, text, length)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_cat_cstr(string: Value, text: *const c_char) -> Value {
    rb_str_buf_cat(string, text, cstr_length(text))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_cmp(first: Value, second: Value) -> i32 {
    integer(send(first, "<=>", vec![to_object(second)])) as i32
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_equal(first: Value, second: Value) -> Value {
    send(first, "==", vec![to_object(second)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_length(string: Value) -> Value {
    send(string, "length", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_strlen(string: Value) -> i64 {
    integer(rb_str_length(string))
}

/// How many characters the first `byte_offset` bytes hold.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_sublen(string: Value, byte_offset: i64) -> i64 {
    let leading = send(
        string,
        "byteslice",
        vec![Object::Int(0), Object::Int(byte_offset)],
    );
    integer(send(leading, "length", Vec::new()))
}

fn byte_size(string: Value) -> i64 {
    integer(send(string, "bytesize", Vec::new()))
}

/// Where the `*length` characters from character `start` begin, setting
/// `*length` to the bytes they take, or NULL when `start` lies outside the
/// string or `*length` is negative.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_subpos(string: Value, start: i64, length: *mut i64) -> *mut c_char {
    let total = rb_str_strlen(string);
    let start = if start < 0 { start + total } else { start };
    // SAFETY: C hands over a pointer to a long.
    let asked = unsafe { *length };
    if start < 0 || start > total || asked < 0 {
        return std::ptr::null_mut();
    }
    let characters = asked.min(total - start);
    let before = rb_str_substr(string, 0, start);
    let taken = rb_str_substr(string, start, characters);
    // SAFETY: as above.
    unsafe { *length = byte_size(taken) };
    rb_rstring_ptr(string).wrapping_add(byte_size(before) as usize)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_subseq(string: Value, start: i64, length: i64) -> Value {
    send(
        string,
        "byteslice",
        vec![Object::Int(start), Object::Int(length)],
    )
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_substr(string: Value, start: i64, length: i64) -> Value {
    send(string, "[]", vec![Object::Int(start), Object::Int(length)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_update(string: Value, start: i64, length: i64, replacement: Value) {
    let arguments = vec![
        Object::Int(start),
        Object::Int(length),
        to_object(replacement),
    ];
    send(string, "[]=", arguments);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_split(string: Value, separator: *const c_char) -> Value {
    send(string, "split", vec![to_object(rb_str_new_cstr(separator))])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_inspect(string: Value) -> Value {
    send(string, "inspect", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_intern(string: Value) -> Value {
    send(string, "to_sym", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_freeze(string: Value) -> Value {
    send(string, "freeze", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_hash(string: Value) -> usize {
    integer(send(string, "hash", Vec::new())) as usize
}

/// The Integer the text at `text` spells in `base`. Strictly, the whole
/// text has to spell one, and otherwise the digits it starts with are read.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cstr_to_inum(text: *const c_char, base: i32, strict: i32) -> Value {
    let spelled = to_object(rb_str_new_cstr(text));
    if strict != 0 || base == 0 {
        let arguments = vec![spelled, Object::Int(i64::from(base))];
        return to_value(&call(top_level_module("Kernel"), "Integer", arguments));
    }
    to_value(&call(spelled, "to_i", vec![Object::Int(i64::from(base))]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cstr2inum(text: *const c_char, base: i32) -> Value {
    rb_cstr_to_inum(text, base, (base == 0) as i32)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str2inum(string: Value, base: i32) -> Value {
    let held = string_value(to_object(string));
    to_value(&call(held, "to_i", vec![Object::Int(i64::from(base))]))
}

/// `string` converted to `encoding`, with the replacement the flags and
/// the options Hash ask for.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_encode(
    string: Value,
    encoding: Value,
    flags: i32,
    options: Value,
) -> Value {
    const INVALID_REPLACE: i32 = 0x02;
    const UNDEF_REPLACE: i32 = 0x20;
    let mut keywords: indexmap::IndexMap<String, Object> = match to_object(options) {
        Object::Dict(pairs) => pairs.borrow().clone(),
        _ => indexmap::IndexMap::new(),
    };
    if flags & INVALID_REPLACE != 0 {
        keywords.insert(":invalid".to_string(), Object::symbol("replace"));
    }
    if flags & UNDEF_REPLACE != 0 {
        keywords.insert(":undef".to_string(), Object::symbol("replace"));
    }
    let mut arguments = vec![
        to_object(encoding),
        Object::Dict(std::rc::Rc::new(RefCell::new(keywords))),
    ];
    super::procs::pass_last_as_keywords(&mut arguments);
    send(string, "encode", arguments)
}

/// `string` converted from `from` to `to`: itself when there is nothing to
/// convert or the conversion fails, retagged when its bytes read the same
/// in `to`, and converted otherwise.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_conv_enc_opts(
    string: Value,
    from: *const RbEncoding,
    to: *const RbEncoding,
    _flags: i32,
    _options: Value,
) -> Value {
    if to.is_null() {
        return string;
    }
    let from = if from.is_null() {
        rb_enc_get(string)
    } else {
        from
    };
    let target = encoding_object(to);
    let target_name = call(target.clone(), "name", Vec::new()).to_string();
    let source_name = call(encoding_object(from), "name", Vec::new()).to_string();
    if source_name == target_name {
        return string;
    }
    let ascii_compatible = call(target.clone(), "ascii_compatible?", Vec::new()).is_truthy();
    if (ascii_compatible && is_ascii_only(string)) || target_name == "ASCII-8BIT" {
        let current = to_object(send(string, "encoding", Vec::new())).to_string();
        if current == target_name {
            return string;
        }
        return send(rb_str_dup(string), "force_encoding", vec![target]);
    }
    let converted = super::control::caught(|| {
        let copy = rb_str_dup(string);
        send(copy, "force_encoding", vec![encoding_object(from)]);
        send(copy, "encode", vec![target])
    });
    converted.unwrap_or(string)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_conv_enc(
    string: Value,
    from: *const RbEncoding,
    to: *const RbEncoding,
) -> Value {
    rb_str_conv_enc_opts(string, from, to, 0, QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_export_to_enc(string: Value, encoding: *const RbEncoding) -> Value {
    rb_str_conv_enc(string, rb_enc_get(string), encoding)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_export(string: Value) -> Value {
    rb_str_export_to_enc(string, rb_default_external_encoding())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_export_locale(string: Value) -> Value {
    rb_str_export_to_enc(string, rb_locale_encoding())
}
