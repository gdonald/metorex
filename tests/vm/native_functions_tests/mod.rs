// Coverage tests for the native functions, grouped by what each one does.
//
// `run` evaluates a program and answers its last value. `run_err` evaluates
// one that is expected to fail and answers the message it failed with.

pub(crate) use metorex::lexer::Lexer;
pub(crate) use metorex::object::Object;
pub(crate) use metorex::parser::Parser;
pub(crate) use metorex::vm::VirtualMachine;
pub(crate) use std::path::Path;

pub(crate) fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

pub(crate) fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

mod address_lookup;
mod assertions;
mod bindings;
mod blocks;
mod command_line_flags;
mod compression;
mod conversions;
mod digests;
mod encodings;
mod kernel_rand;
mod kernel_srand;
mod keyed_digests;
mod language_details;
mod load_paths;
mod loading;
mod method_and_require;
mod numerics;
mod pack_directives;
mod pattern_constants;
mod protected_methods;
mod randomness;
mod reporting;
mod sockets;
mod system_calls;
mod top_level_visibility;
mod tracing;
mod visibility_stubs;
mod writers_and_superclasses;
