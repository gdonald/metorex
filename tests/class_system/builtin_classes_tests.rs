// Unit tests for Metorex built-in classes
// Tests the built-in class hierarchy and method definitions

use metorex::builtin_classes::{
    BuiltinClasses, init_array_methods, init_object_methods, init_string_methods,
};
use metorex::class::Class;
use metorex::object::Object;
use std::rc::Rc;

// ============================================================================
// Built-in Classes Creation Tests
// ============================================================================

#[test]
fn test_builtin_classes_new() {
    let builtins = BuiltinClasses::new();

    assert_eq!(builtins.object_class.name(), "Object");
    assert_eq!(builtins.string_class.name(), "String");
    assert_eq!(builtins.integer_class.name(), "Integer");
    assert_eq!(builtins.float_class.name(), "Float");
    assert_eq!(builtins.array_class.name(), "Array");
    assert_eq!(builtins.hash_class.name(), "Hash");
    assert_eq!(builtins.set_class.name(), "Set");
}

#[test]
fn test_builtin_classes_default() {
    let builtins = BuiltinClasses::default();

    assert_eq!(builtins.object_class.name(), "Object");
}

// ============================================================================
// Inheritance Hierarchy Tests
// ============================================================================

#[test]
fn test_string_inherits_from_object() {
    let builtins = BuiltinClasses::new();

    let superclass = builtins.string_class.superclass();
    assert!(superclass.is_some());
    assert_eq!(superclass.unwrap().name(), "Object");
}

#[test]
fn test_integer_inherits_from_object() {
    let builtins = BuiltinClasses::new();

    let superclass = builtins.integer_class.superclass();
    assert!(superclass.is_some());
    assert_eq!(superclass.unwrap().name(), "Object");
}

#[test]
fn test_array_inherits_from_object() {
    let builtins = BuiltinClasses::new();

    let superclass = builtins.array_class.superclass();
    assert!(superclass.is_some());
    assert_eq!(superclass.unwrap().name(), "Object");
}

#[test]
fn test_object_has_no_superclass() {
    let builtins = BuiltinClasses::new();

    assert!(builtins.object_class.superclass().is_none());
}

// ============================================================================
// Exception Hierarchy Tests
// ============================================================================

#[test]
fn test_exception_hierarchy_creation() {
    let builtins = BuiltinClasses::new();

    assert_eq!(builtins.exception_class.name(), "Exception");
    assert_eq!(builtins.standard_error_class.name(), "StandardError");
    assert_eq!(builtins.runtime_error_class.name(), "RuntimeError");
    assert_eq!(builtins.type_error_class.name(), "TypeError");
    assert_eq!(builtins.value_error_class.name(), "ValueError");
}

#[test]
fn test_standard_error_inherits_from_exception() {
    let builtins = BuiltinClasses::new();

    let superclass = builtins.standard_error_class.superclass();
    assert!(superclass.is_some());
    assert_eq!(superclass.unwrap().name(), "Exception");
}

#[test]
fn test_runtime_error_inherits_from_standard_error() {
    let builtins = BuiltinClasses::new();

    let superclass = builtins.runtime_error_class.superclass();
    assert!(superclass.is_some());
    assert_eq!(superclass.unwrap().name(), "StandardError");
}

#[test]
fn test_type_error_inherits_from_standard_error() {
    let builtins = BuiltinClasses::new();

    let superclass = builtins.type_error_class.superclass();
    assert!(superclass.is_some());
    assert_eq!(superclass.unwrap().name(), "StandardError");
}

#[test]
fn test_value_error_inherits_from_standard_error() {
    let builtins = BuiltinClasses::new();

    let superclass = builtins.value_error_class.superclass();
    assert!(superclass.is_some());
    assert_eq!(superclass.unwrap().name(), "StandardError");
}

#[test]
fn test_exception_inherits_from_object() {
    let builtins = BuiltinClasses::new();

    let superclass = builtins.exception_class.superclass();
    assert!(superclass.is_some());
    assert_eq!(superclass.unwrap().name(), "Object");
}

// ============================================================================
// Class Of Tests
// ============================================================================

#[test]
fn test_class_of_integer() {
    let builtins = BuiltinClasses::new();
    let obj = Object::Int(42);

    let class = builtins.class_of(&obj);
    assert_eq!(class.name(), "Integer");
}

#[test]
fn test_class_of_float() {
    let builtins = BuiltinClasses::new();
    let obj = Object::Float(3.14);

    let class = builtins.class_of(&obj);
    assert_eq!(class.name(), "Float");
}

#[test]
fn test_class_of_string() {
    let builtins = BuiltinClasses::new();
    let obj = Object::string("hello");

    let class = builtins.class_of(&obj);
    assert_eq!(class.name(), "String");
}

#[test]
fn test_class_of_array() {
    let builtins = BuiltinClasses::new();
    let obj = Object::empty_array();

    let class = builtins.class_of(&obj);
    assert_eq!(class.name(), "Array");
}

#[test]
fn test_class_of_dict() {
    let builtins = BuiltinClasses::new();
    let obj = Object::empty_dict();

    let class = builtins.class_of(&obj);
    assert_eq!(class.name(), "Hash");
}

#[test]
fn test_class_of_set() {
    let builtins = BuiltinClasses::new();
    let obj = Object::empty_set();

    let class = builtins.class_of(&obj);
    assert_eq!(class.name(), "Set");
}

#[test]
fn test_class_of_nil() {
    let builtins = BuiltinClasses::new();
    let obj = Object::Nil;

    let class = builtins.class_of(&obj);
    assert_eq!(class.name(), "Object");
}

#[test]
fn test_class_of_bool() {
    let builtins = BuiltinClasses::new();
    let obj = Object::Bool(true);

    let class = builtins.class_of(&obj);
    assert_eq!(class.name(), "Object");
}

