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
fn module_function_method_is_included_as_private() {
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
Foo.new.send(:double, 7)
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
fn module_function_no_arguments_is_the_toggle() {
    let result = run(r#"
module Foo
end
Foo.module_function()
"#);
    assert_eq!(result, Some(Object::Nil));
}
#[test]
fn remove_method_with_symbol_arg() {
    let result = run(r#"
class Foo
  def bar
    42
  end
  def method_missing(name)
    "gone"
  end
end
Foo.remove_method(:bar)
Foo.new.bar
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("gone".to_string())))
    );
}

#[test]
fn remove_method_type_error() {
    let err = run_err(
        r#"
class Foo
  def bar
    42
  end
end
Foo.remove_method(123)
"#,
    );
    assert!(err.contains("String or Symbol"));
}

// ── undef_method with symbol ────────────────────────────────────────────────

#[test]
fn undef_method_with_symbol_arg() {
    let result = run(r#"
class Foo
  def bar
    42
  end
  def method_missing(name)
    "undef"
  end
end
Foo.undef_method(:bar)
Foo.new.bar
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("undef".to_string())))
    );
}

#[test]
fn undef_method_type_error() {
    let err = run_err(
        r#"
class Foo
end
Foo.undef_method(123)
"#,
    );
    assert!(err.contains("String or Symbol"));
}

// ── alias_method with symbol ────────────────────────────────────────────────

#[test]
fn alias_method_new_name_symbol() {
    let result = run(r#"
class Foo
  def greet
    "hi"
  end
end
Foo.alias_method(:hello, "greet")
Foo.new.hello
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hi".to_string())))
    );
}

#[test]
fn alias_method_old_name_symbol() {
    let result = run(r#"
class Foo
  def greet
    "hi"
  end
end
Foo.alias_method("hello", :greet)
Foo.new.hello
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hi".to_string())))
    );
}

#[test]
fn alias_method_type_error_first_arg() {
    let err = run_err(
        r#"
class Foo
end
Foo.alias_method(123, "greet")
"#,
    );
    // MRI raises TypeError for non-string/symbol names that can't be coerced
    // via to_str; metorex matches that. Older tests checked for "String or
    // Symbol" in the error text; we now assert the TypeError class instead.
    assert!(err.contains("TypeError") || err.contains("not a symbol nor a string"));
}

#[test]
fn alias_method_type_error_second_arg() {
    let err = run_err(
        r#"
class Foo
end
Foo.alias_method("hello", 123)
"#,
    );
    assert!(err.contains("TypeError") || err.contains("not a symbol nor a string"));
}

// ── module_function with symbol ─────────────────────────────────────────────

