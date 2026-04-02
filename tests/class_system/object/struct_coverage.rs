// Coverage tests for object structs: Binding, Heap, CallFrame, GlobalRegistry,
// CompiledFunction, ObjectHash, BlockStatement, Instance, Exception, Method

use metorex::object::{
    Binding, BlockStatement, Class, Exception, Instance, Method, Object, ObjectHash,
};
use metorex::vm::{CallFrame, GlobalRegistry, Heap};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ── CallFrame ───────────────────────────────────────────────────────────────

#[test]
fn call_frame_new_and_accessors() {
    let frame = CallFrame::new("my_method", Some("file.rb:10".to_string()));
    assert_eq!(frame.name(), "my_method");
    assert_eq!(frame.location(), Some("file.rb:10"));
}

#[test]
fn call_frame_no_location() {
    let frame = CallFrame::new("top_level", None);
    assert_eq!(frame.name(), "top_level");
    assert_eq!(frame.location(), None);
}

// ── GlobalRegistry ──────────────────────────────────────────────────────────

#[test]
fn global_registry_new_is_empty() {
    let registry = GlobalRegistry::new();
    assert!(!registry.contains("anything"));
    assert!(registry.get("anything").is_none());
}

#[test]
fn global_registry_set_get_contains() {
    let mut registry = GlobalRegistry::new();
    registry.set("x", Object::Int(42));
    assert!(registry.contains("x"));
    assert_eq!(registry.get("x"), Some(Object::Int(42)));
    assert!(!registry.contains("y"));
}

#[test]
fn global_registry_iter() {
    let mut registry = GlobalRegistry::new();
    registry.set("a", Object::Int(1));
    registry.set("b", Object::Int(2));
    let items: Vec<_> = registry.iter().collect();
    assert_eq!(items.len(), 2);
}

// ── Heap ────────────────────────────────────────────────────────────────────

#[test]
fn heap_default_is_empty() {
    let heap = Heap::default();
    assert_eq!(heap.allocation_count(), 0);
}

#[test]
fn heap_allocate_and_count() {
    let mut heap = Heap::default();
    heap.allocate(Object::Int(1));
    heap.allocate(Object::string("hello"));
    heap.allocate(Object::Nil);
    assert_eq!(heap.allocation_count(), 3);
}

// ── Binding ─────────────────────────────────────────────────────────────────

#[test]
fn binding_new_empty() {
    let binding = Binding::new(HashMap::new());
    assert!(binding.get("x").is_none());
    assert!(binding.keys().is_empty());
}

#[test]
fn binding_get_returns_shared_ref() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), Rc::new(RefCell::new(Object::Int(10))));
    let binding = Binding::new(vars);
    let x = binding.get("x").unwrap();
    assert_eq!(*x.borrow(), Object::Int(10));
    // Mutation through the Rc<RefCell<>> is shared
    *x.borrow_mut() = Object::Int(20);
    let x2 = binding.get("x").unwrap();
    assert_eq!(*x2.borrow(), Object::Int(20));
}

// ── ObjectHash ──────────────────────────────────────────────────────────────

#[test]
fn object_hash_from_nil() {
    let h = ObjectHash::from_object(&Object::Nil);
    assert!(h.is_some());
}

#[test]
fn object_hash_from_bool() {
    let h = ObjectHash::from_object(&Object::Bool(true));
    assert!(h.is_some());
}

#[test]
fn object_hash_from_int() {
    let h = ObjectHash::from_object(&Object::Int(42));
    assert!(h.is_some());
}

#[test]
fn object_hash_from_float() {
    let h = ObjectHash::from_object(&Object::Float(3.14));
    assert!(h.is_some());
}

#[test]
fn object_hash_from_string() {
    let h = ObjectHash::from_object(&Object::string("hello"));
    assert!(h.is_some());
}

#[test]
fn object_hash_from_symbol() {
    let h = ObjectHash::from_object(&Object::Symbol(Rc::new("foo".to_string())));
    assert!(h.is_some());
}

#[test]
fn object_hash_from_array_is_none() {
    let h = ObjectHash::from_object(&Object::empty_array());
    assert!(h.is_none());
}

#[test]
fn object_hash_from_dict_is_none() {
    let h = ObjectHash::from_object(&Object::empty_dict());
    assert!(h.is_none());
}

// ── CompiledFunction ────────────────────────────────────────────────────────

#[test]
fn compiled_function_debug_script() {
    use metorex::object::CompiledFunction;
    let f = CompiledFunction::new(String::new(), 0);
    assert_eq!(format!("{:?}", f), "<script>");
    assert_eq!(format!("{}", f), "<script>");
}

#[test]
fn compiled_function_debug_named() {
    use metorex::object::CompiledFunction;
    let f = CompiledFunction::new("foo".to_string(), 2);
    assert_eq!(format!("{:?}", f), "<fn foo>");
    assert_eq!(format!("{}", f), "<fn foo>");
}

