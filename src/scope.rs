// Scope and variable management for Metorex
// This module implements lexical scoping with scope chain traversal

use crate::object::Object;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Represents a single scope in the scope chain
/// Each scope can have a parent scope, forming a chain for variable lookup
#[derive(Debug)]
pub struct Scope {
    /// Variable storage: maps variable names to shared mutable references
    /// This allows closures to mutate captured variables
    variables: HashMap<String, Rc<RefCell<Object>>>,

    /// Reference to the parent scope (None for global scope)
    parent: Option<Rc<RefCell<Scope>>>,

    /// Whether this scope is a method boundary. A method body cannot see the
    /// caller's locals, so `local_variables` stops here.
    is_method_boundary: bool,

    /// Names this scope holds only because a block captured them from the
    /// scope it was written in. They are not locals of this scope.
    captured_names: HashSet<String>,
}

impl Scope {
    /// Creates a new scope with no parent (global scope)
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            parent: None,
            is_method_boundary: false,
            captured_names: HashSet::new(),
        }
    }

    /// Creates a new scope with the given parent scope
    pub fn with_parent(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(parent),
            is_method_boundary: false,
            captured_names: HashSet::new(),
        }
    }

    /// Defines a new variable in the current scope
    /// If the variable already exists in this scope, it will be overwritten
    pub fn define(&mut self, name: String, value: Object) {
        self.captured_names.remove(&name);
        self.variables.insert(name, Rc::new(RefCell::new(value)));
    }

    /// Defines a new variable in the current scope with a shared reference
    /// Used when a closure defines a captured variable
    pub fn define_shared(&mut self, name: String, value: Rc<RefCell<Object>>) {
        self.captured_names.remove(&name);
        self.variables.insert(name, value);
    }

    /// Binds a name a block captured from the scope it was written in. The
    /// binding behaves like any other, but it is not a local of this scope,
    /// so `local_variables` leaves it out.
    pub fn define_captured(&mut self, name: String, value: Rc<RefCell<Object>>) {
        self.captured_names.insert(name.clone());
        self.variables.insert(name, value);
    }

    /// Remove a variable from the current scope only (does not walk parents).
    pub fn undefine(&mut self, name: &str) {
        self.variables.remove(name);
    }

    /// Gets a variable value by traversing the scope chain
    /// Returns None if the variable is not found in any scope
    pub fn get(&self, name: &str) -> Option<Object> {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            return Some(value_ref.borrow().clone());
        }

        // If not found, check the parent scope recursively
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        // Variable not found in any scope
        None
    }

    /// Gets a shared reference to a variable by traversing the scope chain
    /// Used for closure capture to enable mutable closures
    pub fn get_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            return Some(value_ref.clone());
        }

        // If not found, check the parent scope recursively
        if let Some(parent) = &self.parent {
            return parent.borrow().get_ref(name);
        }

        // Variable not found in any scope
        None
    }

    /// Sets a variable value by traversing the scope chain
    /// Returns true if the variable was found and updated, false otherwise
    /// This method will NOT create a new variable if it doesn't exist
    pub fn set(&mut self, name: &str, value: Object) -> bool {
        // First, check if the variable exists in this scope
        if let Some(value_ref) = self.variables.get(name) {
            *value_ref.borrow_mut() = value;
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
            return self.variables.get(name).map(|v| v.borrow().clone());
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
            if let Some(value_ref) = self.variables.get(name) {
                *value_ref.borrow_mut() = value;
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
        for (name, value_ref) in &self.variables {
            all_vars.insert(name.clone(), value_ref.borrow().clone());
        }

        all_vars
    }

    /// Names bound in this scope alone, excluding those a block captured.
    pub fn own_variable_names(&self) -> Vec<String> {
        self.variables
            .keys()
            .filter(|name| !self.captured_names.contains(*name))
            .cloned()
            .collect()
    }

    /// Names bound in this scope and the enclosing scopes a Ruby local would
    /// be visible from. The walk stops at the root, which holds the builtins,
    /// and at a method boundary, whose locals belong to the method rather
    /// than to the block running inside it.
    pub fn collect_local_variable_names(&self) -> Vec<String> {
        let mut names = self.own_variable_names();
        if let Some(parent) = &self.parent {
            let parent_ref = parent.borrow();
            if parent_ref.parent.is_some() && !parent_ref.is_method_boundary {
                names.extend(parent_ref.collect_local_variable_names());
            }
        }
        names
    }

    /// Mark this scope as a method boundary.
    pub fn mark_method_boundary(&mut self) {
        self.is_method_boundary = true;
    }

    /// The reference this scope holds for `name`, ignoring enclosing scopes.
    pub fn own_var_ref(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        self.variables.get(name).cloned()
    }

    /// Whether this scope is the root of the chain.
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Collects all variable references from the entire scope chain
    /// Returns a HashMap with shared references to all visible variables
    /// Used for closure capture to enable mutable closures
    pub fn collect_all_var_refs(&self) -> HashMap<String, Rc<RefCell<Object>>> {
        let mut all_vars = HashMap::new();

        // Start from parent and work backwards, so that closer scopes override farther ones
        if let Some(parent) = &self.parent {
            all_vars = parent.borrow().collect_all_var_refs();
        }

        // Now add this scope's variables (potentially overriding parent values)
        for (name, value_ref) in &self.variables {
            all_vars.insert(name.clone(), value_ref.clone());
        }

        all_vars
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
