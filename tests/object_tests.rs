// Unit tests for Metorex runtime Object system
// Tests object creation, type checking, equality, hashing, and string representation

use metorex::object::{BlockClosure, Class, Exception, Instance, Method, Object, ObjectHash};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ============================================================================
// Basic Object Tests
// ============================================================================

#[test]
fn test_nil_object() {
    let obj = Object::Nil;
    assert_eq!(obj.type_name(), "Nil");
    assert!(!obj.is_truthy());
    assert_eq!(format!("{}", obj), "nil");
}

#[test]
fn test_bool_object() {
    let obj_true = Object::Bool(true);
    let obj_false = Object::Bool(false);

    assert_eq!(obj_true.type_name(), "Bool");
    assert!(obj_true.is_truthy());
    assert!(!obj_false.is_truthy());
    assert_eq!(format!("{}", obj_true), "true");
    assert_eq!(format!("{}", obj_false), "false");
}

#[test]
fn test_int_object() {
    let obj = Object::Int(42);
    assert_eq!(obj.type_name(), "Int");
    assert!(obj.is_truthy());
    assert_eq!(format!("{}", obj), "42");
}

#[test]
fn test_float_object() {
    let obj = Object::Float(3.14);
    assert_eq!(obj.type_name(), "Float");
    assert!(obj.is_truthy());
    assert_eq!(format!("{}", obj), "3.14");
}

#[test]
fn test_string_object() {
    let obj = Object::string("hello");
    assert_eq!(obj.type_name(), "String");
    assert!(obj.is_truthy());
    assert_eq!(format!("{}", obj), "hello");
}

#[test]
fn test_array_object() {
    let obj = Object::array(vec![Object::Int(1), Object::Int(2), Object::Int(3)]);
    assert_eq!(obj.type_name(), "Array");
    assert!(obj.is_truthy());
    assert_eq!(format!("{}", obj), "[1, 2, 3]");
}

#[test]
fn test_empty_array() {
    let obj = Object::empty_array();
    assert_eq!(obj.type_name(), "Array");
    assert_eq!(format!("{}", obj), "[]");
}

#[test]
fn test_dict_object() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), Object::Int(1));
    map.insert("y".to_string(), Object::Int(2));
    let obj = Object::dict(map);
    assert_eq!(obj.type_name(), "Dict");
    assert!(obj.is_truthy());
    // Dict output order is not guaranteed, so just check it contains the values
    let output = format!("{}", obj);
    assert!(output.contains("x: 1") || output.contains("y: 2"));
}

#[test]
fn test_empty_dict() {
    let obj = Object::empty_dict();
    assert_eq!(obj.type_name(), "Dict");
    assert_eq!(format!("{}", obj), "{}");
}

#[test]
fn test_class_object() {
    let class = Rc::new(Class::new("MyClass".to_string(), None));
    let obj = Object::Class(class);
    assert_eq!(obj.type_name(), "Class");
    assert_eq!(format!("{}", obj), "<class MyClass>");
}

#[test]
fn test_instance_object() {
    let class = Rc::new(Class::new("MyClass".to_string(), None));
    let obj = Object::instance(class);
    assert_eq!(obj.type_name(), "Instance");
    assert_eq!(format!("{}", obj), "<MyClass instance>");
}

#[test]
fn test_exception_object() {
    let obj = Object::exception("RuntimeError", "Something went wrong");
    assert_eq!(obj.type_name(), "Exception");
    assert_eq!(format!("{}", obj), "RuntimeError: Something went wrong");
}

#[test]
fn test_result_ok() {
    let obj = Object::ok(Object::Int(42));
    assert_eq!(obj.type_name(), "Result");
    assert_eq!(format!("{}", obj), "Ok(42)");
}

#[test]
fn test_result_err() {
    let obj = Object::err(Object::string("error"));
    assert_eq!(obj.type_name(), "Result");
    assert_eq!(format!("{}", obj), "Err(error)");
}

#[test]
fn test_set_object() {
    let obj = Object::empty_set();
    assert_eq!(obj.type_name(), "Set");
    assert_eq!(format!("{}", obj), "#{}");
}

