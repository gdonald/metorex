//! Error construction functions for the Metorex virtual machine.
//!
//! This module provides helper functions for constructing various runtime, type,
//! and internal errors that can occur during VM execution.

use super::utils::position_to_location;
use crate::ast::{BinaryOp, Expression, Statement, UnaryOp};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

// ============================================================================
// Control Flow Errors
// ============================================================================

/// Produce a runtime error for unsupported control-flow usage (e.g., break outside loop).
pub(super) fn loop_control_error(keyword: &str, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!("{keyword} cannot be used outside of a loop"),
        position_to_location(position),
    )
}

// ============================================================================
// Variable and Assignment Errors
// ============================================================================

/// Produce a runtime error when attempting to assign to an invalid target.
pub(super) fn invalid_assignment_target_error(target: &Expression) -> MetorexError {
    MetorexError::runtime_error(
        "Invalid assignment target",
        position_to_location(target.position()),
    )
}

/// Produce a runtime error for referencing an undefined variable.
pub(super) fn undefined_variable_error(name: &str, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!("Undefined variable '{name}'"),
        position_to_location(position),
    )
}

/// Produce a runtime error when accessing `self` outside of a method context.
pub(super) fn undefined_self_error(position: Position) -> MetorexError {
    MetorexError::runtime_error(
        "Undefined self in current context",
        position_to_location(position),
    )
}

// ============================================================================
// Method and Callable Errors
// ============================================================================

/// Produce a runtime error when invoking an undefined method on a receiver.
pub(super) fn undefined_method_error(
    method: &str,
    receiver: &Object,
    position: Position,
) -> MetorexError {
    MetorexError::runtime_error(
        format!(
            "Undefined method '{}' for type '{}'",
            method,
            receiver.type_name()
        ),
        position_to_location(position),
    )
}

/// Produce a runtime error when a method receives the wrong number of arguments.
pub(super) fn method_argument_error(
    method: &str,
    expected: usize,
    found: usize,
    position: Position,
) -> MetorexError {
    MetorexError::runtime_error(
        format!(
            "Method '{}' expected {} argument(s) but received {}",
            method, expected, found
        ),
        position_to_location(position),
    )
}

/// Produce a type error for invalid method argument type.
pub(super) fn method_argument_type_error(
    method: &str,
    expected: &str,
    found: &Object,
    position: Position,
) -> MetorexError {
    MetorexError::type_error(
        format!(
            "Method '{}' expected argument of type '{}' but found '{}'",
            method,
            expected,
            found.type_name()
        ),
        position_to_location(position),
    )
}

/// Produce a runtime error when attempting to call a non-callable object.
pub(super) fn not_callable_error(value: &Object, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!("Object of type '{}' is not callable", value.type_name()),
        position_to_location(position),
    )
}

/// Produce a runtime error when a callable receives the wrong number of arguments.
pub(super) fn callable_argument_error(
    callable_name: &str,
    expected: usize,
    found: usize,
    position: Position,
) -> MetorexError {
    MetorexError::runtime_error(
        format!(
            "Callable '{}' expected {} argument(s) but received {}",
            callable_name, expected, found
        ),
        position_to_location(position),
    )
}

// ============================================================================
// Operator Errors
// ============================================================================

/// Produce a type error for unary operations.
pub(super) fn unary_type_error(op: &UnaryOp, value: &Object, position: Position) -> MetorexError {
    MetorexError::type_error(
        format!(
            "Cannot apply unary operator '{:?}' to type '{}'",
            op,
            value.type_name()
        ),
        position_to_location(position),
    )
}

/// Produce a type error for binary operations.
pub(super) fn binary_type_error(
    op: BinaryOp,
    left: &Object,
    right: &Object,
    position: Position,
) -> MetorexError {
    MetorexError::type_error(
        format!(
            "Cannot apply operator '{:?}' to types '{}' and '{}'",
            op,
            left.type_name(),
            right.type_name()
        ),
        position_to_location(position),
    )
}

/// Produce a divide-by-zero runtime error.
pub(super) fn divide_by_zero_error(position: Position) -> MetorexError {
    MetorexError::runtime_error("Division by zero", position_to_location(position))
}

// ============================================================================
// Indexing and Collection Errors
// ============================================================================

/// Produce an index out of bounds runtime error.
pub(super) fn index_out_of_bounds_error(
    index: i64,
    length: usize,
    position: Position,
) -> MetorexError {
    MetorexError::runtime_error(
        format!(
            "Index {} is out of bounds for array of length {}",
            index, length
        ),
        position_to_location(position),
    )
}

/// Produce a runtime error when a dictionary key is missing.
pub(super) fn undefined_dictionary_key_error(key: &str, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!("Key '{}' not found in dictionary", key),
        position_to_location(position),
    )
}

// ============================================================================
// Internal Errors
// ============================================================================

/// Produce an internal error for statements that are not yet implemented.
pub(super) fn unimplemented_statement_error(statement: &Statement) -> MetorexError {
    MetorexError::internal_error(format!(
        "Statement execution not implemented for {:?}",
        statement
    ))
}
