// Int error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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
// Int methods - times with continue/exception (lines 79-84)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn int_times_with_continue_skips() {
    let result = run(r#"
sum = 0
5.times do |i|
  if i == 2
    continue
  end
  sum = sum + i
end
sum
"#);
    // 0 + 1 + 3 + 4 = 8
    assert_eq!(result, Some(Object::Int(8)));
}

#[test]
fn int_times_return_in_method_returns_from_method() {
    // Ruby semantics: `return` inside a block returns from the enclosing method.
    let result = run(r#"
def test_times_return
  3.times do |i|
    return i
  end
end
test_times_return
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn int_times_raise_propagates() {
    let err = run_err(
        r#"
3.times do |i|
  raise "times boom"
end
"#,
    );
    assert!(
        err.contains("times boom") || err.contains("Uncaught"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Int methods - abs, to_f, to_i, to_s
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn int_abs_method() {
    let result = run(r#"
(-5).abs
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn int_to_f_method() {
    let result = run(r#"
5.to_f
"#);
    assert_eq!(result, Some(Object::Float(5.0)));
}

#[test]
fn int_to_i_method() {
    let result = run(r#"
5.to_i
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn int_to_s_method() {
    let result = run(r#"
42.to_s
"#);
    assert_eq!(result, Some(Object::String(Rc::new("42".to_string()))));
}