#[test]
fn compiled_function_equality() {
    use metorex::object::CompiledFunction;
    let a = CompiledFunction::new("foo".to_string(), 2);
    let b = CompiledFunction::new("foo".to_string(), 2);
    let c = CompiledFunction::new("bar".to_string(), 2);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ── Instance ────────────────────────────────────────────────────────────────

#[test]
fn instance_new_and_class_name() {
    let class = Rc::new(Class::new("Dog", None));
    let inst = Instance::new(Rc::clone(&class));
    assert_eq!(inst.class_name(), "Dog");
}

#[test]
fn instance_get_set_var() {
    let class = Rc::new(Class::new("Dog", None));
    let mut inst = Instance::new(Rc::clone(&class));
    assert!(inst.get_var("name").is_none());
    inst.set_var("name".to_string(), Object::string("Rex"));
    assert_eq!(inst.get_var("name"), Some(&Object::string("Rex")));
}

#[test]
fn instance_is_var_declared() {
    let class = Rc::new(Class::new("Dog", None));
    class.declare_instance_var("name");
    let inst = Instance::new(Rc::clone(&class));
    assert!(inst.is_var_declared("name"));
    assert!(!inst.is_var_declared("age"));
}

#[test]
fn instance_find_method() {
    let class = Rc::new(Class::new("Dog", None));
    let method = Rc::new(Method::new("bark".to_string(), vec![], vec![]));
    class.define_method("bark", method);
    let inst = Instance::new(Rc::clone(&class));
    assert!(inst.find_method("bark").is_some());
    assert!(inst.find_method("meow").is_none());
}

// ── Exception ───────────────────────────────────────────────────────────────

#[test]
fn exception_new() {
    let exc = Exception::new("RuntimeError".to_string(), "oops".to_string());
    assert_eq!(exc.exception_type, "RuntimeError");
    assert_eq!(exc.message, "oops");
    assert!(exc.backtrace.is_none());
    assert!(exc.location.is_none());
    assert!(exc.cause.is_none());
}

#[test]
fn exception_with_backtrace() {
    let exc = Exception::with_backtrace(
        "Error".to_string(),
        "msg".to_string(),
        vec!["line1".to_string(), "line2".to_string()],
    );
    assert_eq!(exc.backtrace.as_ref().unwrap().len(), 2);
}

#[test]
fn exception_with_location() {
    use metorex::object::SourceLocation;
    let loc = SourceLocation::new("test.rb".to_string(), 10, 5);
    let exc = Exception::with_location("Error".to_string(), "msg".to_string(), loc);
    assert!(exc.location.is_some());
    let loc = exc.location.unwrap();
    assert_eq!(loc.file, "test.rb");
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
}

#[test]
fn exception_with_cause() {
    let cause = Object::string("original");
    let exc = Exception::with_cause("Error".to_string(), "wrapper".to_string(), cause);
    assert!(exc.cause.is_some());
}

#[test]
fn exception_with_all() {
    let exc = Exception::with_all(
        "Error".to_string(),
        "msg".to_string(),
        Some(vec!["bt".to_string()]),
        None,
        Some(Object::Nil),
    );
    assert!(exc.backtrace.is_some());
    assert!(exc.cause.is_some());
}

#[test]
fn exception_chain_no_cause() {
    let exc = Exception::new("Error".to_string(), "msg".to_string());
    let chain = exc.exception_chain();
    assert_eq!(chain, vec!["Error: msg"]);
}

#[test]
fn exception_chain_with_cause() {
    let cause_inner = Exception::new("InnerError".to_string(), "inner".to_string());
    let cause_obj = Object::Exception(Rc::new(RefCell::new(cause_inner)));
    let exc = Exception::with_cause("OuterError".to_string(), "outer".to_string(), cause_obj);
    let chain = exc.exception_chain();
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0], "OuterError: outer");
    assert_eq!(chain[1], "InnerError: inner");
}

// ── Method ──────────────────────────────────────────────────────────────────

#[test]
fn method_undefined_sentinel() {
    let m = Method::undefined("gone".to_string());
    assert!(m.is_undefined);
    assert_eq!(m.name, "gone");
    assert!(m.parameters.is_empty());
}

#[test]
fn method_with_source_location() {
    use metorex::error::SourceLocation;
    let loc = SourceLocation::new(5, 10, 100);
    let m = Method::with_source_location("foo".to_string(), vec![], vec![], loc);
    assert!(m.source_location.is_some());
    assert!(!m.is_undefined);
}

#[test]
fn method_bind_preserves_undefined() {
    let m = Method::undefined("x".to_string());
    let bound = m.bind(Object::Nil);
    assert!(bound.is_undefined);
    assert!(bound.is_bound());
    assert!(bound.receiver().is_some());
}

#[test]
fn method_bind_normal() {
    let m = Method::new("foo".to_string(), vec!["a".to_string()], vec![]);
    assert!(!m.is_bound());
    assert!(m.receiver().is_none());
    let bound = m.bind(Object::Int(42));
    assert!(bound.is_bound());
}

// ── BlockStatement Callable ─────────────────────────────────────────────────

#[test]
fn block_statement_callable_trait() {
    use metorex::callable::Callable;
    let block = BlockStatement::new(
        vec!["x".to_string(), "y".to_string()],
        vec![],
        HashMap::new(),
    );
    assert_eq!(block.name(), "<block>");
    assert_eq!(block.parameters().len(), 2);
    assert_eq!(block.arity(), 2);
    assert!(block.body().is_empty());
}

#[test]
fn block_statement_captured_vars() {
    let mut vars = HashMap::new();
    vars.insert("z".to_string(), Rc::new(RefCell::new(Object::Int(99))));
    let block = BlockStatement::new(vec![], vec![], vars);
    let captured = block.captured_vars();
    assert!(captured.contains_key("z"));
}

// ── Method Callable trait ───────────────────────────────────────────────────

#[test]
fn method_callable_trait() {
    use metorex::callable::Callable;
    let m = Method::new(
        "test".to_string(),
        vec!["a".to_string(), "b".to_string()],
        vec![],
    );
    assert_eq!(m.name(), "test");
    assert_eq!(m.parameters().len(), 2);
    assert_eq!(m.arity(), 2);
    assert!(m.body().is_empty());
}
