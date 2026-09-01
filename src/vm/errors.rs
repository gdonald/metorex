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

/// Produce a `LocalJumpError` for a method that requires a block but was called
/// without one (e.g. `class_exec` / `module_exec`).
pub(super) fn local_jump_error(method_name: &str, position: Position) -> MetorexError {
    let msg = format!("no block given (yield) for {method_name}");
    let exc = Object::exception("LocalJumpError", msg.clone());
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message: msg,
    }
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

/// Produce an error for referencing an undefined variable. Modeled as a
/// Ruby-level NameError so `rescue NameError` (and the mspec
/// `raise_error(NameError)` matcher) catch it the way they would in MRI.
pub(super) fn undefined_variable_error(
    name: &str,
    receiver: Option<Object>,
    position: Position,
) -> MetorexError {
    let msg = format!("Undefined variable '{name}'");
    let exc = crate::object::Object::exception("NameError", msg.clone());
    if let Object::Exception(details) = &exc {
        let mut details = details.borrow_mut();
        details.name = Some(name.to_string());
        details.receiver = receiver.map(Box::new);
    }
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message: msg,
    }
}

/// A NameError whose `#name` answers the very object the caller handed over,
/// which is what `instance_variable_get` and `class_variable_get` report.
pub(super) fn invalid_name_error(
    message: String,
    name: &Object,
    receiver: &Object,
    position: Position,
) -> MetorexError {
    let exception = Object::exception("NameError", message.clone());
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details
            .instance_vars
            .insert(crate::vm::NAME_ERROR_NAME_KEY.to_string(), name.clone());
        details.receiver = Some(Box::new(receiver.clone()));
    }
    MetorexError::UncaughtException {
        exception,
        location: position_to_location(position),
        message,
    }
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
    args: &[Object],
    position: Position,
) -> MetorexError {
    let class_info = if let Object::Instance(inst) = receiver {
        format!("Instance({})", inst.borrow().class.name())
    } else {
        receiver.type_name().to_string()
    };
    let message = format!("Undefined method '{}' for type '{}'", method, class_info);
    let exc = no_method_error(&message, method, receiver, args);
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message,
    }
}

/// A NoMethodError carrying the name it was raised for and the object it was
/// called on, which `NameError#name` and `#receiver` report.
pub(super) fn no_method_error(
    message: &str,
    method: &str,
    receiver: &Object,
    args: &[Object],
) -> Object {
    let exception = Object::exception("NoMethodError", message);
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details.name = Some(method.to_string());
        details.receiver = Some(Box::new(receiver.clone()));
        details.instance_vars.insert(
            crate::vm::NO_METHOD_ARGS_KEY.to_string(),
            Object::array(args.to_vec()),
        );
    }
    exception
}

/// How many arguments a callable accepts, rendered the way Ruby renders it
/// in an arity message: one count, a range, or a minimum with a `+`.
pub(super) enum Arity {
    /// Exactly this many.
    Exact(usize),
    /// From the first count through the second.
    Range(usize, usize),
    /// This many or more, which is what a splat parameter accepts.
    AtLeast(usize),
}

impl std::fmt::Display for Arity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arity::Exact(count) => write!(formatter, "{}", count),
            Arity::Range(low, high) => write!(formatter, "{}..{}", low, high),
            Arity::AtLeast(low) => write!(formatter, "{}+", low),
        }
    }
}

/// Produce an `ArgumentError` when a method receives the wrong number of
/// arguments (Ruby raises ArgumentError, not RuntimeError, for arity
/// mismatches). The method name is left out, as Ruby leaves it out.
pub(super) fn method_argument_error(
    _method: &str,
    expected: usize,
    found: usize,
    position: Position,
) -> MetorexError {
    argument_count_error(Arity::Exact(expected), found, position)
}

/// The same error for a method that accepts a span of counts rather than one.
pub(super) fn argument_count_error(
    expected: Arity,
    found: usize,
    position: Position,
) -> MetorexError {
    let msg = format!(
        "wrong number of arguments (given {}, expected {})",
        found, expected
    );
    let exc = Object::exception("ArgumentError", msg.clone());
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message: msg,
    }
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
/// A LoadError that remembers the feature it could not load, which `#path`
/// answers.
pub(super) fn load_error(message: String, feature: &str) -> Object {
    let exception = Object::exception("LoadError", message);
    if let Object::Exception(details) = &exception {
        details.borrow_mut().instance_vars.insert(
            crate::vm::LOAD_ERROR_PATH_KEY.to_string(),
            Object::string(feature),
        );
    }
    exception
}

/// Raise `class_name` with a fixed message.
pub(super) fn simple_exception(
    class_name: &str,
    message: &str,
    position: Position,
) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception(class_name, message.to_string()),
        location: position_to_location(position),
        message: message.to_string(),
    }
}

pub(super) fn divide_by_zero_error(position: Position) -> MetorexError {
    let message = "divided by 0".to_string();
    MetorexError::UncaughtException {
        exception: Object::exception("ZeroDivisionError", message.clone()),
        location: position_to_location(position),
        message,
    }
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
