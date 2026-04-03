// Coverage tests for int native methods

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

// ── Int times with break ────────────────────────────────────────────────

#[test]
fn int_times_with_break_coverage() {
    let result =
        run("sum = 0\n10.times do |i|\n  if i == 5\n    break\n  end\n  sum = sum + i\nend\nsum");
    assert_eq!(result, Some(Object::Int(10)));
}

// ── int_methods.rs: times with continue in block (lines 79-84) ─────────────

#[test]
fn int_times_with_continue_in_block() {
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
    // 0 + 1 + 3 + 4 = 8 (skipping i==2)
    assert_eq!(result, Some(Object::Int(8)));
}

// ── int_methods.rs: times with return in block error (lines 102-104) ────────

#[test]
fn int_times_return_in_block_error() {
    let err = run_err(
        r#"
5.times do |i|
  return i
end
"#,
    );
    assert!(err.contains("return") || err.contains("control") || err.contains("loop"));
}

// ── int_methods.rs: times with exception in block ───────────────────────────

#[test]
fn int_times_exception_in_block_propagates() {
    let err = run_err(
        r#"
3.times do |i|
  raise "times block error"
end
"#,
    );
    assert!(
        err.contains("times block error") || err.contains("Uncaught") || err.contains("exception")
    );
}

// ── Int times with continue (additional path) ─────────────────────────────

#[test]
fn int_times_continue_skips_iteration() {
    let result = run(r#"
result = []
5.times do |i|
  if i == 2
    continue
  end
  result.push(i)
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(4)));
}
