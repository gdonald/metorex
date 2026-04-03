// Coverage tests for string native methods

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

// ── String + non-string argument error ────────────────────────────────────────

#[test]
fn string_concat_non_string_error() {
    let err = run_err(
        r#"
"hello" + 42
"#,
    );
    assert!(err.contains("String") || err.contains("type") || err.contains("+"));
}

// ── String each_char block path ─────────────────────────────────────────

#[test]
fn string_each_char_basic() {
    let result = run("result = []\n\"abc\".each_char do |c|\n  result.push(c)\nend\nresult.length");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── string_methods.rs: "+" concatenation (lines 58-76) ──────────────────────

#[test]
fn string_plus_concatenation() {
    let result = run(r#"
"hello" + " " + "world"
"#);
    assert_eq!(result, Some(Object::string("hello world")));
}

// ── string_methods.rs: slice/[] operation (line 216) ────────────────────────

#[test]
fn string_slice_basic() {
    let result = run(r#"
"hello".slice(1, 3)
"#);
    assert_eq!(result, Some(Object::string("ell")));
}

#[test]
fn string_slice_beyond_end_returns_empty() {
    let result = run(r#"
"hi".slice(10, 2)
"#);
    assert_eq!(result, Some(Object::string("")));
}

#[test]
fn string_slice_negative_start() {
    let result = run(r#"
"hello".slice(-3, 2)
"#);
    assert_eq!(result, Some(Object::string("ll")));
}

// ── string_methods.rs: each_char iteration with block (lines 297-302) ───────

#[test]
fn string_each_char_returns_receiver() {
    let result = run(r#"
result = "abc".each_char do |c|
  c
end
result
"#);
    assert_eq!(result, Some(Object::string("abc")));
}

#[test]
fn string_each_char_without_block_error() {
    let err = run_err(
        r#"
"abc".each_char
"#,
    );
    assert!(err.contains("block") || err.contains("each_char") || err.contains("requires"));
}

// ── String concatenation ──────────────────────────────────────────────────

#[test]
fn string_concatenation_with_plus() {
    let result = run(r#"
"hello" + " world"
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("hello world".to_string())))
    );
}

#[test]
fn string_chars_method() {
    let result = run(r#"
"abc".chars
"#);
    assert!(result.is_some());
}

#[test]
fn string_bytes_method() {
    let result = run(r#"
"abc".bytes
"#);
    assert!(result.is_some());
}

#[test]
fn string_each_char_with_block() {
    let result = run(r#"
result = []
"abc".each_char do |c|
  result.push(c)
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}
