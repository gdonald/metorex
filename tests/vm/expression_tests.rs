// Coverage tests for vm/expression.rs uncovered paths

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

// ── Dictionary literal with invalid key type ──────────────────────────────────

#[test]
fn dict_literal_array_key_error() {
    let err = run_err(
        r#"
h = {[1, 2] => "val"}
"#,
    );
    assert!(err.contains("Dictionary keys") || err.contains("key") || err.contains("String"));
}

// ── Array index read with non-integer ────────────────────────────────────────

#[test]
fn array_index_read_non_int_error() {
    let err = run_err(
        r#"
arr = [1, 2, 3]
arr["x"]
"#,
    );
    assert!(err.contains("integer") || err.contains("Integer") || err.contains("index"));
}

// ── Dictionary index read with invalid key type ───────────────────────────────

#[test]
fn dict_index_read_invalid_key_error() {
    let err = run_err(
        r#"
h = {}
h["key"] = 1
h[[1, 2]]
"#,
    );
    assert!(err.contains("Dictionary") || err.contains("key") || err.contains("index"));
}

// ── Index on non-container type (read) ───────────────────────────────────────

#[test]
fn index_read_on_integer_error() {
    let err = run_err("42[0]");
    assert!(err.contains("index") || err.contains("type") || err.contains("Cannot"));
}

#[test]
fn index_read_on_bool_error() {
    let err = run_err("true[0]");
    assert!(err.contains("index") || err.contains("type") || err.contains("Cannot"));
}

// ── If expression with elsif branch taken ────────────────────────────────────

#[test]
fn if_expression_elsif_branch_taken() {
    let result = run(r#"
def check(x)
  if x == 1
    "one"
  elsif x == 2
    "two"
  else
    "other"
  end
end
check 2
"#);
    assert_eq!(result, Some(Object::string("two")));
}

#[test]
fn if_expression_elsif_first_false_second_true() {
    let result = run(r#"
def check(x)
  if x > 10
    "big"
  elsif x > 5
    "medium"
  elsif x > 0
    "small"
  else
    "zero or negative"
  end
end
check 7
"#);
    assert_eq!(result, Some(Object::string("medium")));
}

// ── Unless expression else branch ────────────────────────────────────────────

#[test]
fn unless_expression_else_branch_taken() {
    let result = run(r#"
def check(x)
  unless x == 0
    "nonzero"
  else
    "zero"
  end
end
check 0
"#);
    assert_eq!(result, Some(Object::string("zero")));
}

// ── Return inside if expression branch ───────────────────────────────────────

#[test]
fn return_inside_if_expression_branch() {
    let result = run(r#"
def check(x)
  if x > 0
    return "positive"
  elsif x < 0
    return "negative"
  else
    return "zero"
  end
end
check(-5)
"#);
    assert_eq!(result, Some(Object::string("negative")));
}

#[test]
fn return_inside_unless_expression_branch() {
    let result = run(r#"
def check(x)
  unless x > 0
    return "non-positive"
  end
  "positive"
end
check(-1)
"#);
    assert_eq!(result, Some(Object::string("non-positive")));
}

// ── Break inside evaluate_branch_value ───────────────────────────────────────

#[test]
fn break_inside_if_expression_in_while_loop() {
    let result = run(r#"
i = 0
while i < 10
  if i == 3
    break
  end
  i = i + 1
end
i
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Dictionary index on missing key returns nil (Ruby semantics) ─────────────

#[test]
fn dict_index_read_undefined_key_returns_nil() {
    let result = run(r#"
h = {"a" => 1}
h["missing"]
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── unless expression with else (expression.rs line 162) ──

#[test]
fn unless_as_expression_returns_nil_when_true() {
    let result = run(r#"
x = unless true
  "nope"
end
x
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── evaluate_branch_value with return in branch (expression.rs line 176) ──

#[test]
fn if_expression_with_return_in_branch() {
    let result = run(r#"
def test_return
  x = if true
    return "early"
    "unreachable"
  end
  x
end
test_return
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("early".to_string())))
    );
}

// ── evaluate_branch_value with break (expression.rs line 194) ──

#[test]
fn if_expression_with_multiple_statements_in_branch() {
    // Tests evaluate_branch_value with multiple statements (expression.rs line 176)
    let result = run(r#"
def test_if_expr
  x = if true
    a = 10
    return a + 5
    0
  end
  x
end
test_if_expr
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

// ── Break/Continue in for-loop (expression.rs line 194) ─────────────────────

#[test]
fn for_loop_with_break() {
    let result = run(
        "sum = 0\nfor i in [1, 2, 3, 4, 5]\n  if i == 4\n    break\n  end\n  sum = sum + i\nend\nsum",
    );
    assert_eq!(result, Some(Object::Int(6)));
}
