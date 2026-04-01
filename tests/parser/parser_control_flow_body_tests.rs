// Tests for control flow body parsing (comment-only bodies, multiline bodies)

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

// ── Comment-only bodies ─────────────────────────────────────────────────────

#[test]
fn if_then_branch_comment_only() {
    parse_ok("if true\n  # comment\nend");
}

#[test]
fn if_elsif_body_comment_only() {
    parse_ok("if true\n  1\nelsif false\n  # comment\nend");
}

#[test]
fn if_else_body_comment_only() {
    parse_ok("if false\n  1\nelse\n  # comment\nend");
}

#[test]
fn while_body_comment_only() {
    parse_ok("while false\n  # comment\nend");
}

#[test]
fn for_body_comment_only() {
    parse_ok("for x in []\n  # comment\nend");
}

#[test]
fn unless_body_comment_only() {
    parse_ok("unless false\n  # comment\nend");
}

#[test]
fn unless_else_body_comment_only() {
    parse_ok("unless false\n  1\nelse\n  # comment\nend");
}

#[test]
fn case_when_body_comment_only() {
    parse_ok("case 1\nwhen 1\n  # comment\nend");
}

#[test]
fn case_else_body_comment_only() {
    parse_ok("case 1\nwhen 1\n  1\nelse\n  # comment\nend");
}

#[test]
fn case_in_body_comment_only() {
    parse_ok("case 1\nin 1\n  # comment\nend");
}

#[test]
fn case_in_else_body_comment_only() {
    parse_ok("case 1\nin 1\n  1\nelse\n  # comment\nend");
}

#[test]
fn bare_return_without_value() {
    parse_ok("return");
}

// ── Multiline bodies ────────────────────────────────────────────────────────

#[test]
fn parser_if_multiline_body() {
    let result = run("x = 0\nif true\n  x = 1\n  x = x + 1\nend\nx");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn parser_elsif_multiline_body() {
    let result = run("x = 0\nif false\n  x = 1\nelsif true\n  x = 10\n  x = x + 5\nend\nx");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn parser_else_multiline_body() {
    let result = run("x = 0\nif false\n  x = 1\nelse\n  x = 10\n  x = x + 5\nend\nx");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn parser_while_multiline_body() {
    let result = run("x = 0\nwhile x < 3\n  x = x + 1\n  y = x * 2\nend\nx");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn parser_for_multiline_body() {
    let result = run("sum = 0\nfor i in [1, 2, 3]\n  x = i * 2\n  sum = sum + x\nend\nsum");
    assert_eq!(result, Some(Object::Int(12)));
}

// ── If/unless expression multiline bodies ───────────────────────────────────

#[test]
fn parser_if_expression_multiline_then() {
    let result = run("x = if true\n  a = 1\n  b = 2\n  a + b\nend\nx");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn parser_if_expression_multiline_else() {
    let result = run("x = if false\n  1\nelse\n  a = 20\n  a + 2\nend\nx");
    assert_eq!(result, Some(Object::Int(22)));
}

#[test]
fn parser_unless_expression_multiline() {
    let result = run("x = unless false\n  a = 10\n  a * 3\nend\nx");
    assert_eq!(result, Some(Object::Int(30)));
}

#[test]
fn parser_unless_else_multiline() {
    let result = run("x = unless true\n  1\nelse\n  a = 5\n  a * 4\nend\nx");
    assert_eq!(result, Some(Object::Int(20)));
}

// ── If/unless expression comment-only bodies ────────────────────────────────

#[test]
fn if_expression_then_comment_only() {
    parse_ok("x = if true\n  # comment\nend");
}

#[test]
fn if_expression_elsif_comment_only() {
    parse_ok("x = if true\n  1\nelsif false\n  # comment\nend");
}

#[test]
fn if_expression_else_comment_only() {
    parse_ok("x = if false\n  1\nelse\n  # comment\nend");
}

#[test]
fn unless_expression_then_comment_only() {
    parse_ok("x = unless true\n  # comment\nend");
}

#[test]
fn unless_expression_else_comment_only() {
    parse_ok("x = unless true\n  1\nelse\n  # comment\nend");
}

#[test]
fn unless_expression_without_else() {
    let result = run("x = unless false\n  42\nend\nx");
    assert_eq!(result, Some(Object::Int(42)));
}
