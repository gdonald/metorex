// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod integration {
    mod examples {
        mod runner;
        use runner::run_example;

        mod builtins;
        mod errors;
        mod metaprogramming;
    }
}
