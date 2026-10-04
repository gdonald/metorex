// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod ast;
mod bytecode;
mod compiler;
mod environment;
mod lexer;
mod parser;
