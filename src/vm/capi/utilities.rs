//! The odds and ends `ruby/util.h` and the iteration helpers give a C
//! extension: keywords and the block it was called with, `rb_get_kwargs`,
//! breaking out of the block that called it, where it was called from, a C
//! int range check, and `ruby_strtod`.

use super::calls::call;
use super::handles::{QNIL, QUNDEF, Value, to_object, to_value};
use super::{called_from, called_with_block, interpreter, keywords_given, raise};
use crate::error::MetorexError;
use crate::object::Object;
use std::ffi::{CStr, CString, c_char};

fn error(class: &str, message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        class,
        &message,
        called_from(),
    ))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_keyword_given_p() -> i32 {
    keywords_given() as i32
}

/// The block the C method running now was called with, as a Proc.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_block_proc() -> Value {
    match called_with_block() {
        Some(block @ Object::Block(_)) => to_value(&block),
        _ => error(
            "ArgumentError",
            "tried to create Proc object without a block".to_string(),
        ),
    }
}

/// How MRI words a missing or unknown keyword list: `missing keyword: :a`,
/// or `missing keywords: :a, :b`.
fn keyword_error(kind: &str, keys: &[Object]) -> ! {
    let inspected: Vec<String> = keys
        .iter()
        .map(|key| call(key.clone(), "inspect", Vec::new()).to_string())
        .collect();
    let noun = if keys.len() == 1 {
        "keyword"
    } else {
        "keywords"
    };
    error(
        "ArgumentError",
        format!("{} {}: {}", kind, noun, inspected.join(", ")),
    )
}

/// Takes the keywords `names` lists out of `hash`, storing their values
/// through `values` and answering how many were found. The first `required`
/// must be there. Without a negative `optional`, any key left over is
/// refused; a negative one allows `-1 - optional` optional keys and leaves
/// the rest in the hash.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_get_kwargs(
    hash: Value,
    names: *const Value,
    required: i32,
    optional: i32,
    values: *mut Value,
) -> i32 {
    let (optional, rest) = if optional < 0 {
        (-1 - optional, true)
    } else {
        (optional, false)
    };
    let total = (required + optional).max(0) as usize;
    // SAFETY: C hands over `required + optional` IDs at `names`.
    let names = unsafe { std::slice::from_raw_parts(names, total) };
    let hash = to_object(hash);
    let has_hash = !matches!(hash, Object::Nil);
    let store = |index: usize, value: Value| {
        if !values.is_null() {
            // SAFETY: C hands over room for `required + optional` values.
            unsafe { *values.add(index) = value };
        }
    };
    // Takes the key out of the hash, answering its value when it was there.
    let extract = |key: &Object| -> Option<Object> {
        if !has_hash || !call(hash.clone(), "key?", vec![key.clone()]).is_truthy() {
            return None;
        }
        if values.is_null() {
            return Some(Object::Nil);
        }
        Some(call(hash.clone(), "delete", vec![key.clone()]))
    };
    let mut missing = Vec::new();
    for (index, name) in names.iter().take(required.max(0) as usize).enumerate() {
        let key = to_object(*name);
        match extract(&key) {
            Some(found) => store(index, to_value(&found)),
            None => missing.push(key),
        }
    }
    if !missing.is_empty() {
        keyword_error("missing", &missing);
    }
    let mut found_count = required.max(0);
    for (offset, name) in names.iter().skip(required.max(0) as usize).enumerate() {
        let index = required.max(0) as usize + offset;
        match extract(&to_object(*name)) {
            Some(found) => {
                store(index, to_value(&found));
                found_count += 1;
            }
            None => store(index, QUNDEF),
        }
    }
    if !rest && has_hash {
        let left = match call(hash.clone(), "keys", Vec::new()) {
            Object::Array(keys) => keys.borrow().clone(),
            _ => Vec::new(),
        };
        let allowed = if values.is_null() {
            found_count as usize
        } else {
            0
        };
        if left.len() > allowed {
            let known: Vec<Object> = names.iter().map(|name| to_object(*name)).collect();
            let unknown: Vec<Object> = left
                .into_iter()
                .filter(|key| !known.contains(key))
                .collect();
            keyword_error("unknown", &unknown);
        }
    }
    found_count
}

