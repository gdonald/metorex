// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod ast_manipulation_tests;
    mod binding_eval_tests;
    mod block_execution_coverage_tests;
    mod break_target_tests;
    mod control_tests;
    mod core_tests;
    mod execution;
    mod raising_with_a_cause_tests;
    mod redo_target_tests;
    mod rescue_module_tests;
    mod running_a_string_against_a_receiver_tests;
    mod running_code_from_a_string_tests;
    mod statement_tests;
    mod unreachable_paths_tests;
    // The vm test tree mirrors src/vm, which nests a vm module of its own.
    #[allow(clippy::module_inception)]
    mod vm;
}
