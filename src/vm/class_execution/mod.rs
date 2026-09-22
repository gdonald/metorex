// Class and function definition execution for the Metorex VM.
//
// Each module below holds one part of that work: opening a class or
// module, running its body, and the mixins, constants and visibility a
// body sets up.

pub(crate) use crate::ast::{Expression, Statement};
pub(crate) use crate::class::Class;
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::{Method, Object};
pub(crate) use crate::vm::ControlFlow;
pub(crate) use crate::vm::core::VirtualMachine;
pub(in crate::vm) use crate::vm::errors::method_argument_type_error;
pub(crate) use crate::vm::native_methods::MODULE_FUNCTION_VISIBILITY;
pub(in crate::vm) use crate::vm::utils::*;
pub(crate) use std::rc::Rc;

mod aliasing;
mod body_errors;
mod class_body;
mod class_definition;
mod class_eval;
mod constants;
mod extending;
mod freezing;
mod function_definition;
mod including;
mod method_building;
mod module_body;
mod module_definition;
mod visibility;

pub(crate) use body_errors::*;
pub(crate) use method_building::*;
pub(crate) use visibility::*;
