// Object operations - comparison and boolean logic

use super::Object;

impl Object {
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

use std::rc::Rc;
