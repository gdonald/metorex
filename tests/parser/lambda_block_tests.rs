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

/// The message Ruby's parser gives for a `->` whose body opens with
/// neither `{` nor `do`.
const LAMBDA_BODY_UNOPENED: &str = "expected a `do` keyword or a `{` to open the lambda block";

#[test]
fn a_lambda_written_after_its_grouped_parameter_is_refused() {
    assert!(parse_err("f = (x) -> x + 1").contains(LAMBDA_BODY_UNOPENED));
}

#[test]
fn a_lambda_body_written_without_braces_after_an_expression_is_refused() {
    assert!(parse_err("(1 + 2) -> 42").contains(LAMBDA_BODY_UNOPENED));
}

#[test]
fn a_lambda_body_written_without_braces_after_a_string_is_refused() {
    assert!(parse_err(r#""hello" -> 42"#).contains(LAMBDA_BODY_UNOPENED));
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
fn a_lambda_with_parameters_and_an_unbraced_body_is_refused() {
    assert!(parse_err("f = -> (x) x + 1").contains(LAMBDA_BODY_UNOPENED));
}

#[test]
fn a_lambda_in_an_array_with_an_unbraced_body_is_refused() {
    assert!(parse_err("[-> (x) x + 1]").contains(LAMBDA_BODY_UNOPENED));
}

#[test]
fn a_lambda_in_an_array_with_a_braced_body_runs() {
    let result = run("[-> (x) { x + 1 }][0].call(5)");
    assert_eq!(result, Some(Object::Int(6)));
}

// ── Stabby lambda as paren-less method-call argument ────────────────────────

#[test]
fn method_call_with_stabby_lambda_arg_no_parens() {
    let result = run("class Holder\n  def take(p) p.call end\nend\nHolder.new.take -> { 42 }");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn method_call_with_stabby_lambda_arg_multiline_body() {
    let result =
        run("class Holder\n  def take(p) p.call end\nend\nh = Holder.new\nh.take -> {\n  1 + 2\n}");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Trailing comma in argument lists ────────────────────────────────────────

#[test]
fn method_call_allows_trailing_comma() {
    let result = run("def f(a, b) a + b end\nf(1, 2,)");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn method_call_allows_trailing_comma_with_kwargs() {
    let result = run("def f(a, b:) a + b end\nf(1, b: 2,)");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn method_call_allows_trailing_comma_multiline() {
    let result = run("def f(a, b, c) a + b + c end\nf(\n  1,\n  2,\n  3,\n)");
    assert_eq!(result, Some(Object::Int(6)));
}

// ── Parenthesized assignment ─────────────────────────────────────────────────

#[test]
fn parenthesized_assignment_parses() {
    // groups.rs line 18: parse_paren_group sees `=` after identifier, builds BinaryOp::Assign
    let result = run("(x = 5)\nx");
    assert_eq!(result, Some(Object::Int(5)));
}
