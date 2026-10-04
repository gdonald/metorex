// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod integration {
    mod examples {
        mod runner;
        use runner::run_example;

        mod advanced;
        mod algorithms;
        mod array_methods;
        mod basics;
        mod callables;
        mod control_flow;
        mod data_structures;
        mod dsl;
        mod enumerable;
        mod eval;
    }
}
