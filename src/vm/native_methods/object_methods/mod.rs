//! Native method implementations for the Object class.
//!
//! `dispatch` holds the order the groups below are tried in. Each group
//! is a module of its own, and answers `None` for a name that is none of
//! its own.

pub(crate) use crate::class::Class;
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(in crate::vm) use crate::vm::utils::position_to_location;

mod basic_object;
mod clamping;
mod comparisons;
mod copying;
mod describing;
mod dispatch;
mod equality;
mod evaluating;
pub(crate) mod hashing;
mod initializing;
mod inspecting;
mod instance_variables;
mod method_lists;
mod method_lookup;
mod method_names;
mod object_identity;
mod sending;
mod singleton_support;
mod singletons;
mod state;
mod symbol_proc;
mod type_checks;

pub(crate) use basic_object::*;
pub(crate) use object_identity::*;
pub(crate) use singleton_support::*;
pub(crate) use symbol_proc::*;
