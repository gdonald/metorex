// How a class or module body names itself, and what it refuses.

use super::*;

/// How a class or module body names itself in a backtrace: `<class:Name>`,
/// `<module:Name>`, or `singleton class` for one opened with `class << x`.
pub(crate) fn class_body_label(class: &Rc<Class>) -> String {
    if class.is_singleton_class() {
        return "singleton class".to_string();
    }
    let named = class.name();
    let unqualified = named.rsplit("::").next().unwrap_or(named);
    if class.is_module() {
        format!("<module:{unqualified}>")
    } else {
        format!("<class:{unqualified}>")
    }
}

/// The TypeError `module X` raises when X already names something that is not
/// a module.
pub(crate) fn not_a_module(named: &str, held: &Object, position: Position) -> MetorexError {
    let message = format!("{} is not a module", named);
    let _ = held;
    crate::vm::errors::simple_exception("TypeError", &message, position)
}

/// Ruby's parser refuses a `return` written directly in a class or module
/// body, which it reports as a syntax error.
pub(crate) fn return_in_a_body_error(position: Position) -> MetorexError {
    crate::vm::errors::syntax_error(
        "Invalid return in class/module body".to_string(),
        None,
        position,
    )
}

/// A `return` written in a block inside a class or module body has no method
/// to return from, so it is refused where the body ends.
pub(crate) fn refuse_return_from_a_body(
    answer: Result<Object, MetorexError>,
    position: Position,
) -> Result<Object, MetorexError> {
    match answer {
        Err(MetorexError::NonLocalReturn { .. }) => Err(crate::vm::errors::simple_exception(
            "LocalJumpError",
            "unexpected return",
            position,
        )),
        other => other,
    }
}
