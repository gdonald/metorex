// Basic Object Tests - creation, type checking, variables, and ObjectHash

use metorex::object::{BlockStatement, Class, Method, Object, ObjectHash};
use std::collections::HashMap;
use std::rc::Rc;

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
    assert_eq!(format!("{}", obj), "MyClass");
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
fn test_symbol_type_name() {
    let obj = Object::Symbol(Rc::new("hello".to_string()));
    assert_eq!(obj.type_name(), "Symbol");
}

#[test]
fn test_native_function_type_name() {
    let obj = Object::NativeFunction("puts".to_string());
    assert_eq!(obj.type_name(), "NativeFunction");
}

#[test]
fn test_method_type_name() {
    let method = Method::new("foo".to_string(), vec![], vec![]);
    let obj = Object::Method(Rc::new(method));
    assert_eq!(obj.type_name(), "Method");
}

#[test]
fn test_block_type_name() {
    let block = BlockStatement::new(vec![], vec![], HashMap::new());
    let obj = Object::Block(Rc::new(block));
    assert_eq!(obj.type_name(), "Block");
}

#[test]
fn test_module_type_name() {
    let module_class = Rc::new(Class::new("MyModule", None));
    let obj = Object::Module(module_class);
    assert_eq!(obj.type_name(), "Module");
}

#[test]
fn test_binding_type_name() {
    use metorex::object::Binding;
    let binding = Binding::new(HashMap::new());
    let obj = Object::Binding(Rc::new(binding));
    assert_eq!(obj.type_name(), "Binding");
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

    let hash4 = ObjectHash::from_object(&Object::empty_array());
    assert!(hash4.is_none());
}

#[test]
fn test_object_hash_nil() {
    let hash = ObjectHash::from_object(&Object::Nil);
    assert!(hash.is_some());
    let hash2 = ObjectHash::from_object(&Object::Nil);
    assert_eq!(hash, hash2);
}

#[test]
fn test_object_hash_bool() {
    let hash_true = ObjectHash::from_object(&Object::Bool(true));
    let hash_false = ObjectHash::from_object(&Object::Bool(false));
    assert!(hash_true.is_some());
    assert!(hash_false.is_some());
    assert_ne!(hash_true, hash_false);
}

#[test]
fn test_object_hash_float() {
    let hash = ObjectHash::from_object(&Object::Float(3.14));
    assert!(hash.is_some());
    let hash2 = ObjectHash::from_object(&Object::Float(3.14));
    assert_eq!(hash, hash2);
}

#[test]
fn test_object_hash_symbol() {
    let hash = ObjectHash::from_object(&Object::Symbol(Rc::new("foo".to_string())));
    assert!(hash.is_some());
    let str_hash = ObjectHash::from_object(&Object::string("foo"));
    assert_ne!(hash, str_hash);
}
