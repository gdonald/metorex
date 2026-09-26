//! The methods a module answers natively, as against those the prelude
//! writes in Ruby.

pub(crate) use crate::class::Class;
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(crate) use crate::vm::native_methods::is_valid_constant_name;
pub(in crate::vm) use crate::vm::utils::position_to_location;
pub(crate) use std::rc::Rc;

mod clocks;
mod collecting;
mod dispatch;
mod processes;
mod reachable;
mod refinements;

pub(crate) use clocks::*;
pub(crate) use processes::PROCESS_NATIVE_METHODS;
pub(crate) use reachable::*;
pub(crate) use refinements::*;
