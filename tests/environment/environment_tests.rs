// Tests for the Environment scope stack management

use metorex::environment::Environment;
use metorex::object::Object;

// Helper function to create integer objects for tests
fn int(value: i64) -> Object {
    Object::Int(value)
}

#[test]
fn test_environment_creation() {
    let env = Environment::new();
    assert_eq!(
        env.current_depth(),
        0,
        "New environment should start at depth 0"
    );
}

#[test]
fn test_environment_default() {
    let env = Environment::default();
    assert_eq!(
        env.current_depth(),
        0,
        "Default environment should start at depth 0"
    );
}

#[test]
fn test_push_scope() {
    let mut env = Environment::new();

    // Initially at depth 0
    assert_eq!(env.current_depth(), 0);

    // Push first scope
    env.push_scope();
    assert_eq!(env.current_depth(), 1, "After one push, depth should be 1");

    // Push second scope
    env.push_scope();
    assert_eq!(
        env.current_depth(),
        2,
        "After two pushes, depth should be 2"
    );
}

#[test]
fn test_pop_scope() {
    let mut env = Environment::new();

    // Push two scopes
    env.push_scope();
    env.push_scope();
    assert_eq!(env.current_depth(), 2);

    // Pop first scope
    let popped = env.pop_scope();
    assert!(popped.is_some(), "Popping from depth 2 should return Some");
    assert_eq!(env.current_depth(), 1);

    // Pop second scope
    let popped = env.pop_scope();
    assert!(popped.is_some(), "Popping from depth 1 should return Some");
    assert_eq!(env.current_depth(), 0);
}

#[test]
fn test_cannot_pop_global_scope() {
    let mut env = Environment::new();

    // Try to pop the global scope
    let popped = env.pop_scope();
    assert!(popped.is_none(), "Popping global scope should return None");
    assert_eq!(env.current_depth(), 0, "Depth should remain 0");
}

#[test]
fn test_current_scope_accessor() {
    let env = Environment::new();

    // Get current scope at depth 0
    let scope = env.current_scope();
    scope.borrow_mut().define("x".to_string(), int(42));

    // Verify we can retrieve the value
    assert_eq!(env.get("x"), Some(int(42)));
}

#[test]
fn test_global_scope_accessor() {
    let mut env = Environment::new();

    // Define variable in global scope
    env.define("global_var".to_string(), int(100));

    // Push a new scope
    env.push_scope();

    // Global scope accessor should still reference the first scope
    let global = env.global_scope();
    assert_eq!(
        global.borrow().get("global_var"),
        Some(int(100)),
        "Global scope should contain the global variable"
    );
}

#[test]
fn test_define_in_current_scope() {
    let mut env = Environment::new();

    // Define in global scope
    env.define("x".to_string(), int(1));
    assert_eq!(env.get("x"), Some(int(1)));

    // Push new scope and define same variable
    env.push_scope();
    env.define("x".to_string(), int(2));
    assert_eq!(env.get("x"), Some(int(2)), "Should get shadowed value");

    // Pop scope
    env.pop_scope();
    assert_eq!(
        env.get("x"),
        Some(int(1)),
        "Should get original value after pop"
    );
}

#[test]
fn test_get_from_parent_scope() {
    let mut env = Environment::new();

    // Define in global scope
    env.define("outer".to_string(), int(42));

    // Push new scope
    env.push_scope();

    // Should be able to access parent scope variable
    assert_eq!(
        env.get("outer"),
        Some(int(42)),
        "Should access variable from parent scope"
    );
}

#[test]
fn test_get_nonexistent_variable() {
    let env = Environment::new();

    // Try to get a variable that doesn't exist
    assert_eq!(env.get("nonexistent"), None);
}

#[test]
fn test_set_existing_variable() {
    let mut env = Environment::new();

    // Define a variable
    env.define("x".to_string(), int(1));

    // Set should succeed
    let result = env.set("x", int(2));
    assert!(result, "Setting existing variable should return true");
    assert_eq!(env.get("x"), Some(int(2)));
}

#[test]
fn test_set_nonexistent_variable() {
    let mut env = Environment::new();

    // Try to set a variable that doesn't exist
    let result = env.set("nonexistent", int(42));
    assert!(!result, "Setting nonexistent variable should return false");
    assert_eq!(env.get("nonexistent"), None);
}

