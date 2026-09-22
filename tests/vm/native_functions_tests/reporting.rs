// What a program writes out and what it reports about the call running.

use super::*;

#[test]
fn at_exit_returns_the_handler() {
    let result = run("at_exit { puts 'bye' }");
    assert!(matches!(result, Some(Object::Block(_))));
}

// ── warn ─────────────────────────────────────────────────────────────────────

#[test]
fn warn_returns_nil() {
    let result = run("warn 'test warning'");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn warn_multiple_args() {
    let result = run("warn 'a', 'b'");
    assert_eq!(result, Some(Object::Nil));
}

// ── sprintf / format ─────────────────────────────────────────────────────────

#[test]
fn sprintf_basic() {
    let result = run("sprintf '%s is %d', 'age', 25");
    assert!(result.is_some());
}

#[test]
fn sprintf_no_args_error() {
    let err = run_err("sprintf()");
    assert!(err.contains("argument"));
}

#[test]
fn format_alias() {
    let result = run("format '%d', 42");
    assert!(result.is_some());
}

// ── __method__ ───────────────────────────────────────────────────────────────

#[test]
fn dunder_method_returns_symbol() {
    let result = run(r#"
def foo
  __method__()
end
foo
"#);
    assert!(matches!(result, Some(Object::Symbol(_))));
}

// ── caller ───────────────────────────────────────────────────────────────────

#[test]
fn caller_returns_array() {
    let result = run("caller()");
    assert!(matches!(result, Some(Object::Array(_))));
}
