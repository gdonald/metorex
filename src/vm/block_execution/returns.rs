// What a `return` written in a block means.

use super::*;

/// The same definition written without the `self.` that named the receiver.
/// A block running against a receiver has that receiver's singleton class as
/// its definee, so the method belongs there under its own name.
pub(crate) fn statement_for_its_own_receiver(statement: &Statement) -> Option<Statement> {
    match statement {
        Statement::FunctionDef {
            name,
            parameters,
            body,
            singleton_class: Some(named),
            position,
            end_position,
        } if named == "self" => Some(Statement::FunctionDef {
            name: name.clone(),
            parameters: parameters.clone(),
            body: body.clone(),
            singleton_class: None,
            position: *position,
            end_position: *end_position,
        }),
        Statement::MethodDef {
            name,
            parameters,
            body,
            is_class_method: true,
            position,
            end_position,
        } => Some(Statement::MethodDef {
            name: name.clone(),
            parameters: parameters.clone(),
            body: body.clone(),
            is_class_method: false,
            position: *position,
            end_position: *end_position,
        }),
        _ => None,
    }
}

/// A `return` that left a block without finding the lambda or the method it
/// belongs to, which Ruby reports the same way as one whose method has
/// already returned.
pub(crate) fn escaped_return_error(
    value: Object,
    location: crate::error::SourceLocation,
) -> MetorexError {
    let message = "unexpected return".to_string();
    let exception = Object::exception("LocalJumpError", message.clone());
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details
            .instance_vars
            .insert("@exit_value".to_string(), value);
        details
            .instance_vars
            .insert("@reason".to_string(), Object::symbol("return".to_string()));
    }
    MetorexError::UncaughtException {
        exception,
        location,
        message,
    }
}

/// A `return` from a block whose defining method has already returned. Ruby
/// reports it as a LocalJumpError carrying the value and the reason.
pub(crate) fn orphaned_return_error(value: Object, position: Position) -> MetorexError {
    let message = "unexpected return".to_string();
    let exception = Object::exception("LocalJumpError", message.clone());
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details
            .instance_vars
            .insert("@exit_value".to_string(), value);
        details
            .instance_vars
            .insert("@reason".to_string(), Object::symbol("return".to_string()));
    }
    MetorexError::UncaughtException {
        exception,
        location: position_to_location(position),
        message,
    }
}
