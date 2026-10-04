// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod integration {
    mod examples {
        mod runner;
        use runner::run_example;

        mod addresses;
        mod c_extensions;
        mod data_objects;
        mod dates;
        mod encodings;
        mod linear_algebra;
        mod object_space;
        mod scanning;
        mod sets;
        mod structs;
        mod tabular_data;
    }
}
