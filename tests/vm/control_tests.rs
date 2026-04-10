// Coverage tests for vm/control_structures.rs uncovered paths

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

// ── return inside while loop (propagates ControlFlow::Return) ─────────────────

#[test]
fn return_inside_while_loop_exits_method() {
    let result = run(r#"
def find_first_even(arr)
  i = 0
  while i < arr.length
    if arr[i] % 2 == 0
      return arr[i]
    end
    i = i + 1
  end
  nil
end
find_first_even([1, 3, 4, 5, 6])
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

// ── raise inside while loop (propagates ControlFlow::Exception) ───────────────

#[test]
fn raise_inside_while_loop_propagates_exception() {
    let err = run_err(
        r#"
i = 0
while i < 3
  i = i + 1
  raise "loop error"
end
"#,
    );
    assert!(err.contains("loop error") || err.contains("exception") || err.contains("Uncaught"));
}

// ── return inside for loop (propagates ControlFlow::Return) ───────────────────

#[test]
fn return_inside_for_loop_exits_method() {
    let result = run(r#"
def find_value(arr, target)
  for item in arr
    if item == target
      return true
    end
  end
  false
end
find_value([1, 2, 3, 4, 5], 3)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── raise inside for loop (propagates ControlFlow::Exception) ─────────────────

#[test]
fn raise_inside_for_loop_propagates_exception() {
    let err = run_err(
        r#"
for i in [1, 2, 3]
  raise "for loop error"
end
"#,
    );
    assert!(
        err.contains("for loop error") || err.contains("exception") || err.contains("Uncaught")
    );
}

// ── Reverse range in for loop (line 129-130) ──────────────────────────────────

#[test]
fn for_loop_with_reverse_range() {
    let result = run(r#"
result = []
for i in 5..3
  result.push(i)
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Float range bounds error in for loop (lines 136-138) ─────────────────────

#[test]
fn for_loop_float_range_bounds_error() {
    let err = run_err(
        r#"
for i in 1.5..3.5
  i
end
"#,
    );
    assert!(
        err.contains("integer")
            || err.contains("Integer")
            || err.contains("Range")
            || err.contains("bounds")
    );
}

// ── Non-iterable type in for loop ────────────────────────────────────────────

#[test]
fn for_loop_on_non_iterable_error() {
    let err = run_err(
        r#"
for i in 42
  i
end
"#,
    );
    assert!(
        err.contains("iterate")
            || err.contains("Array")
            || err.contains("Range")
            || err.contains("type")
    );
}

// ── From remaining_tests ────────────────────────────────────────────────────

fn run_ctrl(code: &str) -> Option<metorex::object::Object> {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    use metorex::vm::VirtualMachine;
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

#[test]
fn break_in_while_loop_remaining() {
    assert_eq!(
        run_ctrl("x = 0; while true; x = x + 1; break if x >= 3; end; x"),
        Some(metorex::object::Object::Int(3))
    );
}
