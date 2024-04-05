// Runtime object representation for Metorex
// This module defines the core Object type that represents all runtime values

use crate::ast::Statement;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

pub use crate::class::Class;

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
    Block(Rc<BlockClosure>),

    /// Exception object
    Exception(Rc<RefCell<Exception>>),

    /// Set (unordered collection of unique objects)
    Set(Rc<RefCell<HashSet<ObjectHash>>>),

    /// Result type for explicit error handling
    Result(Result<Box<Object>, Box<Object>>),
}

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

/// Trait for callable objects (methods, functions, blocks)
pub trait Callable {
    /// Get the name of the callable
    fn name(&self) -> &str;

    /// Get the parameter names
    fn parameters(&self) -> &[String];

    /// Get the body statements
    fn body(&self) -> &[Statement];

    /// Get the arity (number of required parameters)
    fn arity(&self) -> usize {
        self.parameters().len()
    }
}

/// Method definition (function bound to a class)
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    /// Name of the method
    pub name: String,
    /// Parameter names
    pub parameters: Vec<String>,
    /// Method body (AST statements)
    pub body: Vec<Statement>,
    /// Optional receiver (for bound methods)
    pub receiver: Option<Box<Object>>,
}

impl Method {
    /// Create a new method
    pub fn new(name: String, parameters: Vec<String>, body: Vec<Statement>) -> Self {
        Self {
            name,
            parameters,
            body,
            receiver: None,
        }
    }

    /// Bind this method to a receiver
    pub fn bind(&self, receiver: Object) -> Self {
        Self {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            receiver: Some(Box::new(receiver)),
        }
    }

    /// Check if this method is bound to a receiver
    pub fn is_bound(&self) -> bool {
        self.receiver.is_some()
    }

    /// Get the receiver if this method is bound
    pub fn receiver(&self) -> Option<&Object> {
        self.receiver.as_deref()
    }
}

impl Callable for Method {
    fn name(&self) -> &str {
        &self.name
    }

    fn parameters(&self) -> &[String] {
        &self.parameters
    }

    fn body(&self) -> &[Statement] {
        &self.body
    }
}

/// Block/lambda/closure with captured variables
#[derive(Debug, Clone, PartialEq)]
pub struct BlockClosure {
    /// Parameter names
    pub parameters: Vec<String>,
    /// Block body (AST statements)
    pub body: Vec<Statement>,
    /// Captured variables from outer scope
    pub captured_vars: HashMap<String, Object>,
}

impl BlockClosure {
    /// Create a new block closure
    pub fn new(
        parameters: Vec<String>,
        body: Vec<Statement>,
        captured_vars: HashMap<String, Object>,
    ) -> Self {
        Self {
            parameters,
            body,
            captured_vars,
        }
    }

    /// Get the captured variables
    pub fn captured_vars(&self) -> &HashMap<String, Object> {
        &self.captured_vars
    }
}

impl Callable for BlockClosure {
    fn name(&self) -> &str {
        "<block>"
    }

    fn parameters(&self) -> &[String] {
        &self.parameters
    }

    fn body(&self) -> &[Statement] {
        &self.body
    }
}

/// Source location for exceptions
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    /// File name or path
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self { file, line, column }
    }
}

/// Exception object for error handling
#[derive(Debug, Clone, PartialEq)]
pub struct Exception {
    /// Exception type/class name
    pub exception_type: String,
    /// Error message
    pub message: String,
    /// Optional backtrace
    pub backtrace: Option<Vec<String>>,
    /// Source location where the exception occurred
    pub location: Option<SourceLocation>,
    /// Cause chain (wrapped exception)
    pub cause: Option<Box<Object>>,
}

impl Exception {
    /// Create a new exception
    pub fn new(exception_type: String, message: String) -> Self {
        Self {
            exception_type,
            message,
            backtrace: None,
            location: None,
            cause: None,
        }
    }

    /// Create an exception with backtrace
    pub fn with_backtrace(exception_type: String, message: String, backtrace: Vec<String>) -> Self {
        Self {
            exception_type,
            message,
            backtrace: Some(backtrace),
            location: None,
            cause: None,
        }
    }

    /// Create an exception with source location
    pub fn with_location(
        exception_type: String,
        message: String,
        location: SourceLocation,
    ) -> Self {
        Self {
            exception_type,
            message,
            backtrace: None,
            location: Some(location),
            cause: None,
        }
    }

    /// Create an exception with a cause
    pub fn with_cause(exception_type: String, message: String, cause: Object) -> Self {
        Self {
            exception_type,
            message,
            backtrace: None,
            location: None,
            cause: Some(Box::new(cause)),
        }
    }

    /// Create an exception with all fields
    pub fn with_all(
        exception_type: String,
        message: String,
        backtrace: Option<Vec<String>>,
        location: Option<SourceLocation>,
        cause: Option<Object>,
    ) -> Self {
        Self {
            exception_type,
            message,
            backtrace,
            location,
            cause: cause.map(Box::new),
        }
    }

    /// Get the full exception chain
    pub fn exception_chain(&self) -> Vec<String> {
        let mut chain = vec![format!("{}: {}", self.exception_type, self.message)];

        if let Some(ref cause_obj) = self.cause
            && let Object::Exception(cause_exc) = cause_obj.as_ref()
        {
            let cause = cause_exc.borrow();
            chain.extend(cause.exception_chain());
        }

        chain
    }
}

