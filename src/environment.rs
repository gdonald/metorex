// Environment stack for managing lexical scopes in Metorex
// This module implements a stack-based scope management system

use crate::object::Object;
use crate::scope::Scope;
use std::cell::RefCell;
use std::rc::Rc;

/// Represents the environment with a stack of scopes
/// The environment manages the scope chain and tracks the current depth
#[derive(Debug)]
pub struct Environment {
    /// Stack of scopes, with the top being the current scope
    scopes: Vec<Rc<RefCell<Scope>>>,

    /// Current depth in the scope stack (0 = global scope)
    depth: usize,
}

impl Environment {
    /// Creates a new environment with a global scope
    pub fn new() -> Self {
        let global_scope = Rc::new(RefCell::new(Scope::new()));
        Environment {
            scopes: vec![global_scope],
            depth: 0,
        }
    }

    /// Pushes a new scope onto the stack
    /// The new scope's parent will be the current top scope
    pub fn push_scope(&mut self) {
        let parent = self.scopes.last().unwrap().clone();
        let new_scope = Rc::new(RefCell::new(Scope::with_parent(parent)));
        self.scopes.push(new_scope);
        self.depth += 1;
    }

    /// Pushes an isolated scope (method boundary). The parent is the global
    /// scope, not the caller's scope, so local variable assignment inside a
    /// method body cannot accidentally modify the caller's variables.
    pub fn push_isolated_scope(&mut self) {
        let global = self.scopes[0].clone();
        let new_scope = Rc::new(RefCell::new(Scope::with_parent(global)));
        new_scope.borrow_mut().mark_method_boundary();
        self.scopes.push(new_scope);
        self.depth += 1;
    }

    /// Pops the current scope from the stack
    /// Returns the popped scope, or None if we're at the global scope
    /// Note: The global scope can never be popped
    pub fn pop_scope(&mut self) -> Option<Rc<RefCell<Scope>>> {
        if self.scopes.len() <= 1 {
            // Cannot pop the global scope
            return None;
        }

        self.depth -= 1;
        self.scopes.pop()
    }

    /// Returns a reference to the current (top) scope
    pub fn current_scope(&self) -> Rc<RefCell<Scope>> {
        self.scopes.last().unwrap().clone()
    }

    /// Returns the current scope depth
    /// 0 = global scope, 1 = first nested scope, etc.
    pub fn current_depth(&self) -> usize {
        self.depth
    }

    /// Returns a reference to the global scope
    pub fn global_scope(&self) -> Rc<RefCell<Scope>> {
        self.scopes[0].clone()
    }

    /// Defines a variable in the current scope
    pub fn define(&mut self, name: String, value: Object) {
        self.current_scope().borrow_mut().define(name, value);
    }

    /// Remove a variable from the current scope (does not touch parent scopes).
    pub fn undefine(&mut self, name: &str) {
        self.current_scope().borrow_mut().undefine(name);
    }

    /// Gets a variable value by traversing the scope chain from the current scope
    pub fn get(&self, name: &str) -> Option<Object> {
        self.current_scope().borrow().get(name)
    }

    /// Sets a variable value by traversing the scope chain from the current scope
    /// Returns true if the variable was found and updated, false otherwise
    pub fn set(&mut self, name: &str, value: Object) -> bool {
        self.current_scope().borrow_mut().set(name, value)
    }

    /// Gets a variable at a specific depth relative to the current scope
    pub fn get_at(&self, depth: usize, name: &str) -> Option<Object> {
        self.current_scope().borrow().get_at(depth, name)
    }

    /// Sets a variable at a specific depth relative to the current scope
    pub fn set_at(&mut self, depth: usize, name: &str, value: Object) -> bool {
        self.current_scope().borrow_mut().set_at(depth, name, value)
    }

    /// Collects all variables from the current scope chain
    /// This is used for lambda closure capture
    pub fn current_scope_vars(&self) -> std::collections::HashMap<String, Object> {
        self.current_scope().borrow().collect_all_vars()
    }

    /// Collects all variable references from the current scope chain
    /// This is used for lambda closure capture with mutable closures
    pub fn current_scope_var_refs(
        &self,
    ) -> std::collections::HashMap<String, std::rc::Rc<std::cell::RefCell<Object>>> {
        self.current_scope().borrow().collect_all_var_refs()
    }

    /// Names bound as local variables where execution currently sits. At the
    /// top level that is the root scope's own names; anywhere else it is the
    /// chain up to but not including the root, which holds the builtins.
    pub fn local_variable_names(&self) -> Vec<String> {
        let current = self.current_scope();
        let scope = current.borrow();
        if scope.is_root() {
            scope.own_variable_names()
        } else {
            scope.collect_local_variable_names()
        }
    }

    /// Whether `name` resolves to the very reference the root scope holds.
    /// A block captures enclosing names by reference, so a builtin or a
    /// top-level local reaches a block's own scope as the same reference
    /// rather than as a local of its own.
    pub fn resolves_to_root_binding(&self, name: &str) -> bool {
        if self.current_scope().borrow().is_root() {
            return false;
        }
        let Some(root_ref) = self.scopes[0].borrow().own_var_ref(name) else {
            return false;
        };
        self.get_ref(name)
            .is_some_and(|current| std::rc::Rc::ptr_eq(&current, &root_ref))
    }

    /// Defines a variable in the current scope with a shared reference
    /// Used when a closure defines a captured variable
    pub fn define_shared(&mut self, name: String, value: std::rc::Rc<std::cell::RefCell<Object>>) {
        self.current_scope().borrow_mut().define_shared(name, value);
    }

    /// Defines a name a block captured from its definition site. It resolves
    /// like any other variable but is not reported as a local of this scope.
    pub fn define_captured(
        &mut self,
        name: String,
        value: std::rc::Rc<std::cell::RefCell<Object>>,
    ) {
        self.current_scope()
            .borrow_mut()
            .define_captured(name, value);
    }

    /// Gets a shared reference to a variable
    pub fn get_ref(&self, name: &str) -> Option<std::rc::Rc<std::cell::RefCell<Object>>> {
        self.current_scope().borrow().get_ref(name)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
