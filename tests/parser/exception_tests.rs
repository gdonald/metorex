// Tests for begin/rescue/ensure parsing

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

#[test]
fn parser_begin_rescue_multiline() {
    let result =
        run("x = 0\nbegin\n  a = 1\n  raise \"boom\"\nrescue => e\n  x = 10\n  x = x + 1\nend\nx");
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn parser_rescue_else_multiline() {
    let result = run("x = 0\nbegin\n  x = 1\nrescue\n  x = 2\nelse\n  x = 10\n  x = x + 5\nend\nx");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn parser_ensure_multiline() {
    let result = run("x = 0\nbegin\n  x = 1\nensure\n  y = 42\n  x = y\nend\nx");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── From additional_tests ───────────────────────────────────────────────────

fn parse_exc_ok(code: &str) {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

#[test]
fn parse_begin_rescue_else_ensure_additional() {
    parse_exc_ok("begin\n  1\nrescue\n  2\nelse\n  3\nensure\n  4\nend");
}

#[test]
fn parse_rescue_with_variable_binding_additional() {
    parse_exc_ok("begin\n  1\nrescue => e\n  e\nend");
}

#[test]
fn parse_raise_with_string_additional() {
    parse_exc_ok("raise \"error\"");
}

#[test]
fn parse_raise_with_class_and_message_additional() {
    parse_exc_ok("raise RuntimeError, \"msg\"");
}

#[test]
fn parse_rescue_multiple_exception_types_additional() {
    parse_exc_ok("begin\n  1\nrescue TypeError, RuntimeError => e\n  e\nend");
}
