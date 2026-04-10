// Method Tests - Method structure, Callable trait, and BlockStatement

use metorex::object::{BlockStatement, Method, Object};
use std::collections::HashMap;
use std::rc::Rc;

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

// ── Method constructors (from additional_tests) ─────────────────────────────

#[test]
fn test_method_with_owner() {
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
