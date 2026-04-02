// Range error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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

// ══════════════════════════════════════════════════════════════════════════════
// Range methods - to_a inclusive (lines 42-47)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn range_to_a_inclusive_values() {
    let result = run(r#"
(1..3).to_a
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed[0], Object::Int(1));
        assert_eq!(borrowed[2], Object::Int(3));
    }
}

#[test]
fn range_to_a_exclusive_values() {
    let result = run(r#"
(1...4).to_a
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed[0], Object::Int(1));
        assert_eq!(borrowed[2], Object::Int(3));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Range methods - map with block (lines 154-159)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn range_map_inclusive_transforms() {
    let result = run(r#"
(1..4).map { |x| x * 10 }
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 4);
        assert_eq!(borrowed[0], Object::Int(10));
        assert_eq!(borrowed[3], Object::Int(40));
    }
}

#[test]
fn range_map_exclusive_transforms() {
    let result = run(r#"
(1...4).map { |x| x * 10 }
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed[0], Object::Int(10));
        assert_eq!(borrowed[2], Object::Int(30));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Range methods - each with return/raise
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn range_each_return_in_method_errors() {
    let err = run_err(
        r#"
(1..3).each do |i|
  return i
end
"#,
    );
    assert!(
        err.contains("return") || err.contains("control") || err.contains("loop"),
        "Error was: {}",
        err
    );
}

#[test]
fn range_each_raise_propagates() {
    let err = run_err(
        r#"
(1..3).each do |i|
  raise "range boom"
end
"#,
    );
    assert!(
        err.contains("range boom") || err.contains("Uncaught"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Range include? method
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn range_include_inclusive() {
    let result = run(r#"
(1..5).include?(3)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn range_include_exclusive() {
    let result = run(r#"
(1...5).include?(5)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}
