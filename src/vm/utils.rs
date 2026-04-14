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
        Object::Symbol(s) => Some(format!(":{}", s)),
        Object::Int(i) => Some(i.to_string()),
        Object::Float(f) => Some(f.to_string()),
        Object::Bool(b) => Some(b.to_string()),
        Object::Nil => Some("nil".to_string()),
        other => Some(format!("{}", other)),
    }
}

/// Check if a key value is primitive (reconstructible from string alone).
pub(super) fn is_primitive_key(value: &Object) -> bool {
    matches!(
        value,
        Object::String(_)
            | Object::Symbol(_)
            | Object::Int(_)
            | Object::Float(_)
            | Object::Bool(_)
            | Object::Nil
    )
}

/// Reconstruct a primitive key Object from its string representation.
pub(super) fn dict_key_to_object(key_str: &str) -> Object {
    if key_str == "nil" {
        return Object::Nil;
    }
    if key_str == "true" {
        return Object::Bool(true);
    }
    if key_str == "false" {
        return Object::Bool(false);
    }
    if let Some(sym) = key_str.strip_prefix(':') {
        return Object::Symbol(std::rc::Rc::new(sym.to_string()));
    }
    if let Ok(n) = key_str.parse::<i64>() {
        return Object::Int(n);
    }
    if let Ok(f) = key_str.parse::<f64>() {
        return Object::Float(f);
    }
    Object::String(std::rc::Rc::new(key_str.to_string()))
}

/// Determine if a value is truthy for conditional statements.
/// In Metorex, only `false` and `nil` are falsy; everything else is truthy.
pub(super) fn is_truthy(value: &Object) -> bool {
    !matches!(value, Object::Bool(false) | Object::Nil)
}
