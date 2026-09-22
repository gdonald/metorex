//! Runtime class definition with method table and inheritance.

pub(crate) use crate::object::{Method, Object};
pub(crate) use indexmap::IndexMap;
pub use shape::Class;
pub(crate) use std::cell::RefCell;
pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use std::rc::{Rc, Weak};

mod class_variables;
mod constants;
mod methods;
mod names;
mod shape;
mod visibility;
