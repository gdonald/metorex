//! Operator evaluation functions for the Metorex VM.
//!
//! This module contains the logic for evaluating unary and binary operators including:
//! - Unary operations (+, -)
//! - Binary operations (+, -, *, /, %)
//! - Comparison operations (<, >, <=, >=, ==, !=)
//!
//! `dispatch` reads the operator and hands the operands to the module
//! that answers for it.

pub(crate) use crate::ast::{BinaryOp, UnaryOp};
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::core::VirtualMachine;
pub(in crate::vm) use crate::vm::errors::{
    binary_type_error, divide_by_zero_error, unary_type_error,
};
pub(in crate::vm) use crate::vm::utils::position_to_location;
pub(crate) use std::rc::Rc;

mod arithmetic;
mod bitwise;
mod coercion;
mod dispatch;
mod equality;
mod modules;
mod ordering;
mod unary;

pub(crate) use coercion::*;
pub(crate) use equality::identity_of;
pub(crate) use modules::*;
