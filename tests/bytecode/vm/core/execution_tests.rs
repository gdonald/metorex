// Tests for basic bytecode VM execution: literals, arithmetic, comparison,
// variables, control flow, functions (13.3-13.7).

use metorex::bytecode::vm::BytecodeVm;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use std::rc::Rc;

fn run(source: &str) -> Result<Object, String> {
    let tokens = Lexer::new(source).tokenize();
    let stmts = Parser::new(tokens).parse().map_err(|errs| {
        errs.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    })?;
    let compiler = Compiler::new();
    let chunk = compiler.compile(&stmts).map_err(|e| e.to_string())?;
    let mut vm = BytecodeVm::new();
    vm.execute(&chunk).map_err(|e| e.to_string())
}

fn run_ok(source: &str) -> Object {
    run(source).expect("execution failed")
}

// ── 13.3-13.4 Basic Instruction Execution ───────────────────────────

#[test]
fn execute_empty_program() {
    let result = run_ok("");
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_int_literal() {
    let result = run_ok("42");
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_return_int() {
    let result = run_ok("return 42");
    assert_eq!(result, Object::Int(42));
}

#[test]
fn execute_return_float() {
    let result = run_ok("return 3.14");
    assert_eq!(result, Object::Float(3.14));
}

#[test]
fn execute_return_string() {
    let result = run_ok("return \"hello\"");
    assert_eq!(result, Object::String(Rc::new("hello".to_string())));
}

#[test]
fn execute_return_true() {
    let result = run_ok("return true");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_return_false() {
    let result = run_ok("return false");
    assert_eq!(result, Object::Bool(false));
}

#[test]
fn execute_return_nil() {
    let result = run_ok("return nil");
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_addition() {
    let result = run_ok("return 1 + 2");
    assert_eq!(result, Object::Int(3));
}

#[test]
fn execute_subtraction() {
    let result = run_ok("return 10 - 3");
    assert_eq!(result, Object::Int(7));
}

#[test]
fn execute_multiplication() {
    let result = run_ok("return 4 * 5");
    assert_eq!(result, Object::Int(20));
}

#[test]
fn execute_division() {
    let result = run_ok("return 10 / 3");
    assert_eq!(result, Object::Int(3));
}

#[test]
fn execute_modulo() {
    let result = run_ok("return 10 % 3");
    assert_eq!(result, Object::Int(1));
}

#[test]
fn execute_float_arithmetic() {
    let result = run_ok("return 1.5 + 2.5");
    assert_eq!(result, Object::Float(4.0));
}

#[test]
fn execute_string_concatenation() {
    let result = run_ok("return \"hello\" + \" world\"");
    assert_eq!(result, Object::String(Rc::new("hello world".to_string())));
}

#[test]
fn execute_negate() {
    let result = run_ok("return -42");
    assert_eq!(result, Object::Int(-42));
}

#[test]
fn execute_not() {
    let result = run_ok("return !true");
    assert_eq!(result, Object::Bool(false));
}

#[test]
fn execute_comparison_equal() {
    let result = run_ok("return 1 == 1");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_not_equal() {
    let result = run_ok("return 1 != 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_less() {
    let result = run_ok("return 1 < 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_greater() {
    let result = run_ok("return 2 > 1");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_less_equal() {
    let result = run_ok("return 1 <= 1");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_greater_equal() {
    let result = run_ok("return 2 >= 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_division_by_zero_error() {
    let result = run("return 1 / 0");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Division by zero"));
}

#[test]
fn execute_negate_string_error() {
    let result = run("return -\"hello\"");
    assert!(result.is_err());
}

// ── 13.5 Variable Access Execution ──────────────────────────────────

#[test]
fn execute_global_variable() {
    let result = run_ok("x = 42\nreturn x");
    assert_eq!(result, Object::Int(42));
}

#[test]
fn execute_global_variable_reassignment() {
    let result = run_ok("x = 1\nx = 2\nreturn x");
    assert_eq!(result, Object::Int(2));
}

#[test]
fn execute_multiple_globals() {
    let result = run_ok("x = 10\ny = 20\nreturn x + y");
    assert_eq!(result, Object::Int(30));
}

#[test]
fn execute_undefined_variable_error() {
    let result = run("return undefined_var");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Undefined variable"));
}

// ── 13.6 Control Flow Execution ─────────────────────────────────────

#[test]
fn execute_if_true_branch() {
    let result = run_ok("if true\n  return 1\nend");
    assert_eq!(result, Object::Int(1));
}

#[test]
fn execute_if_false_skips() {
    let result = run_ok("if false\n  return 1\nend\nreturn 2");
    assert_eq!(result, Object::Int(2));
}

#[test]
fn execute_if_else() {
    let result = run_ok("if false\n  return 1\nelse\n  return 2\nend");
    assert_eq!(result, Object::Int(2));
}

#[test]
fn execute_while_loop() {
    let result = run_ok(
        "def count\n  x = 0\n  while x < 5\n    x = x + 1\n  end\n  return x\nend\nreturn count()",
    );
    assert_eq!(result, Object::Int(5));
}

#[test]
fn execute_while_false_skips() {
    let result = run_ok("while false\n  return 1\nend\nreturn 2");
    assert_eq!(result, Object::Int(2));
}

// ── 13.7 Function Call Execution ────────────────────────────────────

#[test]
fn execute_function_definition_and_call() {
    let result = run_ok("def add(a, b)\n  return a + b\nend\nreturn add(3, 4)");
    assert_eq!(result, Object::Int(7));
}

#[test]
fn execute_function_wrong_arg_count_error() {
    let result = run("def f(a)\n  return a\nend\nreturn f(1, 2)");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Expected 1 arguments but got 2")
    );
}

#[test]
fn execute_call_non_function_error() {
    let result = run("x = 42\nreturn x(1)");
    assert!(result.is_err());
}

#[test]
fn execute_nested_arithmetic() {
    let result = run_ok("return (2 + 3) * (4 - 1)");
    assert_eq!(result, Object::Int(15));
}
