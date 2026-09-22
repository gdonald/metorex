// The encoding an argument or a name stands for.

use super::*;

impl VirtualMachine {
    /// The encoding an argument names, whether it arrives as an Encoding or
    /// as its name in text.
    pub(crate) fn encoding_name_argument(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match value {
            Object::Class(encoding) => Ok(encoding.name().to_string()),
            Object::String(named) => {
                let found = self.send_to_object(
                    self.globals().get("Encoding").unwrap_or(Object::Nil),
                    "find",
                    vec![Object::string(named.to_text())],
                    position,
                )?;
                match found {
                    Object::Class(encoding) => Ok(encoding.name().to_string()),
                    // A special name such as "internal" that nothing is set
                    // to names no encoding, and Ruby reads the bytes as
                    // bytes rather than refusing them.
                    Object::Nil => Ok("ASCII-8BIT".to_string()),
                    _ => Ok(named.as_str().to_string()),
                }
            }
            // Anything that reads as a String names an encoding by the
            // characters it reads as.
            other if self.responds_to(other, "to_str") => {
                let read = self.send_to_object(other.clone(), "to_str", vec![], position)?;
                match read {
                    Object::String(_) => self.encoding_name_argument(&read, position),
                    _ => Err(method_argument_type_error(
                        "force_encoding",
                        "String",
                        other,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                "force_encoding",
                "String",
                other,
                position,
            )),
        }
    }

    /// The Encoding object a name stands for, and UTF-8 where the name is one
    /// metorex does not carry a constant for.
    /// The one frozen string that stands for this text in this encoding.
    /// Every equal string deduplicates to it.
    pub(crate) fn deduped_string(&mut self, held: &crate::object::StringValue) -> Object {
        let key = (held.as_str().to_string(), held.encoding_name());
        if let Some(found) = self.deduped_strings.get(&key) {
            return Object::String(Rc::clone(found));
        }
        let made = crate::object::StringValue::with_encoding(held.to_text(), held.encoding_name());
        if held.holds_bytes() {
            made.mark_bytes();
        }
        // Every place writing the same literal shares one string, so the
        // place it is reported from is the first one that wrote it.
        if let Some(written_at) = held.created_at() {
            made.set_created_at(written_at);
        }
        made.mark_deduplicated();
        let made = Rc::new(made);
        self.deduped_strings.insert(key, Rc::clone(&made));
        Object::String(made)
    }

    pub(crate) fn encoding_object(&mut self, name: &str) -> Object {
        for (constant, display, _) in crate::vm::init::ENCODING_NAMES {
            if display == name
                && let Some(found) = self.globals().get(&format!("Encoding::{}", constant))
            {
                return found;
            }
        }
        self.globals().get("Encoding::UTF_8").unwrap_or(Object::Nil)
    }

    /// The encoding `inspect` writes its answer in: the one a program named
    /// as the internal or the external encoding, or US-ASCII when that one
    /// spells ASCII differently from ASCII itself.
    pub(crate) fn inspect_result_encoding(&mut self) -> String {
        let named = match self.globals().get("__Encoding_default_internal") {
            Some(Object::Class(held)) => held.name().to_string(),
            _ => match self.globals().get("__Encoding_default_external") {
                Some(Object::Class(held)) => held.name().to_string(),
                _ => crate::object::string_value::DEFAULT_ENCODING.to_string(),
            },
        };
        if encoding_is_ascii_compatible(&named) {
            named
        } else {
            "US-ASCII".to_string()
        }
    }
}
