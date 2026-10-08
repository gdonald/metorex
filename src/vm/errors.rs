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
    // Ruby tells a constant apart from a name that could be either a local
    // or a method, and names the object the lookup ran against.
    let msg = if name.starts_with(char::is_uppercase) {
        format!("uninitialized constant {name}")
    } else {
        let named = match &receiver {
            Some(Object::Nil) | None => "main".to_string(),
            Some(held) => receiver_wording(held),
        };
        format!("undefined local variable or method '{name}' for {named}")
    };
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
    undefined_method_error_worded(method, receiver, args, receiver_wording(receiver), position)
}

/// The same error, told how to name the receiver. A caller with the machine
/// at hand asks a class that wrote its own `name` what it is called.
pub(super) fn undefined_method_error_worded(
    method: &str,
    receiver: &Object,
    args: &[Object],
    wording: String,
    position: Position,
) -> MetorexError {
    let message = format!("undefined method '{}' for {}", method, wording);
    let exc = no_method_error(&message, method, receiver, args);
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message,
    }
}

/// How Ruby names the receiver in a NoMethodError message: `nil`, `true`,
/// `false`, and the classes and modules by name, everything else as an
/// instance of its class. A class with no name of its own is named by the
/// form it writes itself in.
fn receiver_wording(receiver: &Object) -> String {
    match receiver {
        Object::Nil => "nil".to_string(),
        Object::Bool(true) => "true".to_string(),
        Object::Bool(false) => "false".to_string(),
        Object::Class(class_rc) => format!("class {}", named_or_written(class_rc, receiver)),
        Object::Module(module_rc) => format!("module {}", named_or_written(module_rc, receiver)),
        Object::Instance(instance) => {
            // An object carrying methods of its own is named as itself, since
            // its class is a singleton the program never wrote down.
            if instance.borrow().singleton_class.borrow().is_some()
                || !instance.borrow().singleton_methods.borrow().is_empty()
            {
                return receiver.to_string();
            }
            let class = std::rc::Rc::clone(&instance.borrow().class);
            let written = Object::Class(std::rc::Rc::clone(&class));
            format!("an instance of {}", named_or_written(&class, &written))
        }
        Object::Int(_) | Object::BigInt(_) => "an instance of Integer".to_string(),
        Object::Dict(_) => "an instance of Hash".to_string(),
        Object::Block(_) => "an instance of Proc".to_string(),
        other => format!("an instance of {}", other.type_name()),
    }
}

impl crate::vm::VirtualMachine {
    /// How a NoMethodError names its receiver, asking a class that wrote a
    /// `name` of its own what it is called. A class named the ordinary way
    /// answers through `receiver_wording`.
    /// Record on a NoMethodError for `method_name` whether the call that
    /// raised it named no receiver or named `self`, and the local variables
    /// of the code that made the call. The innermost call of that name the
    /// error passes back through is the one that raised it, so a call further
    /// out leaves what that one recorded.
    #[inline(never)]
    pub(crate) fn note_failed_call(
        &mut self,
        result: &Result<Object, MetorexError>,
        method_name: &str,
        private_call: bool,
        position: Position,
    ) {
        let Err(MetorexError::UncaughtException {
            exception: Object::Exception(details),
            ..
        }) = result
        else {
            return;
        };
        {
            let held = details.borrow();
            if held.name.as_deref() != Some(method_name)
                || held.instance_vars.contains_key(crate::vm::PRIVATE_CALL_KEY)
            {
                return;
            }
        }
        let locals = self.local_variable_names(Vec::new(), position).ok();
        let mut details = details.borrow_mut();
        details.instance_vars.insert(
            crate::vm::PRIVATE_CALL_KEY.to_string(),
            Object::Bool(private_call),
        );
        if let Some(locals) = locals {
            details
                .instance_vars
                .entry(crate::vm::LOCAL_VARIABLES_KEY.to_string())
                .or_insert(locals);
        }
    }

