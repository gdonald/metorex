//! Struct: the classes `Struct.new` generates and what they answer.

pub(crate) use crate::class::Class;
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::{Instance, Object};
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(in crate::vm) use crate::vm::utils::position_to_location;
pub(crate) use indexmap::IndexMap;
pub(crate) use std::cell::RefCell;
pub(crate) use std::rc::Rc;

mod arguments;
mod building;
mod instances;

pub(crate) use arguments::*;
