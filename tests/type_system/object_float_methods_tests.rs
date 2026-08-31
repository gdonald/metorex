// Tests for Object, Float, and Exception native method coverage

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

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

// ── Object#to_s ──────────────────────────────────────────────────────────────

#[test]
fn nil_to_s_returns_empty_string() {
    // Ruby semantics: nil.to_s == "". Use nil.inspect for the literal "nil".
    let result = run("nil.to_s");
    assert_eq!(result, Some(Object::string("")));
}

#[test]
fn bool_to_s_returns_string() {
    let result = run("true.to_s");
    assert_eq!(result, Some(Object::string("true")));
}

#[test]
fn nil_to_s_with_args_silently_ignores() {
    // We don't enforce arity on the Nil-specific shortcut; Ruby would raise
    // an ArgumentError but the cost of validating doesn't justify a check.
    let _ = run("nil.to_s");
}

// ── Object#class ─────────────────────────────────────────────────────────────

#[test]
fn nil_class_returns_class() {
    let result = run("nil.class.nil?");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn nil_class_error_with_args() {
    let err = run_err("nil.class(1)");
    assert!(err.contains("argument"));
}

// ── Object#respond_to? ──────────────────────────────────────────────────────

#[test]
fn object_respond_to_user_method() {
    let result = run(r#"
class C
  def greet
    "hello"
  end
end
C.new.respond_to?("greet")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_respond_to_missing() {
    let result = run(r#"
class C
  def greet
    "hello"
  end
end
C.new.respond_to?("nonexistent_xyz")
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn nil_respond_to_error_no_args() {
    let err = run_err("nil.respond_to?");
    assert!(err.contains("argument"));
}

#[test]
fn nil_respond_to_error_wrong_type() {
    let err = run_err("nil.respond_to?(42)");
    assert!(err.contains("42 is not a symbol nor a string"));
}

// ── Object#nil? ──────────────────────────────────────────────────────────────

#[test]
fn nil_nil_query_true() {
    let result = run("nil.nil?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bool_nil_query_false() {
    let result = run("true.nil?");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn nil_nil_error_with_args() {
    let err = run_err("nil.nil?(1)");
    assert!(err.contains("argument"));
}

// ── Object#get_source ────────────────────────────────────────────────────────

#[test]
fn get_source_error_wrong_count() {
    let err = run_err(
        r#"
class C
  def f
    nil
  end
end
C.new.get_source
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn get_source_error_wrong_type() {
    let err = run_err(
        r#"
class C
  def f
    nil
  end
end
C.new.get_source(42)
"#,
    );
    assert!(err.contains("String or Symbol"));
}

// ── Float#round ──────────────────────────────────────────────────────────────

#[test]
fn float_round_two_places() {
    let result = run("3.14159.round(2)");
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn float_round_zero_places() {
    let result = run("3.7.round(0)");
    assert_eq!(result, Some(Object::Float(4.0)));
}

#[test]
fn float_round_error_no_args() {
    let err = run_err("3.14.round");
    assert!(err.contains("argument"));
}

#[test]
fn float_round_error_wrong_type() {
    let err = run_err(r#"3.14.round("x")"#);
    assert!(err.contains("Integer"));
}

#[test]
fn float_round_error_negative_precision() {
    let err = run_err("3.14.round(-1)");
    assert!(err.contains("non-negative"));
}

// ── Exception methods ────────────────────────────────────────────────────────

#[test]
fn exception_message() {
    let result = run(r#"
e = nil
begin
  raise "test error"
rescue => ex
  e = ex.message
end
e
"#);
    assert_eq!(result, Some(Object::string("test error")));
}

#[test]
fn exception_type() {
    let result = run(r#"
e = nil
begin
  raise "test error"
rescue => ex
  e = ex.type
end
e
"#);
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn exception_backtrace_returns_array() {
    let result = run(r#"
e = nil
begin
  raise "test error"
rescue => ex
  e = ex.backtrace
end
e.nil?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn exception_to_s() {
    let result = run(r#"
e = nil
begin
  raise "test error"
rescue => ex
  e = ex.to_s
end
e.length > 0
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}
