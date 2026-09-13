// Binding - represents a namespace/scope with captured variables

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Object;

/// Binding object represents a namespace/scope containing variable bindings.
/// The variables are held behind a cell because code run through a binding
/// may name a local the binding did not have, which Ruby adds to it.
#[derive(Debug, Clone)]
pub struct Binding {
    /// Captured variables from the binding's scope
    pub variables: RefCell<HashMap<String, Rc<RefCell<Object>>>>,
    /// The names in the order they were bound, which is the order
    /// `local_variables` reports them in.
    order: RefCell<Vec<String>>,
    /// Receiver (self) at the point the binding was captured
    pub receiver: Option<Object>,
    /// Where the binding was captured, which `source_location` reports.
    pub source: RefCell<Option<(String, usize)>>,
}

impl PartialEq for Binding {
    /// The order names were bound in is how they are reported, not part of
    /// what makes two bindings equal.
    fn eq(&self, other: &Self) -> bool {
        self.variables == other.variables
            && self.receiver == other.receiver
            && self.source == other.source
    }
}

impl Binding {
    /// Create a new binding with the given variables
    pub fn new(variables: HashMap<String, Rc<RefCell<Object>>>) -> Self {
        Self {
            order: RefCell::new(variables.keys().cloned().collect()),
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
            order: RefCell::new(variables.keys().cloned().collect()),
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
        if self
            .variables
            .borrow_mut()
            .insert(name.to_string(), cell)
            .is_none()
        {
            self.order.borrow_mut().push(name.to_string());
        }
    }

    /// Replaces the order `local_variables` reports, for a binding built from
    /// a scope chain where the order is already known.
    pub fn set_order(&self, names: Vec<String>) {
        *self.order.borrow_mut() = names;
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
        let held = self.variables.borrow();
        let mut named: Vec<String> = self
            .order
            .borrow()
            .iter()
            .filter(|name| held.contains_key(*name))
            .cloned()
            .collect();
        named.extend(
            held.keys()
                .filter(|name| !self.order.borrow().contains(*name))
                .cloned(),
        );
        named
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
            order: RefCell::new(self.order.borrow().clone()),
            receiver: self.receiver.clone(),
            source: RefCell::new(self.source.borrow().clone()),
        }
    }
}
