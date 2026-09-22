//! Native method implementations for the Complex class.

pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(crate) use crate::vm::native_methods::rational_methods::{
    complex_parts, format_complex, is_zero,
};

mod arithmetic;
mod building;
mod dispatch;
mod parsing;
mod rendering;

pub(crate) use parsing::*;
pub(crate) use rendering::*;
