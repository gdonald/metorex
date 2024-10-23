// Instance struct - represents an instance of a class

use crate::class::Class;
use std::collections::HashMap;
use std::rc::Rc;

use super::{Method, Object};

/// Instance of a class with instance variables
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    /// Reference to the class this is an instance of
    pub class: Rc<Class>,
    /// Instance variables (@variable)
    pub instance_vars: HashMap<String, Object>,
}

impl Instance {
    /// Create a new instance of a class
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            instance_vars: HashMap::new(),
        }
    }

    /// Get an instance variable
    pub fn get_var(&self, name: &str) -> Option<&Object> {
        self.instance_vars.get(name)
    }

    /// Set an instance variable
    pub fn set_var(&mut self, name: String, value: Object) {
        self.instance_vars.insert(name, value);
    }

    /// Check if this instance's class (or a superclass) knows about the variable.
    pub fn is_var_declared(&self, name: &str) -> bool {
        self.class.has_instance_var(name)
    }

    /// Find a method on this instance's class (walks the inheritance chain)
    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
        self.class.find_method(name)
    }

    /// Get the class name of this instance
    pub fn class_name(&self) -> &str {
        self.class.name()
    }
}
