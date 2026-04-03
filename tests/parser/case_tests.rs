// Tests for case/when and case/in expression parsing

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn parse_ok(code: &str) {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

fn parse_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().unwrap_err()[0].to_string()
}

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

#[test]
fn parser_case_when_multiline_body() {
    let result = run("case 2\nwhen 1\n  10\nwhen 2\n  21\nelse\n  0\nend");
    assert_eq!(result, Some(Object::Int(21)));
}

#[test]
fn parser_case_else_multiline_body() {
    let result = run("case 99\nwhen 1\n  10\nelse\n  51\nend");
    assert_eq!(result, Some(Object::Int(51)));
}

#[test]
fn parser_case_in_multiline_body() {
    let result = run("case 42\nin Integer => n\n  n * 2\nend");
    assert_eq!(result, Some(Object::Int(84)));
}

#[test]
fn case_in_bind_non_identifier_error() {
    let err = parse_err("case 1\nin 1 => 42\n  nil\nend");
    assert!(err.contains("identifier") || err.contains("Expected"));
}

#[test]
fn case_when_trailing_comma_before_terminal_token() {
    parse_ok("case 1\nwhen 1, then\n  1\nend");
}

#[test]
fn rest_pattern_at_top_level_is_parse_error() {
    let err = parse_err("case 1\nin *x\n  x\nend");
    assert!(err.contains("pattern") || err.contains("Star") || err.contains("Expected"));
}
