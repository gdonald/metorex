// Coverage tests for Display, is_falsy, equals edge cases, Binding, exceptions,
// Method constructors, Heap, and MetorexError

use metorex::object::{Class, Object, ObjectHash};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ── Display: Set with multiple elements ──────────────────────────────────────

#[test]
fn test_set_display_multiple_elements() {
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

// ── Display: NativeFunction ──────────────────────────────────────────────────

#[test]
fn test_native_function_display() {
    let obj = Object::NativeFunction("puts".to_string());
    assert_eq!(format!("{}", obj), "<native function puts>");
}

// ── is_falsy ─────────────────────────────────────────────────────────────────

#[test]
fn test_is_falsy() {
    assert!(Object::Nil.is_falsy());
    assert!(Object::Bool(false).is_falsy());
    assert!(!Object::Bool(true).is_falsy());
    assert!(!Object::Int(0).is_falsy());
}

// ── equals: Module, Binding ──────────────────────────────────────────────────

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

// ── Binding get/keys ─────────────────────────────────────────────────────────

#[test]
fn test_binding_get_and_keys() {
    use metorex::object::Binding;
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

// ── exception_with_cause ─────────────────────────────────────────────────────

#[test]
fn test_exception_with_cause() {
    let cause = Object::exception("RuntimeError", "original error");
    let exc = Object::exception_with_cause("RuntimeError", "wrapper error", cause);
    if let Object::Exception(exc_ref) = exc {
        let exc_inner = exc_ref.borrow();
        assert_eq!(exc_inner.message, "wrapper error");
        assert!(exc_inner.cause.is_some());
    } else {
        panic!("Expected exception object");
    }
}

// ── Method constructors ──────────────────────────────────────────────────────

#[test]
fn test_method_with_owner() {
    use metorex::object::Method;
    let method = Method::with_owner(
        "foo".to_string(),
        vec!["x".to_string()],
        vec![],
        "MyClass".to_string(),
    );
    assert_eq!(method.name, "foo");
    assert_eq!(method.owner, Some("MyClass".to_string()));
}

#[test]
fn test_method_with_owner_and_location() {
    use metorex::error::SourceLocation;
    use metorex::object::Method;
    let loc = SourceLocation::new(1, 1, 0);
    let method = Method::with_owner_and_location(
        "bar".to_string(),
        vec![],
        vec![],
        "OtherClass".to_string(),
        loc,
    );
    assert_eq!(method.name, "bar");
    assert_eq!(method.owner, Some("OtherClass".to_string()));
    assert!(method.source_location.is_some());
}

// ── Heap ─────────────────────────────────────────────────────────────────────

#[test]
fn test_heap_allocate() {
    use metorex::vm::Heap;
    let mut heap = Heap::default();
    assert_eq!(heap.allocation_count(), 0);
    heap.allocate(Object::Int(1));
    assert_eq!(heap.allocation_count(), 1);
    heap.allocate(Object::string("hello"));
    assert_eq!(heap.allocation_count(), 2);
}

// ── MetorexError From<io::Error> ─────────────────────────────────────────────

#[test]
fn test_metorex_error_from_io_error() {
    use metorex::error::MetorexError;
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let metorex_err: MetorexError = io_err.into();
    let msg = metorex_err.to_string();
    assert!(msg.contains("file not found"));
}
