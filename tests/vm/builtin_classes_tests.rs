// Coverage tests for builtin_classes.rs uncovered paths

use metorex::builtin_classes::BuiltinClasses;
use metorex::object::Object;
use metorex::object::ObjectHash;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ── builtin_classes.rs: class_of for all object types ───────────────────────

#[test]
fn class_of_nil() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Nil);
    assert_eq!(class.name(), "Object");
}

#[test]
fn class_of_bool() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Bool(true));
    assert_eq!(class.name(), "Object");
}

#[test]
fn class_of_int() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Int(42));
    assert_eq!(class.name(), "Integer");
}

#[test]
fn class_of_float() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Float(3.14));
    assert_eq!(class.name(), "Float");
}

#[test]
fn class_of_string() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::string("hello"));
    assert_eq!(class.name(), "String");
}

#[test]
fn class_of_symbol() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Symbol(Rc::new("foo".to_string())));
    assert_eq!(class.name(), "String");
}

#[test]
fn class_of_array() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Array(Rc::new(RefCell::new(vec![]))));
    assert_eq!(class.name(), "Array");
}

#[test]
fn class_of_dict() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Dict(Rc::new(RefCell::new(HashMap::new()))));
    assert_eq!(class.name(), "Hash");
}

#[test]
fn class_of_set() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Set(Rc::new(RefCell::new(
        HashSet::<ObjectHash>::new(),
    ))));
    assert_eq!(class.name(), "Set");
}

#[test]
fn class_of_range() {
    let builtins = BuiltinClasses::new();
    let class = builtins.class_of(&Object::Range {
        start: Box::new(Object::Int(1)),
        end: Box::new(Object::Int(10)),
        exclusive: false,
    });
    assert_eq!(class.name(), "Range");
}

#[test]
fn class_of_exception() {
    let builtins = BuiltinClasses::new();
    use metorex::object::Exception;
    let exc = Object::Exception(Rc::new(RefCell::new(Exception::new(
        "RuntimeError".to_string(),
        "test".to_string(),
    ))));
    let class = builtins.class_of(&exc);
    assert_eq!(class.name(), "Exception");
}

#[test]
fn class_of_method() {
    let builtins = BuiltinClasses::new();
    use metorex::object::Method;
    let method = Object::Method(Rc::new(Method::new("foo".to_string(), vec![], vec![])));
    let class = builtins.class_of(&method);
    assert_eq!(class.name(), "Method");
}

#[test]
fn class_of_block() {
    let builtins = BuiltinClasses::new();
    use metorex::object::BlockStatement;
    let block = Object::Block(Rc::new(BlockStatement {
        parameters: vec![],
        parameter_defaults: Vec::new(),
        body: vec![],
        captured_vars: HashMap::new(),
        captured_def_scope: vec![],
    }));
    let class = builtins.class_of(&block);
    assert_eq!(class.name(), "Proc");
}

#[test]
fn class_of_native_function() {
    let builtins = BuiltinClasses::new();
    let nf = Object::NativeFunction("puts".to_string());
    let class = builtins.class_of(&nf);
    assert_eq!(class.name(), "Object");
}

#[test]
fn class_of_class() {
    let builtins = BuiltinClasses::new();
    use metorex::class::Class;
    let cls = Object::Class(Rc::new(Class::new("MyClass", None)));
    let class = builtins.class_of(&cls);
    assert_eq!(class.name(), "Object");
}

#[test]
fn class_of_module() {
    let builtins = BuiltinClasses::new();
    use metorex::class::Class;
    let module = Object::Module(Rc::new(Class::new("MyModule", None)));
    let class = builtins.class_of(&module);
    assert_eq!(class.name(), "Object");
}

#[test]
fn class_of_binding() {
    let builtins = BuiltinClasses::new();
    use metorex::object::Binding;
    let binding = Object::Binding(Rc::new(Binding::new(HashMap::new())));
    let class = builtins.class_of(&binding);
    assert_eq!(class.name(), "Object");
}

#[test]
fn class_of_result() {
    let builtins = BuiltinClasses::new();
    let result = Object::Result(Ok(Box::new(Object::Int(42))));
    let class = builtins.class_of(&result);
    assert_eq!(class.name(), "Object");
}

#[test]
fn class_of_compiled_function() {
    let builtins = BuiltinClasses::new();
    use metorex::object::CompiledFunction;
    let func = Object::CompiledFunction(Rc::new(CompiledFunction::new("test".to_string(), 0)));
    let class = builtins.class_of(&func);
    assert_eq!(class.name(), "Object");
}

// ── builtin_classes.rs: is_instance_of ──────────────────────────────────────

#[test]
fn is_instance_of_int_is_integer() {
    let builtins = BuiltinClasses::new();
    assert!(builtins.is_instance_of(&Object::Int(42), &builtins.integer_class));
}

#[test]
fn is_instance_of_int_is_object() {
    let builtins = BuiltinClasses::new();
    assert!(builtins.is_instance_of(&Object::Int(42), &builtins.object_class));
}

#[test]
fn is_instance_of_string_is_not_integer() {
    let builtins = BuiltinClasses::new();
    assert!(!builtins.is_instance_of(&Object::string("hello"), &builtins.integer_class));
}

// ── builtin_classes.rs: is_subclass_of ──────────────────────────────────────

#[test]
fn is_subclass_of_runtime_error_is_standard_error() {
    let builtins = BuiltinClasses::new();
    assert!(builtins.is_subclass_of(
        &builtins.runtime_error_class,
        &builtins.standard_error_class
    ));
}

#[test]
fn is_subclass_of_runtime_error_is_exception() {
    let builtins = BuiltinClasses::new();
    assert!(builtins.is_subclass_of(&builtins.runtime_error_class, &builtins.exception_class));
}

#[test]
fn is_subclass_of_string_is_not_integer() {
    let builtins = BuiltinClasses::new();
    assert!(!builtins.is_subclass_of(&builtins.string_class, &builtins.integer_class));
}

// ── builtin_classes.rs: all_classes ─────────────────────────────────────────

#[test]
fn all_classes_has_expected_entries() {
    let builtins = BuiltinClasses::new();
    let classes = builtins.all_classes();
    assert!(classes.contains_key("Object"));
    assert!(classes.contains_key("String"));
    assert!(classes.contains_key("Integer"));
    assert!(classes.contains_key("Float"));
    assert!(classes.contains_key("Array"));
    assert!(classes.contains_key("Hash"));
    assert!(classes.contains_key("Set"));
    assert!(classes.contains_key("Exception"));
    assert!(classes.contains_key("StandardError"));
    assert!(classes.contains_key("RuntimeError"));
    assert!(classes.contains_key("TypeError"));
    assert!(classes.contains_key("ValueError"));
    assert!(classes.contains_key("File"));
}

// ── builtin_classes.rs: Default impl ────────────────────────────────────────

#[test]
fn builtin_classes_default() {
    let builtins = BuiltinClasses::default();
    assert_eq!(builtins.object_class.name(), "Object");
}
