//! Reading a pattern's source into the pieces it names.

pub(crate) use crate::regexp::node::{Class, ClassItem, Greed, Named, Node, Target};
pub(crate) use shape::Read;

mod classes;
mod escapes;
mod pieces;
mod shape;

pub(crate) use classes::{condition_target, folded_letters, name_target};
pub use shape::*;
