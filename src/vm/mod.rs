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
pub(crate) mod coverage;
pub(crate) mod errors;
mod eval;
mod exceptions;
mod expression;
pub(crate) mod fibers;
mod format;
mod global_registry;
mod heap;
pub(crate) mod init;

/// The type the operating system names a resource limit with, which is a
/// plain int on the BSDs and a typed enum on Linux.
#[cfg(target_os = "linux")]
pub(crate) type RlimitResource = libc::__rlimit_resource_t;
#[cfg(not(target_os = "linux"))]
pub(crate) type RlimitResource = libc::c_int;

mod loading;
mod method_execution;
pub(crate) mod method_invocation;
mod method_lookup;
mod native_functions;
pub(crate) mod native_methods;
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

/// What names a file of the core library rather than one of the program's.
/// A backtrace entry sitting in one of these stands for the place that
/// reached it, which is where Ruby names the program's own code.
pub(crate) const INTERNAL_FILE_PREFIX: &str = "<internal:";

/// What names code handed to `eval` with no filename of its own: the place
/// the eval was written, which is not a file and so holds no directory.
pub(crate) const EVAL_FILE_PREFIX: &str = "(eval at ";

/// Where a KeyError keeps the lookup that missed. Not an `@` name, so a
/// program's own instance variables cannot collide with it.
pub(crate) const KEY_ERROR_KEY: &str = "__key__";
/// The hash a `NoMatchingPatternKeyError` found no key in, which the error
/// answers with `#matchee`.
pub(crate) const MATCHEE_KEY: &str = "__matchee__";

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
