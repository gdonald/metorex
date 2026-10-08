//! Virtual machine module for the Metorex interpreter.
//!
//! This module contains the core virtual machine implementation and related support structures.

pub(crate) mod allocation_sites;
mod begin_rescue;
mod block_execution;
mod call_frame;
pub(crate) mod capi;
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
mod ractors;
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
/// The name the C library gives the encoding the locale calls for, which
/// differs between platforms for one and the same locale. A program reads its
/// environment and its file names in it.
pub(crate) fn locale_charmap_name() -> String {
    // Settling the locale changes state the whole process shares, and asking
    // twice from two threads at once is what the C library refuses, so it is
    // asked once and the answer kept.
    static NAMED: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    NAMED
        .get_or_init(|| {
            // SAFETY: both calls read the library's own state and answer a
            // pointer into it, which is read before anything else runs.
            unsafe {
                let empty = std::ffi::CString::new("").expect("a literal with no zero byte");
                libc::setlocale(libc::LC_CTYPE, empty.as_ptr());
                let held = libc::nl_langinfo(libc::CODESET);
                if held.is_null() {
                    return String::new();
                }
                std::ffi::CStr::from_ptr(held).to_string_lossy().to_string()
            }
        })
        .clone()
}

/// The encoding the locale names, spelled the way `Encoding.find` knows it,
/// or `None` when the C library names none.
pub fn locale_encoding_name() -> Option<String> {
    let charmap = locale_charmap_name();
    let named = if charmap == "ANSI_X3.4-1968" {
        "US-ASCII".to_string()
    } else {
        charmap
    };
    (!named.is_empty()).then_some(named)
}

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

/// Where a NoMethodError keeps whether the failed call named no receiver or
/// named `self`, which `#private_call?` answers, and a NameError the local
/// variables in scope where it was raised, which `#local_variables` answers.
pub(crate) const PRIVATE_CALL_KEY: &str = "__private_call__";
pub(crate) const LOCAL_VARIABLES_KEY: &str = "__local_variables__";

/// Where a NoMethodError keeps the arguments the failed call was made with,
/// which `#args` answers. Not an `@` name, so a program's own instance
/// variables cannot collide with it.
pub(crate) const NO_METHOD_ARGS_KEY: &str = "__args__";
/// Where an exception keeps the String it was given as its message, which
/// Marshal writes with its encoding.
pub(crate) const MESSAGE_STRING_KEY: &str = "__message_string__";