#[test]
fn test_set_in_parent_scope() {
    let mut env = Environment::new();

    // Define in global scope
    env.define("x".to_string(), int(1));

    // Push new scope
    env.push_scope();

    // Set should find variable in parent scope
    let result = env.set("x", int(2));
    assert!(result, "Should successfully set variable in parent scope");

    // Verify the change
    assert_eq!(env.get("x"), Some(int(2)));

    // Pop scope and verify change persisted
    env.pop_scope();
    assert_eq!(env.get("x"), Some(int(2)));
}

#[test]
fn test_get_at_depth() {
    let mut env = Environment::new();

    // Define in global scope
    env.define("x".to_string(), int(1));

    // Push scope and define
    env.push_scope();
    env.define("x".to_string(), int(2));

    // Get at depth 0 (current scope)
    assert_eq!(env.get_at(0, "x"), Some(int(2)));

    // Get at depth 1 (parent scope)
    assert_eq!(env.get_at(1, "x"), Some(int(1)));
}

#[test]
fn test_set_at_depth() {
    let mut env = Environment::new();

    // Define in global scope
    env.define("x".to_string(), int(1));

    // Push scope and define
    env.push_scope();
    env.define("x".to_string(), int(2));

    // Set at depth 1 (parent scope)
    let result = env.set_at(1, "x", int(100));
    assert!(result, "Should successfully set at depth 1");

    // Verify current scope value unchanged
    assert_eq!(env.get_at(0, "x"), Some(int(2)));

    // Verify parent scope value changed
    assert_eq!(env.get_at(1, "x"), Some(int(100)));
}

#[test]
fn test_multiple_variables_in_scopes() {
    let mut env = Environment::new();

    // Global scope variables
    env.define("a".to_string(), int(1));
    env.define("b".to_string(), int(2));

    // First nested scope
    env.push_scope();
    env.define("c".to_string(), int(3));
    assert_eq!(env.get("a"), Some(int(1)));
    assert_eq!(env.get("b"), Some(int(2)));
    assert_eq!(env.get("c"), Some(int(3)));

    // Second nested scope
    env.push_scope();
    env.define("d".to_string(), int(4));
    assert_eq!(env.get("a"), Some(int(1)));
    assert_eq!(env.get("b"), Some(int(2)));
    assert_eq!(env.get("c"), Some(int(3)));
    assert_eq!(env.get("d"), Some(int(4)));

    // Pop back to first nested scope
    env.pop_scope();
    assert_eq!(env.get("a"), Some(int(1)));
    assert_eq!(env.get("b"), Some(int(2)));
    assert_eq!(env.get("c"), Some(int(3)));
    assert_eq!(
        env.get("d"),
        None,
        "Variable d should not be accessible after pop"
    );
}

#[test]
fn test_scope_isolation() {
    let mut env = Environment::new();

    // Define in global scope
    env.define("shared".to_string(), int(1));

    // Push first scope and define unique variable
    env.push_scope();
    env.define("scope1_var".to_string(), int(10));
    assert_eq!(env.current_depth(), 1);

    // Pop back to global
    env.pop_scope();
    assert_eq!(env.current_depth(), 0);

    // Push second scope (sibling to first)
    env.push_scope();

    // Should not see scope1_var
    assert_eq!(
        env.get("scope1_var"),
        None,
        "Variables from sibling scopes should not be accessible"
    );

    // But should see shared global variable
    assert_eq!(env.get("shared"), Some(int(1)));
}

#[test]
fn test_deep_nesting() {
    let mut env = Environment::new();

    // Create deep nesting
    for i in 0..10 {
        env.push_scope();
        env.define(format!("var_{}", i), int(i as i64));
        assert_eq!(env.current_depth(), i + 1);
    }

    // Should be able to access all variables from innermost scope
    for i in 0..10 {
        assert_eq!(env.get(&format!("var_{}", i)), Some(int(i as i64)));
    }

    // Pop all scopes
    for i in (0..10).rev() {
        env.pop_scope();
        assert_eq!(env.current_depth(), i);
    }

    // Should be back at global scope
    assert_eq!(env.current_depth(), 0);
}
