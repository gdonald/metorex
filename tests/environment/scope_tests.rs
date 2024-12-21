// Tests for scope and variable management

use metorex::object::Object;
use metorex::scope::Scope;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_create_new_scope() {
    let scope = Scope::new();
    assert_eq!(scope.get("x"), None);
}

#[test]
fn test_define_variable() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Object::Int(42));
    assert_eq!(scope.get("x"), Some(Object::Int(42)));
}

#[test]
fn test_define_multiple_variables() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Object::Int(42));
    scope.define("y".to_string(), Object::Bool(true));
    scope.define(
        "z".to_string(),
        Object::String(Rc::new("hello".to_string())),
    );

    assert_eq!(scope.get("x"), Some(Object::Int(42)));
    assert_eq!(scope.get("y"), Some(Object::Bool(true)));
    assert_eq!(
        scope.get("z"),
        Some(Object::String(Rc::new("hello".to_string())))
    );
}

#[test]
fn test_redefine_variable() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Object::Int(42));
    scope.define("x".to_string(), Object::Int(100));
    assert_eq!(scope.get("x"), Some(Object::Int(100)));
}

#[test]
fn test_get_undefined_variable() {
    let scope = Scope::new();
    assert_eq!(scope.get("undefined"), None);
}

#[test]
fn test_scope_with_parent() {
    let mut parent = Scope::new();
    parent.define("x".to_string(), Object::Int(42));
    let parent_rc = Rc::new(RefCell::new(parent));

    let child = Scope::with_parent(parent_rc);
    assert_eq!(child.get("x"), Some(Object::Int(42)));
}

#[test]
fn test_scope_chain_lookup() {
    // Create a 3-level scope chain: grandparent -> parent -> child
    let mut grandparent = Scope::new();
    grandparent.define("a".to_string(), Object::Int(1));
    let grandparent_rc = Rc::new(RefCell::new(grandparent));

    let mut parent = Scope::with_parent(grandparent_rc);
    parent.define("b".to_string(), Object::Int(2));
    let parent_rc = Rc::new(RefCell::new(parent));

    let mut child = Scope::with_parent(parent_rc);
    child.define("c".to_string(), Object::Int(3));

    // Child should be able to access all variables
    assert_eq!(child.get("a"), Some(Object::Int(1)));
    assert_eq!(child.get("b"), Some(Object::Int(2)));
    assert_eq!(child.get("c"), Some(Object::Int(3)));
}

#[test]
fn test_variable_shadowing() {
    let mut parent = Scope::new();
    parent.define("x".to_string(), Object::Int(42));
    let parent_rc = Rc::new(RefCell::new(parent));

    let mut child = Scope::with_parent(parent_rc.clone());
    child.define("x".to_string(), Object::Int(100));

    // Child should see its own value (shadowing parent)
    assert_eq!(child.get("x"), Some(Object::Int(100)));

    // Parent should still have its original value
    assert_eq!(parent_rc.borrow().get("x"), Some(Object::Int(42)));
}

#[test]
fn test_set_variable_in_current_scope() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Object::Int(42));

    assert!(scope.set("x", Object::Int(100)));
    assert_eq!(scope.get("x"), Some(Object::Int(100)));
}

#[test]
fn test_set_variable_in_parent_scope() {
    let mut parent = Scope::new();
    parent.define("x".to_string(), Object::Int(42));
    let parent_rc = Rc::new(RefCell::new(parent));

    let mut child = Scope::with_parent(parent_rc.clone());

    // Child can modify parent's variable
    assert!(child.set("x", Object::Int(100)));
    assert_eq!(child.get("x"), Some(Object::Int(100)));
    assert_eq!(parent_rc.borrow().get("x"), Some(Object::Int(100)));
}

#[test]
fn test_set_undefined_variable_fails() {
    let mut scope = Scope::new();
    assert!(!scope.set("undefined", Object::Int(42)));
}

#[test]
fn test_set_with_shadowing() {
    let mut parent = Scope::new();
    parent.define("x".to_string(), Object::Int(42));
    let parent_rc = Rc::new(RefCell::new(parent));

    let mut child = Scope::with_parent(parent_rc.clone());
    child.define("x".to_string(), Object::Int(100));

    // Setting should affect the child's variable, not the parent's
    assert!(child.set("x", Object::Int(200)));
    assert_eq!(child.get("x"), Some(Object::Int(200)));
    assert_eq!(parent_rc.borrow().get("x"), Some(Object::Int(42)));
}

#[test]
fn test_get_at_current_scope() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Object::Int(42));

    assert_eq!(scope.get_at(0, "x"), Some(Object::Int(42)));
}

