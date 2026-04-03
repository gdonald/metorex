// String error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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

// ══════════════════════════════════════════════════════════════════════════════
// String methods - chars, bytes, various transforms (lines 58-76)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_chars_returns_array_of_chars() {
    let result = run(r#"
"hello".chars.length
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn string_bytes_returns_array_of_ints() {
    let result = run(r#"
"abc".bytes.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn string_trim_method() {
    let result = run(r#"
"  hello  ".trim
"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_strip_method() {
    let result = run(r#"
"  hello  ".strip
"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_reverse_method() {
    let result = run(r#"
"hello".reverse
"#);
    assert_eq!(result, Some(Object::string("olleh")));
}

#[test]
fn string_upcase_method() {
    let result = run(r#"
"hello".upcase
"#);
    assert_eq!(result, Some(Object::string("HELLO")));
}

#[test]
fn string_downcase_method() {
    let result = run(r#"
"HELLO".downcase
"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_size_method() {
    let result = run(r#"
"hello".size
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ══════════════════════════════════════════════════════════════════════════════
// String methods - slice (line 216 nil return for out-of-bounds)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_slice_out_of_bounds_returns_nil() {
    // start_idx > chars.len() triggers the nil return
    let result = run(r#"
"hi".slice(100, 2)
"#);
    // When start is beyond length, returns nil or empty string
    assert!(
        result == Some(Object::Nil) || result == Some(Object::string("")),
        "Got: {:?}",
        result
    );
}

#[test]
fn string_slice_negative_start_wraps() {
    let result = run(r#"
"hello".slice(-2, 2)
"#);
    assert_eq!(result, Some(Object::string("lo")));
}

// ══════════════════════════════════════════════════════════════════════════════
// String methods - each_char block execution (lines 297-302)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_each_char_accumulates_chars() {
    let result = run(r#"
collected = []
"xyz".each_char do |c|
  collected.push(c)
end
collected.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ══════════════════════════════════════════════════════════════════════════════
// String methods - + concatenation non-string error (lines 72-77)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_plus_non_string_type_error() {
    let err = run_err(
        r#"
"hello" + 42
"#,
    );
    assert!(
        err.contains("String") || err.contains("type"),
        "Error was: {}",
        err
    );
}

#[test]
fn string_plus_nil_type_error() {
    let err = run_err(
        r#"
"hello" + nil
"#,
    );
    assert!(
        err.contains("String") || err.contains("type") || err.contains("nil"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// String include? and starts_with? / ends_with?
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_include_method() {
    let result = run(r#"
"hello world".include?("world")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_starts_with_method() {
    let result = run(r#"
"hello".starts_with?("hel")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_ends_with_method() {
    let result = run(r#"
"hello".ends_with?("llo")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ══════════════════════════════════════════════════════════════════════════════
// String split method
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_split_by_separator() {
    let result = run(r#"
"a,b,c".split(",").length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn string_split_by_whitespace() {
    let result = run(r#"
"hello world".split.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}
