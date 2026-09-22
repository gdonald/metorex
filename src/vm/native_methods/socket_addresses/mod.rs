// Network addresses: reading one out of the text a program writes it as, and
// the sockets a program opens over them.
//
// `dispatch` maps a socket action to the method that carries it out.

pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::core::VirtualMachine;
pub(crate) use std::io::{Read as _, Write as _};

mod addresses;
mod datagrams;
mod descriptors;
mod dispatch;
mod listening;
mod open_sockets;
mod options;
mod system_calls;
mod transfer;
mod unix_sockets;

pub(crate) use dispatch::refused;
pub(crate) use open_sockets::*;
pub(crate) use system_calls::*;
