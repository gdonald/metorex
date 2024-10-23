// Object module - Runtime object representation for Metorex
// This module defines the core Object type that represents all runtime values

// Declare submodules
mod block;
mod constructors;
mod display;
mod exception;
mod hash;
mod instance;
mod method;
mod operations;
mod types;

// Re-export core types and traits
pub use block::BlockStatement;
pub use exception::{Exception, SourceLocation};
pub use hash::ObjectHash;
pub use instance::Instance;
pub use method::Method;
pub use types::Object;

// Re-export from callable and class modules
pub use crate::callable::Callable;
pub use crate::class::Class;
