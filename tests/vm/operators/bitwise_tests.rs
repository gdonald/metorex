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

// ── BitwiseAnd (&) ──────────────────────────────────────────────────────────

#[test]
fn bitwise_and_bool_bool() {
    assert_eq!(run("true & false"), Some(Object::Bool(false)));
}

#[test]
fn bitwise_and_int_int() {
    assert_eq!(run("12 & 10"), Some(Object::Int(8)));
}

#[test]
fn bitwise_and_bool_int() {
    assert_eq!(run("true & 0"), Some(Object::Bool(true)));
}

#[test]
fn bitwise_and_int_bool() {
    assert_eq!(run("1 & true"), Some(Object::Bool(true)));
}

#[test]
fn bitwise_and_type_error() {
    let err = run_err("'a' & 'b'");
    assert!(err.contains("type") || err.contains("Cannot"));
}

#[test]
fn bitwise_and_nil_left() {
    assert_eq!(run("nil & true"), Some(Object::Bool(false)));
}

#[test]
fn bitwise_and_nil_right() {
    assert_eq!(run("true & nil"), Some(Object::Bool(false)));
}

// ── BitwiseOr (|) ───────────────────────────────────────────────────────────

#[test]
fn bitwise_or_bool_bool() {
    assert_eq!(run("false | true"), Some(Object::Bool(true)));
}

#[test]
fn bitwise_or_int_int() {
    assert_eq!(run("12 | 3"), Some(Object::Int(15)));
}

#[test]
fn bitwise_or_bool_int() {
    assert_eq!(run("false | 1"), Some(Object::Bool(true)));
}

#[test]
fn bitwise_or_int_bool() {
    assert_eq!(run("0 | false"), Some(Object::Bool(true)));
}

#[test]
fn bitwise_or_type_error() {
    let err = run_err("'a' | 'b'");
    assert!(err.contains("type") || err.contains("Cannot"));
}

#[test]
fn bitwise_or_nil_left() {
    assert_eq!(run("nil | true"), Some(Object::Bool(true)));
}

#[test]
fn bitwise_or_nil_left_false() {
    assert_eq!(run("nil | false"), Some(Object::Bool(false)));
}

// ── Xor (^) ─────────────────────────────────────────────────────────────────

#[test]
fn xor_bool_true_false() {
    assert_eq!(run("true ^ false"), Some(Object::Bool(true)));
}

#[test]
fn xor_bool_true_true() {
    assert_eq!(run("true ^ true"), Some(Object::Bool(false)));
}

#[test]
fn xor_bool_false_false() {
    assert_eq!(run("false ^ false"), Some(Object::Bool(false)));
}

#[test]
fn xor_int() {
    assert_eq!(run("5 ^ 3"), Some(Object::Int(6)));
}

#[test]
fn xor_bool_with_truthy() {
    assert_eq!(run("true ^ nil"), Some(Object::Bool(true)));
}

#[test]
fn xor_non_bool_left() {
    assert_eq!(run(r#""hello" ^ true"#), Some(Object::Bool(false)));
}

#[test]
fn xor_non_bool_left_false() {
    assert_eq!(run("nil ^ false"), Some(Object::Bool(false)));
}

#[test]
fn xor_nil_left_truthy() {
    assert_eq!(run("nil ^ true"), Some(Object::Bool(true)));
}

#[test]
fn xor_nil_left_falsy() {
    assert_eq!(run("nil ^ false"), Some(Object::Bool(false)));
}

#[test]
fn xor_bool_truthy_other() {
    assert_eq!(run("true ^ 0"), Some(Object::Bool(false)));
}

#[test]
fn xor_other_bool() {
    assert_eq!(run("0 ^ true"), Some(Object::Bool(false)));
}

#[test]
fn xor_bool_int() {
    assert_eq!(run("true ^ 0"), Some(Object::Bool(false)));
}

#[test]
fn xor_int_bool() {
    assert_eq!(run("1 ^ false"), Some(Object::Bool(true)));
}

#[test]
fn xor_type_error() {
    let err = run_err("'a' ^ 'b'");
    assert!(err.contains("type") || err.contains("Cannot"));
}
