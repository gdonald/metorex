//! Native method implementations for the Range class.
//!
//! `dispatch` holds the order the groups below are tried in. Each group
//! is a module of its own, and answers `None` for a name that is none of
//! its own.

pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(in crate::vm) use crate::vm::utils::position_to_location;
pub(crate) use std::cell::RefCell;
pub(crate) use std::rc::Rc;

mod bounds;
mod describing;
mod dispatch;
mod mapping;
mod searching;
mod support;
mod walking;

pub(crate) use searching::*;
pub(crate) use support::*;
