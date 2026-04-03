// Tests for built-in Object methods (class, to_s, respond_to?) on all types

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

// ── .class on built-in types returns a Class object ─────────────────────

#[test]
fn array_class() {
    let result = run("[1, 2, 3].class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Array");
    }
}

#[test]
fn string_class() {
    let result = run(r#""hello".class"#);
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "String");
    }
}

#[test]
fn integer_class() {
    let result = run("42.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Integer");
    }
}

#[test]
fn float_class() {
    let result = run("3.14.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Float");
    }
}

#[test]
fn bool_class() {
    let result = run("true.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Object");
    }
}

#[test]
fn nil_class() {
    let result = run("nil.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Object");
    }
}

#[test]
fn range_class() {
    let result = run("(1..5).class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Range");
    }
}

#[test]
fn hash_class() {
    let result = run(r#"{"a" => 1}.class"#);
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Hash");
    }
}

#[test]
fn set_class() {
    let result = run("Set.new.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Set");
    }
}

#[test]
fn user_instance_class() {
    let result = run(r#"
class Dog
end
Dog.new.class
"#);
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Dog");
    }
}

// ── .to_s on built-in types ─────────────────────────────────────────────

#[test]
fn array_to_s() {
    let result = run("[1, 2, 3].to_s");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[1, 2, 3]".to_string())))
    );
}

#[test]
fn integer_to_s() {
    let result = run("42.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("42".to_string()))));
}

#[test]
fn nil_to_s() {
    let result = run("nil.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("nil".to_string()))));
}

#[test]
fn bool_to_s() {
    let result = run("true.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("true".to_string()))));
}

// ── .respond_to? on built-in types ──────────────────────────────────────

#[test]
fn array_respond_to_length() {
    let result = run(r#"[1, 2].respond_to?("length")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_respond_to_upcase() {
    let result = run(r#""hello".respond_to?("upcase")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn integer_respond_to_class() {
    let result = run(r#"42.respond_to?("class")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}
