// Constructor helper methods for Object

use crate::class::Class;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use super::{Exception, Instance, Object};

impl Object {
    /// Create a string object from a Rust string
    pub fn string(s: impl Into<String>) -> Self {
        Object::String(Rc::new(s.into()))
    }

    /// An Integer from an arbitrary-precision value, narrowed back to `Int`
    /// whenever it fits. Ruby draws no line between the two sizes, so nothing
    /// downstream should ever see a `BigInt` holding a small number.
    pub fn integer(value: num_bigint::BigInt) -> Self {
        match i64::try_from(&value) {
            Ok(small) => Object::Int(small),
            Err(_) => Object::BigInt(Rc::new(value)),
        }
    }

    /// This object's value as an arbitrary-precision integer, for the numeric
    /// kinds that have one.
    pub fn as_big_integer(&self) -> Option<num_bigint::BigInt> {
        match self {
            Object::Int(value) => Some(num_bigint::BigInt::from(*value)),
            Object::BigInt(value) => Some((**value).clone()),
            _ => None,
        }
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
        Object::Dict(Rc::new(RefCell::new(IndexMap::new())))
    }

    /// Create a dictionary from an ordered map
    pub fn dict(map: IndexMap<String, Object>) -> Self {
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

    /// Create an exception with a cause
    pub fn exception_with_cause(
        exception_type: impl Into<String>,
        message: impl Into<String>,
        cause: Object,
    ) -> Self {
        Object::Exception(Rc::new(RefCell::new(Exception::with_cause(
            exception_type.into(),
            message.into(),
            cause,
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
