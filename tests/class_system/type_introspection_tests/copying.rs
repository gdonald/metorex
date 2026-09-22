// The copies `dup` and `clone` hand back.

use super::*;

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
    assert_eq!(result, Some(Object::string("hello".to_string())));
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
p.instance_variable_get(:@name)
"#);
    assert_eq!(result, Some(Object::string("Alice".to_string())));
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
