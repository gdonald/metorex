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

// ── =~ ──────────────────────────────────────────────────────────────────────

#[test]
fn regex_match_returns_position() {
    assert_eq!(run(r#"/hello/ =~ "say hello world""#), Some(Object::Int(4)));
}

#[test]
fn regex_match_returns_nil_on_no_match() {
    assert_eq!(run(r#"/xyz/ =~ "hello""#), Some(Object::Nil));
}

#[test]
fn regex_match_string_left() {
    assert_eq!(run(r#""test123" =~ /\d+/"#), Some(Object::Int(4)));
}

#[test]
fn regex_match_case_insensitive() {
    assert_eq!(run(r#"/hello/i =~ "HELLO world""#), Some(Object::Int(0)));
}

// ── !~ ──────────────────────────────────────────────────────────────────────

#[test]
fn regex_not_match_true() {
    assert_eq!(run(r#""abc" !~ /xyz/"#), Some(Object::Bool(true)));
}

#[test]
fn regex_not_match_false() {
    assert_eq!(run(r#""abc" !~ /abc/"#), Some(Object::Bool(false)));
}

// ── %r literal ──────────────────────────────────────────────────────────────

#[test]
fn percent_r_regex() {
    assert_eq!(run(r#"%r(hello) =~ "say hello""#), Some(Object::Int(4)));
}

#[test]
fn percent_r_brackets() {
    assert_eq!(run(r#"%r[hello] =~ "hello world""#), Some(Object::Int(0)));
}

#[test]
fn percent_r_braces() {
    assert_eq!(run(r#"%r{test} =~ "a test""#), Some(Object::Int(2)));
}
