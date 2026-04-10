// Tests for for-loop parsing

use metorex::lexer::Lexer;
use metorex::parser::Parser;

fn parse_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().unwrap_err()[0].to_string()
}

#[test]
fn parse_for_missing_identifier_error() {
    let err = parse_err("for 42 in [1, 2]\nend");
    assert!(err.contains("identifier") || err.contains("'for'") || err.contains("Expected"));
}

// ── From additional_tests ───────────────────────────────────────────────────

fn parse_for_ok(code: &str) {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

#[test]
fn parse_for_with_do_additional() {
    parse_for_ok("for x in [1,2,3] do\n  x\nend");
}

#[test]
fn parse_for_without_do_additional() {
    parse_for_ok("for x in [1,2,3]\n  x\nend");
}
