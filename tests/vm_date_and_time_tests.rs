// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod calendar_date_tests;
    mod clock_date_tests;
    mod lazy_walk_tests;
    mod pickings_tests;
    mod range_walk_tests;
    mod stepping_and_products_tests;
    mod time_tests;
}
