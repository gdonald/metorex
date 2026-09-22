// What `main` is built from: the command line, the line-reading loop,
// and the setup each flag asks for.

pub(crate) use clap::Parser as ClapParser;
pub(crate) use metorex::lexer::Lexer;
pub(crate) use metorex::parser::Parser;
pub(crate) use metorex::repl::Repl;
pub(crate) use metorex::test_discovery;
pub(crate) use metorex::vm::VirtualMachine;
pub(crate) use std::fs;
pub(crate) use std::path::Path;
pub(crate) use std::process;

mod options;
mod records;
mod startup;

pub(crate) use options::*;
pub(crate) use records::*;
pub(crate) use startup::*;
