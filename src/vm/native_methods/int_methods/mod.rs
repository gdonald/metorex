//! Native method implementations for the Integer class.
//!
//! `dispatch` holds the order the groups below are tried in. Each group
//! is a module of its own, and answers `None` for a name that is none of
//! its own.

pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(crate) use std::rc::Rc;

mod big_integers;
mod bit_operations;
mod bit_reading;
mod bits;
mod class_methods;
mod coercion;
mod digits;
mod dispatch;
mod dividing;
mod encodings;
mod roots;
mod rounding;
mod shared_arithmetic;
mod shifts;
mod signs;
mod text;
mod walks;

pub(crate) use big_integers::*;
pub(crate) use dividing::*;
pub(crate) use encodings::*;
pub(crate) use roots::*;
pub(crate) use rounding::*;
