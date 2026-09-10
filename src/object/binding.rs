// Binding - represents a namespace/scope with captured variables

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Object;

/// Binding object represents a namespace/scope containing variable bindings.
/// The variables are held behind a cell because code run through a binding
/// may name a local the binding did not have, which Ruby adds to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    /// Captured variables from the binding's scope
    pub variables: RefCell<HashMap<String, Rc<RefCell<Object>>>>,
    /// Receiver (self) at the point the binding was captured
    pub receiver: Option<Object>,
    /// Where the binding was captured, which `source_location` reports.
    pub source: RefCell<Option<(String, usize)>>,
}

impl Binding {
    /// Create a new binding with the given variables
    pub fn new(variables: HashMap<String, Rc<RefCell<Object>>>) -> Self {
        Self {
            variables: RefCell::new(variables),
            receiver: None,
            source: RefCell::new(None),
        }
    }

    /// Create a new binding with a receiver.
    pub fn with_receiver(
        variables: HashMap<String, Rc<RefCell<Object>>>,
        receiver: Object,
    ) -> Self {
        Self {
            variables: RefCell::new(variables),
            receiver: Some(receiver),
            source: RefCell::new(None),
        }
    }

    /// Get a variable from the binding
    pub fn get(&self, name: &str) -> Option<Rc<RefCell<Object>>> {
        self.variables.borrow().get(name).map(Rc::clone)
    }

    /// Bind `name` to a cell, adding it when the binding had no such local.
    pub fn set(&self, name: &str, cell: Rc<RefCell<Object>>) {
        self.variables.borrow_mut().insert(name.to_string(), cell);
    }

    /// Whether the binding names this local.
    pub fn has(&self, name: &str) -> bool {
        self.variables.borrow().contains_key(name)
    }

    /// How many locals the binding names.
    pub fn variable_count(&self) -> usize {
        self.variables.borrow().len()
    }

    /// Get all variable names in the binding
    pub fn keys(&self) -> Vec<String> {
        self.variables.borrow().keys().cloned().collect()
    }

    /// A copy that shares nothing with this one, which is what `dup` gives.
    pub fn copied(&self) -> Self {
        let copied = self
            .variables
            .borrow()
            .iter()
            .map(|(name, cell)| (name.clone(), Rc::new(RefCell::new(cell.borrow().clone()))))
            .collect();
        Self {
            variables: RefCell::new(copied),
            receiver: self.receiver.clone(),
            source: RefCell::new(self.source.borrow().clone()),
        }
    }
}
