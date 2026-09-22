// Whether a value belongs to a class or one of its ancestors.

use super::*;

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
    assert_eq!(result, Some(Object::string("Numeric".to_string())));
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
    assert_eq!(result, Some(Object::string("Animal".to_string())));
}

#[test]
fn superclass_of_object_is_basicobject() {
    let result = run("Object.superclass.name");
    assert_eq!(result, Some(Object::string("BasicObject".to_string())));
}

#[test]
fn superclass_of_basicobject_is_nil() {
    let result = run("BasicObject.superclass");
    assert_eq!(result, Some(Object::Nil));
}