#[test]
fn test_instance_variables() {
    let class = Rc::new(Class::new("Person".to_string(), None));
    let obj = Object::instance(Rc::clone(&class));

    if let Object::Instance(inst) = obj {
        let mut instance = inst.borrow_mut();
        instance.set_var("name".to_string(), Object::string("Alice"));
        instance.set_var("age".to_string(), Object::Int(30));

        assert_eq!(instance.get_var("name"), Some(&Object::string("Alice")));
        assert_eq!(instance.get_var("age"), Some(&Object::Int(30)));
        assert_eq!(instance.get_var("nonexistent"), None);
    } else {
        panic!("Expected Instance object");
    }
}

#[test]
fn test_class_methods() {
    let mut class = Class::new("Calculator".to_string(), None);
    let method = Rc::new(Method::new(
        "add".to_string(),
        vec!["x".to_string(), "y".to_string()],
        vec![],
    ));
    class.add_method("add".to_string(), method);

    assert!(class.find_method("add").is_some());
    assert!(class.find_method("nonexistent").is_none());
}

#[test]
fn test_class_variables() {
    let class = Class::new("Counter".to_string(), None);
    class.set_class_var("count".to_string(), Object::Int(0));

    assert_eq!(class.get_class_var("count"), Some(Object::Int(0)));
    assert_eq!(class.get_class_var("nonexistent"), None);
}

#[test]
fn test_object_hash() {
    let hash1 = ObjectHash::from_object(&Object::Int(42));
    let hash2 = ObjectHash::from_object(&Object::Int(42));
    assert_eq!(hash1, hash2);

    let hash3 = ObjectHash::from_object(&Object::string("hello"));
    assert_ne!(hash1, hash3);

    // Non-hashable objects return None
    let hash4 = ObjectHash::from_object(&Object::empty_array());
    assert!(hash4.is_none());
}

// ============================================================================
// Type System Tests - equals() method
// ============================================================================

#[test]
fn test_equals_nil() {
    assert!(Object::Nil.equals(&Object::Nil));
    assert!(!Object::Nil.equals(&Object::Bool(false)));
    assert!(!Object::Nil.equals(&Object::Int(0)));
}

#[test]
fn test_equals_bool() {
    assert!(Object::Bool(true).equals(&Object::Bool(true)));
    assert!(Object::Bool(false).equals(&Object::Bool(false)));
    assert!(!Object::Bool(true).equals(&Object::Bool(false)));
    assert!(!Object::Bool(true).equals(&Object::Int(1)));
}

#[test]
fn test_equals_int() {
    assert!(Object::Int(42).equals(&Object::Int(42)));
    assert!(!Object::Int(42).equals(&Object::Int(43)));
    assert!(!Object::Int(42).equals(&Object::Float(42.0)));
}

#[test]
fn test_equals_float() {
    assert!(Object::Float(3.14).equals(&Object::Float(3.14)));
    assert!(Object::Float(1.0).equals(&Object::Float(1.0 + 1e-10))); // Within epsilon
    assert!(!Object::Float(3.14).equals(&Object::Float(2.71)));
    assert!(!Object::Float(1.0).equals(&Object::Int(1)));
}

#[test]
fn test_equals_string() {
    let s1 = Object::string("hello");
    let s2 = Object::string("hello");
    let s3 = Object::string("world");

    assert!(s1.equals(&s2));
    assert!(!s1.equals(&s3));
    assert!(!s1.equals(&Object::Nil));
}

#[test]
fn test_equals_array_simple() {
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
    let arr3 = Object::Array(Rc::new(RefCell::new(vec![Object::Int(1), Object::Int(2)])));

    assert!(arr1.equals(&arr2));
    assert!(!arr1.equals(&arr3));
}

#[test]
fn test_equals_array_nested() {
    let arr1 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), Object::Int(3)]))),
    ])));
    let arr2 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), Object::Int(3)]))),
    ])));
    let arr3 = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Array(Rc::new(RefCell::new(vec![Object::Int(2), Object::Int(4)]))),
    ])));

    assert!(arr1.equals(&arr2));
    assert!(!arr1.equals(&arr3));
}

