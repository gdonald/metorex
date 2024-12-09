// Core Object enum definition for runtime value representation

use crate::class::Class;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::{BlockStatement, Exception, Instance, Method, ObjectHash};

/// Core object type representing all runtime values in Metorex
#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    /// Nil/null value
    Nil,

    /// Boolean value (true or false)
    Bool(bool),

    /// 64-bit signed integer
    Int(i64),

    /// 64-bit floating point number
    Float(f64),

    /// String value (reference counted for efficient copying)
    String(Rc<String>),

    /// Array/list of objects (mutable, reference counted)
    Array(Rc<RefCell<Vec<Object>>>),

    /// Dictionary/hash map (mutable, reference counted)
    Dict(Rc<RefCell<HashMap<String, Object>>>),

    /// Instance of a class
    Instance(Rc<RefCell<Instance>>),

    /// Class object (used for class definitions and instantiation)
    Class(Rc<Class>),

    /// Method object (bound or unbound)
    Method(Rc<Method>),

    /// Block/lambda/closure (critical for meta-programming)
    Block(Rc<BlockStatement>),

    /// Exception object
    Exception(Rc<RefCell<Exception>>),

    /// Set (unordered collection of unique objects)
    Set(Rc<RefCell<HashSet<ObjectHash>>>),

    /// Result type for explicit error handling
    Result(Result<Box<Object>, Box<Object>>),

    /// Native function (built-in function implemented in Rust)
    NativeFunction(String),

    /// Range object (start..end or start...end)
    Range {
        start: Box<Object>,
        end: Box<Object>,
        exclusive: bool,
    },
}

impl Object {
    /// Get the type name of this object
    pub fn type_name(&self) -> &'static str {
        match self {
            Object::Nil => "Nil",
            Object::Bool(_) => "Bool",
            Object::Int(_) => "Int",
            Object::Float(_) => "Float",
            Object::String(_) => "String",
            Object::Array(_) => "Array",
            Object::Dict(_) => "Dict",
            Object::Instance(_) => "Instance",
            Object::Class(_) => "Class",
            Object::Method(_) => "Method",
            Object::Block(_) => "Block",
            Object::Exception(_) => "Exception",
            Object::Set(_) => "Set",
            Object::Result(_) => "Result",
            Object::NativeFunction(_) => "NativeFunction",
            Object::Range { .. } => "Range",
        }
    }
}
