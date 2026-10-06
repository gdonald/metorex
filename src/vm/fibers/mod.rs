//! Fibers: a block that runs on a stack of its own and can suspend
//! part-way through, and the threads whose bodies run on one.

pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::{BlockStatement, Object};
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use corosensei::stack::DefaultStack;
pub(crate) use corosensei::{Coroutine, CoroutineResult, Yielder};

mod running;
mod scheduling;
mod state;
mod stopping;
mod threads;
mod waiting;

pub(crate) use state::*;
pub(crate) use stopping::*;
pub(crate) use threads::WOKEN;
pub(crate) use waiting::{DYING_OF, WAITING_FOREVER};
