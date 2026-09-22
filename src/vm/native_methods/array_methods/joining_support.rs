// Building the text a `join` answers.

use super::*;

impl VirtualMachine {
    /// The pieces an array joins from, each with the encoding it is written
    /// in. An element that names an array of its own contributes its own
    /// pieces however deeply they nest, and any other element is asked for
    /// `to_str`, then `to_ary`, then `to_s`, which is the order Ruby tries.
    /// An array that reaches itself is refused.
    pub(crate) fn joined_parts(
        &mut self,
        elements: &[Object],
        in_flight: &mut Vec<usize>,
        position: Position,
    ) -> Result<Vec<(String, String)>, MetorexError> {
        let mut written = Vec::with_capacity(elements.len());
        for element in elements {
            self.join_element_into(element, in_flight, &mut written, position)?;
        }
        Ok(written)
    }

    /// One element's contribution to a join, added to what is written so far.
    pub(crate) fn join_element_into(
        &mut self,
        element: &Object,
        in_flight: &mut Vec<usize>,
        written: &mut Vec<(String, String)>,
        position: Position,
    ) -> Result<(), MetorexError> {
        match element {
            Object::Symbol(name) | Object::String(name) => {
                written.push((name.as_str().to_string(), name.encoding_name()));
                return Ok(());
            }
            _ => {}
        }
        let nested = match element {
            Object::Array(held) => Some(Rc::clone(held)),
            other => match crate::vm::native_methods::array_subclass_value(other) {
                Some(Object::Array(held)) => Some(held),
                _ => None,
            },
        };
        if let Some(held) = nested {
            let address = Rc::as_ptr(&held) as usize;
            if in_flight.contains(&address) {
                return Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "recursive array join",
                    position,
                ));
            }
            in_flight.push(address);
            let inner = held.borrow().clone();
            let joined = self.joined_parts(&inner, in_flight, position);
            in_flight.pop();
            written.extend(joined?);
            return Ok(());
        }
        for name in ["to_str", "to_ary", "to_s"] {
            if !self.responds_to(element, name) {
                continue;
            }
            let answered = self.send_to_object(element.clone(), name, vec![], position)?;
            // Only an array answered by `to_ary` joins as one. Anything else
            // it answers leaves the element for `to_s` to spell.
            if name == "to_ary" {
                if matches!(&answered, Object::Array(_))
                    || crate::vm::native_methods::array_subclass_value(&answered).is_some()
                {
                    return self.join_element_into(&answered, in_flight, written, position);
                }
                continue;
            }
            if let Object::String(text) = answered {
                written.push((text.as_str().to_string(), text.encoding_name()));
                return Ok(());
            }
        }
        Err(crate::vm::errors::simple_exception(
            "NoMethodError",
            &format!(
                "undefined method 'to_str' for an instance of {}",
                self.builtins().class_of(element).name()
            ),
            position,
        ))
    }

    /// The encoding a join writes its answer in: the one the first piece is
    /// written in, widened by the first piece that is not all ASCII. Two
    /// pieces that are each written in an encoding of their own and neither
    /// of which is ASCII cannot be joined at all.
    pub(crate) fn joined_encoding(
        &mut self,
        parts: &[(String, String)],
        position: Position,
    ) -> Result<String, MetorexError> {
        let mut writing = "US-ASCII".to_string();
        let mut all_ascii = true;
        for (at, (text, held)) in parts.iter().enumerate() {
            if at == 0 {
                writing = held.clone();
                all_ascii = text.is_ascii();
                continue;
            }
            if text.is_ascii() {
                continue;
            }
            if all_ascii {
                writing = held.clone();
                all_ascii = false;
                continue;
            }
            if held != &writing {
                let message = format!("incompatible character encodings: {} and {}", writing, held);
                return Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &message,
                    position,
                ));
            }
        }
        Ok(writing)
    }
}
