// Constructor helper methods for Object

use crate::class::Class;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::{Exception, Instance, Object};

impl Object {
    /// Create a string object from a Rust string
    pub fn string(s: impl Into<String>) -> Self {
        Object::String(Rc::new(s.into()))
    }

    /// Create an empty array
    pub fn empty_array() -> Self {
        Object::Array(Rc::new(RefCell::new(Vec::new())))
    }

    /// Create an array from a vector of objects
    pub fn array(elements: Vec<Object>) -> Self {
        Object::Array(Rc::new(RefCell::new(elements)))
    }

    /// Create an empty dictionary
    pub fn empty_dict() -> Self {
        Object::Dict(Rc::new(RefCell::new(HashMap::new())))
    }

    /// Create a dictionary from a HashMap
    pub fn dict(map: HashMap<String, Object>) -> Self {
        Object::Dict(Rc::new(RefCell::new(map)))
    }

    /// Create an empty set
    pub fn empty_set() -> Self {
        Object::Set(Rc::new(RefCell::new(HashSet::new())))
    }

    /// Create an instance of a class
    pub fn instance(class: Rc<Class>) -> Self {
        Object::Instance(Rc::new(RefCell::new(Instance::new(class))))
    }

    /// Create an exception
    pub fn exception(exception_type: impl Into<String>, message: impl Into<String>) -> Self {
        Object::Exception(Rc::new(RefCell::new(Exception::new(
            exception_type.into(),
            message.into(),
        ))))
    }

    /// Create an Ok result
    pub fn ok(value: Object) -> Self {
        Object::Result(Ok(Box::new(value)))
    }

    /// Create an Err result
    pub fn err(value: Object) -> Self {
        Object::Result(Err(Box::new(value)))
    }
}
