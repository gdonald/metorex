// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod assignment_target_tests;
    mod defined_forms_tests;
    mod defined_tests;
    mod expression_tests;
    mod identifier_coverage_tests;
    mod identifier_more_coverage_tests;
    mod multiple_assignment_tests;
    mod operators;
    mod operators_coverage_extra_tests;
    mod pattern_matching_tests;
    mod special_global_tests;
}
