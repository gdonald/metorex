// Type System Tests - equals() method

use metorex::object::{BlockStatement, Class, Exception, Instance, Method, Object, ObjectHash};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

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
        singleton_methods: Rc::new(RefCell::new(HashMap::new())),
        singleton_class: Rc::new(RefCell::new(None)),
    }));
    let inst2 = Rc::clone(&inst1);
    let inst3 = Rc::new(RefCell::new(Instance {
        class: Rc::clone(&class),
        instance_vars: HashMap::new(),
        singleton_methods: Rc::new(RefCell::new(HashMap::new())),
        singleton_class: Rc::new(RefCell::new(None)),
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
        default_parameters: vec![],
        keyword_parameters: vec![],
        block_parameter: None,
        variadic_param: None,
        body: vec![],
        receiver: None,
        owner: None,
        source_location: None,
        captured_vars: None,
        is_undefined: false,
        captured_refinements: Vec::new(),
    });
    let method2 = Rc::clone(&method1);
    let method3 = Rc::new(Method {
        name: "foo".to_string(),
        parameters: vec![],
        default_parameters: vec![],
        keyword_parameters: vec![],
        block_parameter: None,
        variadic_param: None,
        body: vec![],
        receiver: None,
        owner: None,
        source_location: None,
        captured_vars: None,
        is_undefined: false,
        captured_refinements: Vec::new(),
    });

    let obj1 = Object::Method(method1);
    let obj2 = Object::Method(method2);
    let obj3 = Object::Method(method3);

    assert!(obj1.equals(&obj2)); // Same reference (Rc::ptr_eq)
    // Per Ruby Method#==, structurally identical methods compare equal even
    // when allocated separately, so two methods with the same name/body/no
    // receiver are equal.
    assert!(obj1.equals(&obj3));
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

// ── Module/Binding equals (from additional_tests) ───────────────────────────

#[test]
fn test_module_equals_same_rc() {
    let module = Rc::new(Class::new("MyMod", None));
    let a = Object::Module(module.clone());
    let b = Object::Module(module);
    assert!(a.equals(&b));
}

#[test]
fn test_module_equals_different_rc() {
    let a = Object::Module(Rc::new(Class::new("MyMod", None)));
    let b = Object::Module(Rc::new(Class::new("MyMod", None)));
    assert!(!a.equals(&b));
}

#[test]
fn test_binding_equals_same_rc() {
    use metorex::object::Binding;
    let binding = Rc::new(Binding::new(HashMap::new()));
    let a = Object::Binding(binding.clone());
    let b = Object::Binding(binding);
    assert!(a.equals(&b));
}

#[test]
fn test_binding_equals_different_rc() {
    use metorex::object::Binding;
    let a = Object::Binding(Rc::new(Binding::new(HashMap::new())));
    let b = Object::Binding(Rc::new(Binding::new(HashMap::new())));
    assert!(!a.equals(&b));
}
