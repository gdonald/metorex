// Whether a value is frozen, and the error a write to one raises.

use super::*;

impl VirtualMachine {
    /// The address a collection lives at, used to record that it is frozen.
    /// An Array, Hash, or Set has nowhere of its own to keep the flag.
    /// An instance variable an object keeps outside itself, which is where a
    /// callable, a String, and the collections hold theirs.
    pub(crate) fn carried_variable(&self, held: &Object, name: &str) -> Option<Object> {
        let address = Self::collection_address(held)?;
        self.collection_variables
            .get(&address)
            .and_then(|held| held.get(name))
            .cloned()
    }

    pub(crate) fn collection_address(receiver: &Object) -> Option<usize> {
        match receiver {
            Object::Array(items) => Some(Rc::as_ptr(items) as usize),
            Object::Dict(entries) => Some(Rc::as_ptr(entries) as usize),
            Object::Set(items) => Some(Rc::as_ptr(items) as usize),
            // A String keeps its instance variables the same way, since the
            // text itself has nowhere to put them.
            Object::String(text) => Some(Rc::as_ptr(text) as usize),
            // A callable and a Binding are the same: what they hold is the
            // code and the scope, with nowhere for a variable or a flag.
            Object::Method(method) => Some(Rc::as_ptr(method) as usize),
            Object::Block(block) => Some(Rc::as_ptr(block) as usize),
            Object::Binding(binding) => Some(Rc::as_ptr(binding) as usize),
            // A Regexp is known by the pattern it holds, which each one made
            // holds a copy of its own.
            Object::Regex(pattern, _) => Some(Rc::as_ptr(pattern) as usize),
            _ => None,
        }
    }

    /// The FrozenError Ruby raises for modifying `receiver`. The message
    /// names the class and inspects the object, falling back to `...` when
    /// that inspection would itself modify the frozen object.
    pub(crate) fn frozen_modification_error(
        &mut self,
        receiver: &Object,
        position: Position,
    ) -> MetorexError {
        let class_name = crate::vm::native_methods::define_method::ruby_class_name(receiver);
        // `inspect` may itself modify the object, which raises here again.
        // Ruby shows `...` rather than recursing.
        let rendered = if self.rendering_frozen_error {
            "...".to_string()
        } else {
            self.rendering_frozen_error = true;
            let rendered = self.get_inspect_representation(receiver, position);
            self.rendering_frozen_error = false;
            rendered.unwrap_or_else(|_| "...".to_string())
        };
        let mut message = format!("can't modify frozen {}: {}", class_name, rendered);
        // A run told to report where a literal was written names the place
        // the refused string came from.
        if let Object::String(text) = receiver
            && let Some(written_at) = text.created_at()
        {
            message.push_str(&format!(", created at {}", written_at));
        }
        let exception = Object::exception("FrozenError", message.clone());
        if let Object::Exception(details) = &exception {
            details.borrow_mut().receiver = Some(Box::new(receiver.clone()));
        }
        MetorexError::UncaughtException {
            exception,
            location: position_to_location(position),
            message,
        }
    }

    /// Whether `receiver` is frozen. Immediates are always frozen; instances
    /// and classes/modules carry their own flag.
    pub(crate) fn object_is_frozen(&self, receiver: &Object) -> bool {
        match receiver {
            Object::Bool(_)
            | Object::Nil
            | Object::Int(_)
            | Object::BigInt(_)
            | Object::Float(_)
            | Object::Symbol(_) => true,
            // A string changes unless it has been frozen, so it carries the
            // flag itself rather than always answering yes.
            Object::String(text) => text.is_frozen(),
            // A range holds its ends and nothing else, and Ruby freezes every
            // one it builds. A copy made by `dup` is not frozen, and it is
            // told apart by the mark it carries. A subclass instance is an
            // ordinary object, so it is not reached here.
            Object::Range { mark, .. } => !self
                .thawed_ranges
                .contains_key(&(Rc::as_ptr(mark) as usize)),
            Object::Class(c) | Object::Module(c) => c.is_frozen(),
            Object::Array(_)
            | Object::Dict(_)
            | Object::Set(_)
            | Object::Method(_)
            | Object::Block(_)
            | Object::Binding(_)
            | Object::Regex(_, _) => Self::collection_address(receiver)
                .is_some_and(|address| self.frozen_collections.contains_key(&address)),
            // Complex and Rational are value objects, frozen from birth.
            Object::Instance(inst) => {
                inst.borrow().frozen || matches!(inst.borrow().class.name(), "Complex" | "Rational")
            }
            _ => false,
        }
    }
}
