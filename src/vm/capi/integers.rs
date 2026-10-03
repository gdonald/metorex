//! `rb_integer_pack`, which writes an Integer into a buffer of words in the
//! order and width C asks for, and `rb_int_positive_pow`.

use super::calls::call;
use super::handles::{Value, to_object, to_value};
use super::{interpreter, or_raise, raise};
use crate::object::Object;
use crate::vm::native_methods::from_big;
use num_bigint::{BigInt, Sign};

const MSWORD_FIRST: i32 = 0x01;
const LSWORD_FIRST: i32 = 0x02;
const MSBYTE_FIRST: i32 = 0x10;
const LSBYTE_FIRST: i32 = 0x20;
const NATIVE_BYTE_ORDER: i32 = 0x40;
const TWOS_COMPLEMENT: i32 = 0x80;
const FORCE_GENERIC_IMPLEMENTATION: i32 = 0x400;
const WORD_ORDER_MASK: i32 = MSWORD_FIRST | LSWORD_FIRST;
const BYTE_ORDER_MASK: i32 = MSBYTE_FIRST | LSBYTE_FIRST | NATIVE_BYTE_ORDER;
const SUPPORTED_FLAGS: i32 =
    WORD_ORDER_MASK | BYTE_ORDER_MASK | TWOS_COMPLEMENT | FORCE_GENERIC_IMPLEMENTATION;

fn argument_error(message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        "ArgumentError",
        &message,
        super::called_from(),
    ))
}

/// Refuses a layout `rb_integer_pack` cannot write, as MRI's
/// `validate_integer_pack_format` does.
fn check_layout(word_count: usize, word_size: usize, nails: usize, flags: i32) {
    if flags & !SUPPORTED_FLAGS != 0 {
        argument_error("unsupported flags specified".to_string());
    }
    let word_order = flags & WORD_ORDER_MASK;
    if word_order == 0 && word_count > 1 {
        argument_error("word order not specified".to_string());
    }
    if word_order == WORD_ORDER_MASK {
        argument_error("unexpected word order".to_string());
    }
    match flags & BYTE_ORDER_MASK {
        0 => argument_error("byte order not specified".to_string()),
        MSBYTE_FIRST | LSBYTE_FIRST | NATIVE_BYTE_ORDER => {}
        _ => argument_error("unexpected byte order".to_string()),
    }
    if word_size == 0 {
        argument_error(format!("invalid wordsize: {}", word_size));
    }
    if word_size > isize::MAX as usize {
        argument_error(format!("too big wordsize: {}", word_size));
    }
    if word_size <= nails / 8 {
        argument_error(format!("too big nails: {}", nails));
    }
    if usize::MAX / word_size < word_count {
        argument_error(format!(
            "too big numwords * wordsize: {} * {}",
            word_count, word_size
        ));
    }
}

/// The Integer `rb_to_int` makes of a VALUE: an Integer itself, a Float cut
/// toward zero, or what `to_int` answers.
fn integer_from(value: Value) -> BigInt {
    match to_object(value) {
        Object::Float(number) if number.is_finite() => BigInt::from(number.trunc() as i128),
        other => or_raise(interpreter().coerce_integer_argument(&other, super::called_from())),
    }
}

/// Whether the bytes of a word are written most significant first.
fn most_significant_byte_first(flags: i32) -> bool {
    match flags & BYTE_ORDER_MASK {
        MSBYTE_FIRST => true,
        LSBYTE_FIRST => false,
        _ => cfg!(target_endian = "big"),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_integer_pack(
    value: Value,
    words: *mut u8,
    word_count: usize,
    word_size: usize,
    nails: usize,
    flags: i32,
) -> i32 {
    let number = integer_from(value);
    check_layout(word_count, word_size, nails, flags);
    let byte_count = word_count * word_size;
    // SAFETY: C hands over a buffer of `word_count` words of `word_size`
    // bytes each.
    let buffer = unsafe { std::slice::from_raw_parts_mut(words, byte_count) };
    if number.sign() == Sign::NoSign {
        buffer.fill(0);
        return 0;
    }
    let bits_per_word = word_size * 8 - nails;
    let limit = BigInt::from(1) << (word_count * bits_per_word);
    let magnitude = BigInt::from_biguint(Sign::Plus, number.magnitude().clone());
    let negative = number.sign() == Sign::Minus;
    let twos_complement = flags & TWOS_COMPLEMENT != 0;
    let overflow = if negative && twos_complement {
        magnitude > limit
    } else {
        magnitude >= limit
    };
    let mut field = &magnitude % &limit;
    if negative && twos_complement && field.sign() != Sign::NoSign {
        field = &limit - field;
    }
    let word_mask = (BigInt::from(1) << bits_per_word) - 1;
    let byte_first = most_significant_byte_first(flags);
    for word_index in 0..word_count {
        let word = (&field >> (word_index * bits_per_word)) & &word_mask;
        let mut bytes = word.to_bytes_le().1;
        bytes.resize(word_size, 0);
        if byte_first {
            bytes.reverse();
        }
        let slot = if flags & MSWORD_FIRST != 0 {
            word_count - 1 - word_index
        } else {
            word_index
        };
        buffer[slot * word_size..(slot + 1) * word_size].copy_from_slice(&bytes);
    }
    let sign = if negative { -1 } else { 1 };
    if overflow { sign * 2 } else { sign }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_int_positive_pow(base: i64, exponent: u64) -> Value {
    let exponent = from_big(BigInt::from(exponent));
    to_value(&call(Object::Int(base), "**", vec![exponent]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_int2inum(number: isize) -> Value {
    to_value(&Object::Int(number as i64))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_uint2inum(number: usize) -> Value {
    to_value(&from_big(BigInt::from(number)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ll2inum(number: i64) -> Value {
    to_value(&Object::Int(number))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ull2inum(number: u64) -> Value {
    to_value(&from_big(BigInt::from(number)))
}
