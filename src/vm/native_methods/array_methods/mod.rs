//! Native method implementations for the Array class.
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

mod access;
mod aggregates;
mod basics;
mod coercion;
mod dispatch;
mod drawing;
mod edges;
mod equality;
mod filtering;
mod flattening;
mod folding;
mod in_place;
mod inspecting;
mod iteration;
mod joining;
mod joining_support;
mod ordering;
mod packing;
mod predicates;
mod searching;
mod sorting;
mod subscripts;
mod walking;
mod writing;

pub(crate) use coercion::*;
pub(crate) use equality::identical;
pub(crate) use flattening::*;
pub(crate) use inspecting::*;
pub(crate) use sorting::*;
pub(crate) use walking::*;
