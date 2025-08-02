// Binding - represents a namespace/scope with captured variables

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Object;

/// Binding object represents a namespace/scope containing variable bindings
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    /// Captured variables from the binding's scope
    pub variables: HashMap<String, Rc<RefCell<Object>>>,
}

impl Binding {
    /// Create a new binding with the given variables
    pub fn new(variables: HashMap<String, Rc<RefCell<Object>>>) -> Self {
        Self { variables }
    }

    /// Get a variable from the binding
    pub fn get(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        self.variables.get(name).map(Rc::clone)
    }

    /// Get all variable names in the binding
    pub fn keys(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }
}
