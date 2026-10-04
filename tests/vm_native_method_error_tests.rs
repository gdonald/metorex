// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod native_methods {
        mod array_error_tests;
        mod block_error_tests;
        mod class_error_tests;
        mod exception_error_tests;
        mod float_error_tests;
        mod hash_error_tests;
        mod int_error_tests;
        mod method_error_tests;
        mod operator_error_tests;
        mod range_error_tests;
        mod set_error_tests;
        mod string_error_tests;
    }
}
