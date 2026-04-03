// Tests for unless statement parsing

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn parse_ok(code: &str) {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

#[test]
fn parse_unless_without_else() {
    parse_ok("unless false\n  x = 1\nend\n");
}

#[test]
fn parse_unless_with_else() {
    let result = run("x = 0\nunless false\n  x = 1\nelse\n  x = 2\nend\nx");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn unless_with_else_takes_else_branch() {
    let result = run("x = 0\nunless true\n  x = 1\nelse\n  x = 2\nend\nx");
    assert_eq!(result, Some(Object::Int(2)));
}
