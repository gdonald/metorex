// Type system integration tests for Metorex runtime objects
// Tests the Object type system including equality, hashing, and type operations

use metorex::object::{BlockStatement, Class, Exception, Instance, Method, Object, ObjectHash};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ============================================================================
// Object Creation Tests
// ============================================================================

#[test]
fn test_create_all_object_types() {
    // Primitive types
    let nil = Object::Nil;
    let bool_obj = Object::Bool(true);
    let int_obj = Object::Int(42);
    let float_obj = Object::Float(3.14);
    let string_obj = Object::string("test");

    // Collection types
    let array_obj = Object::empty_array();
    let dict_obj = Object::empty_dict();
    let set_obj = Object::Set(Rc::new(RefCell::new(HashSet::new())));

    // Verify types exist
    assert!(matches!(nil, Object::Nil));
    assert!(matches!(bool_obj, Object::Bool(_)));
    assert!(matches!(int_obj, Object::Int(_)));
    assert!(matches!(float_obj, Object::Float(_)));
    assert!(matches!(string_obj, Object::String(_)));
    assert!(matches!(array_obj, Object::Array(_)));
    assert!(matches!(dict_obj, Object::Dict(_)));
    assert!(matches!(set_obj, Object::Set(_)));
}

#[test]
fn test_create_complex_types() {
    // Class
    let class = Rc::new(Class::new("TestClass".to_string(), None));
    let class_obj = Object::Class(class);

    // Instance
    let class = Rc::new(Class::new("TestClass".to_string(), None));
    let instance = Rc::new(RefCell::new(Instance::new(Rc::clone(&class))));
    let instance_obj = Object::Instance(instance);

    // Method
    let method = Rc::new(Method::new("test_method".to_string(), vec![], vec![]));
    let method_obj = Object::Method(method);

    // Block
    let block = Rc::new(BlockStatement::new(vec![], vec![], HashMap::new()));
    let block_obj = Object::Block(block);

    // Exception
    let exception = Rc::new(RefCell::new(Exception::new(
        "TestError".to_string(),
        "test message".to_string(),
    )));
    let exception_obj = Object::Exception(exception);

    // Result
    let result_ok = Object::Result(Ok(Box::new(Object::Int(42))));
    let result_err = Object::Result(Err(Box::new(Object::string("error"))));

    // Verify types exist
    assert!(matches!(class_obj, Object::Class(_)));
    assert!(matches!(instance_obj, Object::Instance(_)));
    assert!(matches!(method_obj, Object::Method(_)));
    assert!(matches!(block_obj, Object::Block(_)));
    assert!(matches!(exception_obj, Object::Exception(_)));
    assert!(matches!(result_ok, Object::Result(Ok(_))));
    assert!(matches!(result_err, Object::Result(Err(_))));
}

// ============================================================================
// Deep Equality Tests
// ============================================================================

#[test]
fn test_primitive_equality() {
    assert!(Object::Nil.equals(&Object::Nil));
    assert!(Object::Bool(true).equals(&Object::Bool(true)));
    assert!(Object::Int(42).equals(&Object::Int(42)));
    assert!(Object::Float(3.14).equals(&Object::Float(3.14)));
    assert!(Object::string("test").equals(&Object::string("test")));

    // Different values should not be equal
    assert!(!Object::Bool(true).equals(&Object::Bool(false)));
    assert!(!Object::Int(42).equals(&Object::Int(43)));
    assert!(!Object::Float(3.14).equals(&Object::Float(2.71)));
    assert!(!Object::string("hello").equals(&Object::string("world")));
}

#[test]
fn test_cross_type_inequality() {
    let nil = Object::Nil;
    let bool_obj = Object::Bool(false);
    let int_obj = Object::Int(0);
    let float_obj = Object::Float(0.0);
    let string_obj = Object::string("0");

    // No type should equal any other type
    assert!(!nil.equals(&bool_obj));
    assert!(!nil.equals(&int_obj));
    assert!(!bool_obj.equals(&int_obj));
    assert!(!int_obj.equals(&float_obj));
    assert!(!int_obj.equals(&string_obj));
}

#[test]
fn test_array_deep_equality() {
    // Simple arrays
    let arr1 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Int(2),
        Object::Int(3),
    ])));
    let arr2 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Int(2),
        Object::Int(3),
    ])));
    assert!(arr1.equals(&arr2));

    // Different length arrays
    let arr3 = Object::Array(Rc::new(RefCell::new(vec![Object::Int(1), Object::Int(2)])));
    assert!(!arr1.equals(&arr3));

    // Different values
    let arr4 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Int(2),
        Object::Int(4),
    ])));
    assert!(!arr1.equals(&arr4));

    // Nested arrays
    let nested1 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), Object::Int(3)]))),
    ])));
    let nested2 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), Object::Int(3)]))),
    ])));
    assert!(nested1.equals(&nested2));

    // Different nested arrays
    let nested3 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), Object::Int(4)]))),
    ])));
    assert!(!nested1.equals(&nested3));
}

