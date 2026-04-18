//! Control flow representation for the Metorex virtual machine.
//!
//! This module defines the `ControlFlow` enum that represents signals produced
//! during statement execution (return, break, continue, exceptions).

use crate::lexer::Position;
use crate::object::Object;

/// Represents control-flow signals produced during statement execution.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ControlFlow {
    /// Normal execution, continue with next statement.
    Next,
    /// A return statement was encountered with an associated value.
    Return { value: Object, position: Position },
    /// A case/match arm produced a value (used so that the last statement of a
    /// method body can pick up the case's value without triggering a true return).
    Value(Object),
    /// A break statement was encountered, optionally with a value (Ruby's
    /// `break <expr>` returns the value from the enclosing loop/method call).
    Break { value: Object, position: Position },
    /// A continue statement was encountered.
    Continue { position: Position },
    /// An exception was raised and is propagating.
    Exception {
        exception: Object,
        position: Position,
    },
}
