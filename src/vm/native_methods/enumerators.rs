// The Enumerator a method answers when it is called without the
// block it walks with.

use super::*;

impl VirtualMachine {
    /// The Enumerator a method answers when it is called without the block it
    /// would have yielded to. It remembers the receiver and the call, so
    /// walking it runs the method with a block of the Enumerator's own.
    pub(crate) fn make_enumerator(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: crate::lexer::Position,
    ) -> Result<Object, crate::error::MetorexError> {
        let Some(enumerator @ Object::Class(_)) = self.globals().get("Enumerator") else {
            return Err(crate::error::MetorexError::runtime_error(
                "Enumerator is not defined",
                crate::vm::utils::position_to_location(position),
            ));
        };
        // The size an Enumerator reports without walking it, which for a
        // collection is how many elements it holds. A search reports none,
        // since how far it runs depends on what it finds.
        let counted = !matches!(method_name, "rindex" | "index" | "find_index");
        let size = match receiver {
            Object::Array(elements) if counted => Object::Int(elements.borrow().len() as i64),
            Object::Dict(entries) if counted => Object::Int(entries.borrow().len() as i64),
            _ => Object::Nil,
        };
        let call = vec![
            receiver.clone(),
            Object::symbol(method_name.to_string()),
            Object::array(arguments.to_vec()),
            size,
        ];
        self.send_to_object(enumerator, "over", call, position)
    }

    /// Whether an object answers to a name, asking it with `respond_to?` when
    /// it is one of the program's own, since its answer may be written rather
    /// than declared.
    pub(crate) fn answers_to(
        &mut self,
        held: &Object,
        name: &str,
        position: crate::lexer::Position,
    ) -> Result<bool, crate::error::MetorexError> {
        if self.responds_to(held, name) {
            return Ok(true);
        }
        if !matches!(held, Object::Instance(_)) {
            return Ok(false);
        }
        // Ruby asks after a private method too when it is looking for the
        // one that converts, so the second argument is passed. A
        // `respond_to?` written to take one argument alone is asked again
        // without it.
        let asked = self.send_to_object(
            held.clone(),
            "respond_to?",
            vec![Object::symbol(name.to_string()), Object::Bool(true)],
            position,
        );
        let answer = match asked {
            Ok(answer) => answer,
            Err(_) => self.send_to_object(
                held.clone(),
                "respond_to?",
                vec![Object::symbol(name.to_string())],
                position,
            )?,
        };
        Ok(answer.is_truthy())
    }
}

/// A string cut from another is in the same encoding, so an answer still
/// carrying the encoding a fresh literal gets is retagged with the
/// receiver's. A method that named an encoding of its own keeps it.
pub(crate) fn carry_string_encoding(
    receiver: &Object,
    method_name: &str,
    answer: Option<Object>,
) -> Option<Object> {
    // A method that pads with a string of its own works out which encoding
    // the two have in common, `encode` is named an encoding outright, and
    // `inspect` writes its answer in the encoding answers are written in, so
    // each is already tagged.
    if matches!(
        method_name,
        "center" | "ljust" | "rjust" | "+" | "encode" | "inspect"
    ) {
        return answer;
    }
    let Object::String(source) = receiver else {
        return answer;
    };
    let held = source.encoding_name();
    if held == crate::object::string_value::DEFAULT_ENCODING && !source.holds_bytes() {
        return answer;
    }
    let carry = |derived: &crate::object::StringValue| {
        if derived.encoding_name() != crate::object::string_value::DEFAULT_ENCODING
            || derived.holds_bytes()
        {
            return;
        }
        derived.set_encoding(held.clone());
        // Characters that stand for bytes go on standing for bytes in
        // whatever the method made out of them.
        if source.holds_bytes() {
            derived.mark_bytes();
        }
    };
    match &answer {
        Some(Object::String(derived)) => carry(derived),
        // The pieces a string was cut into read the way the whole one did.
        // A split around a separator hands that separator back as it was
        // written, so its own reading stands.
        Some(Object::Array(pieces)) if !matches!(method_name, "partition" | "rpartition") => {
            for piece in pieces.borrow().iter() {
                if let Object::String(derived) = piece {
                    carry(derived);
                }
            }
        }
        _ => {}
    }
    answer
}
