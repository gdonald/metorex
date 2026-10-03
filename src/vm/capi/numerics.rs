//! Floats, Rationals and Complex numbers made and read from C.

use super::calls::{call, class_name_of, top_level_module};
use super::handles::{Value, to_object, to_value};
use super::raise;
use crate::object::Object;

/// Calls one of Kernel's conversion functions, `Complex` or `Rational`.
fn kernel_conversion(name: &str, arguments: &[Value]) -> Value {
    let arguments = arguments.iter().map(|held| to_object(*held)).collect();
    to_value(&call(top_level_module("Kernel"), name, arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_float_new(number: f64) -> Value {
    to_value(&Object::Float(number))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_float_value(float: Value) -> f64 {
    match to_object(float) {
        Object::Float(number) => number,
        other => raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!(
                "wrong argument type {} (expected Float)",
                class_name_of(other)
            ),
            super::called_from(),
        )),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_float_type_p(value: Value) -> i32 {
    matches!(to_object(value), Object::Float(_)) as i32
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_Float(value: Value) -> Value {
    kernel_conversion("Float", &[value])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_Complex(real: Value, imaginary: Value) -> Value {
    kernel_conversion("Complex", &[real, imaginary])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_complex_new(real: Value, imaginary: Value) -> Value {
    let arguments = vec![to_object(real), to_object(imaginary)];
    to_value(&call(top_level_module("Complex"), "rect", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_Rational(numerator: Value, denominator: Value) -> Value {
    kernel_conversion("Rational", &[numerator, denominator])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rational_new(numerator: Value, denominator: Value) -> Value {
    kernel_conversion("Rational", &[numerator, denominator])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rational_num(rational: Value) -> Value {
    to_value(&call(to_object(rational), "numerator", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rational_den(rational: Value) -> Value {
    to_value(&call(to_object(rational), "denominator", Vec::new()))
}
