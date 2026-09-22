// The class an object alone has.

use super::*;

impl VirtualMachine {
    /// The Symbol a String names. The symbol keeps the encoding the string
    /// was in, and a run of bytes that spells no characters in that encoding
    /// names no symbol at all.
    pub(crate) fn interned_symbol(
        &mut self,
        text: &std::rc::Rc<crate::object::StringValue>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !crate::vm::native_methods::string_methods::holds_valid_text(text) {
            let spelled = Object::String(std::rc::Rc::clone(text));
            let written = self.send_to_object(spelled, "inspect", vec![], position)?;
            let message = format!(
                "invalid symbol in encoding {} :{}",
                text.encoding_name(),
                written
            );
            return Err(crate::vm::errors::simple_exception(
                "EncodingError",
                &message,
                position,
            ));
        }
        crate::symbol_registry::record(&text.as_str());
        let named = crate::object::StringValue::with_encoding(text.to_text(), text.encoding_name());
        if text.holds_bytes() {
            named.mark_bytes();
        }
        Ok(Object::Symbol(std::rc::Rc::new(named)))
    }
}

impl VirtualMachine {
    /// Give the notice a string carries that it will be frozen in a later
    /// release, for the operations that count as changing it. The notice is
    /// given once, the way Ruby gives it.
    pub(crate) fn warn_chilled_string(&mut self, receiver: &Object, position: Position) {
        let Object::String(text) = receiver else {
            return;
        };
        let Some(notice) = text.take_chill() else {
            return;
        };
        if self.warning_category_enabled("deprecated") {
            self.emit_warning_to_stderr(&notice, position);
            self.report_string_birthplace(text, position);
        }
    }

    /// Name where a literal was written, after the notice that a change to it
    /// will be refused in a later release. Only a run told to record the
    /// place has one to name.
    pub(crate) fn report_string_birthplace(
        &mut self,
        text: &crate::object::StringValue,
        position: Position,
    ) {
        let Some(written_at) = text.created_at() else {
            return;
        };
        let message = format!("{}: info: the string was created here", written_at);
        self.emit_warning_to_stderr(&message, position);
    }
}

/// Whether the object stands for a value rather than holding one of its own,
/// which is what keeps it from carrying a singleton class. Every use of the
/// same number, symbol, or interned string names the same object, so a method
/// written on one would be written on all of them.
pub(crate) fn refuses_a_singleton(receiver: &Object) -> bool {
    matches!(receiver, Object::String(held) if held.is_deduplicated())
        || matches!(
            receiver,
            Object::Int(_) | Object::BigInt(_) | Object::Float(_) | Object::Symbol(_)
        )
}

impl VirtualMachine {
    /// Whether the receiver is the object a program runs against at the top
    /// level, which Ruby calls `main`.
    pub(crate) fn is_the_main_object(&self, receiver: &Object) -> bool {
        let Object::Instance(held) = receiver else {
            return false;
        };
        // A wrapped load runs against a main of its own, which stands for
        // main the way the program's own does.
        if held
            .borrow()
            .get_var(crate::vm::loading::MAIN_STAND_IN)
            .is_some()
        {
            return true;
        }
        match self.globals().get("__main__") {
            Some(Object::Instance(main)) => std::rc::Rc::ptr_eq(held, &main),
            _ => false,
        }
    }
}
