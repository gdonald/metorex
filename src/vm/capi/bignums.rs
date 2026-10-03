//! Integers too wide for a fixnum, from C: converting them to and from C
//! numbers and strings, comparing them, and packing them into longs. Also
//! `NUM2DBL`, which reads a C double out of any number.

use super::calls::{call, class_name_of};
use super::handles::{Value, to_object, to_value};
use super::{called_from, interpreter, or_raise, raise};
use crate::object::Object;
use num_bigint::{BigInt, Sign};

fn error(class: &str, message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        class,
        &message,
        called_from(),
    ))
}

fn big_integer(value: Value) -> BigInt {
    or_raise(interpreter().coerce_integer_argument(&to_object(value), called_from()))
}

/// The magnitude of `number` as a C unsigned integer of `bytes` bytes,
/// refusing one that needs more.
fn magnitude_within(number: &BigInt, bytes: u64, type_name: &str) -> u64 {
    if number.bits() > bytes * 8 {
        error(
            "RangeError",
            format!("bignum too big to convert into '{}'", type_name),
        );
    }
    let (_, digits) = number.to_u64_digits();
    digits.first().copied().unwrap_or(0)
}

/// `number` as a signed C integer, wrapping a negative one the way MRI's
/// `rb_big2long` does, or None when it does not fit.
fn signed_within(number: &BigInt, magnitude: u64) -> Option<i64> {
    if number.sign() != Sign::Minus {
        return i64::try_from(magnitude).ok();
    }
    if magnitude <= 1 + (-(i64::MIN + 1)) as u64 {
        return Some((-((magnitude - 1) as i64)).wrapping_sub(1));
    }
    None
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_big2long(value: Value) -> i64 {
    let number = big_integer(value);
    let magnitude = magnitude_within(&number, 8, "long");
    signed_within(&number, magnitude).unwrap_or_else(|| {
        error(
            "RangeError",
            "bignum too big to convert into 'long'".to_string(),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_big2ll(value: Value) -> i64 {
    let number = big_integer(value);
    let magnitude = magnitude_within(&number, 8, "long long");
    signed_within(&number, magnitude).unwrap_or_else(|| {
        error(
            "RangeError",
            "bignum too big to convert into 'long long'".to_string(),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_big2ulong(value: Value) -> u64 {
    let number = big_integer(value);
    let magnitude = magnitude_within(&number, 8, "unsigned long");
    if number.sign() != Sign::Minus {
        return magnitude;
    }
    match signed_within(&number, magnitude) {
        Some(wrapped) => wrapped as u64,
        None => error(
            "RangeError",
            "bignum out of range of unsigned long".to_string(),
        ),
    }
}

/// The Integer as a double, Infinity for one too big, with a warning when
/// `$VERBOSE` is true.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_big2dbl(value: Value) -> f64 {
    let converted = match call(to_object(value), "to_f", Vec::new()) {
        Object::Float(number) => number,
        _ => 0.0,
    };
    if converted.is_infinite() {
        let message = to_value(&Object::string("Integer out of Float range"));
        super::calls::rb_warn_message(message, 1);
    }
    converted
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_dbl2big(number: f64) -> Value {
    if number.is_nan() {
        error("FloatDomainError", "NaN".to_string());
    }
    if number.is_infinite() {
        let named = if number < 0.0 {
            "-Infinity"
        } else {
            "Infinity"
        };
        error("FloatDomainError", named.to_string());
    }
    to_value(&call(Object::Float(number), "to_i", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_big2str(value: Value, base: i32) -> Value {
    to_value(&call(
        to_object(value),
        "to_s",
        vec![Object::Int(i64::from(base))],
    ))
}

/// 1 for an Integer of zero or more, 0 for a negative one.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_big_sign(value: Value) -> i32 {
    (big_integer(value).sign() != Sign::Minus) as i32
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_big_cmp(first: Value, second: Value) -> Value {
    to_value(&call(to_object(first), "<=>", vec![to_object(second)]))
}

/// Packs the Integer into `count` longs, least significant first, in two's
/// complement.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_big_pack(value: Value, longs: *mut u8, count: i64) {
    const LSWORD_FIRST: i32 = 0x02;
    const NATIVE_BYTE_ORDER: i32 = 0x40;
    const TWOS_COMPLEMENT: i32 = 0x80;
    super::integers::rb_integer_pack(
        value,
        longs,
        count.max(0) as usize,
        std::mem::size_of::<i64>(),
        0,
        LSWORD_FIRST | NATIVE_BYTE_ORDER | TWOS_COMPLEMENT,
    );
}

/// How many bytes the magnitude of the Integer takes, storing how many of
/// the top byte's bits are unused through `unused_bits` when it is given.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_absint_size(value: Value, unused_bits: *mut i32) -> usize {
    let bits = big_integer(value).bits();
    let bytes = bits.div_ceil(8);
    if !unused_bits.is_null() {
        // SAFETY: C hands over somewhere to put the count, or NULL.
        unsafe { *unused_bits = (bytes * 8 - bits) as i32 };
    }
    bytes as usize
}

/// Any number as a C double, as MRI's `rb_num2dbl` reads one: Integers,
/// Floats and Rationals directly, nil, booleans, Symbols and Strings
/// refused, and anything else through `to_f`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num2dbl(value: Value) -> f64 {
    let object = to_object(value);
    match &object {
        Object::Float(number) => return *number,
        Object::Int(_) | Object::BigInt(_) => {
            if let Object::Float(number) = call(object, "to_f", Vec::new()) {
                return number;
            }
            return 0.0;
        }
        Object::Nil | Object::Bool(_) | Object::Symbol(_) | Object::String(_) => error(
            "TypeError",
            format!(
                "no implicit conversion of {} into Float",
                interpreter().conversion_name(&object)
            ),
        ),
        _ => {}
    }
    let is_rational = call(
        object.clone(),
        "is_a?",
        vec![super::calls::top_level_module("Rational")],
    )
    .is_truthy();
    if is_rational && let Object::Float(number) = call(object.clone(), "to_f", Vec::new()) {
        return number;
    }
    let class_name = class_name_of(object.clone());
    if !super::calls::answers(&object, "to_f") {
        error(
            "TypeError",
            format!("can't convert {} into Float", class_name),
        );
    }
    match call(object.clone(), "to_f", Vec::new()) {
        Object::Float(number) => number,
        other => error(
            "TypeError",
            format!(
                "can't convert {} to Float ({}#to_f gives {})",
                class_name,
                class_name,
                class_name_of(other)
            ),
        ),
    }
}
