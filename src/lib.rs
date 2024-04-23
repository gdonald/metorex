// Metorex Programming Language
// A modern, Ruby-inspired language with powerful metaprogramming capabilities

pub mod ast;
pub mod builtin_classes;
pub mod class;
pub mod environment;
pub mod error;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod resolver;
pub mod runtime;
pub mod scope;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
