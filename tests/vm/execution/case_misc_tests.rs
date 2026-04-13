// VM tests for case/when, case/in, defined?, misc control flow, multiple assignment.

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

// ── case/when ────────────────────────────────────────────────────────────────

#[test]
fn case_when_as_last_stmt_in_method() {
    let result = run(r#"
def grade(score)
  case score
  when 90
    "A"
  when 80
    "B"
  else
    "F"
  end
end
grade(90)
"#);
    assert_eq!(result, Some(Object::string("A")));
}

#[test]
fn case_when_in_block_returns_value() {
    let result = run(r#"
result = [1, 2].map do |n|
  case n
  when 1
    "one"
  when 2
    "two"
  end
end
result[0]
"#);
    assert_eq!(result, Some(Object::string("one")));
}

#[test]
fn case_when_as_last_stmt_in_nested_context() {
    let result = run(r#"
def last_is_case(x)
  y = x + 1
  case y
  when 2
    "matched two"
  when 3
    "matched three"
  else
    "other"
  end
end
last_is_case(1)
"#);
    assert_eq!(result, Some(Object::string("matched two")));
}

#[test]
fn top_level_if_containing_case_returns_value() {
    let result = run(r#"
if true
  case 1
  when 1 then 5
  end
end
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn begin_body_case_when_covers_execute_statements_for_value() {
    let result = run(r#"
def classify(x)
  begin
    case x
    when 1 then "one"
    when 2 then "two"
    end
  end
end
classify 1
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("one".to_string())))
    );
}

#[test]
fn method_body_case_when_as_last_statement() {
    let result = run(r#"
class Classifier
  def classify(x)
    case x
    when 1 then "one"
    when 2 then "two"
    end
  end
end
Classifier.new.classify 1
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("one".to_string())))
    );
}

#[test]
fn method_body_begin_rescue_as_last_statement() {
    let result = run(r#"
class SafeDiv
  def compute(a, b)
    begin
      a / b
    rescue => e
      0
    end
  end
end
SafeDiv.new.compute 10, 2
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── case/in pattern matching ──────────────────────────────────────────────────

#[test]
fn case_in_pattern_match_in_method() {
    let result = run(r#"
def classify(x)
  case x
  in 1
    "one"
  in 2
    "two"
  end
end
classify(1)
"#);
    assert_eq!(result, Some(Object::string("one")));
}

#[test]
fn case_in_pattern_match_in_block() {
    let result = run(r#"
result = [1, 2].map do |n|
  case n
  in 1
    "one"
  in 2
    "two"
  end
end
result[0]
"#);
    assert_eq!(result, Some(Object::string("one")));
}

// ── defined? ─────────────────────────────────────────────────────────────────

#[test]
fn defined_instance_var_in_class_method_returns_nil() {
    let result = run(r#"
class Checker
  def self.test
    defined?(@x)
  end
end
Checker.test
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn defined_call_to_undefined_function_returns_nil() {
    let result = run(r#"defined?(totally_nonexistent_func_xyz())"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn defined_failing_expression_returns_nil() {
    let result = run(r#"defined?(1 / 0)"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── multiple assignment ───────────────────────────────────────────────────────

#[test]
fn multiple_assignment_with_newline_between_targets() {
    let result = run("a,\nb = 1, 2\na + b");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn multiple_assignment_lookahead_rejects_non_ident() {
    let result = run("a = 1\nb = 2\n[a, b].length");
    assert_eq!(result, Some(Object::Int(2)));
}

// ── control flow in expressions ───────────────────────────────────────────────

#[test]
fn break_inside_while_in_if_expression() {
    let result = run(r#"
x = 0
result = if true
  while x < 5
    x = x + 1
    break if x == 3
  end
  x
end
result
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn if_expression_with_case_in_body_returns_nil() {
    let result = run(r#"
x = if true
  case 1
  when 1 then 5
  end
end
x
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── parenthesized assignment / hash trailing comma ────────────────────────────

#[test]
fn parenthesized_assignment_expression() {
    let result = run(r#"
a = 0
(a = 5)
a
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn hash_literal_with_trailing_comma() {
    let result = run(r#"h = {"a" => 1,}; h["a"]"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── index instance without [] method ─────────────────────────────────────────

#[test]
fn index_instance_without_bracket_method_errors() {
    let err = run_err(
        r#"
class NoIndex
end
NoIndex.new[0]
"#,
    );
    assert!(
        err.contains("Cannot index") || err.contains("type") || err.contains("NoIndex"),
        "Error was: {}",
        err
    );
}
