// Object operations - comparison and boolean logic

use super::{Method, Object};

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
            (Object::BigInt(a), Object::BigInt(b)) => a == b,
            // A normalized BigInt never holds a value an Int could, so the
            // two variants can only be equal through a Float.
            (Object::Int(_), Object::BigInt(_)) | (Object::BigInt(_), Object::Int(_)) => false,
            (Object::Float(a), Object::Float(b)) => {
                // NaN equals nothing, and two infinities of the same sign are
                // equal even though subtracting them is not a number.
                if a.is_nan() || b.is_nan() {
                    false
                } else if a.is_infinite() || b.is_infinite() {
                    a == b
                } else {
                    // Float comparison with epsilon for floating point precision
                    (a - b).abs() < 1e-9
                }
            }
            // A Float equals the Integer it holds exactly, which is how Ruby
            // compares numbers of different kinds.
            (Object::Int(a), Object::Float(b)) | (Object::Float(b), Object::Int(a)) => {
                *a as f64 == *b
            }
            (Object::BigInt(a), Object::Float(b)) | (Object::Float(b), Object::BigInt(a)) => {
                a.to_string().parse::<f64>().is_ok_and(|value| value == *b)
            }
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Symbol(a), Object::Symbol(b)) => a == b,
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
            // An instance of a String subclass compares by its characters,
            // in either order.
            (Object::Instance(_), Object::String(_)) | (Object::String(_), Object::Instance(_)) => {
                match (subclass_string(self), subclass_string(other)) {
                    (Some(text), None) => text.equals(other),
                    (None, Some(text)) => self.equals(&text),
                    _ => false,
                }
            }
            // Instance comparisons: value equality for Rational/Complex, reference for others
            (Object::Instance(a), Object::Instance(b)) => {
                let inst_a = a.borrow();
                let inst_b = b.borrow();
                let class_name = inst_a.class.name();
                if class_name == inst_b.class.name() && matches!(class_name, "Rational" | "Complex")
                {
                    // Compare by stored instance variables
                    let vars_a = &inst_a.instance_vars;
                    let vars_b = &inst_b.instance_vars;
                    vars_a.len() == vars_b.len()
                        && vars_a
                            .iter()
                            .all(|(k, v)| vars_b.get(k).is_some_and(|v2| v.equals(v2)))
                } else {
                    drop(inst_a);
                    drop(inst_b);
                    Rc::ptr_eq(a, b)
                }
            }
            // Classes and modules share one representation, so the same
            // object compares equal whichever variant is holding it.
            (Object::Class(a) | Object::Module(a), Object::Class(b) | Object::Module(b)) => {
                Rc::ptr_eq(a, b)
            }
            (Object::Method(a), Object::Method(b)) => {
                // Two Method objects are equal when they wrap the same
                // underlying definition bound to the same receiver. We use
                // pointer equality first (cheap) and structural body+name+
                // receiver equality second so freshly bound copies of the
                // same source method (e.g. from `obj.method(:x)` called
                // twice, or via aliases sharing the source method) compare
                // as equal — matching MRI's `Method#==`.
                let defined_name = |method: &Method| {
                    method
                        .original_name
                        .clone()
                        .unwrap_or_else(|| method.name.clone())
                };
                Rc::ptr_eq(a, b)
                    || (defined_name(a) == defined_name(b)
                        && a.body.len() == b.body.len()
                        && match (&a.receiver, &b.receiver) {
                            (Some(ra), Some(rb)) => ra.equals(rb),
                            (None, None) => true,
                            _ => false,
                        }
                        && a.body == b.body)
            }
            (Object::Block(a), Object::Block(b)) => Rc::ptr_eq(a, b),
            (Object::Binding(a), Object::Binding(b)) => Rc::ptr_eq(a, b),
            // Ruby compares two exceptions by class, message, and backtrace,
            // so a dup equals its original.
            (Object::Exception(a), Object::Exception(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                let (left, right) = (a.borrow(), b.borrow());
                let same_class = match (&left.class, &right.class) {
                    (Some(one), Some(two)) => Rc::ptr_eq(one, two),
                    _ => left.exception_type == right.exception_type,
                };
                same_class && left.message == right.message && left.backtrace == right.backtrace
            }
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

/// The characters behind an instance of a String subclass.
fn subclass_string(value: &Object) -> Option<Object> {
    let Object::Instance(instance) = value else {
        return None;
    };
    instance.borrow().instance_vars.get("__string__").cloned()
}
