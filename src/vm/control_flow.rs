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
    /// A break statement was encountered.
    Break { position: Position },
    /// A continue statement was encountered.
    Continue { position: Position },
    /// An exception was raised and is propagating.
    Exception {
        exception: Object,
        position: Position,
    },
}
