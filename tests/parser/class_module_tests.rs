// Tests for class and module parsing

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

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
fn module_with_non_ident_name_error() {
    let err = parse_err("module 42\nend");
    assert!(err.contains("module name") || err.contains("Expected") || err.contains("Ident"));
}

#[test]
fn include_with_non_ident_name_error() {
    let err = parse_err("class Foo\n  include 42\nend");
    assert!(err.contains("module name") || err.contains("Expected") || err.contains("include"));
}

#[test]
fn extend_with_non_ident_name_error() {
    let err = parse_err("class Foo\n  extend 42\nend");
    assert!(err.contains("module name") || err.contains("Expected") || err.contains("extend"));
}

#[test]
fn class_with_non_ident_parent_error() {
    let err = parse_err("class Foo < 42\nend");
    assert!(err.contains("class") || err.contains("Expected") || err.contains("Unexpected"));
}

#[test]
fn parser_class_multiline_body() {
    let result = run(
        "class Foo\n  def bar\n    1\n  end\n  def baz\n    2\n  end\nend\nFoo.new.bar + Foo.new.baz",
    );
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn parser_class_with_inheritance_body() {
    let result = run(
        "class Base\n  def x\n    10\n  end\nend\nclass Child < Base\n  def y\n    20\n  end\nend\nc = Child.new\nc.x + c.y",
    );
    assert_eq!(result, Some(Object::Int(30)));
}

#[test]
fn define_method_with_string_name() {
    let result = run(
        "class Greeter\n  define_method(\"hello\") do\n    \"world\"\n  end\nend\nGreeter.new.hello",
    );
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("world".to_string())))
    );
}
