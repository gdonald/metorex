// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod integration {
    mod examples {
        mod runner;
        use runner::run_example;

        mod functions;
        mod globals;
        mod hash_methods;
        mod methods;
        mod programs;
        mod regexp;
        mod strings;
        mod symbols_and_ranges;
        mod syntax;
    }
}
