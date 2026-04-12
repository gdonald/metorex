// Tests for AST manipulation (14.5): eval, parse, code generation

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── eval ────────────────────────────────────────────────────────────────

#[test]
fn eval_simple_expression() {
    let result = run(r#"eval("1 + 2")"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn eval_reads_outer_variables() {
    let result = run(r#"
x = 10
eval("x * 3")
"#);
    assert_eq!(result, Some(Object::Int(30)));
}

#[test]
fn eval_defines_variables_in_scope() {
    let result = run(r#"
eval("z = 42")
z
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn eval_defines_method() {
    let result = run(r#"
eval("def triple(n)\n  n * 3\nend")
triple(5)
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn eval_defines_class() {
    let result = run(r#"
eval("class Greeter\n  def greet\n    \"hi\"\n  end\nend")
Greeter.new.greet
"#);
    assert_eq!(result, Some(Object::String(Rc::new("hi".to_string()))));
}

#[test]
fn eval_returns_last_expression() {
    let result = run(r#"
eval("1\n2\n3")
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn eval_wrong_arg_count() {
    let err = run_err(r#"eval()"#);
    assert!(err.contains("expects 1"));
}

#[test]
fn eval_wrong_arg_type() {
    let err = run_err(r#"eval(42)"#);
    assert!(err.contains("String"));
}

#[test]
fn eval_parse_error() {
    // `(` followed by `)` with nothing inside: still a parse error.
    let err = run_err(r#"eval("def )")"#);
    assert!(
        err.contains("parse") || err.contains("function name") || err.contains("Unexpected"),
        "got: {}",
        err
    );
}

#[test]
fn eval_runtime_error() {
    let err = run_err(r#"eval("undefined_xyz")"#);
    assert!(err.contains("Undefined"));
}

// ── parse ───────────────────────────────────────────────────────────────

#[test]
fn parse_returns_array() {
    let result = run(r#"parse("1 + 2")"#);
    assert!(matches!(result, Some(Object::Array(_))));
}

#[test]
fn parse_captures_ast_structure() {
    let result = run(r#"
ast = parse("x = 1")
ast.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn parse_wrong_arg_count() {
    let err = run_err(r#"parse()"#);
    assert!(err.contains("expects 1"));
}

#[test]
fn parse_wrong_arg_type() {
    let err = run_err(r#"parse(42)"#);
    assert!(err.contains("String"));
}

#[test]
fn parse_error_in_code() {
    let err = run_err(r#"parse("def")"#);
    assert!(err.contains("parse error"));
}

// ── code generation via eval ────────────────────────────────────────────

#[test]
fn eval_generates_multiple_methods() {
    let result = run(r#"
eval("def add(a, b)\n  a + b\nend")
eval("def mul(a, b)\n  a * b\nend")
add(2, 3) + mul(4, 5)
"#);
    assert_eq!(result, Some(Object::Int(25)));
}

// ── runtime method modification via define_method ───────────────────────

#[test]
fn define_method_replaces_existing() {
    let result = run(r#"
class Calc
  def op(a, b)
    a + b
  end
end
c = Calc.new
first = c.op(2, 3)
Calc.define_method("op") do |a, b|
  a * b
end
second = c.op(2, 3)
first + second
"#);
    // first = 5 (add), second = 6 (multiply), total = 11
    assert_eq!(result, Some(Object::Int(11)));
}

// ── block modification via define_method ────────────────────────────────

#[test]
fn block_body_can_be_replaced() {
    let result = run(r#"
class Transformer
  def transform(x)
    x
  end
end
t = Transformer.new
Transformer.define_method("transform") do |x|
  x * 10
end
t.transform(5)
"#);
    assert_eq!(result, Some(Object::Int(50)));
}
