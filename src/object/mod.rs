// Object module - Runtime object representation for Metorex
// This module defines the core Object type that represents all runtime values

// Declare submodules
mod binding;
mod block;
mod compiled_function;
mod constructors;
mod display;
pub mod string_value;
pub(crate) use display::{
    begin_rendering, end_rendering, inspect_symbol, render_guarded, rendering_in_progress,
};
mod exception;
mod hash;
mod instance;
mod method;
mod operations;
mod types;

// Re-export core types and traits
pub use binding::Binding;
pub use block::{
    BlockStatement, DESTRUCTURED_GROUP_PREFIX, KEYWORD_PARAM_PREFIX, TRAILING_COMMA_PARAM,
};
pub use compiled_function::CompiledFunction;
pub use exception::{Exception, SourceLocation};
pub use hash::ObjectHash;
pub use instance::Instance;
pub use method::Method;
pub use types::Object;

// Re-export from callable and class modules
pub use crate::callable::Callable;
pub use crate::class::Class;

pub use string_value::StringValue;