// ============================================================================
// Subclass Tests
// ============================================================================

#[test]
fn test_is_subclass_of_same_class() {
    let builtins = BuiltinClasses::new();

    assert!(builtins.is_subclass_of(&builtins.string_class, &builtins.string_class));
}

#[test]
fn test_is_subclass_of_parent() {
    let builtins = BuiltinClasses::new();

    assert!(builtins.is_subclass_of(&builtins.string_class, &builtins.object_class));
}

#[test]
fn test_is_subclass_of_grandparent() {
    let builtins = BuiltinClasses::new();

    // RuntimeError -> StandardError -> Exception -> Object
    assert!(builtins.is_subclass_of(&builtins.runtime_error_class, &builtins.exception_class));
    assert!(builtins.is_subclass_of(&builtins.runtime_error_class, &builtins.object_class));
}

#[test]
fn test_is_not_subclass_of_sibling() {
    let builtins = BuiltinClasses::new();

    // String and Integer are both subclasses of Object, but not of each other
    assert!(!builtins.is_subclass_of(&builtins.string_class, &builtins.integer_class));
    assert!(!builtins.is_subclass_of(&builtins.integer_class, &builtins.string_class));
}

#[test]
fn test_is_not_subclass_of_unrelated() {
    let builtins = BuiltinClasses::new();

    // RuntimeError is not a subclass of TypeError
    assert!(!builtins.is_subclass_of(&builtins.runtime_error_class, &builtins.type_error_class));
}

// ============================================================================
// Instance Of Tests
// ============================================================================

#[test]
fn test_is_instance_of_direct_class() {
    let builtins = BuiltinClasses::new();
    let obj = Object::Int(42);

    assert!(builtins.is_instance_of(&obj, &builtins.integer_class));
}

#[test]
fn test_is_instance_of_parent_class() {
    let builtins = BuiltinClasses::new();
    let obj = Object::string("test");

    assert!(builtins.is_instance_of(&obj, &builtins.object_class));
}

#[test]
fn test_is_not_instance_of_sibling_class() {
    let builtins = BuiltinClasses::new();
    let obj = Object::Int(42);

    assert!(!builtins.is_instance_of(&obj, &builtins.string_class));
}

// ============================================================================
// All Classes Tests
// ============================================================================

#[test]
fn test_all_classes() {
    let builtins = BuiltinClasses::new();
    let all = builtins.all_classes();

    assert_eq!(all.len(), 12);
    assert!(all.contains_key("Object"));
    assert!(all.contains_key("String"));
    assert!(all.contains_key("Integer"));
    assert!(all.contains_key("Float"));
    assert!(all.contains_key("Array"));
    assert!(all.contains_key("Hash"));
    assert!(all.contains_key("Set"));
    assert!(all.contains_key("Exception"));
    assert!(all.contains_key("StandardError"));
    assert!(all.contains_key("RuntimeError"));
    assert!(all.contains_key("TypeError"));
    assert!(all.contains_key("ValueError"));
}

#[test]
fn test_all_classes_returns_correct_classes() {
    let builtins = BuiltinClasses::new();
    let all = builtins.all_classes();

    assert_eq!(all.get("String").unwrap().name(), "String");
    assert_eq!(all.get("Integer").unwrap().name(), "Integer");
    assert_eq!(all.get("RuntimeError").unwrap().name(), "RuntimeError");
}

// ============================================================================
// Method Initialization Tests
// ============================================================================

#[test]
fn test_init_object_methods() {
    let object_class = Class::new("Object", None);
    init_object_methods(&object_class);

    assert!(object_class.find_method("to_s").is_some());
    assert!(object_class.find_method("class").is_some());
    assert!(object_class.find_method("respond_to?").is_some());
}

#[test]
fn test_init_string_methods() {
    let object_class = Rc::new(Class::new("Object", None));
    let string_class = Class::new("String", Some(object_class));
    init_string_methods(&string_class);

    assert!(string_class.find_method("length").is_some());
    assert!(string_class.find_method("upcase").is_some());
    assert!(string_class.find_method("downcase").is_some());
    assert!(string_class.find_method("+").is_some());
}

#[test]
fn test_init_array_methods() {
    let object_class = Rc::new(Class::new("Object", None));
    let array_class = Class::new("Array", Some(object_class));
    init_array_methods(&array_class);

    assert!(array_class.find_method("length").is_some());
    assert!(array_class.find_method("push").is_some());
    assert!(array_class.find_method("pop").is_some());
    assert!(array_class.find_method("[]").is_some());
}

// ============================================================================
// Method Parameters Tests
// ============================================================================

#[test]
fn test_object_respond_to_parameters() {
    let object_class = Class::new("Object", None);
    init_object_methods(&object_class);

    let respond_to = object_class.find_method("respond_to?").unwrap();
    assert_eq!(respond_to.parameters.len(), 1);
    assert_eq!(respond_to.parameters[0], "method_name");
}

#[test]
fn test_string_concat_parameters() {
    let object_class = Rc::new(Class::new("Object", None));
    let string_class = Class::new("String", Some(object_class));
    init_string_methods(&string_class);

    let concat = string_class.find_method("+").unwrap();
    assert_eq!(concat.parameters.len(), 1);
    assert_eq!(concat.parameters[0], "other");
}

#[test]
fn test_array_index_parameters() {
    let object_class = Rc::new(Class::new("Object", None));
    let array_class = Class::new("Array", Some(object_class));
    init_array_methods(&array_class);

    let index = array_class.find_method("[]").unwrap();
    assert_eq!(index.parameters.len(), 1);
    assert_eq!(index.parameters[0], "index");
}
