//! Native (built-in) function implementations for the virtual machine.
//!
//! `dispatch` maps a name to the method that answers it. The modules
//! beside it hold those methods, grouped by what they do.

pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use std::rc::Rc;

mod assertions;
mod backtraces;
mod bindings;
mod blocks;
mod control_flow;
mod conversions;
mod dispatch;
mod evaluation;
mod exiting;
mod generator;
mod input;
mod math;
mod method_objects;
mod output;
mod output_support;
mod processes;
mod randomness;
mod reading;
mod reporting;
mod requiring;
mod shell;
mod support;
mod timing;
mod variables;
mod visibility;

pub(crate) use backtraces::*;
pub(crate) use generator::*;
pub(crate) use math::*;
pub(crate) use output_support::*;
pub(crate) use processes::*;
pub(crate) use support::*;