#[test]
fn test_dict_deep_equality() {
    // Simple dicts
    let mut map1 = HashMap::new();
    map1.insert("x".to_string(), Object::Int(10));
    map1.insert("y".to_string(), Object::Int(20));
    let dict1 = Object::Dict(Rc::new(RefCell::new(map1)));

    let mut map2 = HashMap::new();
    map2.insert("x".to_string(), Object::Int(10));
    map2.insert("y".to_string(), Object::Int(20));
    let dict2 = Object::Dict(Rc::new(RefCell::new(map2)));

    assert!(dict1.equals(&dict2));

    // Different values
    let mut map3 = HashMap::new();
    map3.insert("x".to_string(), Object::Int(10));
    map3.insert("y".to_string(), Object::Int(30));
    let dict3 = Object::Dict(Rc::new(RefCell::new(map3)));

    assert!(!dict1.equals(&dict3));

    // Different keys
    let mut map4 = HashMap::new();
    map4.insert("x".to_string(), Object::Int(10));
    map4.insert("z".to_string(), Object::Int(20));
    let dict4 = Object::Dict(Rc::new(RefCell::new(map4)));

    assert!(!dict1.equals(&dict4));

    // Nested dicts
    let mut inner1 = HashMap::new();
    inner1.insert("a".to_string(), Object::Int(1));

    let mut outer1 = HashMap::new();
    outer1.insert(
        "nested".to_string(),
        Object::Dict(Rc::new(RefCell::new(inner1))),
    );
    let nested_dict1 = Object::Dict(Rc::new(RefCell::new(outer1)));

    let mut inner2 = HashMap::new();
    inner2.insert("a".to_string(), Object::Int(1));

    let mut outer2 = HashMap::new();
    outer2.insert(
        "nested".to_string(),
        Object::Dict(Rc::new(RefCell::new(inner2))),
    );
    let nested_dict2 = Object::Dict(Rc::new(RefCell::new(outer2)));

    assert!(nested_dict1.equals(&nested_dict2));
}

#[test]
fn test_set_equality() {
    let mut set1 = HashSet::new();
    set1.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    set1.insert(ObjectHash::from_object(&Object::Int(2)).unwrap());
    set1.insert(ObjectHash::from_object(&Object::Int(3)).unwrap());

    let mut set2 = HashSet::new();
    set2.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    set2.insert(ObjectHash::from_object(&Object::Int(2)).unwrap());
    set2.insert(ObjectHash::from_object(&Object::Int(3)).unwrap());

    let obj1 = Object::Set(Rc::new(RefCell::new(set1)));
    let obj2 = Object::Set(Rc::new(RefCell::new(set2)));

    assert!(obj1.equals(&obj2));

    // Different size sets
    let mut set3 = HashSet::new();
    set3.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    let obj3 = Object::Set(Rc::new(RefCell::new(set3)));

    assert!(!obj1.equals(&obj3));
}

#[test]
fn test_result_equality() {
    let ok1 = Object::Result(Ok(Box::new(Object::Int(42))));
    let ok2 = Object::Result(Ok(Box::new(Object::Int(42))));
    let ok3 = Object::Result(Ok(Box::new(Object::Int(43))));

    let err1 = Object::Result(Err(Box::new(Object::string("error"))));
    let err2 = Object::Result(Err(Box::new(Object::string("error"))));
    let err3 = Object::Result(Err(Box::new(Object::string("different"))));

    // Ok equality
    assert!(ok1.equals(&ok2));
    assert!(!ok1.equals(&ok3));

    // Err equality
    assert!(err1.equals(&err2));
    assert!(!err1.equals(&err3));

    // Ok vs Err
    assert!(!ok1.equals(&err1));
}

#[test]
fn test_reference_equality() {
    // Instances with same reference should be equal
    let class = Rc::new(Class::new("Test".to_string(), None));
    let inst1 = Rc::new(RefCell::new(Instance::new(class)));
    let inst2 = Rc::clone(&inst1);

    let obj1 = Object::Instance(inst1);
    let obj2 = Object::Instance(inst2);

    assert!(obj1.equals(&obj2));

    // Classes with same reference should be equal
    let class1 = Rc::new(Class::new("Test".to_string(), None));
    let class2 = Rc::clone(&class1);

    let obj1 = Object::Class(class1);
    let obj2 = Object::Class(class2);

    assert!(obj1.equals(&obj2));
}

