// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod comma_separated_tests;
    mod data_object_tests;
    mod matrix_shapes_tests;
    mod matrix_tests;
    mod open_struct_tests;
    mod regexp_encoding_tests;
    mod regexp_match_tests;
    mod string_io_tests;
    mod string_scanner_tests;
}
