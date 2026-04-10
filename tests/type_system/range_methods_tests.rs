// Tests for Range native method coverage

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

// ── each ────────────────────────────────────────────────────────────────────

#[test]
fn range_each_iterates() {
    let result = run(r#"
sum = 0
(1..5).each { |i| sum = sum + i }
sum
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn range_each_exclusive() {
    let result = run(r#"
sum = 0
(1...5).each { |i| sum = sum + i }
sum
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn range_each_error_with_args() {
    let err = run_err("(1..5).each(1) { |i| i }");
    assert!(err.contains("argument"));
}

#[test]
fn range_each_error_no_block() {
    let err = run_err("(1..5).each");
    assert!(err.contains("block"));
}

#[test]
fn range_each_with_break() {
    let result = run(r#"
sum = 0
(1..10).each do |i|
  if i == 4
    break
  end
  sum = sum + i
end
sum
"#);
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn range_each_with_continue() {
    let result = run(r#"
sum = 0
(1..5).each do |i|
  if i == 3
    continue
  end
  sum = sum + i
end
sum
"#);
    assert_eq!(result, Some(Object::Int(12)));
}

// ── to_a ────────────────────────────────────────────────────────────────────

#[test]
fn range_to_a_inclusive() {
    let result = run("(1..4).to_a");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
            Object::Int(4),
        ]))
    );
}

#[test]
fn range_to_a_exclusive() {
    let result = run("(1...4).to_a");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

#[test]
fn range_to_a_error_with_args() {
    let err = run_err("(1..5).to_a(1)");
    assert!(err.contains("argument"));
}

// ── include? ────────────────────────────────────────────────────────────────

#[test]
fn range_include_true() {
    let result = run("(1..10).include?(5)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn range_include_false() {
    let result = run("(1..10).include?(11)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn range_include_exclusive_boundary() {
    let result = run("(1...10).include?(10)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn range_include_error_no_args() {
    let err = run_err("(1..5).include?");
    assert!(err.contains("argument"));
}

#[test]
fn range_include_error_too_many_args() {
    let err = run_err("(1..5).include?(1, 2)");
    assert!(err.contains("argument"));
}

// ── map ─────────────────────────────────────────────────────────────────────

#[test]
fn range_map_squares() {
    let result = run("(1..3).map { |n| n * n }");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(4),
            Object::Int(9),
        ]))
    );
}

#[test]
fn range_map_exclusive() {
    let result = run("(1...4).map { |n| n * 2 }");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6),
        ]))
    );
}

#[test]
fn range_map_error_no_block() {
    let err = run_err("(1..5).map");
    assert!(err.contains("block"));
}

#[test]
fn range_map_error_with_args() {
    let err = run_err("(1..5).map(1) { |n| n }");
    assert!(err.contains("argument"));
}

// ── each/map: return and exception inside block ───────────────────────────────

#[test]
fn range_each_return_inside_block_error() {
    let err = run_err(
        r#"
(1..3).each do |i|
  return i
end
"#,
    );
    assert!(err.contains("return") || err.contains("loop"));
}

#[test]
fn range_each_exception_inside_block_error() {
    let err = run_err(r#"(1..3).each { |i| raise "range error" }"#);
    assert!(err.contains("range error") || err.contains("exception") || err.contains("Uncaught"));
}

#[test]
fn range_map_exception_inside_block_error() {
    let err = run_err(r#"(1..3).map { |i| raise "range map error" }"#);
    assert!(
        err.contains("range map error") || err.contains("exception") || err.contains("Uncaught")
    );
}

#[test]
fn range_each_basic_coverage() {
    let result = run("result = []\n(1..3).each do |x|\n  result << x\nend\nresult");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3)
        ]))
    );
}

#[test]
fn range_map_basic_coverage() {
    let result = run("(1..3).map do |x|\n  x * 2\nend");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6)
        ]))
    );
}
