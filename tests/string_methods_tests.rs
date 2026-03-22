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

#[test]
fn string_length() {
    let result = run(r#""hello".length"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn string_size_alias() {
    let result = run(r#""hello".size"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn string_upcase() {
    let result = run(r#""hello".upcase"#);
    assert_eq!(result, Some(Object::string("HELLO")));
}

#[test]
fn string_downcase() {
    let result = run(r#""HELLO".downcase"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_reverse() {
    let result = run(r#""hello".reverse"#);
    assert_eq!(result, Some(Object::string("olleh")));
}

#[test]
fn string_trim() {
    let result = run(r#""  hello  ".trim"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_strip_alias() {
    let result = run(r#""  hello  ".strip"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_split_with_separator() {
    let result = run(r#""one,two,three".split(",")"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("one"),
            Object::string("two"),
            Object::string("three"),
        ]))
    );
}

#[test]
fn string_split_whitespace() {
    let result = run(r#""hello world".split"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("hello"),
            Object::string("world"),
        ]))
    );
}

#[test]
fn string_slice() {
    let result = run(r#""hello world".slice(0, 5)"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_slice_negative_index() {
    let result = run(r#""hello world".slice(-5, 5)"#);
    assert_eq!(result, Some(Object::string("world")));
}

#[test]
fn string_include_true() {
    let result = run(r#""hello world".include?("hello")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_include_false() {
    let result = run(r#""hello world".include?("xyz")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_contains_alias() {
    let result = run(r#""hello world".contains?("world")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_starts_with_true() {
    let result = run(r#""hello world".starts_with?("hello")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_starts_with_false() {
    let result = run(r#""hello world".starts_with?("world")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_ends_with_true() {
    let result = run(r#""hello world".ends_with?("world")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_ends_with_false() {
    let result = run(r#""hello world".ends_with?("hello")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn array_join_with_separator() {
    let result = run(r#"["one", "two", "three"].join(", ")"#);
    assert_eq!(result, Some(Object::string("one, two, three")));
}

#[test]
fn array_join_no_separator() {
    let result = run(r#"["one", "two", "three"].join"#);
    assert_eq!(result, Some(Object::string("onetwothree")));
}
