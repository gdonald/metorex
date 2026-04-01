// CompiledFunction — a function compiled to bytecode.
//
// Stored in the constant pool and referenced by OP_CONSTANT / OP_CLOSURE.

use crate::bytecode::chunk::Chunk;
use std::fmt;

/// A function that has been compiled to bytecode.
#[derive(Clone)]
pub struct CompiledFunction {
    /// Function name (empty string for anonymous / script-level).
    pub name: String,
    /// Number of required positional parameters.
    pub arity: u8,
    /// The compiled bytecode for this function's body.
    pub chunk: Chunk,
}

impl CompiledFunction {
    pub fn new(name: String, arity: u8) -> Self {
        Self {
            name,
            arity,
            chunk: Chunk::new(),
        }
    }
}

impl fmt::Debug for CompiledFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.name.is_empty() {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", self.name)
        }
    }
}

impl fmt::Display for CompiledFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.name.is_empty() {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", self.name)
        }
    }
}

impl PartialEq for CompiledFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}
