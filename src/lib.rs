// Metorex Programming Language
// A modern, Ruby-inspired language with powerful metaprogramming capabilities

pub mod ast;
pub mod error;
pub mod lexer;
pub mod runtime;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
