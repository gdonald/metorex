// Type System Tests - to_string() method

use indexmap::IndexMap;
use metorex::object::{Class, Object};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_to_string_primitives() {
    assert_eq!(Object::Nil.to_string(), "nil");
    assert_eq!(Object::Bool(true).to_string(), "true");
    assert_eq!(Object::Bool(false).to_string(), "false");
    assert_eq!(Object::Int(42).to_string(), "42");
    assert_eq!(Object::Float(3.14).to_string(), "3.14");
    assert_eq!(Object::string("hello").to_string(), "hello");
}

#[test]
fn test_to_string_array() {
    let arr = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Int(2),
        Object::Int(3),
    ])));
    assert_eq!(arr.to_string(), "[1, 2, 3]");
}

#[test]
fn test_to_string_dict() {
    let mut map = IndexMap::new();
    map.insert("x".to_string(), Object::Int(10));
    let dict = Object::Dict(Rc::new(RefCell::new(map)));
    let s = dict.to_string();
    assert!(s.starts_with('{') && s.ends_with('}'));
    assert!(s.contains("x: 10"));
}

#[test]
fn test_to_string_class() {
    let class = Rc::new(Class::new("MyClass", None));
    let obj = Object::Class(class);
    assert_eq!(obj.to_string(), "MyClass");
}

// ── Set/NativeFunction display (from additional_tests) ──────────────────────

#[test]
fn test_set_display_multiple_elements() {
    use metorex::object::ObjectHash;
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    set.insert(ObjectHash::from_object(&Object::Int(2)).unwrap());
    let obj = Object::Set(Rc::new(RefCell::new(set)));
    let display = format!("{}", obj);
    assert!(display.starts_with("#{"));
    assert!(display.ends_with("}"));
    assert!(display.contains("1"));
    assert!(display.contains("2"));
}

#[test]
fn test_native_function_display() {
    let obj = Object::NativeFunction("puts".to_string());
    assert_eq!(format!("{}", obj), "<native function puts>");
}
