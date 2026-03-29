// Tests for Hash native method coverage

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

// ── keys ────────────────────────────────────────────────────────────────────

#[test]
fn hash_keys_returns_array() {
    let result = run(r#"{"a" => 1, "b" => 2}.keys.length"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_keys_error_with_args() {
    let err = run_err(r#"{"a" => 1}.keys(1)"#);
    assert!(err.contains("argument"));
}

// ── values ──────────────────────────────────────────────────────────────────

#[test]
fn hash_values_returns_array() {
    let result = run(r#"{"a" => 1, "b" => 2}.values.length"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_values_error_with_args() {
    let err = run_err(r#"{"a" => 1}.values(1)"#);
    assert!(err.contains("argument"));
}

// ── has_key? ─────────────────────────────────────────────────────────────────

#[test]
fn hash_has_key_string() {
    let result = run(r#"{"a" => 1}.has_key?("a")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_missing() {
    let result = run(r#"{"a" => 1}.has_key?("z")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn hash_has_key_float_key() {
    let result = run(r#"{1.5 => "x"}.has_key?(1.5)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_bool_key() {
    let result = run(r#"{true => "x"}.has_key?(true)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_nil_key() {
    let result = run(r#"{nil => "x"}.has_key?(nil)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_error_no_args() {
    let err = run_err(r#"{"a" => 1}.has_key?"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_has_key_error_wrong_type() {
    let err = run_err(r#"{"a" => 1}.has_key?([1, 2])"#);
    assert!(err.contains("String"));
}

#[test]
fn hash_key_alias() {
    let result = run(r#"{"a" => 1}.key?("a")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── entries / to_a ──────────────────────────────────────────────────────────

#[test]
fn hash_entries_returns_pairs() {
    let result = run(r#"{"a" => 1}.entries.length"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_to_a_alias() {
    let result = run(r#"{"a" => 1}.to_a.length"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_entries_error_with_args() {
    let err = run_err(r#"{"a" => 1}.entries(1)"#);
    assert!(err.contains("argument"));
}

// ── length / size ────────────────────────────────────────────────────────────

#[test]
fn hash_length() {
    let result = run(r#"{"a" => 1, "b" => 2, "c" => 3}.length"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn hash_size_alias() {
    let result = run(r#"{"a" => 1}.size"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_length_error_with_args() {
    let err = run_err(r#"{"a" => 1}.length(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_size_error_with_args() {
    let err = run_err(r#"{"a" => 1}.size(1)"#);
    assert!(err.contains("argument"));
}

// ── has_key? with integer key ────────────────────────────────────────────────

#[test]
fn hash_has_key_integer_key_true() {
    let result = run(r#"{1 => "x"}.has_key?(1)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_integer_key_false() {
    let result = run(r#"{1 => "x"}.has_key?(2)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}
