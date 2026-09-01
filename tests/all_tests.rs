// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

// Main test integration file that organizes all tests by topic

mod array;
mod ast;
mod blocks;
mod bytecode;
mod class_system;
pub mod common;
mod compiler;
mod control_flow;
mod environment;
mod errors;
mod exceptions;
mod file_loader;
mod integration;
mod lexer;
mod module_tests;
mod parser;
mod repl_tests;
mod require_relative;
mod string_methods_tests;
mod test_discovery;
mod type_system;
mod vm;
