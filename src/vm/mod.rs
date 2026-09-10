//! Virtual machine module for the Metorex interpreter.
//!
//! This module contains the core virtual machine implementation and related support structures.

mod begin_rescue;
mod block_execution;
mod call_frame;
mod class_execution;
mod control_flow;
mod control_structures;
pub(crate) mod core;
pub(crate) mod errors;
mod eval;
mod exceptions;
mod expression;
mod global_registry;
mod heap;
mod init;
mod loading;
mod method_execution;
pub(crate) mod method_invocation;
mod method_lookup;
mod native_functions;
mod native_methods;
mod operators;
pub(super) mod param_binding;
mod pattern_matching;
mod prelude;
mod program;
pub mod signals;
pub(crate) mod stdlib;
mod warn;
pub(crate) use native_methods::{REFINEMENT_KEY_PREFIX, REFINEMENT_LABEL_KEY};
pub(crate) mod statement;
pub(crate) mod system_call_error;
mod tracepoint;
pub(crate) mod utils;

pub use call_frame::{CallFrame, FrameKind};
pub use core::VirtualMachine;
pub use global_registry::GlobalRegistry;
pub use heap::Heap;

pub(crate) use control_flow::ControlFlow;

/// Where a KeyError keeps the lookup that missed. Not an `@` name, so a
/// program's own instance variables cannot collide with it.
pub(crate) const KEY_ERROR_KEY: &str = "__key__";

/// Where a LoadError keeps the feature that could not be loaded, and a
/// SyntaxError the file it was raised for. Not an `@` name, so a program's own
/// instance variables cannot collide with it.
pub(crate) const EXCEPTION_PATH_KEY: &str = "__path__";

/// Where an UncaughtThrowError keeps the tag `throw` was called with and the
/// value it carried. Not `@` names, so a program's own instance variables
/// cannot collide with them.
pub(crate) const THROW_TAG_KEY: &str = "__throw_tag__";
pub(crate) const THROW_VALUE_KEY: &str = "__throw_value__";

/// Where a NameError keeps the name it was handed, when that name has to come
/// back as the very object the caller passed rather than as a Symbol. Not an
/// `@` name, so a program's own instance variables cannot collide with it.
pub(crate) const NAME_ERROR_NAME_KEY: &str = "__name_value__";

/// Where a NoMethodError keeps the arguments the failed call was made with,
/// which `#args` answers. Not an `@` name, so a program's own instance
/// variables cannot collide with it.
pub(crate) const NO_METHOD_ARGS_KEY: &str = "__args__";
