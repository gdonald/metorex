// ObjectHash - wrapper for making Objects hashable for use in a Set

use super::Object;

/// Wrapper that makes an Object usable as a set element. The rendered
/// `hash_value` is what decides membership, and the element itself travels
/// alongside so the set gives back what was put in.
#[derive(Debug, Clone)]
pub struct ObjectHash {
    /// String representation of the object for hashing
    pub(crate) hash_value: String,
    /// The element itself, which `to_a` and `each` hand back.
    pub(crate) value: Object,
}

impl PartialEq for ObjectHash {
    fn eq(&self, other: &Self) -> bool {
        self.hash_value == other.hash_value
    }
}

impl Eq for ObjectHash {}

impl std::hash::Hash for ObjectHash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash_value.hash(state);
    }
}

impl ObjectHash {
    /// Create a hashable wrapper from an object. Anything with a stable
    /// rendering can be an element; an Instance is told apart by identity,
    /// which is what its rendering carries.
    pub fn from_object(obj: &Object) -> Option<Self> {
        let hash_value = match obj {
            Object::Nil => "nil".to_string(),
            Object::Bool(value) => value.to_string(),
            Object::Int(value) => value.to_string(),
            // A Float keeps its fraction, so 1.0 and 1 are different
            // elements the way `eql?` makes them.
            Object::Float(value) if value.fract() == 0.0 && value.is_finite() => {
                format!("{:.1}", value)
            }
            Object::Float(value) => value.to_string(),
            Object::String(text) => format!("\"{}\"", text),
            Object::Symbol(name) => format!(":{}", name),
            Object::BigInt(value) => value.to_string(),
            // A collection is told apart by what it holds, which its own
            // rendering reports. That rendering guards against a collection
            // that reaches itself, where a structural walk would not.
            Object::Array(_) | Object::Dict(_) | Object::Range { .. } | Object::Regex(_, _) => {
                format!("{}{}", obj.type_name(), obj)
            }
            Object::Instance(instance) => {
                format!("#<{:p}>", std::rc::Rc::as_ptr(instance))
            }
            // A Set is told apart by what it holds, in an order-independent
            // way, so two sets with the same elements are one element.
            Object::Set(set) => {
                let mut keys: Vec<String> = set
                    .borrow()
                    .iter()
                    .map(|held| held.hash_value.clone())
                    .collect();
                keys.sort();
                format!("Set[{}]", keys.join(", "))
            }
            _ => return None,
        };
        Some(Self {
            hash_value,
            value: obj.clone(),
        })
    }
}
