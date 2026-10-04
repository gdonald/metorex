// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod integration {
    mod cli_tests;
    mod full_pipeline;
    mod lexer_parser;
    mod multi_feature;
    mod parser_vm;
    mod test_runner;
    mod version_test;
}
