//! Utility functions for the Metorex virtual machine.
//!
//! This module provides common helper functions used throughout the VM implementation.

use crate::error::SourceLocation;
use crate::lexer::Position;
use crate::object::Object;

/// Convert a lexer position into a runtime source location.
pub(super) fn position_to_location(position: Position) -> SourceLocation {
    SourceLocation::new(position.line, position.column, position.offset)
}

/// Format an exception object for display.
pub(super) fn format_exception(exception: &Object) -> String {
    match exception {
        Object::Exception(ex) => {
            let exc = ex.borrow();
            format!("{}: {}", exc.exception_type, exc.message)
        }
        _ => format!("{:?}", exception),
    }
}

/// Convert an object into a dictionary key string representation.
pub(super) fn object_to_dict_key(value: &Object) -> Option<String> {
    match value {
        Object::String(s) => Some((**s).clone()),
        Object::Int(i) => Some(i.to_string()),
        Object::Float(f) => Some(f.to_string()),
        Object::Bool(b) => Some(b.to_string()),
        Object::Nil => Some("nil".to_string()),
        _ => None,
    }
}

/// Determine if a value is truthy for conditional statements.
/// In Metorex, only `false` and `nil` are falsy; everything else is truthy.
pub(super) fn is_truthy(value: &Object) -> bool {
    !matches!(value, Object::Bool(false) | Object::Nil)
}
