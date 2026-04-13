// Tests for lambda, arrow lambda, and block parameter parsing

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

// ── Arrow lambda ────────────────────────────────────────────────────────────

#[test]
fn arrow_lambda_grouped_single_param() {
    let result = run("f = (x) -> x + 1\nf.call(5)");
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn arrow_lambda_grouped_non_ident_parse_error() {
    let err = parse_err("(1 + 2) -> 42");
    assert!(
        err.contains("parameter")
            || err.contains("identifier")
            || err.contains("arrow")
            || err.contains("Identifier")
    );
}

#[test]
fn arrow_lambda_non_ident_lhs_parse_error() {
    let err = parse_err(r#""hello" -> 42"#);
    assert!(
        err.contains("parameter")
            || err.contains("identifier")
            || err.contains("Left")
            || err.contains("arrow")
    );
}

#[test]
fn parser_arrow_lambda_basic() {
    let result = run("f = lambda do\n  42\nend\nf.call");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── Block empty params ──────────────────────────────────────────────────────

#[test]
fn do_block_with_empty_pipe_params() {
    let result = run("result = 0\n[1, 2, 3].each do ||\n  result = result + 1\nend\nresult");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn brace_block_with_empty_pipe_params() {
    let result = run("result = 0\n[1, 2, 3].each { || result = result + 1 }\nresult");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Block param errors ──────────────────────────────────────────────────────

#[test]
fn do_block_non_ident_param_parse_error() {
    let err = parse_err(r#"[1].each do |42| x end"#);
    assert!(
        err.contains("parameter")
            || err.contains("identifier")
            || err.contains("Expected")
            || err.contains("Ident")
    );
}

#[test]
fn brace_block_non_ident_param_parse_error() {
    let err = parse_err(r#"[1,2,3].map { |42| 1 }"#);
    assert!(
        err.contains("parameter")
            || err.contains("identifier")
            || err.contains("Expected")
            || err.contains("Ident")
    );
}

// ── Lambda param errors ─────────────────────────────────────────────────────

#[test]
fn lambda_with_invalid_param_name_error() {
    let err = parse_err("lambda do |123|\n  nil\nend");
    assert!(err.contains("parameter name") || err.contains("Expected"));
}

#[test]
fn lambda_with_comment_only_body() {
    parse_ok("lambda do\n  # nothing\nend");
}

// ── Do-block edge cases ─────────────────────────────────────────────────────

#[test]
fn do_block_with_invalid_param_name_error() {
    let err = parse_err("x = do |123|\n  nil\nend");
    assert!(err.contains("parameter name") || err.contains("Expected"));
}

#[test]
fn do_block_with_comment_only_body() {
    parse_ok("x = do\n  # nothing\nend");
}

// ── Multiline lambda body ───────────────────────────────────────────────────

#[test]
fn parser_lambda_multiline_body() {
    let result = run("f = lambda do |x|\n  y = x + 1\n  z = y * 2\n  z\nend\nf.call(3)");
    assert_eq!(result, Some(Object::Int(8)));
}

// ── From additional_tests ───────────────────────────────────────────────────

fn parse_lb_ok(code: &str) {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

#[test]
fn parse_lambda_with_do_keyword_additional() {
    parse_lb_ok("f = lambda do\n  42\nend");
}

#[test]
fn parse_lambda_empty_params_additional() {
    parse_lb_ok("f = lambda { || 42 }");
}

#[test]
fn parse_do_block_with_params_additional() {
    parse_lb_ok("[1].each do |x|\n  x\nend");
}

// ── Block params with default values ────────────────────────────────────────

#[test]
fn brace_block_param_with_default_value() {
    // mod.rs lines 405-407: brace block param default parsing.
    // Default value must be followed by comma (not |) so parse_expression stops
    // at the comma, not consuming the closing |.
    parse_ok("[1].map { |x = 0, y| x }");
}

#[test]
fn do_block_param_with_default_value() {
    // mod.rs lines 337-339: do-block param default parsing (comma stops parse_expression)
    parse_ok("[1].map do |x = 0, y|\n  x\nend");
}

#[test]
fn lambda_param_with_default_value() {
    // blocks.rs lines 153-155: lambda do-block param default parsing
    let result = run("f = lambda do |x = 99, y|\n  [x, y]\nend\nf.call(1, 2)");
    assert_eq!(
        result,
        Some(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
            vec![Object::Int(1), Object::Int(2),]
        ))))
    );
}

// ── Stabby lambda with parens and expression body ────────────────────────────

#[test]
fn stabby_lambda_with_parens_and_expression_body() {
    // blocks.rs lines 244-252: stabby lambda with (params) and expression body
    // -> (x) expr (no brace block)
    let result = run("f = -> (x) x + 1\nf.call(5)");
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn stabby_lambda_as_primary_expression_in_array() {
    // blocks.rs lines 244-252: stabby_lambda_with_params via parse_primary (Arrow token
    // inside an array literal goes through parse_primary -> parse_stabby_lambda).
    // This differs from statement-level `f = -> (x) ...` which uses parse_arrow_lambda.
    let result = run("[-> (x) x + 1][0].call(5)");
    assert_eq!(result, Some(Object::Int(6)));
}

// ── Parenthesized assignment ─────────────────────────────────────────────────

#[test]
fn parenthesized_assignment_parses() {
    // groups.rs line 18: parse_paren_group sees `=` after identifier, builds BinaryOp::Assign
    let result = run("(x = 5)\nx");
    assert_eq!(result, Some(Object::Int(5)));
}
