//! Runtime class representation for Metorex
//! Handles method tables, inheritance, and instance variable declarations.

use crate::object::{Method, Object};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Runtime class definition with method table and inheritance.
#[derive(Debug)]
pub struct Class {
    name: String,
    superclass: Option<Rc<Class>>,
    methods: RefCell<HashMap<String, Rc<Method>>>,
    instance_variables: RefCell<HashSet<String>>,
    class_variables: RefCell<HashMap<String, crate::object::Object>>,
}

impl Class {
    /// Create a new class with an optional superclass.
    pub fn new(name: impl Into<String>, superclass: Option<Rc<Class>>) -> Self {
        Self {
            name: name.into(),
            superclass,
            methods: RefCell::new(HashMap::new()),
            instance_variables: RefCell::new(HashSet::new()),
            class_variables: RefCell::new(HashMap::new()),
        }
    }

    /// Return the class name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the superclass if present.
    pub fn superclass(&self) -> Option<Rc<Class>> {
        self.superclass.as_ref().map(Rc::clone)
    }

    /// Declare a new instance variable on this class.
    pub fn declare_instance_var(&self, name: impl Into<String>) {
        self.instance_variables.borrow_mut().insert(name.into());
    }

    /// Check if this class (or a superclass) declares the given instance variable.
    pub fn has_instance_var(&self, name: &str) -> bool {
        if self.instance_variables.borrow().contains(name) {
            return true;
        }

        self.superclass
            .as_ref()
            .is_some_and(|superclass| superclass.has_instance_var(name))
    }

    /// Return the list of instance variable names defined directly on this class.
    pub fn instance_variables(&self) -> Vec<String> {
        let mut vars = self
            .instance_variables
            .borrow()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        vars.sort();
        vars
    }

    /// Define or replace a method on this class.
    pub fn define_method(&self, name: impl Into<String>, method: Rc<Method>) {
        self.methods.borrow_mut().insert(name.into(), method);
    }

    /// Determine whether this class defines a method (without checking superclasses).
    pub fn has_own_method(&self, name: &str) -> bool {
        self.methods.borrow().contains_key(name)
    }

    /// Look up a method by walking the inheritance chain.
    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
        if let Some(method) = self.methods.borrow().get(name) {
            return Some(Rc::clone(method));
        }

        self.superclass
            .as_ref()
            .and_then(|superclass| superclass.find_method(name))
    }

    /// Return a list of method names defined directly on this class.
    pub fn method_names(&self) -> Vec<String> {
        let mut names = self.methods.borrow().keys().cloned().collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Set a class variable on this class.
    pub fn set_class_var(&self, name: impl Into<String>, value: Object) {
        self.class_variables.borrow_mut().insert(name.into(), value);
    }

    /// Retrieve a class variable from this class.
    pub fn get_class_var(&self, name: &str) -> Option<Object> {
        self.class_variables.borrow().get(name).cloned()
    }
}

impl Clone for Class {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            superclass: self.superclass.clone(),
            methods: RefCell::new(self.methods.borrow().clone()),
            instance_variables: RefCell::new(self.instance_variables.borrow().clone()),
            class_variables: RefCell::new(self.class_variables.borrow().clone()),
        }
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        let self_super = self.superclass.as_ref().map(Rc::as_ptr);
        let other_super = other.superclass.as_ref().map(Rc::as_ptr);
        if self_super != other_super {
            return false;
        }

        {
            let vars = self.instance_variables.borrow();
            let other_vars = other.instance_variables.borrow();
            if *vars != *other_vars {
                return false;
            }
        }

        let self_methods = self.methods.borrow();
        let other_methods = other.methods.borrow();
        if self_methods.len() != other_methods.len() {
            return false;
        }
        if self.class_variables.borrow().len() != other.class_variables.borrow().len() {
            return false;
        }

        self_methods.iter().all(|(name, method)| {
            other_methods.get(name).is_some_and(|other_method| {
                Rc::ptr_eq(method, other_method) || method == other_method
            })
        }) && {
            let class_vars = self.class_variables.borrow();
            let other_class_vars = other.class_variables.borrow();
            class_vars
                .iter()
                .all(|(name, value)| other_class_vars.get(name) == Some(value))
        }
    }
}

impl Eq for Class {}
