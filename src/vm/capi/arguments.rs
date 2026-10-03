//! The errors `rb_scan_args` raises when a C method is handed the wrong
//! number of arguments or is given a format it cannot read.

use super::raise;
use crate::vm::errors::{Arity, argument_count_error};

/// What `max` is when a C method takes any number past `min`.
const UNLIMITED_ARGUMENTS: i32 = -1;

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_error_arity(given: i32, min: i32, max: i32) -> ! {
    let expected = match max {
        UNLIMITED_ARGUMENTS => Arity::AtLeast(min as usize),
        _ if max == min => Arity::Exact(min as usize),
        _ => Arity::Range(min as usize, max as usize),
    };
    raise(argument_count_error(
        expected,
        given as usize,
        super::called_from(),
    ))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_scan_args_bad_format(format: *const std::ffi::c_char) -> ! {
    raise(crate::vm::errors::simple_exception(
        "RuntimeError",
        &format!("bad scan arg format: {}", super::exports::text(format)),
        super::called_from(),
    ))
}
