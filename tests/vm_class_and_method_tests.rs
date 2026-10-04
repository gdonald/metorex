// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod block_parameter_binding_tests;
    mod call_argument_tests;
    mod class;
    mod class_execution_coverage_tests;
    mod class_execution_more_coverage_tests;
    mod constant_resolution_tests;
    mod enumerable_dispatch_tests;
    mod lambda_parameter_tests;
    mod method;
    mod method_definition_tests;
    mod method_shape_tests;
    mod object_coverage_tests;
    mod reflection_tests;
    mod super_expr_coverage_tests;
    mod super_resolution_tests;
}
