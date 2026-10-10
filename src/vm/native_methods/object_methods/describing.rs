// What an object reads back as, and the class it belongs to.

use super::*;

impl VirtualMachine {
    /// What an object reads back as, and the class it belongs to.
    pub(crate) fn call_object_describing_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "to_s" | "inspect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // A String already is one, so `to_s` answers the receiver
                // itself and the encoding it is tagged with travels with it.
                if method_name == "to_s" && matches!(receiver, Object::String(_)) {
                    return Ok(Some(receiver.clone()));
                }
                // `true` and `false` each write themselves as one string
                // that never changes, so asking twice answers the same
                // object.
                if method_name == "to_s"
                    && let Some((slot, text)) = match receiver {
                        Object::Bool(true) => Some(("__true_to_s", "true")),
                        Object::Bool(false) => Some(("__false_to_s", "false")),
                        _ => None,
                    }
                {
                    return Ok(Some(self.memoized_text(slot, text)));
                }
                if let Object::Symbol(s) = receiver {
                    // A name past ASCII in an encoding other than the one the
                    // answer is written in cannot be shown as it is, so it is
                    // quoted and escaped the way the String would be.
                    let named = s.encoding_name();
                    if method_name != "to_s"
                        && !matches!(named.as_str(), "UTF-8" | "US-ASCII" | "ASCII-8BIT")
                        && crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(
                            &named,
                        )
                        && !crate::vm::native_methods::string_methods::binary_bytes(s).is_ascii()
                    {
                        let made = crate::object::StringValue::with_encoding(s.to_text(), named);
                        if s.holds_bytes() {
                            made.mark_bytes();
                        }
                        let text = Object::String(std::rc::Rc::new(made));
                        let shown = self.send_to_object(text, "inspect", Vec::new(), position)?;
                        let shown = match shown {
                            Object::String(held) => held.as_str().to_string(),
                            _ => String::new(),
                        };
                        return Ok(Some(Object::string(format!(":{shown}"))));
                    }
                    if method_name != "to_s" {
                        return Ok(Some(Object::string(
                            crate::vm::native_methods::string_methods::symbol_inspect_text(s),
                        )));
                    }
                    // The name keeps the encoding it was interned in, bytes
                    // and all, which is what `Symbol#encoding` reports.
                    let made =
                        crate::object::StringValue::with_encoding(s.to_text(), s.encoding_name());
                    if s.holds_bytes() {
                        made.mark_bytes();
                    }
                    return Ok(Some(Object::String(std::rc::Rc::new(made))));
                }
                if method_name == "inspect"
                    && let Object::Instance(_) = receiver
                {
                    return self.default_instance_inspect(receiver, position).map(Some);
                }
                // A callable describes itself in bytes rather than in text,
                // which is what Ruby tags the description with.
                if matches!(receiver, Object::Block(_)) {
                    return Ok(Some(Object::String(std::rc::Rc::new(
                        crate::object::StringValue::with_encoding(
                            receiver.to_string(),
                            "ASCII-8BIT",
                        ),
                    ))));
                }
                Ok(Some(Object::string(receiver.to_string())))
            }
            "class" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // For exceptions, return the specific exception class, not generic Exception
                if let Object::Exception(exc_ref) = receiver {
                    if let Some(class) = exc_ref.borrow().class.clone() {
                        return Ok(Some(Object::Class(class)));
                    }
                    let exc_type = exc_ref.borrow().exception_type.clone();
                    if let Some(Object::Class(cls)) = self.globals().get(&exc_type) {
                        return Ok(Some(Object::Class(cls)));
                    }
                    // A namespaced type such as `Errno::EINVAL` lives as a
                    // constant on its module rather than at the top level.
                    if let Some(class @ Object::Class(_)) =
                        self.resolve_qualified_constant(&exc_type)
                    {
                        return Ok(Some(class));
                    }
                }
                // A class is an instance of Class, a module an instance of
                // Module. `class_of` answers Object for both because it drives
                // `is_a?` over the inheritance chain.
                // An encoding is held as a class whose parent is Encoding,
                // and what it stands for is an Encoding rather than a class.
                if self.names_an_encoding(receiver)
                    && let Some(held @ Object::Class(_)) = self.globals().get("Encoding")
                {
                    return Ok(Some(held));
                }
                match receiver {
                    Object::Class(_) => {
                        if let Some(Object::Class(cls)) = self.globals().get("Class") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    Object::Module(_) => {
                        if let Some(Object::Class(cls)) = self.globals().get("Module") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    _ => {}
                }
                // For booleans and nil, return the specific class
                match receiver {
                    Object::Bool(true) => {
                        if let Some(Object::Class(cls)) = self.globals().get("TrueClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    Object::Bool(false) => {
                        if let Some(Object::Class(cls)) = self.globals().get("FalseClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    Object::Nil => {
                        if let Some(Object::Class(cls)) = self.globals().get("NilClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    _ => {}
                }
                Ok(Some(Object::Class(self.builtins().class_of(receiver))))
            }
            _ => Ok(None),
        }
    }
}
