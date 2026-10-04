// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod integration {
    mod examples {
        mod runner;
        use runner::run_example;

        mod command_options;
        mod fibers;
        mod file_loading;
        mod filesystem;
        mod paths;
        mod ractors;
        mod signals;
        mod threads;
    }
}
