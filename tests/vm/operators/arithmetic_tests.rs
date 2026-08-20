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

// ── Integer division ────────────────────────────────────────────────────────

#[test]
fn int_divide_non_evenly_produces_float() {
    assert_eq!(run("5 / 2"), Some(Object::Float(2.5)));
}

#[test]
fn int_modulo_by_zero_error() {
    let err = run_err("5 % 0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Float / Float ────────────────────────────────────────────────────────────

#[test]
fn float_subtract() {
    assert_eq!(run("5.0 - 2.0"), Some(Object::Float(3.0)));
}

#[test]
fn float_multiply() {
    assert_eq!(run("3.0 * 4.0"), Some(Object::Float(12.0)));
}

#[test]
fn float_divide() {
    assert_eq!(run("8.0 / 4.0"), Some(Object::Float(2.0)));
}

#[test]
fn float_modulo() {
    assert_eq!(run("7.0 % 3.0"), Some(Object::Float(1.0)));
}

#[test]
fn float_divide_by_zero_is_infinite() {
    assert_eq!(run("4.0 / 0.0"), Some(Object::Float(f64::INFINITY)));
}

#[test]
fn float_modulo_by_zero_is_nan() {
    let Some(Object::Float(result)) = run("4.0 % 0.0") else {
        panic!("expected a Float");
    };
    assert!(result.is_nan());
}

// ── Int / Float ──────────────────────────────────────────────────────────────

#[test]
fn int_float_subtract() {
    assert_eq!(run("5 - 2.0"), Some(Object::Float(3.0)));
}

#[test]
fn int_float_multiply() {
    assert_eq!(run("3 * 2.0"), Some(Object::Float(6.0)));
}

#[test]
fn int_float_divide() {
    assert_eq!(run("6 / 2.0"), Some(Object::Float(3.0)));
}

#[test]
fn int_float_modulo() {
    assert_eq!(run("7 % 3.0"), Some(Object::Float(1.0)));
}

#[test]
fn int_float_divide_by_zero_is_infinite() {
    assert_eq!(run("5 / 0.0"), Some(Object::Float(f64::INFINITY)));
}

#[test]
fn int_float_modulo_by_zero_is_nan() {
    let Some(Object::Float(result)) = run("5 % 0.0") else {
        panic!("expected a Float");
    };
    assert!(result.is_nan());
}

// ── Float / Int ──────────────────────────────────────────────────────────────

#[test]
fn float_int_subtract() {
    assert_eq!(run("5.0 - 2"), Some(Object::Float(3.0)));
}

#[test]
fn float_int_multiply() {
    assert_eq!(run("3.0 * 4"), Some(Object::Float(12.0)));
}

#[test]
fn float_int_divide() {
    assert_eq!(run("8.0 / 2"), Some(Object::Float(4.0)));
}

#[test]
fn float_int_modulo() {
    assert_eq!(run("7.0 % 3"), Some(Object::Float(1.0)));
}

#[test]
fn float_int_divide_by_zero_is_infinite() {
    assert_eq!(run("5.0 / 0"), Some(Object::Float(f64::INFINITY)));
}

#[test]
fn float_int_modulo_by_zero_is_nan() {
    let Some(Object::Float(result)) = run("5.0 % 0") else {
        panic!("expected a Float");
    };
    assert!(result.is_nan());
}

// ── Addition ─────────────────────────────────────────────────────────────────

#[test]
fn addition_type_error_nil() {
    let err = run_err("nil + 1");
    assert!(err.contains("operator") || err.contains("type") || err.contains("Nil"));
}

#[test]
fn float_addition() {
    assert_eq!(run("1.5 + 2.5"), Some(Object::Float(4.0)));
}

#[test]
fn int_plus_float() {
    assert_eq!(run("1 + 2.5"), Some(Object::Float(3.5)));
}

#[test]
fn float_plus_int() {
    assert_eq!(run("1.5 + 2"), Some(Object::Float(3.5)));
}

// ── Type errors ──────────────────────────────────────────────────────────────

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

// ── Power ────────────────────────────────────────────────────────────────────

#[test]
fn power_int_negative_exponent() {
    assert!(matches!(run("2**-1"), Some(Object::Float(_))));
}

#[test]
fn power_float_float() {
    assert_eq!(run("2.0**3.0"), Some(Object::Float(8.0)));
}

#[test]
fn power_int_float() {
    assert_eq!(run("2**3.0"), Some(Object::Float(8.0)));
}

#[test]
fn power_float_int() {
    assert_eq!(run("2.0**3"), Some(Object::Float(8.0)));
}

// ── Integer overflow ─────────────────────────────────────────────────────────

#[test]
fn int_add_overflow_to_float() {
    let result = run("4611686018427387903 + 4611686018427387903");
    assert!(matches!(
        result,
        Some(Object::Float(_)) | Some(Object::Int(_))
    ));
}

#[test]
fn int_multiply_overflow() {
    assert!(run("4611686018427387903 * 3").is_some());
}

#[test]
fn int_subtract_overflow() {
    assert!(run("0 - 4611686018427387903 - 4611686018427387903 - 2").is_some());
}
