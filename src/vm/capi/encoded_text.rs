//! Text in an encoding, from C: setting and reading the encoding an object
//! is tagged with, building strings in one, and reading the characters a
//! run of bytes holds in a given encoding.

use super::calls::{call, top_level_module};
use super::encodings::{
    RbEncoding, encoding_object, encoding_struct, rb_ascii8bit_encindex, rb_enc_from_index,
    rb_enc_get, rb_enc_to_index,
};
use super::handles::{Value, to_object, to_value};
use super::{called_from, interpreter, raise};
use crate::object::Object;
use std::ffi::c_char;

const CODERANGE_7BIT: i32 = 0x100000;
const CODERANGE_VALID: i32 = 0x200000;
const CODERANGE_BROKEN: i32 = 0x300000;

/// The encodings Onigmo reads as Unicode.
const UNICODE_ENCODINGS: [&str; 10] = [
    "UTF-8",
    "UTF8-DoCoMo",
    "UTF8-KDDI",
    "UTF8-MAC",
    "UTF8-SoftBank",
    "CESU-8",
    "UTF-16LE",
    "UTF-16BE",
    "UTF-32LE",
    "UTF-32BE",
];

fn error(class_name: &str, message: &str) -> ! {
    raise(crate::vm::errors::simple_exception(
        class_name,
        message,
        called_from(),
    ))
}

/// The bytes from `start` up to `end`, none when `end` is not past `start`.
fn bytes_between<'a>(start: *const c_char, end: *const c_char) -> &'a [u8] {
    let length = (end as isize - start as isize).max(0) as usize;
    if length == 0 {
        return &[];
    }
    // SAFETY: C hands over a run of bytes from `start` to `end`.
    unsafe { std::slice::from_raw_parts(start.cast::<u8>(), length) }
}

/// The Encoding `encoding` stands for, or BINARY for NULL.
fn encoding_or_binary(encoding: *const RbEncoding) -> Object {
    if encoding.is_null() {
        encoding_object(rb_enc_from_index(rb_ascii8bit_encindex()))
    } else {
        encoding_object(encoding)
    }
}

/// A String holding `bytes`, tagged with `encoding`.
fn encoded(bytes: &[u8], encoding: *const RbEncoding) -> Object {
    let made = to_object(super::strings::rb_str_new(
        bytes.as_ptr(),
        bytes.len() as i64,
    ));
    call(
        made.clone(),
        "force_encoding",
        vec![encoding_or_binary(encoding)],
    );
    made
}

fn integer(object: Object) -> i64 {
    super::numbers::rb_num2long(to_value(&object))
}

fn byte_size(string: &Object) -> i64 {
    integer(call(string.clone(), "bytesize", Vec::new()))
}

fn is_valid(string: &Object) -> bool {
    call(string.clone(), "valid_encoding?", Vec::new()).is_truthy()
}

fn encoding_name(encoding: *const RbEncoding) -> String {
    call(encoding_or_binary(encoding), "name", Vec::new()).to_string()
}

/// How many bytes the first character of `bytes` takes in a wide UTF
/// encoding, a surrogate pair taking two code units, or None for an
/// encoding that is not one.
fn wide_length(bytes: &[u8], named: &str) -> Option<usize> {
    let shape = crate::vm::native_methods::string_methods::wide_encoding(named)?;
    let unit = shape.unit();
    let high = if shape.big_endian() {
        bytes[0]
    } else {
        *bytes.get(1).unwrap_or(&0)
    };
    Some(if unit == 2 && (0xD8..=0xDB).contains(&high) {
        4
    } else {
        unit
    })
}

/// The first character of the non-empty run `bytes` in `encoding`.
fn first_character(bytes: &[u8], encoding: *const RbEncoding) -> Object {
    match wide_length(bytes, &encoding_name(encoding)) {
        Some(length) => encoded(&bytes[..length.min(bytes.len())], encoding),
        None => call(encoded(bytes, encoding), "[]", vec![Object::Int(0)]),
    }
}

/// The bytes of `string`.
fn bytes_of(string: &Object) -> Vec<u8> {
    super::arrays::array_of(call(string.clone(), "bytes", Vec::new()))
        .unwrap_or_default()
        .into_iter()
        .map(|byte| integer(byte) as u8)
        .collect()
}

/// The index of the encoding `object` is tagged with, or -1 for an object
/// that carries none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_get_index(object: Value) -> i32 {
    let encoding = rb_enc_get(object);
    if encoding.is_null() {
        return -1;
    }
    rb_enc_to_index(encoding)
}

