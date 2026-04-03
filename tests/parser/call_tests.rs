// Tests for call expression parsing (keyword args, no-paren calls, dict-like syntax)

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

// ── Keyword argument calls ──────────────────────────────────────────────────

#[test]
fn keyword_arg_call_without_parens_single() {
    let result = run("def greet(name:)\n  name\nend\ngreet name: \"Alice\"");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Alice".to_string())))
    );
}

#[test]
fn keyword_arg_call_without_parens_multiple() {
    let result = run("def describe(name:, age:)\n  name\nend\ndescribe name: \"Bob\", age: 30");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Bob".to_string())))
    );
}

// ── No-paren call ───────────────────────────────────────────────────────────

#[test]
fn no_paren_call_two_positional_args() {
    let result = run("def add(a, b)\n  a + b\nend\nadd 1 + 0, 2");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn no_paren_call_with_trailing_do_block() {
    let result = run("def run_with(x)\n  x\nend\nrun_with !false do\n  99\nend");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn no_paren_call_with_trailing_brace_block() {
    let result = run("def run_with(x)\n  x\nend\nrun_with !false { 99 }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn no_paren_call_trailing_comma_before_end() {
    let result =
        run("def noop(x)\n  x\nend\nresult = nil\nif true\n  result = noop 1 + 0,\nend\nresult");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn should_parse_as_arg_returns_false_for_non_ident_before_colon() {
    parse_ok("x = {42 => 1}");
}

// ── Dict-like syntax errors ─────────────────────────────────────────────────

#[test]
fn no_paren_call_not_triggered_before_rbrace() {
    let result = run("x = {\"a\" => 1}\nx");
    if let Some(Object::Dict(_)) = result {
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn no_paren_call_with_colon_after_expr_is_error() {
    let err = parse_err("foo 1 : 2");
    assert!(
        err.contains("dictionary")
            || err.contains("Expected")
            || err.contains("syntax")
            || err.contains("Unexpected")
    );
}

// ── Parser attr methods ─────────────────────────────────────────────────────

#[test]
fn parser_attr_accessor_multiple() {
    let result = run(
        "class Foo\n  attr_accessor :name, :age\nend\nf = Foo.new\nf.name = \"test\"\nf.age = 25\nf.age",
    );
    assert_eq!(result, Some(Object::Int(25)));
}

#[test]
fn attr_reader_with_non_symbol_error() {
    let err = parse_err("class Foo\n  attr_reader 42\nend");
    assert!(err.contains("attribute") || err.contains("Expected") || err.contains("Unexpected"));
}
