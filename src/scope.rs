// Scope and variable management for Metorex
// This module implements lexical scoping with scope chain traversal

use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Represents a single scope in the scope chain
/// Each scope can have a parent scope, forming a chain for variable lookup
#[derive(Debug)]
pub struct Scope {
    /// Variable storage: maps variable names to their values
    variables: HashMap<String, Object>,

    /// Reference to the parent scope (None for global scope)
    parent: Option<Rc<RefCell<Scope>>>,
}

impl Scope {
    /// Creates a new scope with no parent (global scope)
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            parent: None,
        }
    }

    /// Creates a new scope with the given parent scope
    pub fn with_parent(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Defines a new variable in the current scope
    /// If the variable already exists in this scope, it will be overwritten
    pub fn define(&mut self, name: String, value: Object) {
        self.variables.insert(name, value);
    }

    /// Gets a variable value by traversing the scope chain
    /// Returns None if the variable is not found in any scope
    pub fn get(&self, name: &str) -> Option<Object> {
        // First, check if the variable exists in this scope
        if let Some(value) = self.variables.get(name) {
            return Some(value.clone());
        }

        // If not found, check the parent scope recursively
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        // Variable not found in any scope
        None
    }

    /// Sets a variable value by traversing the scope chain
    /// Returns true if the variable was found and updated, false otherwise
    /// This method will NOT create a new variable if it doesn't exist
    pub fn set(&mut self, name: &str, value: Object) -> bool {
        // First, check if the variable exists in this scope
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return true;
        }

        // If not found, try to set it in the parent scope
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set(name, value);
        }

        // Variable not found in any scope
        false
    }

    /// Gets a variable at a specific depth in the scope chain
    /// depth=0 means current scope, depth=1 means parent, etc.
    /// This is useful for closure resolution where we know the exact depth
    pub fn get_at(&self, depth: usize, name: &str) -> Option<Object> {
        if depth == 0 {
            return self.variables.get(name).cloned();
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get_at(depth - 1, name);
        }

        None
    }

    /// Sets a variable at a specific depth in the scope chain
    /// depth=0 means current scope, depth=1 means parent, etc.
    /// Returns true if successful, false if the depth is invalid or variable doesn't exist
    pub fn set_at(&mut self, depth: usize, name: &str, value: Object) -> bool {
        if depth == 0 {
            if self.variables.contains_key(name) {
                self.variables.insert(name.to_string(), value);
                return true;
            }
            return false;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set_at(depth - 1, name, value);
        }

        false
    }

    /// Collects all variables from the entire scope chain
    /// Returns a HashMap with all visible variables (parent scope vars may be shadowed)
    pub fn collect_all_vars(&self) -> HashMap<String, Object> {
        let mut all_vars = HashMap::new();

        // Start from parent and work backwards, so that closer scopes override farther ones
        if let Some(parent) = &self.parent {
            all_vars = parent.borrow().collect_all_vars();
        }

        // Now add this scope's variables (potentially overriding parent values)
        for (name, value) in &self.variables {
            all_vars.insert(name.clone(), value.clone());
        }

        all_vars
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
