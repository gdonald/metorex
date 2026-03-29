// Coverage tests for vm/method_invocation.rs uncovered paths

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

// ── Function called with wrong arg count ──────────────────────────────────────

#[test]
fn function_too_few_args_error() {
    let err = run_err(
        r#"
def add(x, y)
  x + y
end
add 1
"#,
    );
    assert!(err.contains("argument") || err.contains("add") || err.contains("expected"));
}

#[test]
fn function_too_many_args_error() {
    let err = run_err(
        r#"
def greet(name)
  name
end
greet("Alice", "Bob")
"#,
    );
    assert!(err.contains("argument") || err.contains("greet") || err.contains("expected"));
}

// ── Exception.new with non-string argument ────────────────────────────────────

#[test]
fn exception_new_with_non_string_arg() {
    let result = run(r#"
e = RuntimeError.new(42)
e
"#);
    // Creates an exception object with a non-string arg (debug-formatted)
    assert!(result.is_some());
}

// ── Exception.new with more than one argument ─────────────────────────────────

#[test]
fn exception_new_with_too_many_args_error() {
    let err = run_err(
        r#"
RuntimeError.new("a", "b")
"#,
    );
    assert!(err.contains("argument") || err.contains("Exception") || err.contains("0 or 1"));
}

// ── Unless as last statement in standalone function body ──────────────────────

#[test]
fn function_body_ends_with_unless_else_taken() {
    let result = run(r#"
def check(x)
  unless x > 0
    "negative or zero"
  else
    "positive"
  end
end
check 5
"#);
    assert_eq!(result, Some(Object::string("positive")));
}

#[test]
fn function_body_ends_with_unless_then_taken() {
    let result = run(r#"
def check(x)
  unless x > 0
    "negative or zero"
  else
    "positive"
  end
end
check(-3)
"#);
    assert_eq!(result, Some(Object::string("negative or zero")));
}

// ── Unless as last statement in instance method body ─────────────────────────

#[test]
fn method_body_ends_with_unless_else_taken() {
    let result = run(r#"
class Checker
  def check(x)
    unless x > 0
      "non-positive"
    else
      "positive"
    end
  end
end
Checker.new.check(10)
"#);
    assert_eq!(result, Some(Object::string("positive")));
}

#[test]
fn method_body_ends_with_unless_then_taken() {
    let result = run(r#"
class Checker
  def check(x)
    unless x > 0
      "non-positive"
    else
      "positive"
    end
  end
end
Checker.new.check(-2)
"#);
    assert_eq!(result, Some(Object::string("non-positive")));
}

// ── Missing required keyword argument ─────────────────────────────────────────

#[test]
fn missing_required_keyword_arg_error() {
    let err = run_err(
        r#"
def greet(name:)
  name
end
greet()
"#,
    );
    assert!(
        err.contains("keyword")
            || err.contains("name")
            || err.contains("Missing")
            || err.contains("argument")
    );
}

// ── Break/Continue in method body (non-loop context) ─────────────────────────

#[test]
fn break_inside_method_body_error() {
    let err = run_err(
        r#"
class C
  def foo
    break
  end
end
C.new.foo
"#,
    );
    assert!(err.contains("break") || err.contains("loop") || err.contains("control"));
}

#[test]
fn continue_inside_method_body_error() {
    let err = run_err(
        r#"
class C
  def foo
    continue
  end
end
C.new.foo
"#,
    );
    assert!(err.contains("continue") || err.contains("loop") || err.contains("control"));
}

// ── Exception.new with zero args ──────────────────────────────────────────────

#[test]
fn exception_new_with_no_args() {
    let result = run(r#"
e = RuntimeError.new
e
"#);
    assert!(result.is_some());
}
