// Class methods answered by the interpreter itself.

use super::*;

impl VirtualMachine {
    /// The class methods that reach straight into the interpreter: a socket
    /// address, a stream, the symbols the program has spelled.
    pub(crate) fn call_primitive_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        if let Some(result) =
            self.call_warning_methods(class_rc, method_name, arguments, position)?
        {
            return Ok(Answered(result));
        }
        // A refinement names the class it refines.
        if matches!(method_name, "target" | "refined_class")
            && let Some(found) = class_rc
                .get_class_var(crate::vm::native_methods::module_methods::REFINEMENT_TARGET_KEY)
        {
            return Ok(Answered(found));
        }
        // Every symbol the program has spelled, which the parser records as
        // it reads them and the constructor records as they are made.
        if class_rc.name() == "Symbol" && method_name == "all_symbols" {
            let named = crate::symbol_registry::all()
                .into_iter()
                .map(Object::symbol)
                .collect();
            return Ok(Answered(Object::array(named)));
        }
        // The shape of a network address belongs to the operating system, so
        // the socket library reads and writes them here.
        if class_rc.name() == "Socket" && method_name == "__address__" {
            return self.socket_address(arguments, position).map(Answered);
        }
        // The name the C library gives the encoding the locale calls for,
        // which differs between platforms for one and the same locale.
        if class_rc.name() == "Encoding" && method_name == "__charmap__" {
            return Ok(Answered(Object::string(crate::vm::locale_charmap_name())));
        }
        if class_rc.name() == "Socket" && method_name == "__net__" {
            return self.socket_net(arguments, position).map(Answered);
        }
        if class_rc.name() == "IO" && method_name == "__stream__" {
            return self.stream_action(arguments, position).map(Answered);
        }
        // `IO.__write_standard__(name, text)` writes through the
        // interpreter's own writer, which is what keeps a program's output in
        // the order it wrote it whichever route each piece took.
        if class_rc.name() == "IO" && method_name == "__write_standard__" {
            let named = match arguments.first() {
                Some(Object::String(held)) => held.as_str().to_string(),
                _ => String::new(),
            };
            let text = match arguments.get(1) {
                Some(Object::String(held)) => held.as_str().to_string(),
                other => other.map(|held| held.to_string()).unwrap_or_default(),
            };
            crate::vm::native_functions::write_to_standard_stream(&named, &text);
            return Ok(Answered(Object::Nil));
        }
        // A class that defines `new` of its own builds its instances that
        // way, rather than through the allocate-and-initialize every class
        // is given.
        if method_name == "new" && self.class_method_of(class_rc, "new").is_some() {
            return Ok(Deferred);
        }
        // `Integer.sqrt` and `Integer.try_convert` belong to the class rather
        // than to a number.
        if class_rc.name() == "Integer" && matches!(method_name, "sqrt" | "try_convert") {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            return self
                .integer_class_method(method_name, &arguments[0], position)
                .map(Answered);
        }
        // `String.try_convert` belongs to the class rather than to a string.
        // `Encoding.__source__` names the encoding the source running now is
        // written in, which is what `__ENCODING__` answers.
        if class_rc.name() == "Encoding"
            && method_name == "__running_source__"
            && arguments.is_empty()
        {
            let named = self
                .current_source_encoding
                .clone()
                .map(|held| {
                    crate::vm::native_methods::string_methods::canonical_encoding_name(&held)
                })
                .unwrap_or_else(|| crate::object::string_value::DEFAULT_ENCODING.to_string());
            let found = self.send_to_object(
                Object::Class(Rc::clone(class_rc)),
                "find",
                vec![Object::string(named)],
                position,
            )?;
            return Ok(Answered(found));
        }
        if class_rc.name() == "String" && method_name == "try_convert" {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            return self
                .string_try_convert(&arguments[0], position)
                .map(Answered);
        }
        Ok(Unclaimed)
    }
}
