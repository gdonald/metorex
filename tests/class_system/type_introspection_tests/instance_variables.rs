// Reading the instance variables an object carries.

use super::*;

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
    assert_eq!(result, Some(Object::string("Alice".to_string())));
}

#[test]
fn instance_variable_get_string_without_at_prefix_raises_name_error() {
    let error = run_err(
        r#"
class Person
  def initialize(name)
    @name = name
  end
end
p = Person.new("Bob")
p.instance_variable_get("name")
"#,
    );
    assert!(error.contains("`name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_set_without_at_prefix_raises_name_error() {
    let error = run_err(r#"Object.new.instance_variable_set("name", 1)"#);
    assert!(error.contains("`name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_set_validates_name_before_frozen_receiver() {
    let error = run_err(r#""".instance_variable_set(:name, 1)"#);
    assert!(error.contains("`name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_set_converts_argument_with_to_str() {
    let result = run(r#"
class Name
  def to_str
    "@test"
  end
end
obj = Object.new
obj.instance_variable_set(Name.new, 7)
obj.instance_variable_get(:@test)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn instance_variable_set_integer_raises_type_error() {
    let error = run_err("Object.new.instance_variable_set(10, 1)");
    assert!(error.contains("no implicit conversion of Integer into String"));
}

#[test]
fn instance_variable_set_accepts_a_non_ascii_name() {
    let result = run(r#"
obj = Object.new
obj.instance_variable_set(:@été, 5)
obj.instance_variable_get(:@été)
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn instance_variable_get_symbol_without_at_prefix_raises_name_error() {
    let error = run_err("Object.new.instance_variable_get(:name)");
    assert!(error.contains("`name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_get_bare_at_raises_name_error() {
    let error = run_err(r#"Object.new.instance_variable_get("@")"#);
    assert!(error.contains("`@' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_get_digit_start_raises_name_error() {
    let error = run_err(r#"Object.new.instance_variable_get("@0")"#);
    assert!(error.contains("`@0' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_get_class_variable_name_raises_name_error() {
    let error = run_err(r#"Object.new.instance_variable_get("@@name")"#);
    assert!(error.contains("`@@name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_get_integer_raises_type_error() {
    let error = run_err("Object.new.instance_variable_get(10)");
    assert!(error.contains("no implicit conversion of Integer into String"));
}

#[test]
fn instance_variable_get_converts_argument_with_to_str() {
    let result = run(r#"
class Name
  def to_str
    "@test"
  end
end
obj = Object.new
obj.instance_variable_set(:@test, 7)
obj.instance_variable_get(Name.new)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn instance_variable_get_to_str_returning_non_string_raises_type_error() {
    let error = run_err(
        r#"
class Name
  def to_str
    123
  end
end
Object.new.instance_variable_get(Name.new)
"#,
    );
    assert!(error.contains("can't convert Integer to String"));
}

#[test]
fn instance_variable_get_on_nil_returns_nil() {
    let result = run("nil.instance_variable_get(:@foo)");
    assert_eq!(result, Some(Object::Nil));
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
