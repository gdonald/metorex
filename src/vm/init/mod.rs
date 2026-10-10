//! VM initialization functions.
//!
//! Each module below registers one part of what a fresh VM starts with.

pub(crate) use crate::builtin_classes::{self, BuiltinClasses};
pub(crate) use crate::class::Class;
pub(crate) use crate::environment::Environment;
pub(crate) use crate::object::{Binding, Object};
pub(in crate::vm) use crate::vm::GlobalRegistry;
pub(crate) use indexmap::IndexMap;
pub(crate) use std::cell::RefCell;
pub(crate) use std::collections::HashMap;
pub(crate) use std::rc::Rc;

mod classes;
mod encoding_table;
mod encodings;
mod errno;
mod exceptions;
mod globals;
mod modules;

pub(crate) use classes::*;
pub(crate) use encoding_table::*;
pub(crate) use encodings::*;
pub(crate) use errno::*;
pub(crate) use exceptions::*;
pub(crate) use globals::*;
pub(crate) use modules::*;
