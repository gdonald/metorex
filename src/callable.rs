// Callable trait abstraction for Metorex runtime objects.
// Extracted to its own module so multiple runtime entities (methods, blocks)
// can share a common interface for invocation metadata.

use crate::ast::Statement;

/// Trait implemented by runtime callables (methods, blocks, procs).
pub trait Callable {
    /// Human-readable name used for debugging/call stack.
    fn name(&self) -> &str;

    /// Parameter names in declaration order.
    fn parameters(&self) -> &[String];

    /// Statements that make up the callable body.
    fn body(&self) -> &[Statement];

    /// Number of parameters expected (alias for `parameters().len()`).
    fn arity(&self) -> usize {
        self.parameters().len()
    }
}
