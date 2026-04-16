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
fn unary_plus_on_string_returns_string() {
    let result = run(r#"+"hello""#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

#[test]
fn unary_minus_float() {
    let result = run("-3.14");
    assert_eq!(result, Some(Object::Float(-3.14)));
}

#[test]
fn unary_minus_error_on_string() {
    let err = run_err(r#"-"hello""#);
    assert!(err.contains("unary") || err.contains("-") || err.contains("operator"));
}