/// Breaks out of the block whose call reached this C method, as `break`
/// written in that block would, with `value` or nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iter_break_value(value: Value) -> ! {
    raise(MetorexError::BlockBreak {
        value: to_object(value),
        location: crate::vm::utils::position_to_location(called_from()),
        home_frame: None,
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iter_break() -> ! {
    rb_iter_break_value(QNIL)
}

thread_local! {
    /// The file names `rb_sourcefile` has answered, kept for the life of
    /// the program since C may hold on to any of them.
    static SOURCE_FILES: std::cell::RefCell<Vec<CString>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The file of the Ruby code that called into C.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_sourcefile() -> *const c_char {
    let machine = interpreter();
    let named = machine
        .current_source_file
        .clone()
        .or_else(|| {
            machine
                .reported_current_file()
                .map(|file| file.display().to_string())
        })
        .unwrap_or_default();
    SOURCE_FILES.with(|held| {
        let mut held = held.borrow_mut();
        if let Some(found) = held.iter().find(|kept| kept.to_bytes() == named.as_bytes()) {
            return found.as_ptr();
        }
        held.push(CString::new(named).unwrap_or_default());
        held.last().map_or(std::ptr::null(), |kept| kept.as_ptr())
    })
}

/// The line of the Ruby code that called into C.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_sourceline() -> i32 {
    called_from().line as i32
}

/// Refuses a C long that does not fit a C int, as `rb_long2int` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_out_of_int(number: i64) -> ! {
    let direction = if number < 0 { "small" } else { "big" };
    error(
        "RangeError",
        format!("integer {} too {} to convert to 'int'", number, direction),
    )
}

/// The hex form MRI's strtod reads after `0x`: hex digits, an optional
/// fraction, and an optional binary exponent after `p`. Answers the value
/// and where reading stopped, or None for no digits at all.
fn hex_value(bytes: &[u8], start: usize) -> Option<(f64, usize)> {
    let digit = |at: usize| bytes.get(at).and_then(|held| (*held as char).to_digit(16));
    let mut at = start;
    if bytes.get(at).is_none() || (digit(at).is_none() && bytes.get(at) != Some(&b'.')) {
        return None;
    }
    let mut value = 0.0_f64;
    let mut scale = 1.0_f64;
    let mut exponent: i64 = -4;
    while bytes.get(at) == Some(&b'0') {
        at += 1;
    }
    if at == bytes.len() {
        return Some((0.0, at));
    }
    while let Some(held) = digit(at) {
        value += scale * f64::from(held);
        exponent += 4;
        scale /= 16.0;
        at += 1;
    }
    if bytes.get(at) == Some(&b'.') && digit(at + 1).is_some() {
        at += 1;
        if exponent < 0 {
            while bytes.get(at) == Some(&b'0') {
                at += 1;
                exponent -= 4;
            }
        }
        while let Some(held) = digit(at) {
            value += scale * f64::from(held);
            at += 1;
            scale /= 16.0;
            if scale == 0.0 {
                while digit(at).is_some() {
                    at += 1;
                }
                break;
            }
        }
    }
    if matches!(bytes.get(at), Some(b'p' | b'P')) {
        at += 1;
        let sign = match bytes.get(at) {
            Some(b'+') => {
                at += 1;
                1
            }
            Some(b'-') => {
                at += 1;
                -1
            }
            _ => 1,
        };
        if !bytes.get(at).is_some_and(u8::is_ascii_digit) {
            return None;
        }
        let mut power: i64 = 0;
        while let Some(held) = bytes.get(at).filter(|held| held.is_ascii_digit()) {
            power = (power * 10 + i64::from(held - b'0')).min(100_000);
            at += 1;
        }
        exponent += power * sign;
    }
    let scaled = value * 2f64.powi(exponent.clamp(-2200, 2200) as i32);
    Some((scaled, at))
}

/// Where a decimal number written from `start` ends, as MRI's strtod reads
/// one: digits, a fraction after a point, and an exponent with digits. None
/// when there are no digits.
fn decimal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let digits_from = |mut at: usize| {
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        at
    };
    let mut at = digits_from(start);
    let mut saw_digit = at > start;
    if bytes.get(at) == Some(&b'.') && bytes.get(at + 1).is_some_and(u8::is_ascii_digit) {
        let fraction_end = digits_from(at + 1);
        saw_digit = true;
        at = fraction_end;
    } else if bytes.get(at) == Some(&b'.') && saw_digit {
        at += 1;
    }
    if !saw_digit {
        return None;
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        let mut exponent = at + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        if bytes.get(exponent).is_some_and(u8::is_ascii_digit) {
            at = digits_from(exponent);
        }
    }
    Some(at)
}

/// Reads a double from the start of a C string the way MRI's own strtod
/// does, storing where reading stopped through `end`. It reads no `inf` or
/// `nan`, and with no number at all it answers 0 and stops at the start.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn ruby_strtod(text: *const c_char, end: *mut *mut c_char) -> f64 {
    // SAFETY: C hands over a NUL-terminated string.
    let bytes = unsafe { CStr::from_ptr(text) }.to_bytes();
    let mut at = 0;
    while matches!(
        bytes.get(at),
        Some(b'\t' | b'\n' | 0x0b | 0x0c | b'\r' | b' ')
    ) {
        at += 1;
    }
    let negative = bytes.get(at) == Some(&b'-');
    if matches!(bytes.get(at), Some(b'+' | b'-')) {
        at += 1;
    }
    let read = if bytes.get(at) == Some(&b'0') && matches!(bytes.get(at + 1), Some(b'x' | b'X')) {
        hex_value(bytes, at + 2)
    } else {
        decimal_end(bytes, at).map(|stop| {
            let written = std::str::from_utf8(&bytes[at..stop]).unwrap_or("0");
            (written.parse::<f64>().unwrap_or(0.0), stop)
        })
    };
    let (value, stop) = read.unwrap_or((0.0, 0));
    if !end.is_null() {
        // SAFETY: C hands over somewhere to put the end, or NULL.
        unsafe { *end = text.add(stop) as *mut c_char };
    }
    if read.is_some() && negative {
        -value
    } else {
        value
    }
}