/// Tags a String or Regexp with the encoding at `index`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_set_index(object: Value, index: i32) {
    let held = to_object(object);
    let encoding = encoding_object(rb_enc_from_index(index));
    if let Object::Regex(pattern, _) = &held {
        let name = call(encoding, "name", Vec::new()).to_string();
        interpreter().force_pattern_encoding(pattern, name);
        return;
    }
    if !call(held.clone(), "is_a?", vec![top_level_module("String")]).is_truthy() {
        error(
            "ArgumentError",
            "cannot set encoding on non-encoding capable object",
        );
    }
    call(held, "force_encoding", vec![encoding]);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_associate_index(object: Value, index: i32) -> Value {
    super::objects::rb_check_frozen(object);
    rb_enc_set_index(object, index);
    object
}

/// Tags `object` with `encoding`, or with BINARY for NULL.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_associate(object: Value, encoding: *const RbEncoding) -> Value {
    let index = if encoding.is_null() {
        rb_ascii8bit_encindex()
    } else {
        rb_enc_to_index(encoding)
    };
    rb_enc_associate_index(object, index)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_copy(destination: Value, source: Value) {
    rb_enc_associate(destination, rb_enc_get(source));
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_encoding(object: Value) -> Value {
    to_value(&call(to_object(object), "encoding", Vec::new()))
}

/// The encoding `first` and `second` can be read in together, or NULL.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_compatible(first: Value, second: Value) -> *const RbEncoding {
    let arguments = vec![to_object(first), to_object(second)];
    match call(top_level_module("Encoding"), "compatible?", arguments) {
        Object::Nil => std::ptr::null(),
        encoding => encoding_struct(encoding),
    }
}

