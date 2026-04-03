// Tests for runtime class modification (14.3)

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

// ── alias_method ────────────────────────────────────────────────────

#[test]
fn alias_method_creates_alias() {
    let result = run(r#"
class Foo
  def original
    42
  end
end
Foo.alias_method("copy", "original")
Foo.new.copy
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn alias_method_original_still_works() {
    let result = run(r#"
class Foo
  def original
    42
  end
end
Foo.alias_method("copy", "original")
Foo.new.original
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn alias_method_with_symbol() {
    let result = run(r#"
class Foo
  def greet
    "hello"
  end
end
Foo.alias_method(:hi, :greet)
Foo.new.hi
"#);
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
}

#[test]
fn alias_method_error_nonexistent_source() {
    let err = run_err(
        r#"
class Foo
end
Foo.alias_method("copy", "nonexistent")
"#,
    );
    assert!(err.contains("undefined method 'nonexistent'"));
}

#[test]
fn alias_method_inherits_from_parent() {
    let result = run(r#"
class Parent
  def greet
    "parent"
  end
end
class Child < Parent
end
Child.alias_method("hi", "greet")
Child.new.hi
"#);
    assert_eq!(result, Some(Object::String(Rc::new("parent".to_string()))));
}

// ── remove_method ───────────────────────────────────────────────────

#[test]
fn remove_method_removes_own_method() {
    let result = run(r#"
class Foo
  def bar
    42
  end
  def method_missing(name)
    "removed: #{name}"
  end
end
Foo.remove_method("bar")
Foo.new.bar
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("removed: bar".to_string())))
    );
}

#[test]
fn remove_method_error_not_defined() {
    let err = run_err(
        r#"
class Foo
end
Foo.remove_method("nonexistent")
"#,
    );
    assert!(err.contains("method 'nonexistent' not defined"));
}

#[test]
fn remove_method_exposes_parent_method() {
    let result = run(r#"
class Parent
  def greet
    "parent"
  end
end
class Child < Parent
  def greet
    "child"
  end
end
Child.remove_method("greet")
Child.new.greet
"#);
    assert_eq!(result, Some(Object::String(Rc::new("parent".to_string()))));
}

// ── undef_method ────────────────────────────────────────────────────

#[test]
fn undef_method_blocks_own_method() {
    let result = run(r#"
class Foo
  def bar
    42
  end
  def method_missing(name)
    "undef: #{name}"
  end
end
Foo.undef_method("bar")
Foo.new.bar
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("undef: bar".to_string())))
    );
}

#[test]
fn undef_method_blocks_inherited_method() {
    let result = run(r#"
class Parent
  def greet
    "parent"
  end
end
class Child < Parent
  def method_missing(name)
    "blocked: #{name}"
  end
end
Child.undef_method("greet")
Child.new.greet
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("blocked: greet".to_string())))
    );
}

#[test]
fn undef_method_without_method_missing_raises_error() {
    let err = run_err(
        r#"
class Foo
  def bar
    42
  end
end
Foo.undef_method("bar")
Foo.new.bar
"#,
    );
    assert!(err.contains("Undefined method 'bar'"));
}

#[test]
fn undef_method_does_not_affect_parent() {
    let result = run(r#"
class Parent
  def greet
    "parent"
  end
end
class Child < Parent
end
Child.undef_method("greet")
Parent.new.greet
"#);
    assert_eq!(result, Some(Object::String(Rc::new("parent".to_string()))));
}

// ── module_function ─────────────────────────────────────────────────

#[test]
fn module_function_makes_method_callable_on_module() {
    let result = run(r#"
module Helper
  def double(x)
    x * 2
  end
end
Helper.module_function("double")
Helper.double(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn module_function_error_nonexistent() {
    let err = run_err(
        r#"
module Helper
end
Helper.module_function("nonexistent")
"#,
    );
    assert!(err.contains("undefined method 'nonexistent'"));
}

#[test]
fn module_function_method_still_includable() {
    let result = run(r#"
module Helper
  def double(x)
    x * 2
  end
end
Helper.module_function("double")

class Foo
  include Helper
end
Foo.new.double(7)
"#);
    assert_eq!(result, Some(Object::Int(14)));
}

// ── argument validation ─────────────────────────────────────────────

#[test]
fn alias_method_wrong_arg_count() {
    let err = run_err(
        r#"
class Foo
end
Foo.alias_method("one")
"#,
    );
    assert!(err.contains("expected 2"));
}

#[test]
fn remove_method_wrong_arg_count() {
    let err = run_err(
        r#"
class Foo
end
Foo.remove_method()
"#,
    );
    assert!(err.contains("expected 1"));
}

#[test]
fn undef_method_wrong_arg_count() {
    let err = run_err(
        r#"
class Foo
end
Foo.undef_method()
"#,
    );
    assert!(err.contains("expected 1"));
}

#[test]
fn module_function_wrong_arg_count() {
    let err = run_err(
        r#"
module Foo
end
Foo.module_function()
"#,
    );
    assert!(err.contains("expected 1"));
}
