// Unit tests for Metorex runtime Object system
// Tests object creation, type checking, equality, hashing, and string representation

use metorex::object::{BlockStatement, Class, Exception, Instance, Method, Object, ObjectHash};
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
    let class = Rc::new(Class::new("MyClass", None));
    let obj = Object::Class(class);
    assert_eq!(obj.type_name(), "Class");
    assert_eq!(format!("{}", obj), "<class MyClass>");
}

#[test]
fn test_instance_object() {
    let class = Rc::new(Class::new("MyClass", None));
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
    let class = Rc::new(Class::new("Person", None));
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
    let class = Class::new("Calculator", None);
    let method = Rc::new(Method::new(
        "add".to_string(),
        vec!["x".to_string(), "y".to_string()],
        vec![],
    ));
    class.define_method("add", method);

    assert!(class.find_method("add").is_some());
    assert!(class.find_method("nonexistent").is_none());
}

#[test]
fn test_class_variables() {
    let class = Class::new("Counter", None);
    class.set_class_var("count", Object::Int(0));

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
    let class = Rc::new(Class::new("TestClass", None));

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
    let class1 = Rc::new(Class::new("Class1", None));
    let class2 = Rc::clone(&class1);
    let class3 = Rc::new(Class::new("Class1", None));

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
    let block1 = Rc::new(BlockStatement {
        parameters: vec![],
        body: vec![],
        captured_vars: HashMap::new(),
    });
    let block2 = Rc::clone(&block1);
    let block3 = Rc::new(BlockStatement {
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
    let exc1 = Rc::new(RefCell::new(Exception::new(
        "RuntimeError".to_string(),
        "error".to_string(),
    )));
    let exc2 = Rc::clone(&exc1);
    let exc3 = Rc::new(RefCell::new(Exception::new(
        "RuntimeError".to_string(),
        "error".to_string(),
    )));

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
    let class = Rc::new(Class::new("Test", None));
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
    let class = Rc::new(Class::new("MyClass", None));
    let obj = Object::Class(class);
    assert_eq!(obj.to_string(), "<class MyClass>");
}

// ============================================================================
// Instance Tests - Comprehensive method dispatch and variable tests
// ============================================================================

#[test]
fn test_instance_new() {
    let class = Rc::new(Class::new("TestClass", None));
    let instance = Instance::new(Rc::clone(&class));

    assert_eq!(instance.class_name(), "TestClass");
    assert!(instance.instance_vars.is_empty());
}

#[test]
fn test_instance_set_and_get_var() {
    let class = Rc::new(Class::new("TestClass", None));
    let mut instance = Instance::new(Rc::clone(&class));

    instance.set_var("name".to_string(), Object::string("Bob"));
    instance.set_var("age".to_string(), Object::Int(25));

    assert_eq!(instance.get_var("name"), Some(&Object::string("Bob")));
    assert_eq!(instance.get_var("age"), Some(&Object::Int(25)));
    assert_eq!(instance.get_var("nonexistent"), None);
}

#[test]
fn test_instance_var_declared() {
    let class = Rc::new(Class::new("Person", None));
    class.declare_instance_var("name");
    class.declare_instance_var("age");

    let instance = Instance::new(Rc::clone(&class));

    assert!(instance.is_var_declared("name"));
    assert!(instance.is_var_declared("age"));
    assert!(!instance.is_var_declared("height"));
}

#[test]
fn test_instance_var_declared_with_inheritance() {
    let parent = Rc::new(Class::new("Parent", None));
    parent.declare_instance_var("parent_var");

    let child = Rc::new(Class::new("Child", Some(Rc::clone(&parent))));
    child.declare_instance_var("child_var");

    let instance = Instance::new(Rc::clone(&child));

    assert!(instance.is_var_declared("child_var"));
    assert!(instance.is_var_declared("parent_var"));
    assert!(!instance.is_var_declared("unknown_var"));
}

#[test]
fn test_instance_find_method() {
    let class = Rc::new(Class::new("Calculator", None));
    let method = Rc::new(Method::new(
        "add".to_string(),
        vec!["x".to_string(), "y".to_string()],
        vec![],
    ));
    class.define_method("add", Rc::clone(&method));

    let instance = Instance::new(Rc::clone(&class));

    let found = instance.find_method("add");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "add");

    assert!(instance.find_method("nonexistent").is_none());
}

#[test]
fn test_instance_find_method_with_inheritance() {
    let parent = Rc::new(Class::new("Parent", None));
    let parent_method = Rc::new(Method::new("parent_method".to_string(), vec![], vec![]));
    parent.define_method("parent_method", Rc::clone(&parent_method));

    let child = Rc::new(Class::new("Child", Some(Rc::clone(&parent))));
    let child_method = Rc::new(Method::new("child_method".to_string(), vec![], vec![]));
    child.define_method("child_method", Rc::clone(&child_method));

    let instance = Instance::new(Rc::clone(&child));

    // Should find method on child class
    assert!(instance.find_method("child_method").is_some());

    // Should find method on parent class through inheritance
    assert!(instance.find_method("parent_method").is_some());

    // Should not find nonexistent method
    assert!(instance.find_method("nonexistent").is_none());
}

#[test]
fn test_instance_class_name() {
    let class = Rc::new(Class::new("MyTestClass", None));
    let instance = Instance::new(Rc::clone(&class));

    assert_eq!(instance.class_name(), "MyTestClass");
}

#[test]
fn test_instance_multiple_vars() {
    let class = Rc::new(Class::new("DataHolder", None));
    let mut instance = Instance::new(Rc::clone(&class));

    instance.set_var("string_var".to_string(), Object::string("hello"));
    instance.set_var("int_var".to_string(), Object::Int(42));
    instance.set_var("bool_var".to_string(), Object::Bool(true));
    instance.set_var("nil_var".to_string(), Object::Nil);
    instance.set_var(
        "array_var".to_string(),
        Object::array(vec![Object::Int(1), Object::Int(2)]),
    );

    assert_eq!(
        instance.get_var("string_var"),
        Some(&Object::string("hello"))
    );
    assert_eq!(instance.get_var("int_var"), Some(&Object::Int(42)));
    assert_eq!(instance.get_var("bool_var"), Some(&Object::Bool(true)));
    assert_eq!(instance.get_var("nil_var"), Some(&Object::Nil));
    assert!(matches!(
        instance.get_var("array_var"),
        Some(Object::Array(_))
    ));
}

#[test]
fn test_instance_var_update() {
    let class = Rc::new(Class::new("Counter", None));
    let mut instance = Instance::new(Rc::clone(&class));

    instance.set_var("count".to_string(), Object::Int(0));
    assert_eq!(instance.get_var("count"), Some(&Object::Int(0)));

    instance.set_var("count".to_string(), Object::Int(1));
    assert_eq!(instance.get_var("count"), Some(&Object::Int(1)));

    instance.set_var("count".to_string(), Object::Int(2));
    assert_eq!(instance.get_var("count"), Some(&Object::Int(2)));
}

#[test]
fn test_instance_with_object_wrapper() {
    let class = Rc::new(Class::new("TestClass", None));
    let obj = Object::instance(Rc::clone(&class));

    if let Object::Instance(inst) = obj {
        let mut instance = inst.borrow_mut();
        instance.set_var("test".to_string(), Object::Int(100));

        assert_eq!(instance.get_var("test"), Some(&Object::Int(100)));
        assert_eq!(instance.class_name(), "TestClass");
    } else {
        panic!("Expected Instance object");
    }
}

// ============================================================================
// Method Tests - Comprehensive tests for Method structure and Callable trait
// ============================================================================

#[test]
fn test_method_new() {
    let method = Method::new("test_method".to_string(), vec![], vec![]);

    assert_eq!(method.name, "test_method");
    assert!(method.parameters.is_empty());
    assert!(method.body.is_empty());
    assert!(!method.is_bound());
}

#[test]
fn test_method_with_parameters() {
    let params = vec!["x".to_string(), "y".to_string(), "z".to_string()];
    let method = Method::new("add".to_string(), params.clone(), vec![]);

    assert_eq!(method.name, "add");
    assert_eq!(method.parameters, params);
    assert!(method.body.is_empty());
}

#[test]
fn test_method_bind() {
    let method = Method::new("test".to_string(), vec![], vec![]);
    let receiver = Object::Int(42);

    let bound_method = method.bind(receiver.clone());

    assert!(bound_method.is_bound());
    assert_eq!(bound_method.receiver(), Some(&receiver));
    assert_eq!(bound_method.name, "test");
}

#[test]
fn test_method_unbound() {
    let method = Method::new("test".to_string(), vec![], vec![]);

    assert!(!method.is_bound());
    assert_eq!(method.receiver(), None);
}

#[test]
fn test_method_callable_trait() {
    use metorex::object::Callable;

    let params = vec!["a".to_string(), "b".to_string()];
    let method = Method::new("multiply".to_string(), params.clone(), vec![]);

    assert_eq!(method.name(), "multiply");
    assert_eq!(method.parameters(), &params[..]);
    assert_eq!(method.body(), &[][..]);
    assert_eq!(method.arity(), 2);
}

#[test]
fn test_method_bind_preserves_properties() {
    let params = vec!["x".to_string()];
    let method = Method::new("square".to_string(), params.clone(), vec![]);
    let receiver = Object::string("receiver");

    let bound = method.bind(receiver);

    assert_eq!(bound.name, "square");
    assert_eq!(bound.parameters, params);
    assert!(bound.is_bound());
}

#[test]
fn test_method_equality() {
    let method1 = Method::new("test".to_string(), vec![], vec![]);
    let method2 = Method::new("test".to_string(), vec![], vec![]);

    // Methods with same content are equal
    assert_eq!(method1, method2);
}

#[test]
fn test_method_inequality_different_name() {
    let method1 = Method::new("test1".to_string(), vec![], vec![]);
    let method2 = Method::new("test2".to_string(), vec![], vec![]);

    assert_ne!(method1, method2);
}

#[test]
fn test_method_inequality_different_params() {
    let method1 = Method::new("test".to_string(), vec!["a".to_string()], vec![]);
    let method2 = Method::new("test".to_string(), vec!["b".to_string()], vec![]);

    assert_ne!(method1, method2);
}

#[test]
fn test_block_closure_callable_trait() {
    use metorex::object::Callable;

    let params = vec!["x".to_string()];
    let block = BlockStatement::new(params.clone(), vec![], HashMap::new());

    assert_eq!(block.name(), "<block>");
    assert_eq!(block.parameters(), &params[..]);
    assert_eq!(block.body(), &[][..]);
    assert_eq!(block.arity(), 1);
}

#[test]
fn test_block_closure_captured_vars() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut captured = HashMap::new();
    captured.insert("outer".to_string(), Rc::new(RefCell::new(Object::Int(10))));
    captured.insert("count".to_string(), Rc::new(RefCell::new(Object::Int(0))));

    let block = BlockStatement::new(vec![], vec![], captured.clone());

    assert_eq!(block.captured_vars().len(), 2);
    assert_eq!(
        *block.captured_vars().get("outer").unwrap().borrow(),
        Object::Int(10)
    );
    assert_eq!(
        *block.captured_vars().get("count").unwrap().borrow(),
        Object::Int(0)
    );
}

#[test]
fn test_block_closure_empty_captures() {
    let block = BlockStatement::new(vec!["x".to_string()], vec![], HashMap::new());

    assert!(block.captured_vars().is_empty());
}