/// `rb_enc_compatible`, raising Encoding::CompatibilityError when there
/// is no such encoding.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_check(first: Value, second: Value) -> *const RbEncoding {
    let compatible = rb_enc_compatible(first, second);
    if compatible.is_null() {
        let named = |object: Value| call(to_object(object), "encoding", Vec::new()).to_string();
        error(
            "Encoding::CompatibilityError",
            &format!(
                "incompatible character encodings: {} and {}",
                named(first),
                named(second)
            ),
        );
    }
    compatible
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_str_new(
    text: *const c_char,
    length: i64,
    encoding: *const RbEncoding,
) -> Value {
    let end = text.wrapping_add(length.max(0) as usize);
    to_value(&encoded(bytes_between(text, end), encoding))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_str_new_cstr(
    text: *const c_char,
    encoding: *const RbEncoding,
) -> Value {
    // SAFETY: C hands over a NUL-terminated string.
    let length = unsafe { std::ffi::CStr::from_ptr(text) }.to_bytes().len();
    rb_enc_str_new(text, length as i64, encoding)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_str_new_static(
    text: *const c_char,
    length: i64,
    encoding: *const RbEncoding,
) -> Value {
    rb_enc_str_new(text, length, encoding)
}

/// Whether `string` holds only ASCII, other text its encoding reads, or
/// bytes its encoding cannot read.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_str_coderange(string: Value) -> i32 {
    let held = to_object(string);
    if call(held.clone(), "ascii_only?", Vec::new()).is_truthy() {
        CODERANGE_7BIT
    } else if is_valid(&held) {
        CODERANGE_VALID
    } else {
        CODERANGE_BROKEN
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_str_asciionly_p(string: Value) -> i32 {
    (rb_enc_str_coderange(string) == CODERANGE_7BIT) as i32
}

/// The character `code` stands for in `encoding`. A wide UTF encoding
/// spells the Unicode character, made in UTF-8 and converted.
fn character(code: i64, encoding: *const RbEncoding) -> Object {
    let named = encoding_name(encoding);
    if crate::vm::native_methods::string_methods::wide_encoding(&named).is_some() {
        let unicode = call(Object::Int(code), "chr", vec![Object::string("UTF-8")]);
        return call(unicode, "encode", vec![encoding_object(encoding)]);
    }
    call(Object::Int(code), "chr", vec![encoding_object(encoding)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_codelen(code: i32, encoding: *const RbEncoding) -> i32 {
    byte_size(&character(i64::from(code), encoding)) as i32
}

/// Writes the bytes of the character `code` stands for into `buffer`,
/// answering how many there are.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_mbcput(
    code: u32,
    buffer: *mut u8,
    encoding: *const RbEncoding,
) -> i32 {
    let bytes = bytes_of(&character(i64::from(code), encoding));
    // SAFETY: C hands over a buffer of ONIGENC_CODE_TO_MBC_MAXLEN bytes,
    // which holds any character.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len()) };
    bytes.len() as i32
}

/// How many characters the bytes from `start` to `end` hold in `encoding`,
/// each byte it cannot read counting as one.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_strlen(
    start: *const c_char,
    end: *const c_char,
    encoding: *const RbEncoding,
) -> i64 {
    integer(call(
        encoded(bytes_between(start, end), encoding),
        "length",
        Vec::new(),
    ))
}

/// The codepoint of the first character, its first byte when that does
/// not start a character, or 0 when there are no bytes.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_mbc_to_codepoint(
    start: *const c_char,
    end: *const c_char,
    encoding: *const RbEncoding,
) -> u32 {
    let bytes = bytes_between(start, end);
    if bytes.is_empty() {
        return 0;
    }
    let first = first_character(bytes, encoding);
    if !is_valid(&first) {
        return u32::from(bytes[0]);
    }
    integer(call(first, "ord", Vec::new())) as u32
}

/// What `rb_enc_precise_mbclen` answers when `missing` more bytes would
/// complete the character.
fn need_more(missing: usize) -> i32 {
    -1 - missing as i32
}

/// How many bytes the character a UTF-8 lead byte starts takes, or None
/// for a byte that starts none.
fn utf8_length(lead: u8) -> Option<usize> {
    match lead {
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

/// Whether one more byte after `bytes` would complete a character in
/// `encoding`.
fn one_byte_short(bytes: &[u8], encoding: *const RbEncoding) -> bool {
    (0..=u8::MAX).any(|next| {
        let mut longer = bytes.to_vec();
        longer.push(next);
        let first = first_character(&longer, encoding);
        is_valid(&first) && byte_size(&first) as usize == longer.len()
    })
}

/// How many more bytes would complete the character `bytes` start in
/// `encoding`, or None when they start none. A wide UTF encoding counts
/// code units and UTF-8 reads its lead byte. Any other encoding is asked
/// whether one more byte would do.
fn missing_bytes(bytes: &[u8], encoding: *const RbEncoding) -> Option<usize> {
    let named = encoding_name(encoding);
    if let Some(wanted) = wide_length(bytes, &named) {
        return (bytes.len() < wanted).then(|| wanted - bytes.len());
    }
    if named.starts_with("UTF-8") || named.starts_with("UTF8") {
        let wanted = utf8_length(bytes[0])?;
        let continuing = bytes[1..].iter().all(|byte| (0x80..=0xBF).contains(byte));
        return (continuing && bytes.len() < wanted).then(|| wanted - bytes.len());
    }
    one_byte_short(bytes, encoding).then_some(1)
}

/// The length of the character the bytes start: its byte count when they
/// hold all of it, how many more it needs as MBCLEN_NEEDMORE when they hold
/// the start of one, and -1 when they start none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_precise_mbclen(
    start: *const c_char,
    end: *const c_char,
    encoding: *const RbEncoding,
) -> i32 {
    let bytes = bytes_between(start, end);
    if bytes.is_empty() {
        return need_more(1);
    }
    let first = first_character(bytes, encoding);
    if is_valid(&first) {
        return byte_size(&first) as i32;
    }
    missing_bytes(bytes, encoding).map_or(-1, need_more)
}

/// Where the character `index` characters in starts, or the end when there
/// are fewer.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_nth(
    start: *const c_char,
    end: *const c_char,
    index: i64,
    encoding: *const RbEncoding,
) -> *const c_char {
    let string = encoded(bytes_between(start, end), encoding);
    let leading = call(
        string,
        "[]",
        vec![Object::Int(0), Object::Int(index.max(0))],
    );
    start.wrapping_add(byte_size(&leading) as usize)
}

/// The codepoint of the first character and, through `length`, its byte
/// count, refusing an empty run or one that does not start a character.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_codepoint_len(
    start: *const c_char,
    end: *const c_char,
    length: *mut i32,
    encoding: *const RbEncoding,
) -> u32 {
    let bytes = bytes_between(start, end);
    if bytes.is_empty() {
        error("ArgumentError", "empty string");
    }
    let first = first_character(bytes, encoding);
    if !is_valid(&first) {
        let named = call(encoding_object(encoding), "name", Vec::new());
        error(
            "ArgumentError",
            &format!("invalid byte sequence in {}", named),
        );
    }
    // SAFETY: C hands over a pointer to an int.
    unsafe { *length = byte_size(&first) as i32 };
    integer(call(first, "ord", Vec::new())) as u32
}

/// Where the character holding the byte at `at` starts, or `at` itself
/// when it lies past the last character.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_left_char_head(
    start: *const c_char,
    at: *const c_char,
    end: *const c_char,
    encoding: *const RbEncoding,
) -> *const c_char {
    let offset = (at as isize - start as isize).max(0) as i64;
    let string = encoded(bytes_between(start, end), encoding);
    let characters = super::arrays::array_of(call(string, "chars", Vec::new())).unwrap_or_default();
    let mut head = 0;
    for character in characters {
        let size = byte_size(&character);
        if head + size > offset {
            return start.wrapping_add(head as usize);
        }
        head += size;
    }
    at
}

/// The Unicode character `code` stands for in `encoding`, or None when it
/// stands for none.
fn unicode_character(code: i32, encoding: *const RbEncoding) -> Option<char> {
    let converted = super::control::caught(|| {
        let made = character(i64::from(code), encoding);
        call(made, "encode", vec![Object::string("UTF-8")]).to_string()
    });
    converted.ok().and_then(|text| text.chars().next())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_isalnum(code: i32, encoding: *const RbEncoding) -> i32 {
    unicode_character(code, encoding).is_some_and(char::is_alphanumeric) as i32
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enc_isspace(code: i32, encoding: *const RbEncoding) -> i32 {
    unicode_character(code, encoding).is_some_and(char::is_whitespace) as i32
}

/// Writes `code` as UTF-8 into `buffer`, with the five and six byte forms
/// the original UTF-8 allowed, answering the byte count.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_uv_to_utf8(buffer: *mut u8, code: u64) -> i32 {
    const LEADS: [(u64, u8); 6] = [
        (0x7F, 0x00),
        (0x7FF, 0xC0),
        (0xFFFF, 0xE0),
        (0x1F_FFFF, 0xF0),
        (0x3FF_FFFF, 0xF8),
        (0x7FFF_FFFF, 0xFC),
    ];
    let Some(length) = LEADS.iter().position(|(most, _)| code <= *most) else {
        error("RangeError", "pack(U): value out of range");
    };
    let mut bytes = vec![0_u8; length + 1];
    let mut rest = code;
    for byte in bytes.iter_mut().skip(1).rev() {
        *byte = 0x80 | (rest & 0x3F) as u8;
        rest >>= 6;
    }
    bytes[0] = LEADS[length].1 | rest as u8;
    // SAFETY: C hands over a buffer of six bytes, which holds any form.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len()) };
    bytes.len() as i32
}

/// Raises `klass` with `message` tagged with `encoding`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_enc_raise(
    encoding: *const RbEncoding,
    klass: Value,
    message: Value,
) -> ! {
    call(
        to_object(message),
        "force_encoding",
        vec![encoding_object(encoding)],
    );
    super::exceptions::rb_exc_raise(super::exceptions::rb_exc_new_str(klass, message))
}

/// Writes the case fold of the character at `*at` into `folded`, moves
/// `*at` past it, and answers how many bytes the fold took, or 0 when no
/// character starts there.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_mbc_case_fold(
    encoding: *const RbEncoding,
    _flag: i32,
    at: *mut *const c_char,
    end: *const c_char,
    folded: *mut u8,
) -> i32 {
    // SAFETY: C hands over a pointer to its position in the text.
    let start = unsafe { *at };
    let bytes = bytes_between(start, end);
    if bytes.is_empty() {
        return 0;
    }
    let first = first_character(bytes, encoding);
    let fold = call(first.clone(), "downcase", vec![Object::symbol("fold")]);
    let fold_bytes = bytes_of(&fold);
    // SAFETY: C hands over a buffer that holds the longest fold, and the
    // position moves by the bytes of one character it held.
    unsafe {
        std::ptr::copy_nonoverlapping(fold_bytes.as_ptr(), folded, fold_bytes.len());
        *at = start.wrapping_add(byte_size(&first) as usize);
    }
    fold_bytes.len() as i32
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_is_unicode(encoding: *const RbEncoding) -> i32 {
    let named = call(encoding_object(encoding), "name", Vec::new()).to_string();
    UNICODE_ENCODINGS.contains(&named.as_str()) as i32
}
