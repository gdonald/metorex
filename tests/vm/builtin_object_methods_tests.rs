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

// ── instance_variable_set/get ───────────────────────────────────────────────

#[test]
fn instance_variable_set_on_instance() {
    let result = run(r#"
class Foo; end
f = Foo.new
f.instance_variable_set(:@x, 42)
f.instance_variable_get(:@x)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn instance_variable_set_on_class() {
    let result = run(r#"
class Foo
  def self.setup
    instance_variable_set(:@val, 99)
  end
  def self.val
    instance_variable_get(:@val)
  end
end
Foo.setup
Foo.val
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

fn run_err(code: &str) -> String {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    use metorex::vm::VirtualMachine;
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

#[test]
fn instance_variable_set_with_string_name() {
    let result = run(r#"
class Foo; end
f = Foo.new
f.instance_variable_set("@x", 7)
f.instance_variable_get("@x")
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn instance_variable_set_wrong_arg_count_errors() {
    let err = run_err(
        r#"
class Foo; end
Foo.new.instance_variable_set(:@x)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_set_non_string_name_errors() {
    let err = run_err(
        r#"
class Foo; end
Foo.new.instance_variable_set(42, "bad")
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol"));
}

#[test]
fn instance_variable_set_on_module() {
    let result = run(r#"
module M
end
M.instance_variable_set(:@v, 11)
M.instance_variable_get(:@v)
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

// ── dup / clone ───────────────────────────────────────────────────────────

#[test]
fn array_dup_creates_independent_array() {
    let result = run(r#"
a = [1, 2, 3]
b = a.dup
b << 4
a.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_clone_creates_independent_array() {
    let result = run(r#"
a = [1, 2, 3]
b = a.clone
b << 4
a.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn dict_dup_creates_independent_dict() {
    let result = run(r#"
h = {"a" => 1}
g = h.dup
g["b"] = 2
h["b"]
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn dup_on_integer_returns_same() {
    let result = run("42.dup");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn instance_dup_creates_independent_instance() {
    let result = run(r#"
class Foo
  def initialize(v)
    @v = v
  end
  def v
    @v
  end
  def v=(x)
    @v = x
  end
end
a = Foo.new(1)
b = a.dup
b.v = 99
a.v
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_with_args_errors() {
    let err = run_err("[1, 2].dup(3)");
    assert!(err.contains("argument"));
}