#[test]
fn test_get_at_parent_scope() {
    let mut parent = Scope::new();
    parent.define("x".to_string(), Object::Int(42));
    let parent_rc = Rc::new(RefCell::new(parent));

    let child = Scope::with_parent(parent_rc);

    assert_eq!(child.get_at(1, "x"), Some(Object::Int(42)));
}

#[test]
fn test_get_at_grandparent_scope() {
    let mut grandparent = Scope::new();
    grandparent.define("x".to_string(), Object::Int(42));
    let grandparent_rc = Rc::new(RefCell::new(grandparent));

    let parent = Scope::with_parent(grandparent_rc);
    let parent_rc = Rc::new(RefCell::new(parent));

    let child = Scope::with_parent(parent_rc);

    assert_eq!(child.get_at(2, "x"), Some(Object::Int(42)));
}

#[test]
fn test_get_at_invalid_depth() {
    let scope = Scope::new();
    assert_eq!(scope.get_at(10, "x"), None);
}

#[test]
fn test_set_at_current_scope() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Object::Int(42));

    assert!(scope.set_at(0, "x", Object::Int(100)));
    assert_eq!(scope.get("x"), Some(Object::Int(100)));
}

#[test]
fn test_set_at_parent_scope() {
    let mut parent = Scope::new();
    parent.define("x".to_string(), Object::Int(42));
    let parent_rc = Rc::new(RefCell::new(parent));

    let mut child = Scope::with_parent(parent_rc.clone());

    assert!(child.set_at(1, "x", Object::Int(100)));
    assert_eq!(parent_rc.borrow().get("x"), Some(Object::Int(100)));
}

#[test]
fn test_set_at_invalid_depth() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Object::Int(42));

    assert!(!scope.set_at(10, "x", Object::Int(100)));
}

#[test]
fn test_set_at_undefined_variable() {
    let mut scope = Scope::new();
    assert!(!scope.set_at(0, "undefined", Object::Int(42)));
}

#[test]
fn test_complex_scope_chain() {
    // Create a complex scope hierarchy
    let mut global = Scope::new();
    global.define(
        "global_var".to_string(),
        Object::String(Rc::new("global".to_string())),
    );
    let global_rc = Rc::new(RefCell::new(global));

    let mut function_scope = Scope::with_parent(global_rc.clone());
    function_scope.define("func_var".to_string(), Object::Int(42));
    let function_rc = Rc::new(RefCell::new(function_scope));

    let mut block_scope = Scope::with_parent(function_rc.clone());
    block_scope.define("block_var".to_string(), Object::Bool(true));

    // Test accessing variables from all levels
    assert_eq!(
        block_scope.get("global_var"),
        Some(Object::String(Rc::new("global".to_string())))
    );
    assert_eq!(block_scope.get("func_var"), Some(Object::Int(42)));
    assert_eq!(block_scope.get("block_var"), Some(Object::Bool(true)));

    // Test get_at for each level
    assert_eq!(block_scope.get_at(0, "block_var"), Some(Object::Bool(true)));
    assert_eq!(block_scope.get_at(1, "func_var"), Some(Object::Int(42)));
    assert_eq!(
        block_scope.get_at(2, "global_var"),
        Some(Object::String(Rc::new("global".to_string())))
    );
}

#[test]
fn test_nil_value() {
    let mut scope = Scope::new();
    scope.define("nil_var".to_string(), Object::Nil);
    assert_eq!(scope.get("nil_var"), Some(Object::Nil));
}

#[test]
fn test_float_value() {
    let mut scope = Scope::new();
    scope.define("pi".to_string(), Object::Float(3.14159));
    assert_eq!(scope.get("pi"), Some(Object::Float(3.14159)));
}

#[test]
fn test_multiple_children_same_parent() {
    let mut parent = Scope::new();
    parent.define("shared".to_string(), Object::Int(42));
    let parent_rc = Rc::new(RefCell::new(parent));

    let mut child1 = Scope::with_parent(parent_rc.clone());
    child1.define("child1_var".to_string(), Object::Int(1));

    let mut child2 = Scope::with_parent(parent_rc.clone());
    child2.define("child2_var".to_string(), Object::Int(2));

    // Both children should see the parent's variable
    assert_eq!(child1.get("shared"), Some(Object::Int(42)));
    assert_eq!(child2.get("shared"), Some(Object::Int(42)));

    // But they shouldn't see each other's variables
    assert_eq!(child1.get("child2_var"), None);
    assert_eq!(child2.get("child1_var"), None);
}

#[test]
fn test_scope_isolation() {
    let mut scope1 = Scope::new();
    scope1.define("x".to_string(), Object::Int(42));

    let mut scope2 = Scope::new();
    scope2.define("x".to_string(), Object::Int(100));

    // Two independent scopes should not interfere
    assert_eq!(scope1.get("x"), Some(Object::Int(42)));
    assert_eq!(scope2.get("x"), Some(Object::Int(100)));
}
