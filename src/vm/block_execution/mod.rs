//! Block execution for the virtual machine.
//!
//! `parameters` binds what a block was written to take, `running` and
//! `receivers` run one, and `returns` reads a `return` written inside.

pub(crate) use crate::ast::{Statement, collect_assigned_locals};
pub(crate) use crate::callable::Callable;
pub(crate) use crate::error::{MetorexError, StackFrame};
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::{BlockStatement, Object};
pub(in crate::vm) use crate::vm::errors::loop_control_error;
pub(in crate::vm) use crate::vm::utils::*;
pub(crate) use crate::vm::{CallFrame, ControlFlow, VirtualMachine};

mod parameters;
mod receivers;
mod returns;
mod running;

pub(crate) use parameters::*;
pub(crate) use returns::*;
