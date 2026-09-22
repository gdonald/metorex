//! Native method implementations for the Hash class.
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
pub(crate) use indexmap::IndexMap;
pub(crate) use std::cell::RefCell;
pub(crate) use std::rc::Rc;

mod comparing;
mod copying;
mod defaults;
mod digging;
mod dispatch;
mod environment;
mod environment_methods;
mod identity;
mod inspecting;
mod iteration;
mod lookup;
mod queries;
mod rebuilding;
mod sentinels;
mod slots;
mod transforming;
mod writing;

pub(crate) use copying::*;
pub(crate) use environment::*;
pub(crate) use inspecting::*;
pub(crate) use sentinels::*;
pub(crate) use slots::*;
