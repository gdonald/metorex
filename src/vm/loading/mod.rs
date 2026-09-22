//! Loading files, and the globals a command line flag sets.

pub(crate) use crate::error::{MetorexError, SourceLocation};
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::core::VirtualMachine;
pub(in crate::vm) use crate::vm::errors::keep_exception;
pub(crate) use std::path::PathBuf;
pub(crate) use std::rc::Rc;

mod autoload;
mod command_line;
mod files;
mod running;

pub(crate) use autoload::*;
pub(crate) use command_line::*;
