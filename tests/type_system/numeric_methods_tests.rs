// Tests for Integer and Float native methods

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

// ============================================================================
// Integer#abs
// ============================================================================

#[test]
fn int_abs_positive() {
    let result = run("42.abs");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn int_abs_negative() {
    let result = run("x = -7\nx.abs");
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn int_abs_zero() {
    let result = run("0.abs");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn int_abs_error_with_args() {
    let err = run_err("42.abs(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// Integer#to_f
// ============================================================================

#[test]
fn int_to_f() {
    let result = run("42.to_f");
    assert_eq!(result, Some(Object::Float(42.0)));
}

#[test]
fn int_to_f_negative() {
    let result = run("x = -3\nx.to_f");
    assert_eq!(result, Some(Object::Float(-3.0)));
}

#[test]
fn int_to_f_error_with_args() {
    let err = run_err("42.to_f(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// Integer#to_i
// ============================================================================

#[test]
fn int_to_i_returns_self() {
    let result = run("42.to_i");
    assert_eq!(result, Some(Object::Int(42)));
}

// ============================================================================
// Integer#to_s
// ============================================================================

#[test]
fn int_to_s() {
    let result = run("42.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("42".to_string()))));
}

#[test]
fn int_to_s_negative() {
    let result = run("x = -7\nx.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("-7".to_string()))));
}

// ============================================================================
// Integer#times
// ============================================================================

#[test]
fn int_times_basic() {
    let result = run(r#"
sum = 0
5.times { |i| sum += i }
sum
"#);
    // 0 + 1 + 2 + 3 + 4 = 10
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn int_times_zero() {
    let result = run(r#"
count = 0
0.times { |i| count += 1 }
count
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn int_times_with_do_block() {
    let result = run(r#"
arr = []
3.times do |i|
  arr.push(i)
end
arr
"#);
    if let Some(Object::Array(arr)) = result {
        let arr = arr.borrow();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Object::Int(0));
        assert_eq!(arr[1], Object::Int(1));
        assert_eq!(arr[2], Object::Int(2));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn int_times_returns_self() {
    let result = run("5.times { |i| i }");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn int_times_with_break() {
    let result = run(r#"
count = 0
10.times do |i|
  count += 1
  if i == 3
    break
  end
end
count
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn int_times_no_block_returns_range_array() {
    let result = run("5.times");
    let arr = match result {
        Some(Object::Array(a)) => a,
        other => panic!("expected array, got {other:?}"),
    };
    let expected: Vec<Object> = (0..5).map(Object::Int).collect();
    assert_eq!(*arr.borrow(), expected);
}

#[test]
fn int_times_error_with_args() {
    let err = run_err("5.times(1) { |i| i }");
    assert!(err.contains("argument"));
}

// ============================================================================
// Float#abs
// ============================================================================

#[test]
fn float_abs_positive() {
    let result = run("3.14.abs");
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn float_abs_negative() {
    let result = run("x = -2.5\nx.abs");
    assert_eq!(result, Some(Object::Float(2.5)));
}

#[test]
fn float_abs_error_with_args() {
    let err = run_err("3.14.abs(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// Float#ceil
// ============================================================================

#[test]
fn float_ceil() {
    let result = run("3.2.ceil");
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn float_ceil_negative() {
    let result = run("x = -2.7\nx.ceil");
    assert_eq!(result, Some(Object::Int(-2)));
}

#[test]
fn float_ceil_whole() {
    let result = run("5.0.ceil");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn float_ceil_error_with_args() {
    let err = run_err("3.14.ceil(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// Float#floor
// ============================================================================

#[test]
fn float_floor() {
    let result = run("3.7.floor");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn float_floor_negative() {
    let result = run("x = -2.3\nx.floor");
    assert_eq!(result, Some(Object::Int(-3)));
}

#[test]
fn float_floor_error_with_args() {
    let err = run_err("3.14.floor(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// Float#to_i
// ============================================================================

#[test]
fn float_to_i() {
    let result = run("3.7.to_i");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn float_to_i_negative() {
    let result = run("x = -2.9\nx.to_i");
    assert_eq!(result, Some(Object::Int(-2)));
}

#[test]
fn float_to_i_error_with_args() {
    let err = run_err("3.14.to_i(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// Float#to_f
// ============================================================================

#[test]
fn float_to_f_returns_self() {
    let result = run("3.14.to_f");
    assert_eq!(result, Some(Object::Float(3.14)));
}

// ============================================================================
// Float#to_s
// ============================================================================

#[test]
fn float_to_s() {
    let result = run("3.14.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("3.14".to_string()))));
}

#[test]
fn float_to_s_error_with_args() {
    let err = run_err("3.14.to_s(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// Additional error paths for coverage
// ============================================================================

#[test]
fn int_to_i_error_with_args() {
    let err = run_err("42.to_i(1)");
    assert!(err.contains("argument"));
}

#[test]
fn int_to_s_error_with_args() {
    let err = run_err("42.to_s(1)");
    assert!(err.contains("argument"));
}

#[test]
fn float_to_f_error_with_args() {
    let err = run_err("3.14.to_f(1)");
    assert!(err.contains("argument"));
}

#[test]
fn int_times_with_exception_in_block() {
    let err = run_err(
        r#"
3.times do |i|
  if i == 1
    raise "error in times"
  end
end
"#,
    );
    assert!(err.contains("error in times") || err.contains("exception"));
}

#[test]
fn float_abs_zero() {
    let result = run("0.0.abs");
    assert_eq!(result, Some(Object::Float(0.0)));
}

#[test]
fn float_floor_whole() {
    let result = run("5.0.floor");
    assert_eq!(result, Some(Object::Int(5)));
}
