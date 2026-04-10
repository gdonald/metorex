// Tests for function definition parsing

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
fn def_with_integer_name_is_parse_error() {
    let err = parse_err("def 123\n  nil\nend");
    assert!(err.contains("function name") || err.contains("Expected"));
}

#[test]
fn def_with_comment_only_body() {
    parse_ok("def silent\n  # nothing here\nend");
}

#[test]
fn def_with_variadic_parameter_parses_ok() {
    parse_ok("def sum(*args)\n  args\nend");
}

#[test]
fn def_block_param_with_invalid_name_error() {
    let err = parse_err("def foo(&123)\n  nil\nend");
    assert!(err.contains("parameter name") || err.contains("Expected"));
}

#[test]
fn variadic_param_with_non_ident_error() {
    let err = parse_err("def foo(*42)\nend");
    assert!(err.contains("parameter") || err.contains("Expected") || err.contains("Unexpected"));
}

#[test]
fn parser_function_multiline_body() {
    let result = run("def compute(a, b)\n  x = a + b\n  y = x * 2\n  y\nend\ncompute(3, 4)");
    assert_eq!(result, Some(Object::Int(14)));
}

// ── Setter method def ───────────────────────────────────────────────────────

#[test]
fn setter_method_def() {
    let result = run(r#"
class Foo
  def name=(val)
    @name = val
  end
  def name
    @name
  end
end
f = Foo.new
f.name = "hello"
f.name
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

#[test]
fn singleton_setter_method() {
    parse_ok(
        r#"
class Foo
  def self.name=(val)
    @name = val
  end
end
"#,
    );
}

// ── Method rescue/else/ensure ───────────────────────────────────────────────

#[test]
fn method_rescue_ensure_parsing() {
    parse_ok("def test\n  raise \"oops\"\nrescue\n  42\nensure\n  99\nend");
}

#[test]
fn method_rescue_else_parsing() {
    parse_ok("def safe\n  42\nrescue => e\n  -1\nelse\n  99\nend");
}

#[test]
fn method_rescue_else_ensure_parsing() {
    parse_ok("def full\n  42\nrescue => e\n  -1\nelse\n  99\nensure\n  0\nend");
}