// ============================================================================
// Hashing Tests
// ============================================================================

#[test]
fn test_primitive_hashing() {
    // Nil
    let nil_hash = Object::Nil.hash();
    assert!(nil_hash.is_some());

    // Bool
    let bool_hash = Object::Bool(true).hash();
    assert!(bool_hash.is_some());

    // Int
    let int_hash = Object::Int(42).hash();
    assert!(int_hash.is_some());

    // Float
    let float_hash = Object::Float(3.14).hash();
    assert!(float_hash.is_some());

    // String
    let string_hash = Object::string("test").hash();
    assert!(string_hash.is_some());
}

#[test]
fn test_hash_consistency() {
    // Same values should always produce same hash
    for _ in 0..100 {
        let hash1 = Object::Int(42).hash().unwrap();
        let hash2 = Object::Int(42).hash().unwrap();
        assert_eq!(hash1, hash2);
    }

    // Same strings should hash the same
    for _ in 0..100 {
        let hash1 = Object::string("test").hash().unwrap();
        let hash2 = Object::string("test").hash().unwrap();
        assert_eq!(hash1, hash2);
    }
}

#[test]
fn test_different_values_different_hashes() {
    let hash1 = Object::Int(1).hash().unwrap();
    let hash2 = Object::Int(2).hash().unwrap();
    let hash3 = Object::Int(3).hash().unwrap();

    assert_ne!(hash1, hash2);
    assert_ne!(hash2, hash3);
    assert_ne!(hash1, hash3);

    let str_hash1 = Object::string("a").hash().unwrap();
    let str_hash2 = Object::string("b").hash().unwrap();
    let str_hash3 = Object::string("c").hash().unwrap();

    assert_ne!(str_hash1, str_hash2);
    assert_ne!(str_hash2, str_hash3);
    assert_ne!(str_hash1, str_hash3);
}

#[test]
fn test_non_hashable_types() {
    // Arrays should not be hashable
    let arr = Object::empty_array();
    assert!(arr.hash().is_none());

    // Dicts should not be hashable
    let dict = Object::empty_dict();
    assert!(dict.hash().is_none());

    // Instances should not be hashable
    let class = Rc::new(Class::new("Test".to_string(), None));
    let instance = Object::Instance(Rc::new(RefCell::new(Instance::new(class))));
    assert!(instance.hash().is_none());

    // Classes should not be hashable
    let class = Rc::new(Class::new("Test".to_string(), None));
    let class_obj = Object::Class(class);
    assert!(class_obj.hash().is_none());

    // Methods should not be hashable
    let method = Rc::new(Method::new("test".to_string(), vec![], vec![]));
    let method_obj = Object::Method(method);
    assert!(method_obj.hash().is_none());

    // Blocks should not be hashable
    let block = Rc::new(BlockStatement::new(vec![], vec![], HashMap::new()));
    let block_obj = Object::Block(block);
    assert!(block_obj.hash().is_none());

    // Exceptions should not be hashable
    let exc = Rc::new(RefCell::new(Exception::new(
        "Error".to_string(),
        "msg".to_string(),
    )));
    let exc_obj = Object::Exception(exc);
    assert!(exc_obj.hash().is_none());

    // Results should not be hashable
    let result = Object::Result(Ok(Box::new(Object::Int(42))));
    assert!(result.hash().is_none());
}

#[test]
fn test_object_hash_wrapper() {
    // Test ObjectHash wrapper creation
    let hash1 = ObjectHash::from_object(&Object::Int(42));
    let hash2 = ObjectHash::from_object(&Object::Int(42));
    let hash3 = ObjectHash::from_object(&Object::Int(43));

    assert!(hash1.is_some());
    assert!(hash2.is_some());
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);

    // Non-hashable should return None
    let arr_hash = ObjectHash::from_object(&Object::empty_array());
    assert!(arr_hash.is_none());
}

// ============================================================================
// String Representation Tests
// ============================================================================

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
fn test_to_string_collections() {
    // Array
    let arr = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Int(2),
        Object::Int(3),
    ])));
    assert_eq!(arr.to_string(), "[1, 2, 3]");

    // Empty array
    let empty_arr = Object::empty_array();
    assert_eq!(empty_arr.to_string(), "[]");

    // Nested array
    let nested = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), Object::Int(3)]))),
    ])));
    assert_eq!(nested.to_string(), "[1, [2, 3]]");
}

#[test]
fn test_to_string_dict() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), Object::Int(10));
    let dict = Object::Dict(Rc::new(RefCell::new(map)));
    let s = dict.to_string();

    // Dict output order is not guaranteed
    assert!(s.starts_with('{'));
    assert!(s.ends_with('}'));
    assert!(s.contains("x: 10"));

    // Empty dict
    let empty_dict = Object::empty_dict();
    assert_eq!(empty_dict.to_string(), "{}");
}

