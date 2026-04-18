// Instance struct - represents an instance of a class

use crate::class::Class;
use std::cell::RefCell;
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
    /// Singleton methods defined directly on this instance (def obj.method).
    pub singleton_methods: Rc<RefCell<HashMap<String, Rc<Method>>>>,
    /// Lazily-allocated singleton class for this instance (Ruby semantics:
    /// every object conceptually has one, but we only materialize on demand).
    pub singleton_class: Rc<RefCell<Option<Rc<Class>>>>,
}

impl Instance {
    /// Create a new instance of a class
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            instance_vars: HashMap::new(),
            singleton_methods: Rc::new(RefCell::new(HashMap::new())),
            singleton_class: Rc::new(RefCell::new(None)),
        }
    }

    /// Attach a singleton method to this instance.
    pub fn define_singleton_method(&self, name: String, method: Rc<Method>) {
        self.singleton_methods.borrow_mut().insert(name, method);
    }

    /// Look up a singleton method on this instance.
    pub fn find_singleton_method(&self, name: &str) -> Option<Rc<Method>> {
        self.singleton_methods.borrow().get(name).cloned()
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
