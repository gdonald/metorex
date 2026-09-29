// Instance Tests - method dispatch and variable tests

use metorex::object::{Class, Instance, Method, Object};
use std::rc::Rc;

#[test]
fn test_instance_new() {
    let class = Class::new("TestClass", None);
    let shared = Instance::new(Rc::clone(&class));
    let instance = shared.borrow();

    assert_eq!(instance.class_name(), "TestClass");
    assert!(instance.instance_vars.is_empty());
}

#[test]
fn test_instance_set_and_get_var() {
    let class = Class::new("TestClass", None);
    let shared = Instance::new(Rc::clone(&class));
    let mut instance = shared.borrow_mut();

    instance.set_var("name".to_string(), Object::string("Bob"));
    instance.set_var("age".to_string(), Object::Int(25));

    assert_eq!(instance.get_var("name"), Some(&Object::string("Bob")));
    assert_eq!(instance.get_var("age"), Some(&Object::Int(25)));
    assert_eq!(instance.get_var("nonexistent"), None);
}

#[test]
fn test_instance_var_declared() {
    let class = Class::new("Person", None);
    class.declare_instance_var("name");
    class.declare_instance_var("age");

    let shared = Instance::new(Rc::clone(&class));

    let instance = shared.borrow();

    assert!(instance.is_var_declared("name"));
    assert!(instance.is_var_declared("age"));
    assert!(!instance.is_var_declared("height"));
}

#[test]
fn test_instance_var_declared_with_inheritance() {
    let parent = Class::new("Parent", None);
    parent.declare_instance_var("parent_var");

    let child = Class::new("Child", Some(Rc::clone(&parent)));
    child.declare_instance_var("child_var");

    let shared = Instance::new(Rc::clone(&child));

    let instance = shared.borrow();

    assert!(instance.is_var_declared("child_var"));
    assert!(instance.is_var_declared("parent_var"));
    assert!(!instance.is_var_declared("unknown_var"));
}

#[test]
fn test_instance_find_method() {
    let class = Class::new("Calculator", None);
    let method = Rc::new(Method::new(
        "add".to_string(),
        vec!["x".to_string(), "y".to_string()],
        vec![],
    ));
    class.define_method("add", Rc::clone(&method));

    let shared = Instance::new(Rc::clone(&class));

    let instance = shared.borrow();

    let found = instance.find_method("add");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "add");

    assert!(instance.find_method("nonexistent").is_none());
}

#[test]
fn test_instance_find_method_with_inheritance() {
    let parent = Class::new("Parent", None);
    let parent_method = Rc::new(Method::new("parent_method".to_string(), vec![], vec![]));
    parent.define_method("parent_method", Rc::clone(&parent_method));

    let child = Class::new("Child", Some(Rc::clone(&parent)));
    let child_method = Rc::new(Method::new("child_method".to_string(), vec![], vec![]));
    child.define_method("child_method", Rc::clone(&child_method));

    let shared = Instance::new(Rc::clone(&child));

    let instance = shared.borrow();

    assert!(instance.find_method("child_method").is_some());
    assert!(instance.find_method("parent_method").is_some());
    assert!(instance.find_method("nonexistent").is_none());
}

#[test]
fn test_instance_class_name() {
    let class = Class::new("MyTestClass", None);
    let shared = Instance::new(Rc::clone(&class));
    let instance = shared.borrow();

    assert_eq!(instance.class_name(), "MyTestClass");
}

#[test]
fn test_instance_multiple_vars() {
    let class = Class::new("DataHolder", None);
    let shared = Instance::new(Rc::clone(&class));
    let mut instance = shared.borrow_mut();

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
    let class = Class::new("Counter", None);
    let shared = Instance::new(Rc::clone(&class));
    let mut instance = shared.borrow_mut();

    instance.set_var("count".to_string(), Object::Int(0));
    assert_eq!(instance.get_var("count"), Some(&Object::Int(0)));

    instance.set_var("count".to_string(), Object::Int(1));
    assert_eq!(instance.get_var("count"), Some(&Object::Int(1)));

    instance.set_var("count".to_string(), Object::Int(2));
    assert_eq!(instance.get_var("count"), Some(&Object::Int(2)));
}

#[test]
fn test_instance_with_object_wrapper() {
    let class = Class::new("TestClass", None);
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

// ── Binding get/keys (from additional_tests) ────────────────────────────────

#[test]
fn test_binding_get_and_keys() {
    use metorex::object::Binding;
    use std::cell::RefCell;
    use std::collections::HashMap;
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), Rc::new(RefCell::new(Object::Int(42))));
    vars.insert(
        "y".to_string(),
        Rc::new(RefCell::new(Object::string("hello"))),
    );
    let binding = Binding::new(vars);
    let x = binding.get("x");
    assert!(x.is_some());
    assert_eq!(*x.unwrap().borrow(), Object::Int(42));
    assert!(binding.get("z").is_none());
    let keys = binding.keys();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"x".to_string()));
    assert!(keys.contains(&"y".to_string()));
}
