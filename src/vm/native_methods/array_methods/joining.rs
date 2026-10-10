// Joining an array to itself or to text, and flattening one out.

use super::*;

impl VirtualMachine {
    /// Joining an array to itself or to text, and flattening one out.
    pub(crate) fn call_array_joining_method(
        &mut self,
        receiver: &Object,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `ary * other` joins with a separator when the other object
            // names one, and repeats the array when it names a count. Ruby
            // asks for the separator first.
            "*" => {
                if arguments.len() != 1 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Exact(1),
                        arguments.len(),
                        position,
                    ));
                }
                let given = arguments[0].clone();
                if matches!(given, Object::String(_)) {
                    return self.call_array_method(receiver, "join", &[given], position);
                }
                if self.responds_to(&given, "to_str") {
                    let separator = self.send_to_object(given, "to_str", vec![], position)?;
                    return self.call_array_method(receiver, "join", &[separator], position);
                }
                let count = match given {
                    Object::Int(_) => given,
                    other if self.responds_to(&other, "to_int") => {
                        self.send_to_object(other, "to_int", vec![], position)?
                    }
                    other => {
                        return Err(crate::vm::errors::integer_conversion_error(
                            &other, position,
                        ));
                    }
                };
                let Object::Int(count) = count else {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "can't convert to Integer",
                        position,
                    ));
                };
                let Ok(count) = usize::try_from(count) else {
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        "negative argument",
                        position,
                    ));
                };
                let source = array_rc.borrow().clone();
                let mut repeated = Vec::with_capacity(source.len() * count);
                for _ in 0..count {
                    repeated.extend(source.iter().cloned());
                }
                Ok(Some(Object::array(repeated)))
            }
            "join" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // With no separator, or with nil for one, `$,` says what
                // goes between. Reading it warns, since a later release will
                // drop it.
                let default_separator = |vm: &mut Self| -> String {
                    match vm.globals().get(",") {
                        Some(Object::String(held)) => {
                            let text = held.as_str().to_string();
                            vm.emit_warning_to_stderr(
                                "warning: $, is set to non-nil value",
                                position,
                            );
                            text
                        }
                        _ => String::new(),
                    }
                };
                let sep = if arguments.is_empty() || matches!(arguments[0], Object::Nil) {
                    default_separator(self)
                } else {
                    // An empty array never asks the separator for anything,
                    // which is what Ruby does.
                    let given = if array_rc.borrow().is_empty() {
                        Object::Nil
                    } else if matches!(arguments[0], Object::String(_) | Object::Nil) {
                        arguments[0].clone()
                    } else if self.responds_to(&arguments[0], "to_str") {
                        self.send_to_object(arguments[0].clone(), "to_str", vec![], position)?
                    } else {
                        arguments[0].clone()
                    };
                    match &given {
                        Object::String(s) => s.as_str().to_string(),
                        // A nil separator joins with nothing between.
                        Object::Nil => String::new(),
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                &arguments[0],
                                position,
                            ));
                        }
                    }
                };
                // Each element joins as its `to_s`, so a Symbol contributes
                // the name it is spelled with rather than the leading colon.
                // An element that is itself an array is joined with the same
                // separator, however deeply they nest.
                let elements = array_rc.borrow().clone();
                let mut in_flight = vec![Rc::as_ptr(array_rc) as usize];
                let parts = self.joined_parts(&elements, &mut in_flight, position)?;
                let writing = self.joined_encoding(&parts, position)?;
                let text = parts
                    .iter()
                    .map(|(written, _)| written.as_str())
                    .collect::<Vec<&str>>()
                    .join(&sep);
                let made = crate::object::StringValue::with_encoding(text, writing.clone());
                // A run of bytes joined with others is still a run of bytes,
                // so the result says so rather than reading them as
                // characters.
                if matches!(writing.as_str(), "ASCII-8BIT" | "BINARY") {
                    made.mark_bytes();
                }
                Ok(Some(Object::String(Rc::new(made))))
            }
            // `flatten` walks all the way down by default, or as many levels
            // as the argument names. `flatten!` writes the result back and
            // answers nil when there was nothing nested to flatten.
            "flatten" | "flatten!" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let depth = match arguments.first() {
                    None | Some(Object::Nil) => -1,
                    Some(Object::Int(level)) => *level,
                    Some(other) => self
                        .coerce_integer_argument(other, position)?
                        .try_into()
                        .unwrap_or(-1),
                };
                let elements = array_rc.borrow().clone();
                let mut flat = Vec::new();
                let mut in_flight = vec![Rc::as_ptr(array_rc) as usize];
                self.flatten_into(&elements, depth, &mut in_flight, &mut flat, position)?;
                if method_name == "flatten" {
                    return Ok(Some(Object::array(flat)));
                }
                // Told to flatten nothing, the array is left as it was, which
                // is what `flatten!` reports by answering nil.
                let changed = depth != 0
                    && (flat.len() != elements.len()
                        || elements
                            .iter()
                            .any(|element| matches!(element, Object::Array(_))));
                *array_rc.borrow_mut() = flat;
                Ok(Some(if changed {
                    receiver.clone()
                } else {
                    Object::Nil
                }))
            }
            "compact" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let array = array_rc.borrow();
                let compacted: Vec<Object> = array
                    .iter()
                    .filter(|obj| !matches!(obj, Object::Nil))
                    .cloned()
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(compacted)))))
            }
            _ => Ok(None),
        }
    }
}
