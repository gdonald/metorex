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
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
