//! Reading a C integer out of a VALUE: `NUM2LONG`, `NUM2INT`, `FIX2INT` and
//! their unsigned forms, which take any Integer, a Float, or an object that
//! answers `to_int`, and refuse one that does not fit the C type.

use super::handles::{Value, to_object};
use super::{interpreter, or_raise, raise};
use crate::object::Object;
use num_bigint::BigInt;

fn range_error(message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        "RangeError",
        &message,
        super::called_from(),
    ))
}

/// The whole number a VALUE stands for, a Float cut toward zero.
fn integer_from(value: Value) -> BigInt {
    match to_object(value) {
        Object::Nil => raise(crate::vm::errors::simple_exception(
            "TypeError",
            "no implicit conversion from nil to integer",
            super::called_from(),
        )),
        Object::Float(number) if number.is_finite() => BigInt::from(number.trunc() as i128),
        Object::Float(number) => range_error(format!("float {} out of range of integer", number)),
        other => or_raise(interpreter().coerce_integer_argument(&other, super::called_from())),
    }
}

/// The number as a C `long`.
pub(super) fn long_from(value: Value) -> i64 {
    let number = integer_from(value);
    i64::try_from(&number)
        .unwrap_or_else(|_| range_error("bignum too big to convert into 'long'".to_string()))
}

/// The number as a C `unsigned long`, where a negative one that fits a
/// `long` wraps around as C's conversion does.
fn unsigned_long_from(value: Value) -> u64 {
    let number = integer_from(value);
    match (u64::try_from(&number), i64::try_from(&number)) {
        (Ok(unsigned), _) => unsigned,
        (_, Ok(signed)) => signed as u64,
        _ => range_error("bignum out of range of unsigned long".to_string()),
    }
}

/// The number as a C `int`, held in a `long` as MRI's functions answer it.
fn int_from(value: Value) -> i64 {
    let number = long_from(value);
    if number > i64::from(i32::MAX) {
        range_error(format!("integer {} too big to convert to 'int'", number));
    }
    if number < i64::from(i32::MIN) {
        range_error(format!("integer {} too small to convert to 'int'", number));
    }
    number
}

/// The number as a C `unsigned int`, held in an `unsigned long`. A negative
/// one that fits an `int` wraps around.
fn unsigned_int_from(value: Value) -> u64 {
    let number = unsigned_long_from(value);
    let signed = number as i64;
    if signed < 0 {
        if signed < i64::from(i32::MIN) {
            range_error(format!(
                "integer {} too small to convert to 'unsigned int'",
                signed
            ));
        }
        return number;
    }
    if number > u64::from(u32::MAX) {
        range_error(format!(
            "integer {} too big to convert to 'unsigned int'",
            number
        ));
    }
    number
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num2long(value: Value) -> i64 {
    long_from(value)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num2ulong(value: Value) -> u64 {
    unsigned_long_from(value)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num2int(value: Value) -> i64 {
    int_from(value)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fix2int(value: Value) -> i64 {
    int_from(value)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num2uint(value: Value) -> u64 {
    unsigned_int_from(value)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_fix2uint(value: Value) -> u64 {
    unsigned_int_from(value)
}
