// Coverage tests for vm/operators.rs — unary plus, float arithmetic,
// mixed int/float arithmetic, not-equal, and type-error paths.

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

// ── UnaryOp::Plus ─────────────────────────────────────────────────────────────

#[test]
fn unary_plus_int() {
    let result = run("+3");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn unary_plus_float() {
    let result = run("+3.14");
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn unary_plus_error_on_string() {
    let err = run_err(r#"+"hello""#);
    assert!(err.contains("unary") || err.contains("+") || err.contains("operator"));
}

// ── UnaryOp::Minus on Float (line 32) ────────────────────────────────────────

#[test]
fn unary_minus_float() {
    let result = run("-3.14");
    assert_eq!(result, Some(Object::Float(-3.14)));
}

// ── UnaryOp::Minus error ──────────────────────────────────────────────────────

#[test]
fn unary_minus_error_on_string() {
    let err = run_err(r#"-"hello""#);
    assert!(err.contains("unary") || err.contains("-") || err.contains("operator"));
}

// ── NotEqual ──────────────────────────────────────────────────────────────────

#[test]
fn not_equal_ints_true() {
    let result = run("3 != 4");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn not_equal_ints_false() {
    let result = run("3 != 3");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn equal_ints_true() {
    let result = run("5 == 5");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Integer division producing Float ─────────────────────────────────────────

#[test]
fn int_divide_non_evenly_produces_float() {
    let result = run("5 / 2");
    assert_eq!(result, Some(Object::Float(2.5)));
}

#[test]
fn int_modulo_by_zero_error() {
    let err = run_err("5 % 0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Float / Float arithmetic ──────────────────────────────────────────────────

#[test]
fn float_subtract() {
    let result = run("5.0 - 2.0");
    assert_eq!(result, Some(Object::Float(3.0)));
}

#[test]
fn float_multiply() {
    let result = run("3.0 * 4.0");
    assert_eq!(result, Some(Object::Float(12.0)));
}

#[test]
fn float_divide() {
    let result = run("8.0 / 4.0");
    assert_eq!(result, Some(Object::Float(2.0)));
}

#[test]
fn float_modulo() {
    let result = run("7.0 % 3.0");
    assert_eq!(result, Some(Object::Float(1.0)));
}

#[test]
fn float_divide_by_zero_error() {
    let err = run_err("4.0 / 0.0");
    assert!(err.contains("zero") || err.contains("Division"));
}

#[test]
fn float_modulo_by_zero_error() {
    let err = run_err("4.0 % 0.0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Int / Float arithmetic ────────────────────────────────────────────────────

#[test]
fn int_float_subtract() {
    let result = run("5 - 2.0");
    assert_eq!(result, Some(Object::Float(3.0)));
}

#[test]
fn int_float_multiply() {
    let result = run("3 * 2.0");
    assert_eq!(result, Some(Object::Float(6.0)));
}

#[test]
fn int_float_divide() {
    let result = run("6 / 2.0");
    assert_eq!(result, Some(Object::Float(3.0)));
}

#[test]
fn int_float_modulo() {
    let result = run("7 % 3.0");
    assert_eq!(result, Some(Object::Float(1.0)));
}

#[test]
fn int_float_divide_by_zero_error() {
    let err = run_err("5 / 0.0");
    assert!(err.contains("zero") || err.contains("Division"));
}

#[test]
fn int_float_modulo_by_zero_error() {
    let err = run_err("5 % 0.0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Float / Int arithmetic ────────────────────────────────────────────────────

#[test]
fn float_int_subtract() {
    let result = run("5.0 - 2");
    assert_eq!(result, Some(Object::Float(3.0)));
}

#[test]
fn float_int_multiply() {
    let result = run("3.0 * 4");
    assert_eq!(result, Some(Object::Float(12.0)));
}

#[test]
fn float_int_divide() {
    let result = run("8.0 / 2");
    assert_eq!(result, Some(Object::Float(4.0)));
}

#[test]
fn float_int_modulo() {
    let result = run("7.0 % 3");
    assert_eq!(result, Some(Object::Float(1.0)));
}

#[test]
fn float_int_divide_by_zero_error() {
    let err = run_err("5.0 / 0");
    assert!(err.contains("zero") || err.contains("Division"));
}

#[test]
fn float_int_modulo_by_zero_error() {
    let err = run_err("5.0 % 0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Comparison type errors ────────────────────────────────────────────────────

#[test]
fn comparison_string_int_error() {
    let err = run_err(r#""a" > 1"#);
    assert!(err.contains("operator") || err.contains("type") || err.contains("String"));
}

#[test]
fn comparison_float_string_error() {
    let err = run_err(r#"1.5 < "b""#);
    assert!(err.contains("operator") || err.contains("type") || err.contains("Float"));
}

// ── Addition type error (non-string + non-numeric) ────────────────────────────

#[test]
fn addition_type_error_nil() {
    let err = run_err("nil + 1");
    assert!(err.contains("operator") || err.contains("type") || err.contains("Nil"));
}

// ── Float/Float addition ──────────────────────────────────────────────────────

#[test]
fn float_addition() {
    let result = run("1.5 + 2.5");
    assert_eq!(result, Some(Object::Float(4.0)));
}

// ── Int + Float and Float + Int addition ─────────────────────────────────────

#[test]
fn int_plus_float() {
    let result = run("1 + 2.5");
    assert_eq!(result, Some(Object::Float(3.5)));
}

#[test]
fn float_plus_int() {
    let result = run("1.5 + 2");
    assert_eq!(result, Some(Object::Float(3.5)));
}

// ── Float comparison ──────────────────────────────────────────────────────────

#[test]
fn float_comparison_less() {
    let result = run("1.5 < 2.5");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn float_int_comparison() {
    let result = run("1.5 > 1");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Numeric binary type error (line 183) ─────────────────────────────────────
// Triggered when neither operand is a numeric type for -, *, /, %

#[test]
fn subtract_type_error_nil_minus_int() {
    let err = run_err("nil - 1");
    assert!(err.contains("operator") || err.contains("type") || err.contains("Nil"));
}

#[test]
fn multiply_type_error_string_times_string() {
    let err = run_err(r#""a" * "b""#);
    assert!(err.contains("operator") || err.contains("type") || err.contains("String"));
}

#[test]
fn int_float_comparison() {
    let result = run("2 >= 1.5");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── operators.rs: And/Or error path (lines 62-64) ───────────────────────────
// These should never be reached in normal execution because And/Or are
// handled by short-circuit evaluation. We cannot easily trigger them
// through normal code execution, but we test the short-circuit paths work.

#[test]
fn logical_and_short_circuit() {
    let result = run("false && 1 / 0");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn logical_or_short_circuit() {
    let result = run("true || 1 / 0");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── operators.rs: assignment error path (lines 67-69) ───────────────────────
// These should never be reached because assignments are handled by
// statement execution. We test that normal assignments work.

#[test]
fn add_assign_operator() {
    let result = run("x = 5\nx += 3\nx");
    assert_eq!(result, Some(Object::Int(8)));
}

#[test]
fn subtract_assign_operator() {
    let result = run("x = 10\nx -= 3\nx");
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn multiply_assign_operator() {
    let result = run("x = 4\nx *= 3\nx");
    assert_eq!(result, Some(Object::Int(12)));
}

#[test]
fn divide_assign_operator() {
    let result = run("x = 12\nx /= 4\nx");
    assert_eq!(result, Some(Object::Int(3)));
}
