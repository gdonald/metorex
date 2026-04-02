// Operator error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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

// ══════════════════════════════════════════════════════════════════════════════
// Operators - And/Or short-circuit paths (lines 62-64, 67-69)
// These are defensive guards; exercise normal And/Or paths thoroughly
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn operator_and_short_circuit_false() {
    let result = run(r#"
false && true
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn operator_and_short_circuit_nil() {
    let result = run(r#"
nil && 42
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn operator_and_evaluates_both() {
    let result = run(r#"
true && 42
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn operator_or_short_circuit_true() {
    let result = run(r#"
true || false
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_or_short_circuit_int() {
    let result = run(r#"
42 || false
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn operator_or_evaluates_right() {
    let result = run(r#"
false || 99
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn operator_or_nil_fallthrough() {
    let result = run(r#"
nil || "fallback"
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("fallback".to_string())))
    );
}

#[test]
fn operator_and_complex_expressions() {
    let result = run(r#"
x = 5
x > 3 && x < 10
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_or_complex_expressions() {
    let result = run(r#"
x = 5
x > 10 || x < 3
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn operator_assign_via_plus_equals() {
    let result = run(r#"
x = 10
x += 5
x
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn operator_assign_via_minus_equals() {
    let result = run(r#"
x = 10
x -= 3
x
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn operator_assign_via_multiply_equals() {
    let result = run(r#"
x = 10
x *= 2
x
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn operator_assign_via_divide_equals() {
    let result = run(r#"
x = 10
x /= 2
x
"#);
    assert_eq!(result, Some(Object::Int(5)));
}
