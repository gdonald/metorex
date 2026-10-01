// The integer `Object#hash` answers.

use super::*;

impl VirtualMachine {
    /// The integer `Object#hash` answers. Value types digest their canonical
    /// string form so equal values agree; everything else uses its identity.
    /// The digest a value carries when it sits inside a collection. A
    /// collection there stands for its kind and how much it holds, which is
    /// what lets one holding itself be hashed at all.
    pub(crate) fn shallow_digest(
        &mut self,
        held: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let marked = |kind: i64, count: usize| kind.wrapping_mul(1_000_003) ^ count as i64;
        match held {
            Object::Dict(entries) => Ok(marked(
                1,
                entries
                    .borrow()
                    .keys()
                    .filter(|key| !crate::vm::native_methods::hash_methods::is_internal_key(key))
                    .count(),
            )),
            Object::Array(items) => Ok(marked(2, items.borrow().len())),
            Object::Set(items) => Ok(marked(3, items.borrow().len())),
            other => self.hash_digest(other, position),
        }
    }

    /// The number Ruby mixes in for a collection that was found to hold
    /// itself, which is the hash of the Array class in MRI.
    const LOOPED_COLLECTION_MIX: i64 = 0x5bf0_3635;

    /// The hash of an Array or a Hash: the length mixed with each element's
    /// own `#hash`, read through `to_int`. A walk that reaches a collection
    /// it is already inside answers from the outermost length alone, so a
    /// list holding itself hashes the same however deeply it is wrapped.
    fn collection_hash_digest(
        &mut self,
        receiver: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let address = match receiver {
            Object::Array(held) => std::rc::Rc::as_ptr(held) as usize,
            Object::Dict(held) => std::rc::Rc::as_ptr(held) as usize,
            _ => return Ok(0),
        };
        if self.hash_walk.contains(&address) {
            self.hash_walk_looped = true;
            return Ok(0);
        }
        let outermost = self.hash_walk.is_empty();
        if outermost {
            self.hash_walk_looped = false;
        }
        self.hash_walk.push(address);
        let parts: Vec<Object> = match receiver {
            Object::Array(held) => held.borrow().clone(),
            Object::Dict(held) => {
                let mut listed = Vec::new();
                for (key, value) in self.hash_pairs_for_digest(held) {
                    listed.push(key);
                    listed.push(value);
                }
                listed
            }
            _ => Vec::new(),
        };
        let length = match receiver {
            Object::Dict(_) => parts.len() / 2,
            _ => parts.len(),
        };
        let mut digest = (length as i64).wrapping_mul(0x9e37_79b9);
        let mut walked = Ok(());
        for part in parts {
            match self.element_hash_number(&part, position) {
                Ok(number) => {
                    digest = digest.rotate_left(5).wrapping_mul(31).wrapping_add(number);
                }
                Err(trouble) => {
                    walked = Err(trouble);
                    break;
                }
            }
        }
        self.hash_walk.pop();
        walked?;
        if outermost && self.hash_walk_looped {
            self.hash_walk_looped = false;
            let folded = (length as i64).wrapping_mul(0x9e37_79b9);
            return Ok(folded
                .rotate_left(5)
                .wrapping_mul(31)
                .wrapping_add(Self::LOOPED_COLLECTION_MIX));
        }
        Ok(digest)
    }

    /// The number an element contributes: its own `#hash`, read through
    /// `to_int` when what it answers is not already an Integer.
    fn element_hash_number(
        &mut self,
        element: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let hashed = self.send_to_object(element.clone(), "hash", vec![], position)?;
        match hashed {
            Object::Int(number) => Ok(number),
            other => match self.send_to_object(other, "to_int", vec![], position)? {
                Object::Int(number) => Ok(number),
                _ => Ok(0),
            },
        }
    }

    pub(crate) fn hash_digest(
        &mut self,
        receiver: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        // Two names for one method hash alike, so the hash comes from the
        // definition they share rather than from the object holding it.
        if let Object::Method(method) = receiver {
            // The name the method was defined under tells two aliases apart
            // from two different methods.
            let named = method
                .original_name
                .clone()
                .or_else(|| method.native_alias.clone())
                .unwrap_or_else(|| method.name.clone());
            // Two names for one native method stand for that method, so they
            // hash alike.
            let owner_name = method
                .owner_class
                .as_ref()
                .map(|held| held.name().to_string())
                .or_else(|| method.owner.clone())
                .unwrap_or_default();
            let named = native_alias_target(&owner_name, &named)
                .unwrap_or(named.as_str())
                .to_string();
            // A method bound to an object belongs to that object's class,
            // whatever the lookup walked past to reach it. Two names for one
            // native method are reported under different owners, and they are
            // still the one method.
            let owner = match method.receiver.as_deref() {
                Some(held) => self.builtins().class_of(held).name().to_string(),
                None => method.owner.clone().unwrap_or_default(),
            };
            let mut digest: i64 = 0;
            for byte in named.bytes().chain(owner.bytes()) {
                digest = digest.wrapping_mul(31).wrapping_add(byte as i64);
            }
            if let Some(held) = method.receiver.as_deref() {
                let id = self.call_object_method(held, "object_id", &[], position)?;
                if let Some(Object::Int(id)) = id {
                    digest = digest.wrapping_mul(31).wrapping_add(id);
                }
            }
            return Ok(seeded_hash(digest));
        }
        // A collection is hashed from what it holds, asking each element for
        // its own `#hash`, and a collection that reaches itself is answered
        // from its length the way Ruby answers one. A subclass hashes as the
        // collection behind it, so the class it was made from makes no
        // difference to the number.
        if matches!(receiver, Object::Array(_) | Object::Dict(_)) {
            return self
                .collection_hash_digest(receiver, position)
                .map(seeded_hash);
        }
        if let Some(backing @ (Object::Array(_) | Object::Dict(_))) =
            crate::vm::native_methods::array_subclass_value(receiver)
                .or_else(|| crate::vm::native_methods::hash_subclass_value(receiver))
        {
            return self
                .collection_hash_digest(&backing, position)
                .map(seeded_hash);
        }
        if let Some(hashable) = crate::object::ObjectHash::from_object(receiver) {
            return Ok(seeded_text_hash(&hashable.hash_value));
        }
        match self.call_object_method(receiver, "object_id", &[], position)? {
            Some(Object::Int(id)) => Ok(id),
            _ => Ok(0),
        }
    }
}

/// A hash value mixed with a number this process drew when it started, so
/// two processes answer different hashes for the same value. Ruby does this
/// to keep a hash table from being filled with values chosen to collide.
pub(crate) fn seeded_hash(digest: i64) -> i64 {
    static SEED: std::sync::OnceLock<i64> = std::sync::OnceLock::new();
    let seed = *SEED.get_or_init(|| {
        use std::hash::{BuildHasher, Hasher};
        std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish() as i64
    });
    (digest ^ seed).wrapping_mul(0x9E37_79B9_7F4A_7C15_u64 as i64)
}

/// A hash value for text, seeded for this process.
pub(crate) fn seeded_text_hash(text: &str) -> i64 {
    seeded_hash(text.bytes().fold(0_i64, |digest, byte| {
        digest.wrapping_mul(31).wrapping_add(byte as i64)
    }))
}
