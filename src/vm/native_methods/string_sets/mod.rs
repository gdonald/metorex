//! The String methods that read a character set the way `tr` writes one,
//! plus the clusters and byte runs a string breaks into.

pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;

mod byte_runs;
mod character_sets;
mod clusters;
mod dispatch;
mod reading_back;

pub(crate) use byte_runs::text_in_encoding;
pub(crate) use character_sets::*;
pub(crate) use clusters::*;
pub(crate) use reading_back::*;
