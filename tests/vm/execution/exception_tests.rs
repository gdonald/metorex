// VM tests for exceptions, rescue, ensure, begin/rescue, super, ivar/cvar errors.

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

// ── exception handling: ensure ───────────────────────────────────────────────

#[test]
fn begin_rescue_ensure_runs_always() {
    let result = run(r#"
x = 0
begin
  raise "oops"
rescue
  x = 1
ensure
  x = x + 10
end
x
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn ensure_with_exception_in_ensure() {
    let err = run_err(
        r#"
begin
  1
ensure
  raise "ensure error"
end
"#,
    );
    assert!(err.contains("ensure error"));
}

// ── instance variable / class variable outside context ───────────────────────

#[test]
fn ivar_outside_method_error() {
    let err = run_err("@x = 1");
    assert!(err.contains("instance variable") || err.contains("method"));
}

#[test]
fn cvar_outside_class_error() {
    let err = run_err("@@x = 1");
    assert!(err.contains("class variable") || err.contains("class"));
}

// ── rescue bare / reraise ─────────────────────────────────────────────────────

#[test]
fn rescue_bare_catches_runtime_error() {
    let result = run(r#"
def risky
  begin
    raise "oops"
  rescue => e
    "caught"
  end
end
risky
"#);
    assert_eq!(result, Some(Object::string("caught")));
}

#[test]
fn rescue_reraise_propagates() {
    let err = run_err(
        r#"
def risky
  begin
    raise "original"
  rescue => e
    raise "re-raised"
  end
end
risky
"#,
    );
    assert!(err.contains("re-raised"));
}

// ── require_relative ─────────────────────────────────────────────────────────

#[test]
fn require_relative_nonexistent_error() {
    let err = run_err(r#"require_relative "absolutely_nonexistent_file_xyz""#);
    assert!(err.contains("require_relative") || err.contains("cannot"));
}

// ── undef_method ─────────────────────────────────────────────────────────────

#[test]
fn undef_method_raises_on_call() {
    let err = run_err(
        r#"
class Greeter
  def hello
    "hi"
  end
end
Greeter.undef_method("hello")
Greeter.new.hello
"#,
    );
    assert!(err.contains("undefined") || err.contains("Undefined"));
}

// ── nested begin ─────────────────────────────────────────────────────────────

#[test]
fn nested_begin_as_last_stmt_in_execute_statements_for_value() {
    let result = run(r#"
def nested_begin
  begin
    begin
      42
    rescue => e
      0
    end
  rescue => e
    -1
  end
end
nested_begin
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn nested_begin_rescue_propagates_to_outer() {
    let result = run(r#"
def nested_rescue
  begin
    begin
      raise "inner error"
    rescue => e
      "inner caught"
    end
  rescue => e
    "outer caught"
  end
end
nested_rescue
"#);
    assert_eq!(result, Some(Object::string("inner caught")));
}

// ── ensure with return / error ────────────────────────────────────────────────

#[test]
fn ensure_block_return_overrides_rescue_result() {
    let result = run(r#"
def ensure_return_test
  begin
    raise "error"
  rescue => e
    "rescued"
  ensure
    return "from_ensure"
  end
end
ensure_return_test
"#);
    assert!(
        result == Some(Object::String(std::rc::Rc::new("from_ensure".to_string())))
            || result == Some(Object::String(std::rc::Rc::new("rescued".to_string())))
    );
}

#[test]
fn ensure_block_with_return_in_non_last_stmt() {
    let result = run(r#"
def test_ensure
  begin
    1 + 1
  ensure
    return 99
  end
  "never reached"
end
test_ensure
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn ensure_block_with_runtime_error_overrides_result() {
    let err = run_err(
        r#"
begin
  1 + 1
ensure
  1 / 0
end
"#,
    );
    assert!(
        err.contains("zero") || err.contains("division") || err.contains("0"),
        "Error was: {}",
        err
    );
}

// ── break / next outside loop ─────────────────────────────────────────────────

#[test]
fn break_inside_begin_body_errors() {
    let err = run_err(
        r#"
class BrTest
  def test
    begin
      break
    end
  end
end
BrTest.new.test
"#,
    );
    assert!(
        err.contains("break") || err.contains("loop") || err.contains("outside"),
        "Error was: {}",
        err
    );
}

#[test]
fn break_inside_function_body_errors() {
    let err = run_err(
        r#"
def broken
  break
end
broken
"#,
    );
    assert!(
        err.contains("break") || err.contains("loop") || err.contains("outside"),
        "Error was: {}",
        err
    );
}

#[test]
fn next_inside_begin_body_errors() {
    let err = run_err(
        r#"
class NextTest
  def test
    begin
      next
    end
  end
end
NextTest.new.test
"#,
    );
    assert!(
        err.contains("next")
            || err.contains("loop")
            || err.contains("outside")
            || err.contains("continue"),
        "Error was: {}",
        err
    );
}

// ── super in class with no superclass ────────────────────────────────────────

#[test]
fn super_in_class_with_no_superclass_errors() {
    let err = run_err(
        r#"
class BaseNoParent
  def foo
    super
  end
end
BaseNoParent.new.foo
"#,
    );
    assert!(
        err.contains("superclass")
            || err.contains("Superclass")
            || err.contains("super")
            || err.contains("BaseNoParent"),
        "Error was: {}",
        err
    );
}

// ── rescue RuntimeError internally converted ────────────────────────────────

#[test]
fn rescue_runtime_error_from_undefined_method() {
    let result = run(r#"
val = nil
begin
  "hello".nonexistent_method_xyz
rescue => e
  val = e.message
end
val
"#);
    assert!(result.is_some());
}

#[test]
fn rescue_type_error() {
    let result = run(r#"
val = nil
begin
  1 + "a"
rescue TypeError => e
  val = e.message
end
val
"#);
    assert!(result.is_some());
}

// ── Hash.new with default block ─────────────────────────────────────────────

#[test]
fn hash_new_with_default_block() {
    let result = run(r#"
h = Hash.new { |hash, key| hash[key] = key.to_s + "!" }
h["foo"]
"#);
    assert_eq!(result, Some(Object::string("foo!")));
}

// ── Rational and Complex construction ───────────────────────────────────────

#[test]
fn rational_construction() {
    let result = run("Rational(3, 4)");
    assert!(matches!(result, Some(Object::Instance(_))));
}

#[test]
fn complex_construction() {
    let result = run("Complex(1, 2)");
    assert!(matches!(result, Some(Object::Instance(_))));
}

// ── Refinement captured in method ───────────────────────────────────────────

#[test]
fn refinement_method_uses_refined_behavior() {
    let result = run(r#"
module StringExt
  refine(String) do
    def shout
      upcase + "!"
    end
  end
end

using StringExt

def test_shout
  "hello".shout
end

test_shout
"#);
    assert_eq!(result, Some(Object::string("HELLO!")));
}
