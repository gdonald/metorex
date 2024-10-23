// ObjectHash - wrapper for making Objects hashable for use in HashSet

use super::Object;

/// Wrapper for Object to make it hashable (for use in HashSet)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectHash {
    /// String representation of the object for hashing
    pub(crate) hash_value: String,
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
