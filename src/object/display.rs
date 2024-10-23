// Display trait implementation for Object

use std::fmt;

use super::Object;

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
