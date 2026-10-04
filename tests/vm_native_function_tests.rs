// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod builtin_classes_tests;
    mod builtin_object_methods_tests;
    mod native_functions_coverage_extra_tests;
    mod native_functions_coverage_tests;
    mod native_functions_tests;
}
