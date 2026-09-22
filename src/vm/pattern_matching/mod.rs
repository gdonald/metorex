//! Pattern matching: `case/in`, and the patterns each clause is
//! written with.

pub(crate) use crate::ast::node::ExprMatchCase;
pub(crate) use crate::ast::{Expression, Statement};
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::ControlFlow;
pub(crate) use crate::vm::MATCHEE_KEY;
pub(crate) use crate::vm::core::VirtualMachine;
pub(in crate::vm) use crate::vm::utils::*;
pub(crate) use std::cell::RefCell;
pub(crate) use std::collections::HashMap;
pub(crate) use std::rc::Rc;

mod cases;
mod collections;
mod failures;
mod patterns;
mod reading;

pub(crate) use cases::names_the_same_value;
pub(crate) use failures::*;
