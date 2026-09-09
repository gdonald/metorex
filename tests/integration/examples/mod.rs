mod addresses;
mod advanced;
mod algorithms;
mod array_methods;
mod basics;
mod builtins;
mod callables;
mod command_options;
mod control_flow;
mod data_objects;
mod data_structures;
mod dates;
mod dsl;
mod enumerable;
mod errors;
mod file_loading;
mod filesystem;
mod functions;
mod globals;
mod hash_methods;
mod introspection;
mod linear_algebra;
mod metaprogramming;
mod methods;
mod oop;
mod paths;
mod programs;
mod regexp;
mod scanning;
mod sets;
mod signals;
mod stdlib;
mod stdlib_libraries;
mod structs;
mod symbols_and_ranges;
mod syntax;
mod tabular_data;

use crate::common::EXAMPLES_DIR;
use std::process::Command;

fn run_example(path: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/{}", EXAMPLES_DIR, path);
    let mut cmd = Command::new(binary);
    cmd.current_dir(manifest_dir).arg(&full_path);

    let output = cmd.output().expect("failed to execute example");
    assert!(
        output.status.success(),
        "example {} exited with status {:?}",
        path,
        output.status
    );

    String::from_utf8(output.stdout).expect("stdout was not utf8")
}