#[test]
fn module_function_with_symbol_arg() {
    let result = run(r#"
module Helper
  def double(x)
    x * 2
  end
end
Helper.module_function(:double)
Helper.double(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn module_function_type_error() {
    let err = run_err(
        r#"
module Foo
end
Foo.module_function(123)
"#,
    );
    assert!(
        err.contains("123 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

// ── Module receiver: remove_method, undef_method, alias_method ──────────────

#[test]
fn module_remove_method() {
    let result = run(r#"
module Mod
  def foo
    1
  end
  def bar
    2
  end
end
Mod.remove_method("foo")
class C
  include Mod
end
C.new.bar
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn module_remove_method_with_symbol() {
    let result = run(r#"
module Mod
  def foo
    1
  end
  def bar
    2
  end
end
Mod.remove_method(:foo)
class C
  include Mod
end
C.new.bar
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn module_remove_method_not_found() {
    let err = run_err(
        r#"
module Mod
end
Mod.remove_method("nonexistent")
"#,
    );
    assert!(err.contains("not defined"));
}

#[test]
fn module_remove_method_type_error() {
    let err = run_err(
        r#"
module Mod
end
Mod.remove_method(123)
"#,
    );
    assert!(err.contains("String or Symbol"));
}

#[test]
fn module_remove_method_wrong_args() {
    let err = run_err(
        r#"
module Mod
end
Mod.remove_method()
"#,
    );
    assert!(err.contains("expected 1"));
}

#[test]
fn module_undef_method() {
    let result = run(r#"
module Mod
  def foo
    1
  end
end
Mod.undef_method("foo")
class C
  include Mod
  def method_missing(name)
    "blocked"
  end
end
C.new.foo
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("blocked".to_string())))
    );
}

#[test]
fn module_undef_method_with_symbol() {
    let result = run(r#"
module Mod
  def foo
    1
  end
end
Mod.undef_method(:foo)
class C
  include Mod
  def method_missing(name)
    "blocked"
  end
end
C.new.foo
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("blocked".to_string())))
    );
}

#[test]
fn module_undef_method_type_error() {
    let err = run_err(
        r#"
module Mod
end
Mod.undef_method(123)
"#,
    );
    assert!(err.contains("String or Symbol"));
}

#[test]
fn module_undef_method_wrong_args() {
    let err = run_err(
        r#"
module Mod
end
Mod.undef_method()
"#,
    );
    assert!(err.contains("expected 1"));
}

#[test]
fn module_alias_method() {
    let result = run(r#"
module Mod
  def greet
    "hi"
  end
end
Mod.alias_method("hello", "greet")
class C
  include Mod
end
C.new.hello
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hi".to_string())))
    );
}

#[test]
fn module_alias_method_with_symbols() {
    let result = run(r#"
module Mod
  def greet
    "hi"
  end
end
Mod.alias_method(:hello, :greet)
class C
  include Mod
end
C.new.hello
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hi".to_string())))
    );
}

#[test]
fn module_alias_method_not_found() {
    let err = run_err(
        r#"
module Mod
end
Mod.alias_method("hello", "nonexistent")
"#,
    );
    assert!(err.contains("undefined method"));
}

#[test]
fn module_alias_method_type_error_first() {
    let err = run_err(
        r#"
module Mod
end
Mod.alias_method(123, "greet")
"#,
    );
    assert!(err.contains("TypeError") || err.contains("not a symbol nor a string"));
}

#[test]
fn module_alias_method_type_error_second() {
    let err = run_err(
        r#"
module Mod
end
Mod.alias_method("hello", 123)
"#,
    );
    assert!(err.contains("TypeError") || err.contains("not a symbol nor a string"));
}

#[test]
fn module_alias_method_wrong_args() {
    let err = run_err(
        r#"
module Mod
end
Mod.alias_method("one")
"#,
    );
    assert!(err.contains("expected 2"));
}

#[test]
fn module_module_function_not_found() {
    let err = run_err(
        r#"
module Mod
end
Mod.module_function("nonexistent")
"#,
    );
    assert!(err.contains("undefined method"));
}

#[test]
fn module_module_function_with_symbol() {
    let result = run(r#"
module Mod
  def double(x)
    x * 2
  end
end
Mod.module_function(:double)
Mod.double(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn module_module_function_type_error() {
    let err = run_err(
        r#"
module Mod
end
Mod.module_function(123)
"#,
    );
    assert!(
        err.contains("123 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

#[test]
fn module_module_function_no_arguments_is_the_toggle() {
    let result = run(r#"
module Mod
end
Mod.module_function()
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Module.name ─────────────────────────────────────────────────────────────

#[test]
fn module_name_method() {
    let result = run(r#"
module MyMod
  def foo
    1
  end
end
MyMod.name
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("MyMod".to_string())))
    );
}

// ── define_method with block that has captured vars ─────────────────────────

#[test]
fn define_method_captures_closure_vars() {
    let result = run(r#"
class Foo
end
multiplier = 3
Foo.define_method("triple") do |x|
  x * multiplier
end
Foo.new.triple(7)
"#);
    assert_eq!(result, Some(Object::Int(21)));
}

// ── File.write write failure path ───────────────────────────────────────────

#[test]
fn file_write_failure() {
    let err = run_err(
        r#"
File.write("/nonexistent/path/file.txt", "content")
"#,
    );
    assert!(err.contains("Failed to write file"));
}

// ── module_function on Class receiver ───────────────────────────────────────

#[test]
fn class_module_function_raises_type_error() {
    let err = run_err(
        r#"
class Foo
  def double(x)
    x * 2
  end
end
Foo.module_function("double")
"#,
    );
    assert!(
        err.contains("module_function must be called for modules"),
        "unexpected error: {err}"
    );
}

#[test]
fn class_module_function_not_found() {
    let err = run_err(
        r#"
class Foo
end
Foo.module_function("nonexistent")
"#,
    );
    assert!(
        err.contains("module_function must be called for modules"),
        "unexpected error: {err}"
    );
}