#[test]
fn test_to_string_class_and_instance() {
    let class = Rc::new(Class::new("MyClass".to_string(), None));
    let class_obj = Object::Class(Rc::clone(&class));
    assert_eq!(class_obj.to_string(), "<class MyClass>");

    let instance = Object::Instance(Rc::new(RefCell::new(Instance::new(class))));
    let s = instance.to_string();
    assert_eq!(s, "<MyClass instance>");
}

// ============================================================================
// Complex Scenarios
// ============================================================================

#[test]
fn test_mixed_type_collections() {
    // Array with mixed types
    let mixed_arr = Object::Array(Rc::new(RefCell::new(vec![
        Object::Nil,
        Object::Bool(true),
        Object::Int(42),
        Object::Float(3.14),
        Object::string("hello"),
    ])));

    let expected = "[nil, true, 42, 3.14, hello]";
    assert_eq!(mixed_arr.to_string(), expected);

    // Dict with mixed value types
    let mut mixed_map = HashMap::new();
    mixed_map.insert("nil".to_string(), Object::Nil);
    mixed_map.insert("bool".to_string(), Object::Bool(true));
    mixed_map.insert("int".to_string(), Object::Int(42));
    let mixed_dict = Object::Dict(Rc::new(RefCell::new(mixed_map)));

    let s = mixed_dict.to_string();
    assert!(s.contains("nil: nil"));
    assert!(s.contains("bool: true"));
    assert!(s.contains("int: 42"));
}

#[test]
fn test_deeply_nested_structures() {
    // Create a deeply nested array
    let level3 = Object::Array(Rc::new(RefCell::new(vec![Object::Int(3)])));
    let level2 = Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), level3])));
    let level1 = Object::Array(Rc::new(RefCell::new(vec![Object::Int(1), level2])));

    assert_eq!(level1.to_string(), "[1, [2, [3]]]");

    // Deep equality should work
    let level3_copy = Object::Array(Rc::new(RefCell::new(vec![Object::Int(3)])));
    let level2_copy = Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), level3_copy])));
    let level1_copy = Object::Array(Rc::new(RefCell::new(vec![Object::Int(1), level2_copy])));

    assert!(level1.equals(&level1_copy));
}

#[test]
fn test_float_epsilon_equality() {
    // Very close floats should be equal (within epsilon)
    let f1 = Object::Float(1.0);
    let f2 = Object::Float(1.0 + 1e-10);
    assert!(f1.equals(&f2));

    // Floats outside epsilon should not be equal
    let f3 = Object::Float(1.0);
    let f4 = Object::Float(1.01);
    assert!(!f3.equals(&f4));
}

#[test]
fn test_instance_variables() {
    let class = Rc::new(Class::new("Person".to_string(), None));
    let inst = Rc::new(RefCell::new(Instance::new(class)));

    // Set instance variables
    inst.borrow_mut()
        .set_var("name".to_string(), Object::string("Alice"));
    inst.borrow_mut()
        .set_var("age".to_string(), Object::Int(30));

    // Get instance variables
    let name = inst.borrow().get_var("name").cloned();
    let age = inst.borrow().get_var("age").cloned();

    assert!(name.is_some());
    assert!(age.is_some());
    assert_eq!(name.unwrap().to_string(), "Alice");
    assert_eq!(age.unwrap().to_string(), "30");

    // Non-existent variable
    let missing = inst.borrow().get_var("missing").cloned();
    assert!(missing.is_none());
}

#[test]
fn test_class_variables() {
    let class = Class::new("Counter".to_string(), None);

    // Set class variable
    class.set_class_var("count".to_string(), Object::Int(0));

    // Get class variable
    let count = class.get_class_var("count");
    assert!(count.is_some());
    assert_eq!(count.unwrap().to_string(), "0");

    // Update class variable
    class.set_class_var("count".to_string(), Object::Int(1));
    let count = class.get_class_var("count");
    assert_eq!(count.unwrap().to_string(), "1");
}

#[test]
fn test_result_type_operations() {
    // Ok result
    let ok = Object::Result(Ok(Box::new(Object::Int(42))));
    assert!(matches!(ok, Object::Result(Ok(_))));

    // Error result
    let err = Object::Result(Err(Box::new(Object::string("error message"))));
    assert!(matches!(err, Object::Result(Err(_))));

    // Nested results
    let nested_ok = Object::Result(Ok(Box::new(Object::Result(Ok(Box::new(Object::Int(42)))))));
    assert!(matches!(nested_ok, Object::Result(Ok(_))));
}
