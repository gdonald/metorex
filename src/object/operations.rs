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
        self.equals_within(other, &mut Vec::new())
    }

    /// Compare two objects, remembering the pairs of collections already being
    /// compared. An array that holds itself is equal to another built the same
    /// way, and the record of what is in flight is what stops the comparison
    /// from following the cycle forever.
    fn equals_within(&self, other: &Object, in_flight: &mut Vec<(usize, usize)>) -> bool {
        let pair = match (self, other) {
            (Object::Array(a), Object::Array(b)) => Some((
                std::rc::Rc::as_ptr(a) as usize,
                std::rc::Rc::as_ptr(b) as usize,
            )),
            (Object::Dict(a), Object::Dict(b)) => Some((
                std::rc::Rc::as_ptr(a) as usize,
                std::rc::Rc::as_ptr(b) as usize,
            )),
            _ => None,
        };
        if let Some(pair) = pair {
            if in_flight.contains(&pair) {
                return true;
            }
            in_flight.push(pair);
            let answer = self.equals_inner(other, in_flight);
            in_flight.pop();
            return answer;
        }
        self.equals_inner(other, in_flight)
    }

    fn equals_inner(&self, other: &Object, in_flight: &mut Vec<(usize, usize)>) -> bool {
        // An instance of an Array subclass compares by the elements it holds,
        // so it equals a plain Array with the same contents.
        // An instance of a Hash subclass compares by the entries it holds,
        // so it equals a plain Hash with the same contents.
        match (backing_hash(self), backing_hash(other)) {
            (Some(left), None) => return left.equals_within(other, in_flight),
            (None, Some(right)) => return self.equals_within(&right, in_flight),
            (Some(left), Some(right)) => return left.equals_within(&right, in_flight),
            (None, None) => {}
        }
        match (backing_array(self), backing_array(other)) {
            (Some(left), None) => return left.equals_within(other, in_flight),
            (None, Some(right)) => return self.equals_within(&right, in_flight),
            (Some(left), Some(right)) => return left.equals_within(&right, in_flight),
            (None, None) => {}
        }
        match (self, other) {
            (Object::Nil, Object::Nil) => true,
            (Object::Bool(a), Object::Bool(b)) => a == b,
            (Object::Int(a), Object::Int(b)) => a == b,
            (Object::BigInt(a), Object::BigInt(b)) => a == b,
            // A normalized BigInt never holds a value an Int could, so the
            // two variants can only be equal through a Float.
            (Object::Int(_), Object::BigInt(_)) | (Object::BigInt(_), Object::Int(_)) => false,
            // Two Floats are equal when they are the same number, which is
            // what Ruby compares: `0.1 + 0.2 == 0.3` is false, and a number as
            // small as 1e-16 is not zero. NaN equals nothing, itself included.
            (Object::Float(a), Object::Float(b)) => a == b,
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
                arr_a
                    .iter()
                    .zip(arr_b.iter())
                    .all(|(left, right)| left.equals_within(right, in_flight))
            }
            (Object::Dict(a), Object::Dict(b)) => {
                // The sentinels a hash keeps for its default, its key objects,
                // and its identity setting are bookkeeping, not contents.
                let dict_a = a.borrow();
                let dict_b = b.borrow();
                let entries = |dict: &indexmap::IndexMap<String, Object>| {
                    dict.iter()
                        .filter(|(key, _)| !key.starts_with("__MX_"))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect::<Vec<_>>()
                };
                let left_entries = entries(&dict_a);
                let right_entries = entries(&dict_b);
                if left_entries.len() != right_entries.len() {
                    return false;
                }
                left_entries.iter().all(|(key, value)| {
                    right_entries.iter().any(|(other_key, other_value)| {
                        key == other_key && value.equals_within(other_value, in_flight)
                    })
                })
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

/// The elements an instance of an Array subclass holds, or None for anything
/// else. Kept here so equality can compare such an instance to a plain Array
/// without reaching into the VM.
fn backing_array(value: &Object) -> Option<Object> {
    let Object::Instance(instance) = value else {
        return None;
    };
    instance.borrow().instance_vars.get("__array__").cloned()
}

/// The entries an instance of a Hash subclass holds, or None for anything
/// else.
fn backing_hash(value: &Object) -> Option<Object> {
    let Object::Instance(instance) = value else {
        return None;
    };
    instance.borrow().instance_vars.get("__hash__").cloned()
}
