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

// ── Invalid symbol pattern in case/in (control_flow.rs lines 771-772) ─────────

#[test]
fn case_in_invalid_symbol_pattern_error() {
    // control_flow.rs lines 771-772: symbol pattern with non-ident token after ':'
    let err = parse_err("case x\nin :123\n  1\nend");
    assert!(err.contains("Expected") || err.contains("identifier") || err.contains("pattern"));
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

// ── From additional_tests ───────────────────────────────────────────────────

fn parse_case_ok(code: &str) {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

#[test]
fn parse_case_with_then_additional() {
    parse_case_ok("case x\nwhen 1 then 10\nwhen 2 then 20\nend");
}

#[test]
fn parse_case_with_guard_additional() {
    parse_case_ok("case x\nwhen 1\n  10\nend");
}

#[test]
fn parse_case_in_additional() {
    parse_case_ok("case x\nin 1\n  10\nin 2\n  20\nend");
}

#[test]
fn parse_case_with_object_pattern_additional() {
    parse_case_ok("case h\nin { name: n }\n  n\nend");
}

#[test]
fn parse_case_with_range_pattern_additional() {
    parse_case_ok("case x\nwhen 1..10\n  \"range\"\nend");
}

#[test]
fn parse_case_with_string_pattern_additional() {
    parse_case_ok("case x\nwhen \"hello\"\n  1\nend");
}
