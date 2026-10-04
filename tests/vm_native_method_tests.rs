// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod native_methods {
        mod alias_method_spec_additions_tests;
        mod array_coverage_extra_tests;
        mod array_tests;
        mod ast_coverage_tests;
        mod class_coverage_extra_tests;
        mod hash_tests;
        mod int_tests;
        mod module_coverage_extra_tests;
        mod native_methods_mod_coverage_tests;
        mod object_coverage_extra_tests;
        mod range_tests;
        mod set_tests;
        mod string_tests;
    }
}