/// Wrapper for Object to make it hashable (for use in HashSet)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectHash {
    /// String representation of the object for hashing
    hash_value: String,
}

impl ObjectHash {
    /// Create a hashable wrapper from an object
    pub fn from_object(obj: &Object) -> Option<Self> {
        match obj {
            Object::Nil => Some(Self {
                hash_value: "nil".to_string(),
            }),
            Object::Bool(b) => Some(Self {
                hash_value: b.to_string(),
            }),
            Object::Int(i) => Some(Self {
                hash_value: i.to_string(),
            }),
            Object::Float(f) => Some(Self {
                hash_value: f.to_string(),
            }),
            Object::String(s) => Some(Self {
                hash_value: s.to_string(),
            }),
            // Arrays, Dicts, Instances, etc. are not hashable
            _ => None,
        }
    }
}

// Implement Display for Object to provide string representation
impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Bool(b) => write!(f, "{}", b),
            Object::Int(i) => write!(f, "{}", i),
            Object::Float(fl) => write!(f, "{}", fl),
            Object::String(s) => write!(f, "{}", s),
            Object::Array(arr) => {
                write!(f, "[")?;
                let elements = arr.borrow();
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Object::Dict(dict) => {
                write!(f, "{{")?;
                let map = dict.borrow();
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            Object::Instance(inst) => {
                let instance = inst.borrow();
                write!(f, "<{} instance>", instance.class.name())
            }
            Object::Class(class) => write!(f, "<class {}>", class.name()),
            Object::Method(method) => write!(f, "<method {}>", method.name),
            Object::Block(_) => write!(f, "<block>"),
            Object::Exception(exc) => {
                let exception = exc.borrow();
                write!(f, "{}: {}", exception.exception_type, exception.message)
            }
            Object::Set(set) => {
                write!(f, "#{{")?;
                let elements = set.borrow();
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem.hash_value)?;
                }
                write!(f, "}}")
            }
            Object::Result(result) => match result {
                Ok(obj) => write!(f, "Ok({})", obj),
                Err(obj) => write!(f, "Err({})", obj),
            },
        }
    }
}

// Helper methods for Object
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
        }
    }

    /// Check if this object is truthy (for conditional evaluation)
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Nil => false,
            Object::Bool(b) => *b,
            // All other values are truthy
            _ => true,
        }
    }

    /// Check if this object is falsy
    pub fn is_falsy(&self) -> bool {
        !self.is_truthy()
    }

    /// Deep equality comparison between objects
    pub fn equals(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::Nil, Object::Nil) => true,
            (Object::Bool(a), Object::Bool(b)) => a == b,
            (Object::Int(a), Object::Int(b)) => a == b,
            (Object::Float(a), Object::Float(b)) => {
                // Float comparison with epsilon for floating point precision
                (a - b).abs() < 1e-9
            }
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Array(a), Object::Array(b)) => {
                let arr_a = a.borrow();
                let arr_b = b.borrow();
                if arr_a.len() != arr_b.len() {
                    return false;
                }
                arr_a.iter().zip(arr_b.iter()).all(|(x, y)| x.equals(y))
            }
            (Object::Dict(a), Object::Dict(b)) => {
                let dict_a = a.borrow();
                let dict_b = b.borrow();
                if dict_a.len() != dict_b.len() {
                    return false;
                }
                dict_a
                    .iter()
                    .all(|(key, val)| dict_b.get(key).is_some_and(|v| val.equals(v)))
            }
            (Object::Set(a), Object::Set(b)) => {
                let set_a = a.borrow();
                let set_b = b.borrow();
                if set_a.len() != set_b.len() {
                    return false;
                }
                set_a.iter().all(|item| set_b.contains(item))
            }
            (Object::Result(a), Object::Result(b)) => match (a, b) {
                (Ok(a_val), Ok(b_val)) => a_val.equals(b_val),
                (Err(a_err), Err(b_err)) => a_err.equals(b_err),
                _ => false,
            },
            // Instance, Class, Method, Block, and Exception comparisons by reference
            (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
            (Object::Class(a), Object::Class(b)) => Rc::ptr_eq(a, b),
            (Object::Method(a), Object::Method(b)) => Rc::ptr_eq(a, b),
            (Object::Block(a), Object::Block(b)) => Rc::ptr_eq(a, b),
            (Object::Exception(a), Object::Exception(b)) => Rc::ptr_eq(a, b),
            // Different types are not equal
            _ => false,
        }
    }

    /// Compute hash for hashable types (for use in dictionaries)
    pub fn hash(&self) -> Option<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        match self {
            Object::Nil => {
                let mut hasher = DefaultHasher::new();
                "nil".hash(&mut hasher);
                Some(hasher.finish())
            }
            Object::Bool(b) => {
                let mut hasher = DefaultHasher::new();
                b.hash(&mut hasher);
                Some(hasher.finish())
            }
            Object::Int(i) => {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                Some(hasher.finish())
            }
            Object::Float(f) => {
                let mut hasher = DefaultHasher::new();
                // Convert float to bits for consistent hashing
                f.to_bits().hash(&mut hasher);
                Some(hasher.finish())
            }
            Object::String(s) => {
                let mut hasher = DefaultHasher::new();
                s.as_str().hash(&mut hasher);
                Some(hasher.finish())
            }
            // Complex types are not hashable
            _ => None,
        }
    }
}
