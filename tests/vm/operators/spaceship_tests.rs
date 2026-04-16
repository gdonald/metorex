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

#[test]
fn spaceship_int_int() {
    assert_eq!(run("1 <=> 2"), Some(Object::Int(-1)));
    assert_eq!(run("2 <=> 2"), Some(Object::Int(0)));
    assert_eq!(run("3 <=> 2"), Some(Object::Int(1)));
}

#[test]
fn spaceship_float_float() {
    assert_eq!(run("1.0 <=> 2.0"), Some(Object::Int(-1)));
}

#[test]
fn spaceship_int_float() {
    assert_eq!(run("1 <=> 2.0"), Some(Object::Int(-1)));
}

#[test]
fn spaceship_float_int() {
    assert_eq!(run("2.0 <=> 1"), Some(Object::Int(1)));
}

#[test]
fn spaceship_string() {
    assert_eq!(run(r#""abc" <=> "def""#), Some(Object::Int(-1)));
    assert_eq!(run(r#""abc" <=> "abc""#), Some(Object::Int(0)));
    assert_eq!(run(r#""def" <=> "abc""#), Some(Object::Int(1)));
}

#[test]
fn spaceship_string_string() {
    assert_eq!(run("'a' <=> 'b'"), Some(Object::Int(-1)));
}

#[test]
fn spaceship_type_error() {
    let err = run_err("1 <=> 'a'");
    assert!(err.contains("type") || err.contains("Cannot"));
}
