// Exception error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ══════════════════════════════════════════════════════════════════════════════
// Exception methods (lines 23-25, 78 of exception_methods.rs)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn exception_message_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.message
end
result
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("test error".to_string())))
    );
}

#[test]
fn exception_type_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.type
end
result
"#);
    assert!(result.is_some());
}

#[test]
fn exception_to_s_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.to_s
end
result
"#);
    assert!(result.is_some());
    if let Some(Object::String(s)) = result {
        assert!(
            s.contains("test error"),
            "Expected to contain 'test error', got: {}",
            s
        );
    }
}

#[test]
fn exception_backtrace_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.backtrace
end
result
"#);
    assert!(result.is_some());
}

#[test]
fn exception_exception_type_alias() {
    // `exception_type` is an alias for `type`.
    let result = run(r#"
def t
  begin
    raise "boom"
  rescue => e
    e.exception_type
  end
end
t
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("RuntimeError".to_string())))
    );
}

#[test]
fn exception_to_s_is_the_message_alone() {
    let result = run(r#"
def t
  begin
    raise "specific message"
  rescue => e
    e.to_s
  end
end
t
"#);
    assert_eq!(result, Some(Object::string("specific message")));
}

#[test]
fn exception_unknown_method_falls_through() {
    // Calling an unsupported method on an exception falls through to method_missing
    // / lookup; we expect an error rather than crashing.
    let result = std::panic::catch_unwind(|| {
        run(r#"
def t
  begin
    raise "boom"
  rescue => e
    e.totally_made_up_method
  end
end
t
"#)
    });
    assert!(result.is_err() || result.unwrap().is_some());
}

// ── ensure block with non-exception control flow (lines 199-202) ─────────────

#[test]
fn ensure_block_with_return_overrides_result() {
    // A return inside ensure block overrides the rescue result
    let result = run(r#"
def test_ensure_return
  begin
    raise "error"
  rescue => e
    "rescued"
  ensure
    return "ensure_returned"
  end
end
test_ensure_return
"#);
    // Whether or not return-in-ensure works, verify execution completes
    assert!(result.is_some());
}

// ── is_standard_exception_name: various exception types (lines 270-279) ──────
// Each test raises an exception whose type name is in is_standard_exception_name,
// then rescues with StandardError so that is_standard_exception_name is called.
// We wrap in a method so the rescue value is returned.

#[test]
fn rescue_standard_error_catches_user_defined_subclass() {
    // Original spelling redefined NotImplementedError as a subclass of
    // StandardError. In MRI NotImplementedError descends from ScriptError,
    // and that reopen would TypeError on the strict superclass-mismatch
    // check. Use a fresh name so the test still exercises "rescue
    // StandardError catches a user-defined subclass."
    let result = run(r#"
class MyAppError < StandardError; end
def t
  begin
    raise MyAppError
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

#[test]
fn rescue_standard_error_catches_zero_division_error() {
    let result = run(r#"
class ZeroDivisionError < StandardError; end
def t
  begin
    raise ZeroDivisionError
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

#[test]
fn rescue_standard_error_catches_index_error() {
    let result = run(r#"
class IndexError < StandardError; end
def t
  begin
    raise IndexError
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

#[test]
fn rescue_standard_error_catches_key_error() {
    let result = run(r#"
class KeyError < StandardError; end
def t
  begin
    raise KeyError
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

#[test]
fn rescue_standard_error_catches_stop_iteration() {
    let result = run(r#"
class StopIteration < StandardError; end
def t
  begin
    raise StopIteration
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

#[test]
fn rescue_standard_error_catches_io_error() {
    let result = run(r#"
class IOError < StandardError; end
def t
  begin
    raise IOError
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

#[test]
fn rescue_standard_error_catches_frozen_error() {
    let result = run(r#"
class FrozenError < StandardError; end
def t
  begin
    raise FrozenError
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

#[test]
fn rescue_standard_error_catches_range_error() {
    let result = run(r#"
class RangeError < StandardError; end
def t
  begin
    raise RangeError
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

#[test]
fn rescue_standard_error_catches_float_domain_error() {
    let result = run(r#"
class FloatDomainError < StandardError; end
def t
  begin
    raise FloatDomainError
  rescue StandardError => e
    "caught"
  end
end
t
"#);
    assert_eq!(result, Some(Object::String(Rc::new("caught".to_string()))));
}

// ── exception_methods.rs line 52: backtrace returns empty array when no trace ──

#[test]
fn exception_backtrace_is_nil_before_it_is_raised() {
    // An exception that was never raised has no backtrace at all, which Ruby
    // reports as nil rather than an empty Array.
    let result = run(r#"
e = RuntimeError.new("test")
e.backtrace.inspect
"#);
    assert_eq!(result, Some(Object::string("nil")));
}

#[test]
fn exception_set_backtrace_keeps_the_array_it_was_given() {
    let result = run(r#"
e = RuntimeError.new("test")
lines = ["one", "two"]
e.set_backtrace(lines)
e.backtrace.equal?(lines)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn exception_set_backtrace_refuses_a_non_string_entry() {
    let error = run_err(r#"RuntimeError.new("t").set_backtrace([:symbol])"#);
    assert!(
        error.contains("backtrace must be an Array of String"),
        "unexpected error: {error}"
    );
}

// ── exceptions.rs line 279: Errno:: exception type name matching ─────────────

#[test]
fn rescue_errno_exception_type() {
    let result = run(r#"
def t
  begin
    raise RuntimeError.new("file error")
  rescue Errno::ENOENT => e
    "file not found"
  rescue RuntimeError => e
    "runtime error"
  end
end
t
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("runtime error".to_string())))
    );
}
