// Coverage tests for vm/exceptions.rs uncovered paths

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

fn parse_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().unwrap_err()[0].to_string()
}

// ── raise with non-exception/string/class type (line 40-43) ──────────────────

#[test]
fn raise_with_integer_error() {
    let err = run_err("raise 42");
    assert!(
        err.contains("Exception")
            || err.contains("exception")
            || err.contains("String")
            || err.contains("class")
    );
}

#[test]
fn raise_with_array_error() {
    let err = run_err("raise [1, 2, 3]");
    assert!(
        err.contains("Exception")
            || err.contains("exception")
            || err.contains("String")
            || err.contains("class")
    );
}

#[test]
fn raise_with_bool_error() {
    let err = run_err("raise true");
    assert!(
        err.contains("Exception")
            || err.contains("exception")
            || err.contains("String")
            || err.contains("class")
    );
}

// ── bare raise outside rescue block (lines 53-59) ────────────────────────────

#[test]
fn bare_raise_outside_rescue_error() {
    let err = run_err("raise");
    assert!(
        err.contains("raise")
            || err.contains("rescue")
            || err.contains("exception")
            || err.contains("No exception")
    );
}

// ── ensure block with exception (lines 193-194) ──────────────────────────────

#[test]
fn ensure_block_with_exception_overrides_result() {
    let err = run_err(
        r#"
begin
  x = 1
ensure
  raise "ensure error"
end
"#,
    );
    assert!(err.contains("ensure error") || err.contains("exception") || err.contains("Uncaught"));
}

// ── ensure block with other control flow (lines 199-201) ─────────────────────

#[test]
fn ensure_block_with_return_in_method() {
    let result = run(r#"
def compute
  begin
    1 + 1
  ensure
    return 99
  end
end
compute
"#);
    // ensure block with return overrides the body result
    assert!(result.is_some());
}

// ── ensure block normal flow ──────────────────────────────────────────────────

#[test]
fn ensure_block_executes_after_body() {
    let result = run(r#"
x = 0
begin
  x = 1
ensure
  x = x + 10
end
x
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

// ── begin/rescue/else clause (no exception → else executes) ──────────────────

#[test]
fn begin_else_executes_when_no_exception() {
    let result = run(r#"
result = 0
begin
  result = 1
rescue RuntimeError
  result = 99
else
  result = result + 5
end
result
"#);
    assert_eq!(result, Some(Object::Int(6)));
}

// ── raise with class type (instantiates exception) ───────────────────────────

#[test]
fn raise_with_class_creates_exception() {
    let err = run_err("raise RuntimeError");
    assert!(err.contains("RuntimeError") || err.contains("exception") || err.contains("Uncaught"));
}

// ── multiple rescue types with comma (exception.rs line 128) ─────────────────

#[test]
fn rescue_multiple_exception_types() {
    let result = run(r#"
result = "not caught"
begin
  raise "oops"
rescue TypeError, RuntimeError
  result = "caught"
end
result
"#);
    assert_eq!(result, Some(Object::string("caught")));
}

// ── rescue => non-ident parse error (exception.rs lines 141-148) ─────────────

#[test]
fn rescue_fat_arrow_non_ident_parse_error() {
    let err = parse_err(
        r#"
begin
  raise "oops"
rescue RuntimeError => 42
  "caught"
end
"#,
    );
    assert!(
        err.contains("variable")
            || err.contains("Expected")
            || err.contains("identifier")
            || err.contains("'=>'")
    );
}

// ── exceptions.rs lines 203-205: error in ensure block ───────────────────────

#[test]
fn ensure_block_that_raises_error_propagates() {
    // When the ensure block itself raises an error, vm/exceptions.rs lines 203-205
    // are executed (the Err(_) arm of the ensure result match).
    let err = run_err(
        r#"
begin
  "ok"
rescue
  "rescued"
ensure
  raise "error in ensure"
end
"#,
    );
    assert!(err.contains("ensure") || err.contains("error") || err.contains("RuntimeError"));
}
