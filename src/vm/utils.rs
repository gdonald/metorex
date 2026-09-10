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
            // Ruby reports a RuntimeError with an empty message, which a bare
            // `raise` produces, as "unhandled exception".
            if exc.message.is_empty() && exc.exception_type == "RuntimeError" {
                return format!("{}: unhandled exception", exc.exception_type);
            }
            format!("{}: {}", exc.exception_type, exc.message)
        }
        _ => format!("{:?}", exception),
    }
}

/// Convert an object into a dictionary key string representation.
pub(super) fn object_to_dict_key(value: &Object) -> Option<String> {
    match value {
        Object::String(s) => Some(s.as_str().to_string()),
        Object::Symbol(s) => Some(format!(":{}", s)),
        Object::Int(i) => Some(i.to_string()),
        // A Float key keeps its fraction, so 4.0 and 4 are different keys the
        // way Ruby's `eql?` makes them.
        Object::Float(f) if f.fract() == 0.0 && f.is_finite() => Some(format!("{:.1}", f)),
        Object::Float(f) => Some(f.to_string()),
        Object::Bool(b) => Some(b.to_string()),
        Object::Nil => Some("nil".to_string()),
        other => Some(format!("{}", other)),
    }
}

/// Whether a string reads back as a number, a boolean, or nil, which is what
/// makes it ambiguous as a hash key.
fn reads_as_another_kind(text: &str) -> bool {
    text == "nil"
        || text == "true"
        || text == "false"
        || text.parse::<i64>().is_ok()
        || text.parse::<f64>().is_ok()
}

/// Check if a key value is primitive (reconstructible from string alone).
pub(super) fn is_primitive_key(value: &Object) -> bool {
    // A String that reads back as some other kind is not reconstructible from
    // its text, so `{"1" => x}` keeps the key object beside the entry.
    if let Object::String(text) = value {
        return !reads_as_another_kind(&text.as_str());
    }
    matches!(
        value,
        Object::Symbol(_) | Object::Int(_) | Object::Float(_) | Object::Bool(_) | Object::Nil
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
        return Object::symbol(sym.to_string());
    }
    if let Ok(n) = key_str.parse::<i64>() {
        return Object::Int(n);
    }
    if let Ok(f) = key_str.parse::<f64>() {
        return Object::Float(f);
    }
    Object::string(key_str.to_string())
}

/// Determine if a value is truthy for conditional statements.
/// In Metorex, only `false` and `nil` are falsy; everything else is truthy.
pub(super) fn is_truthy(value: &Object) -> bool {
    !matches!(value, Object::Bool(false) | Object::Nil)
}
