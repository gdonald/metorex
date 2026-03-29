// Tests for type introspection built-ins:
// is_a?, kind_of?, superclass, ancestors, instance_variables,
// instance_variable_get, dup, clone

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

// ============================================================================
// is_a? / kind_of?
// ============================================================================

#[test]
fn is_a_integer() {
    let result = run("42.is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_float() {
    let result = run("3.14.is_a?(Float)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_string() {
    let result = run(r#""hello".is_a?(String)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_array() {
    let result = run("[1, 2].is_a?(Array)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_hash() {
    let result = run(r#"{"a" => 1}.is_a?(Hash)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_returns_false_for_wrong_type() {
    let result = run("42.is_a?(String)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn is_a_checks_inheritance() {
    let result = run("42.is_a?(Object)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_custom_class() {
    let result = run(r#"
class Dog
end
d = Dog.new
d.is_a?(Dog)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_custom_class_with_inheritance() {
    let result = run(r#"
class Animal
end
class Dog < Animal
end
d = Dog.new
d.is_a?(Animal)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn kind_of_is_alias_for_is_a() {
    let result = run("42.kind_of?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_error_no_args() {
    let err = run_err("42.is_a?");
    assert!(err.contains("argument"));
}

#[test]
fn is_a_error_non_class_arg() {
    let err = run_err("42.is_a?(42)");
    assert!(err.contains("Class") || err.contains("type"));
}

// ============================================================================
// superclass
// ============================================================================

#[test]
fn superclass_of_integer() {
    let result = run("Integer.superclass.name");
    assert_eq!(result, Some(Object::String(Rc::new("Object".to_string()))));
}

#[test]
fn superclass_of_custom_class() {
    let result = run(r#"
class Animal
end
class Dog < Animal
end
Dog.superclass.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Animal".to_string()))));
}

#[test]
fn superclass_of_object_is_nil() {
    let result = run("Object.superclass");
    assert_eq!(result, Some(Object::Nil));
}

// ============================================================================
// ancestors
// ============================================================================

#[test]
fn ancestors_of_integer() {
    let result = run("Integer.ancestors.length");
    assert_eq!(result, Some(Object::Int(2))); // Integer, Object
}

#[test]
fn ancestors_of_custom_chain() {
    let result = run(r#"
class A
end
class B < A
end
class C < B
end
C.ancestors.length
"#);
    assert_eq!(result, Some(Object::Int(3))); // C, B, A
}

// ============================================================================
// instance_variables
// ============================================================================

#[test]
fn instance_variables_returns_array() {
    let result = run(r#"
class Person
  def initialize(name, age)
    @name = name
    @age = age
  end
end
p = Person.new("Alice", 30)
p.instance_variables.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn instance_variables_empty_for_non_instance() {
    let result = run("42.instance_variables.length");
    assert_eq!(result, Some(Object::Int(0)));
}

// ============================================================================
// instance_variable_get
// ============================================================================

#[test]
fn instance_variable_get_existing() {
    let result = run(r#"
class Person
  def initialize(name)
    @name = name
  end
end
p = Person.new("Alice")
p.instance_variable_get("@name")
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Alice".to_string()))));
}

#[test]
fn instance_variable_get_without_at_prefix() {
    let result = run(r#"
class Person
  def initialize(name)
    @name = name
  end
end
p = Person.new("Bob")
p.instance_variable_get("name")
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Bob".to_string()))));
}

#[test]
fn instance_variable_get_missing_returns_nil() {
    let result = run(r#"
class Foo
end
f = Foo.new
f.instance_variable_get("@missing")
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn instance_variable_get_on_non_instance_returns_nil() {
    let result = run(r#"42.instance_variable_get("@x")"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn instance_variable_get_error_no_args() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.instance_variable_get
"#,
    );
    assert!(err.contains("argument"));
}

// ============================================================================
// dup / clone
// ============================================================================

#[test]
fn dup_instance_creates_copy() {
    let result = run(r#"
class Point
  def initialize(x, y)
    @x = x
    @y = y
  end
  def x
    @x
  end
end
p1 = Point.new(1, 2)
p2 = p1.dup
p2.x
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_instance_is_independent() {
    let result = run(r#"
class Box
  attr_accessor :value
  def initialize(v)
    @value = v
  end
end
b1 = Box.new(10)
b2 = b1.dup
b2.value = 99
b1.value
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn clone_array_creates_independent_copy() {
    let result = run(r#"
a = [1, 2, 3]
b = a.clone
b.push(4)
a.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn clone_hash_creates_independent_copy() {
    let result = run(r#"
h = {"a" => 1}
h2 = h.clone
h.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_integer_returns_same_value() {
    let result = run("42.dup");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn dup_string_returns_same_value() {
    let result = run(r#""hello".dup"#);
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
}

// ============================================================================
// Error paths for coverage
// ============================================================================

#[test]
fn instance_variables_error_with_args() {
    let err = run_err("42.instance_variables(1)");
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_get_error_non_string_arg() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.instance_variable_get(42)
"#,
    );
    assert!(err.contains("String") || err.contains("type"));
}

#[test]
fn dup_error_with_args() {
    let err = run_err("42.dup(1)");
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_get_with_symbol() {
    let result = run(r#"
class Person
  def initialize(name)
    @name = name
  end
end
p = Person.new("Alice")
p.instance_variable_get(:name)
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Alice".to_string()))));
}

#[test]
fn clone_dict_creates_independent_copy() {
    let result = run(r#"
h1 = {"a" => 1, "b" => 2}
h2 = h1.dup
h2["c"] = 3
h1.size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}
