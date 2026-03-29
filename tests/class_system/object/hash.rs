// Type System Tests - hash() method

use metorex::object::{Class, Instance, Object};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_hash_nil() {
    let hash = Object::Nil.hash();
    assert!(hash.is_some());

    let hash1 = Object::Nil.hash().unwrap();
    let hash2 = Object::Nil.hash().unwrap();
    assert_eq!(hash1, hash2);
}

#[test]
fn test_hash_bool() {
    let hash_true1 = Object::Bool(true).hash().unwrap();
    let hash_true2 = Object::Bool(true).hash().unwrap();
    assert_eq!(hash_true1, hash_true2);

    let hash_false = Object::Bool(false).hash().unwrap();
    assert_ne!(hash_true1, hash_false);
}

#[test]
fn test_hash_int() {
    let hash1 = Object::Int(42).hash().unwrap();
    let hash2 = Object::Int(42).hash().unwrap();
    assert_eq!(hash1, hash2);

    let hash3 = Object::Int(43).hash().unwrap();
    assert_ne!(hash1, hash3);
}

#[test]
fn test_hash_float() {
    let hash1 = Object::Float(3.14).hash().unwrap();
    let hash2 = Object::Float(3.14).hash().unwrap();
    assert_eq!(hash1, hash2);

    let hash3 = Object::Float(2.71).hash().unwrap();
    assert_ne!(hash1, hash3);
}

#[test]
fn test_hash_string() {
    let hash1 = Object::string("hello").hash().unwrap();
    let hash2 = Object::string("hello").hash().unwrap();
    assert_eq!(hash1, hash2);

    let hash3 = Object::string("world").hash().unwrap();
    assert_ne!(hash1, hash3);
}

#[test]
fn test_hash_non_hashable() {
    let arr = Object::empty_array();
    assert!(arr.hash().is_none());

    let dict = Object::empty_dict();
    assert!(dict.hash().is_none());

    let class = Rc::new(Class::new("Test", None));
    let inst = Object::Instance(Rc::new(RefCell::new(Instance {
        class,
        instance_vars: HashMap::new(),
    })));
    assert!(inst.hash().is_none());
}

#[test]
fn test_hash_consistency() {
    for _ in 0..10 {
        let hash1 = Object::Int(100).hash().unwrap();
        let hash2 = Object::Int(100).hash().unwrap();
        assert_eq!(hash1, hash2);
    }
}
