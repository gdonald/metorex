// 10.3.1 — Lexer + Parser Integration Tests
//
// These tests verify that the lexer and parser work correctly together:
// tokenizing source code and producing valid AST nodes.

use metorex::ast::Statement;
use metorex::lexer::{Lexer, TokenKind};
use metorex::parser::Parser;

fn parse(code: &str) -> Vec<Statement> {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed")
}

fn parse_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let errors = Parser::new(tokens).parse().unwrap_err();
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Literals flow through lexer → parser correctly ──

#[test]
fn integer_literal_parses() {
    let stmts = parse("42");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn float_literal_parses() {
    let stmts = parse("3.14");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn string_literal_parses() {
    let stmts = parse("\"hello world\"");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn boolean_literals_parse() {
    let stmts = parse("true\nfalse");
    assert_eq!(stmts.len(), 2);
}

#[test]
fn nil_literal_parses() {
    let stmts = parse("nil");
    assert_eq!(stmts.len(), 1);
}

// ── Operators tokenize and parse into correct binary expressions ──

#[test]
fn arithmetic_operators_parse() {
    let stmts = parse("1 + 2 * 3 - 4 / 2");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn comparison_operators_parse() {
    let stmts = parse("x == 1\nx != 2\nx < 3\nx > 4\nx <= 5\nx >= 6");
    assert_eq!(stmts.len(), 6);
}

#[test]
fn logical_operators_parse() {
    let stmts = parse("true && false\ntrue || false");
    assert_eq!(stmts.len(), 2);
}

#[test]
fn compound_assignment_operators_parse() {
    let stmts = parse("x = 1\nx += 2\nx -= 3\nx *= 4\nx /= 5");
    assert_eq!(stmts.len(), 5);
}

// ── Complex structures tokenize and parse correctly ──

#[test]
fn function_definition_parses() {
    let stmts = parse("def greet(name)\n  puts name\nend");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(stmts[0], Statement::FunctionDef { .. }));
}

#[test]
fn class_definition_parses() {
    let stmts = parse("class Dog < Animal\n  def bark\n    \"woof\"\n  end\nend");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(stmts[0], Statement::ClassDef { .. }));
}

#[test]
fn if_elsif_else_parses() {
    let code = "if x > 0\n  1\nelsif x == 0\n  0\nelse\n  -1\nend";
    let stmts = parse(code);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn while_loop_parses() {
    let stmts = parse("while x > 0\n  x -= 1\nend");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn for_loop_parses() {
    let stmts = parse("for i in 1..10\n  puts i\nend");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn block_with_do_end_parses() {
    let stmts = parse("[1, 2, 3].each do |x|\n  puts x\nend");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn block_with_braces_parses() {
    let stmts = parse("[1, 2, 3].map { |x| x * 2 }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn begin_rescue_ensure_parses() {
    let code = "begin\n  risky\nrescue => e\n  handle\nensure\n  cleanup\nend";
    let stmts = parse(code);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn case_when_parses() {
    let code = "case x\nwhen 1\n  \"one\"\nwhen 2\n  \"two\"\nelse\n  \"other\"\nend";
    let stmts = parse(code);
    assert_eq!(stmts.len(), 1);
}

#[test]
fn lambda_parses() {
    let stmts = parse("f = lambda do |x| x * 2 end");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn string_interpolation_tokenizes_and_parses() {
    let stmts = parse("name = \"world\"\n\"hello #{name}\"");
    assert_eq!(stmts.len(), 2);
}

#[test]
fn array_and_hash_literals_parse() {
    let stmts = parse("[1, 2, 3]\n{\"a\" => 1, \"b\" => 2}");
    assert_eq!(stmts.len(), 2);
}

#[test]
fn method_chaining_parses() {
    let stmts = parse("[1, 2, 3].length.to_s");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn nested_class_with_methods_parses() {
    let code = r#"class Outer
  def initialize
    @x = 1
  end

  def method_a(a, b)
    a + b
  end

  def method_b
    @x
  end
end"#;
    let stmts = parse(code);
    assert_eq!(stmts.len(), 1);
    if let Statement::ClassDef { name, body, .. } = &stmts[0] {
        assert_eq!(name, "Outer");
        assert_eq!(body.len(), 3); // initialize, method_a, method_b
    } else {
        panic!("Expected ClassDef");
    }
}

// ── Error recovery: lexer errors propagate through parser ──

#[test]
fn unterminated_string_lexes_to_end_of_input() {
    let tokens = Lexer::new("\"unterminated").tokenize();
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(kinds, vec![TokenKind::EOF]);
}

#[test]
fn unterminated_string_parses_to_an_empty_program() {
    let tokens = Lexer::new("\"unterminated").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    assert!(statements.is_empty());
}

#[test]
fn unbalanced_end_produces_error() {
    let err = parse_err("end");
    assert!(!err.is_empty());
}

#[test]
fn missing_end_produces_error() {
    let err = parse_err("def foo\n  1");
    assert!(!err.is_empty());
}
