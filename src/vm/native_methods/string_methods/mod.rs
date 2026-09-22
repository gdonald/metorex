//! Native method implementations for the String class.
//!
//! `dispatch` holds the order the groups below are tried in. Each group
//! is a module of its own, and answers `None` for a name that is none of
//! its own.

pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(in crate::vm) use crate::vm::utils::position_to_location;
pub(crate) use std::cell::RefCell;
pub(crate) use std::rc::Rc;

mod answer_text;
mod byte_search;
mod bytes;
mod case_mapping;
mod casing;
mod characters;
mod conversions;
mod dispatch;
mod encode;
mod encode_support;
mod encoding_lookup;
mod encoding_names;
mod encoding_tags;
mod escapes;
mod inspection;
mod latin_tables;
mod matching;
mod numbers;
mod padding;
mod searching;
mod slicing;
mod splitting;
mod substitution;
mod substitution_support;
mod succ;
mod trimming;
mod validity;
mod wide;

pub(crate) use answer_text::*;
pub(crate) use byte_search::*;
pub(crate) use casing::*;
pub(crate) use encode_support::*;
pub(crate) use encoding_names::*;
pub(crate) use escapes::*;
pub(crate) use latin_tables::*;
pub(crate) use numbers::*;
pub(crate) use splitting::*;
pub(crate) use succ::*;
pub(crate) use validity::*;
pub(crate) use wide::*;
