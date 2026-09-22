//! Statement execution for the virtual machine.

pub(crate) use crate::ast::{Expression, Statement};
pub(crate) use crate::class::Class;
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::ControlFlow;
pub(crate) use crate::vm::core::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(in crate::vm) use crate::vm::utils::*;
pub(crate) use std::rc::Rc;

mod assignment;
mod running;

pub(crate) use assignment::name_constant_value;