#[test]
fn test_equals_dict_simple() {
    let mut map1 = HashMap::new();
    map1.insert("x".to_string(), Object::Int(10));
    map1.insert("y".to_string(), Object::Int(20));
    let dict1 = Object::Dict(Rc::new(RefCell::new(map1)));

    let mut map2 = HashMap::new();
    map2.insert("x".to_string(), Object::Int(10));
    map2.insert("y".to_string(), Object::Int(20));
    let dict2 = Object::Dict(Rc::new(RefCell::new(map2)));

    let mut map3 = HashMap::new();
    map3.insert("x".to_string(), Object::Int(10));
    let dict3 = Object::Dict(Rc::new(RefCell::new(map3)));

    assert!(dict1.equals(&dict2));
    assert!(!dict1.equals(&dict3));
}

#[test]
fn test_equals_dict_nested() {
    let mut inner1 = HashMap::new();
    inner1.insert("a".to_string(), Object::Int(1));

    let mut map1 = HashMap::new();
    map1.insert("x".to_string(), Object::Dict(Rc::new(RefCell::new(inner1))));
    let dict1 = Object::Dict(Rc::new(RefCell::new(map1)));

    let mut inner2 = HashMap::new();
    inner2.insert("a".to_string(), Object::Int(1));

    let mut map2 = HashMap::new();
    map2.insert("x".to_string(), Object::Dict(Rc::new(RefCell::new(inner2))));
    let dict2 = Object::Dict(Rc::new(RefCell::new(map2)));

    let mut inner3 = HashMap::new();
    inner3.insert("a".to_string(), Object::Int(2));

    let mut map3 = HashMap::new();
    map3.insert("x".to_string(), Object::Dict(Rc::new(RefCell::new(inner3))));
    let dict3 = Object::Dict(Rc::new(RefCell::new(map3)));

    assert!(dict1.equals(&dict2));
    assert!(!dict1.equals(&dict3));
}

#[test]
fn test_equals_instance() {
    let class = Rc::new(Class {
        name: "TestClass".to_string(),
        superclass: None,
        methods: HashMap::new(),
        class_vars: RefCell::new(HashMap::new()),
    });

    let inst1 = Rc::new(RefCell::new(Instance {
        class: Rc::clone(&class),
        instance_vars: HashMap::new(),
    }));
    let inst2 = Rc::clone(&inst1);
    let inst3 = Rc::new(RefCell::new(Instance {
        class: Rc::clone(&class),
        instance_vars: HashMap::new(),
    }));

    let obj1 = Object::Instance(inst1);
    let obj2 = Object::Instance(inst2);
    let obj3 = Object::Instance(inst3);

    assert!(obj1.equals(&obj2)); // Same reference
    assert!(!obj1.equals(&obj3)); // Different reference
}

#[test]
fn test_equals_class() {
    let class1 = Rc::new(Class {
        name: "Class1".to_string(),
        superclass: None,
        methods: HashMap::new(),
        class_vars: RefCell::new(HashMap::new()),
    });
    let class2 = Rc::clone(&class1);
    let class3 = Rc::new(Class {
        name: "Class1".to_string(),
        superclass: None,
        methods: HashMap::new(),
        class_vars: RefCell::new(HashMap::new()),
    });

    let obj1 = Object::Class(class1);
    let obj2 = Object::Class(class2);
    let obj3 = Object::Class(class3);

    assert!(obj1.equals(&obj2)); // Same reference
    assert!(!obj1.equals(&obj3)); // Different reference even with same name
}

#[test]
fn test_equals_method() {
    let method1 = Rc::new(Method {
        name: "foo".to_string(),
        parameters: vec![],
        body: vec![],
        receiver: None,
    });
    let method2 = Rc::clone(&method1);
    let method3 = Rc::new(Method {
        name: "foo".to_string(),
        parameters: vec![],
        body: vec![],
        receiver: None,
    });

    let obj1 = Object::Method(method1);
    let obj2 = Object::Method(method2);
    let obj3 = Object::Method(method3);

    assert!(obj1.equals(&obj2)); // Same reference
    assert!(!obj1.equals(&obj3)); // Different reference
}

#[test]
fn test_equals_block() {
    let block1 = Rc::new(BlockClosure {
        parameters: vec![],
        body: vec![],
        captured_vars: HashMap::new(),
    });
    let block2 = Rc::clone(&block1);
    let block3 = Rc::new(BlockClosure {
        parameters: vec![],
        body: vec![],
        captured_vars: HashMap::new(),
    });

    let obj1 = Object::Block(block1);
    let obj2 = Object::Block(block2);
    let obj3 = Object::Block(block3);

    assert!(obj1.equals(&obj2)); // Same reference
    assert!(!obj1.equals(&obj3)); // Different reference
}

