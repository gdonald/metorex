// The odds and ends the functions above are written against.

use super::*;

impl VirtualMachine {
    /// Call one method on a receiver by name, which native code needs when
    /// the operation belongs to whatever type the program handed over.
    pub(crate) fn apply_named_method(
        &mut self,
        receiver: &Object,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some((class, method)) = self.lookup_method(receiver, name) else {
            return Err(bad_range_value(position));
        };
        self.invoke_method(class, method, receiver.clone(), arguments, position)
    }

    /// Apply `private` / `public` visibility modifier to top-level methods.
    /// At top level, method definitions target the Object class.
    pub(crate) fn apply_visibility_modifier(
        &mut self,
        modifier: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // No arguments: no-op (in Ruby this toggles subsequent-definition visibility).
        if arguments.is_empty() {
            return Ok(Object::Nil);
        }

        // A single array argument is unpacked.
        let flat: Vec<Object> = if arguments.len() == 1 {
            if let Object::Array(arr) = &arguments[0] {
                arr.borrow().clone()
            } else {
                arguments.clone()
            }
        } else {
            arguments.clone()
        };

        let Some(Object::Class(object_class)) = self.globals().get("Object") else {
            return Ok(Object::Nil);
        };

        let mut names: Vec<String> = Vec::with_capacity(flat.len());
        for arg in &flat {
            let n = match arg {
                Object::Symbol(s) => s.as_str().to_string(),
                Object::String(s) => s.as_str().to_string(),
                _ => {
                    let exc = Object::exception(
                        "TypeError",
                        format!("{} is not a symbol nor a string", arg),
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: crate::vm::utils::position_to_location(position),
                        message: format!("{} is not a symbol nor a string", arg),
                    });
                }
            };
            if object_class.find_method(&n).is_none() {
                let msg = format!("undefined method '{}' for class 'Object'", n);
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: crate::vm::utils::position_to_location(position),
                    message: msg,
                });
            }
            names.push(n);
        }

        for n in &names {
            if modifier == "private" {
                object_class.set_method_private(n.clone());
            } else {
                object_class.set_method_public(n);
            }
        }

        // Return first symbol argument (Ruby returns single sym or array for multi).
        match flat.len() {
            1 => Ok(Object::symbol(names[0].clone())),
            _ => Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                names.into_iter().map(Object::symbol).collect(),
            )))),
        }
    }
}

/// Whether a thrown tag names the same object a `catch` is holding. Ruby
/// matches by identity, so two equal Strings are different tags while a
/// Symbol is only ever itself.
pub(crate) fn throw_tags_match(live: &Object, thrown: &Object) -> bool {
    use std::rc::Rc;
    match (live, thrown) {
        (Object::String(a), Object::String(b)) => Rc::ptr_eq(a, b),
        (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
        (Object::Array(a), Object::Array(b)) => Rc::ptr_eq(a, b),
        (Object::Dict(a), Object::Dict(b)) => Rc::ptr_eq(a, b),
        (Object::Class(a), Object::Class(b)) => Rc::ptr_eq(a, b),
        (Object::Module(a), Object::Module(b)) => Rc::ptr_eq(a, b),
        (Object::Symbol(a), Object::Symbol(b)) => a == b,
        (Object::Int(a), Object::Int(b)) => a == b,
        (Object::Bool(a), Object::Bool(b)) => a == b,
        (Object::Nil, Object::Nil) => true,
        _ => false,
    }
}

/// The numeric value of a Range endpoint, when it has one.
pub(crate) fn numeric_value(object: &Object) -> Option<f64> {
    match object {
        Object::Int(value) => Some(*value as f64),
        Object::Float(value) => Some(*value),
        _ => None,
    }
}

/// A line with its trailing separator removed. With no separator given, a
/// trailing "\r\n", "\n", or "\r" goes, which is what `$/` names by default.
pub(crate) fn chomped(line: &str, separator: Option<&str>) -> String {
    match separator {
        Some(separator) if separator != "\n" => match line.strip_suffix(separator) {
            Some(rest) => rest.to_string(),
            None => line.to_string(),
        },
        _ => {
            for ending in ["\r\n", "\n", "\r"] {
                if let Some(rest) = line.strip_suffix(ending) {
                    return rest.to_string();
                }
            }
            line.to_string()
        }
    }
}

/// Libraries metorex provides itself, which `require` answers for without
/// looking for a file.
/// What the interpreter carries itself, which `$LOADED_FEATURES` lists from
/// the start and which a `require` of answers false.
pub(crate) const BUILT_IN_FEATURES: &[&str] = &[
    "complex",
    "enumerator",
    "fiber",
    "pathname",
    "prettyprint",
    "rational",
    "ruby2_keywords",
    "set",
    "stringio",
    "thread",
];
