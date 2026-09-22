//! Native class methods for File and Dir.
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
pub(crate) use std::rc::Rc;

mod arguments;
mod attributes;
mod directories;
mod directory_entries;
mod directory_support;
mod dispatch;
mod fnmatch;
mod globbing;
mod links;
mod opening;
mod ownership;
mod paths;
mod reading;

pub(crate) use arguments::*;
pub(crate) use directory_support::*;
pub(crate) use fnmatch::*;
pub(crate) use globbing::*;