#[test]
fn test_equals_exception() {
    let exc1 = Rc::new(RefCell::new(Exception {
        exception_type: "RuntimeError".to_string(),
        message: "error".to_string(),
        backtrace: None,
    }));
    let exc2 = Rc::clone(&exc1);
    let exc3 = Rc::new(RefCell::new(Exception {
        exception_type: "RuntimeError".to_string(),
        message: "error".to_string(),
        backtrace: None,
    }));

    let obj1 = Object::Exception(exc1);
    let obj2 = Object::Exception(exc2);
    let obj3 = Object::Exception(exc3);

    assert!(obj1.equals(&obj2)); // Same reference
    assert!(!obj1.equals(&obj3)); // Different reference
}

#[test]
fn test_equals_set() {
    let mut set1 = HashSet::new();
    set1.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    set1.insert(ObjectHash::from_object(&Object::Int(2)).unwrap());

    let mut set2 = HashSet::new();
    set2.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());
    set2.insert(ObjectHash::from_object(&Object::Int(2)).unwrap());

    let mut set3 = HashSet::new();
    set3.insert(ObjectHash::from_object(&Object::Int(1)).unwrap());

    let obj1 = Object::Set(Rc::new(RefCell::new(set1)));
    let obj2 = Object::Set(Rc::new(RefCell::new(set2)));
    let obj3 = Object::Set(Rc::new(RefCell::new(set3)));

    assert!(obj1.equals(&obj2));
    assert!(!obj1.equals(&obj3));
}

#[test]
fn test_equals_result() {
    let ok1 = Object::Result(Ok(Box::new(Object::Int(42))));
    let ok2 = Object::Result(Ok(Box::new(Object::Int(42))));
    let ok3 = Object::Result(Ok(Box::new(Object::Int(43))));
    let err1 = Object::Result(Err(Box::new(Object::string("error"))));
    let err2 = Object::Result(Err(Box::new(Object::string("error"))));

    assert!(ok1.equals(&ok2));
    assert!(!ok1.equals(&ok3));
    assert!(!ok1.equals(&err1));
    assert!(err1.equals(&err2));
}

#[test]
fn test_equals_different_types() {
    let int_obj = Object::Int(42);
    let float_obj = Object::Float(42.0);
    let string_obj = Object::string("42");
    let nil_obj = Object::Nil;
    let bool_obj = Object::Bool(false);

    assert!(!int_obj.equals(&float_obj));
    assert!(!int_obj.equals(&string_obj));
    assert!(!int_obj.equals(&nil_obj));
    assert!(!int_obj.equals(&bool_obj));
}

// ============================================================================
// Type System Tests - hash() method
// ============================================================================

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
    // Arrays are not hashable
    let arr = Object::empty_array();
    assert!(arr.hash().is_none());

    // Dicts are not hashable
    let dict = Object::empty_dict();
    assert!(dict.hash().is_none());

    // Instances are not hashable
    let class = Rc::new(Class {
        name: "Test".to_string(),
        superclass: None,
        methods: HashMap::new(),
        class_vars: RefCell::new(HashMap::new()),
    });
    let inst = Object::Instance(Rc::new(RefCell::new(Instance {
        class,
        instance_vars: HashMap::new(),
    })));
    assert!(inst.hash().is_none());
}

#[test]
fn test_hash_consistency() {
    // Same values should always produce same hash
    for _ in 0..10 {
        let hash1 = Object::Int(100).hash().unwrap();
        let hash2 = Object::Int(100).hash().unwrap();
        assert_eq!(hash1, hash2);
    }
}

// ============================================================================
// Type System Tests - to_string() method
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
    let mut map = HashMap::new();
    map.insert("x".to_string(), Object::Int(10));
    let dict = Object::Dict(Rc::new(RefCell::new(map)));
    let s = dict.to_string();
    assert!(s.starts_with('{') && s.ends_with('}'));
    assert!(s.contains("x: 10"));
}

#[test]
fn test_to_string_class() {
    let class = Rc::new(Class {
        name: "MyClass".to_string(),
        superclass: None,
        methods: HashMap::new(),
        class_vars: RefCell::new(HashMap::new()),
    });
    let obj = Object::Class(class);
    assert_eq!(obj.to_string(), "<class MyClass>");
}