    pub(crate) fn receiver_wording_for(&mut self, receiver: &Object, position: Position) -> String {
        if self.is_the_main_object(receiver) {
            return "main".to_string();
        }
        let named = match receiver {
            Object::Class(class) | Object::Module(class) => {
                self.written_class_name(class, position)
            }
            Object::Instance(instance) => {
                let has_singleton = instance.borrow().singleton_class.borrow().is_some()
                    || !instance.borrow().singleton_methods.borrow().is_empty();
                if has_singleton {
                    None
                } else {
                    let class = std::rc::Rc::clone(&instance.borrow().class);
                    self.written_class_name(&class, position)
                }
            }
            _ => None,
        };
        match (receiver, named) {
            (Object::Class(_), Some(name)) => format!("class {}", name),
            (Object::Module(_), Some(name)) => format!("module {}", name),
            (Object::Instance(_), Some(name)) => format!("an instance of {}", name),
            _ => receiver_wording(receiver),
        }
    }

    /// The name a class with none of its own answers from a `name` method the
    /// program wrote for it, or None when there is no such method.
    fn written_class_name(
        &mut self,
        class: &std::rc::Rc<crate::class::Class>,
        position: Position,
    ) -> Option<String> {
        if !class.ruby_name().is_empty() {
            return None;
        }
        let receiver = Object::Class(std::rc::Rc::clone(class));
        let holds_name = matches!(
            self.lookup_method(&receiver, "name"),
            Some((_, method)) if !method.is_undefined && !method.body.is_empty()
        );
        if !holds_name {
            return None;
        }
        match self.send_to_object(receiver, "name", Vec::new(), position) {
            Ok(Object::String(text)) => Some(text.as_str().to_string()),
            _ => None,
        }
    }
}

/// The name a class or module goes by, or the form it writes itself in when
/// it has none.
fn named_or_written(class: &std::rc::Rc<crate::class::Class>, written: &Object) -> String {
    let named = class.ruby_name();
    if named.is_empty() {
        return written.to_string();
    }
    named.to_string()
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
    // A value with no arithmetic of its own has no such method, which is what
    // Ruby reports rather than a mismatch of types. Text carries none of the
    // number operators either, so the same goes for a String.
    let bitwise = matches!(
        op,
        BinaryOp::BitwiseOr | BinaryOp::BitwiseAnd | BinaryOp::Xor
    );
    if (matches!(left, Object::Nil | Object::Bool(_))
        || (bitwise && matches!(left, Object::String(_))))
        && let Some(named) = crate::vm::eval::binary_op_method_name(&op)
    {
        return undefined_method_error(named, left, std::slice::from_ref(right), position);
    }
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
pub(crate) fn load_error(message: String, feature: &str) -> Object {
    let exception = Object::exception("LoadError", message);
    if let Object::Exception(details) = &exception {
        details.borrow_mut().instance_vars.insert(
            crate::vm::EXCEPTION_PATH_KEY.to_string(),
            Object::string(feature),
        );
    }
    exception
}

/// A LoadError naming no path. Ruby reports what the loader said rather than
/// the file it was looking for when the file is there but cannot be loaded.
pub(super) fn load_error_without_path(message: String) -> Object {
    Object::exception("LoadError", message)
}

/// A SyntaxError carrying the file the unparsable code came from, which
/// `#path` answers. `path` is None for code with no file behind it.
pub(super) fn syntax_error(
    message: String,
    path: Option<&str>,
    position: Position,
) -> MetorexError {
    let exception = Object::exception("SyntaxError", message.clone());
    if let (Object::Exception(details), Some(path)) = (&exception, path) {
        details.borrow_mut().instance_vars.insert(
            crate::vm::EXCEPTION_PATH_KEY.to_string(),
            Object::string(path),
        );
    }
    MetorexError::UncaughtException {
        exception,
        location: position_to_location(position),
        message,
    }
}

/// An exception raised while a file was loading keeps its class and message
/// on the way out of the load, the way MRI propagates it. Any other error is
/// handed to `wrap`, which adds the load's own context to its message.
pub(crate) fn keep_exception(
    error: MetorexError,
    wrap: impl FnOnce(&str) -> MetorexError,
) -> MetorexError {
    match error {
        MetorexError::UncaughtException { .. } => error,
        other => wrap(other.message()),
    }
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
