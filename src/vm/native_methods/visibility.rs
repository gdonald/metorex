use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn apply_class_visibility_modifier(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        modifier: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // With no arguments the modifier is a toggle: the methods defined
        // after it in this body take the given visibility.
        if arguments.is_empty() {
            class_rc.set_current_visibility(modifier);
            return Ok(Object::Nil);
        }
        let flat: Vec<Object> = if arguments.len() == 1 {
            if let Object::Array(arr) = &arguments[0] {
                arr.borrow().clone()
            } else {
                arguments.to_vec()
            }
        } else {
            arguments.to_vec()
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
                        location: position_to_location(position),
                        message: format!("{} is not a symbol nor a string", arg),
                    });
                }
            };
            // A visibility declaration for a method the receiver only
            // inherits gives the receiver its own entry, and that new entry
            // fires `method_added`. Kernel is reached through the Object
            // fallback, since a method defined at the top level lives on
            // Object rather than anywhere in Kernel's ancestry.
            // Marking a method that the receiver only inherits, and that
            // already has the visibility being asked for, changes nothing:
            // Ruby does not copy the method down in that case.
            let already_set = match modifier {
                "private" => {
                    class_rc.find_method(&n).is_some() && self.inherits_private(class_rc, &n)
                }
                "protected" => {
                    class_rc.find_method(&n).is_some() && self.inherits_protected(class_rc, &n)
                }
                _ => false,
            };
            if !already_set && class_rc.find_own_method(&n).is_none() {
                let inherited =
                    class_rc
                        .find_method(&n)
                        .or_else(|| match self.globals().get("Object") {
                            Some(Object::Class(object_class)) => object_class.find_method(&n),
                            _ => None,
                        });
                let Some(method) = inherited else {
                    let msg = format!("undefined method '{}' for class '{}'", n, class_rc.name());
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                };
                class_rc.define_method(&n, method);
                self.invoke_class_hook(class_rc, "method_added", &n, position)?;
            }
            names.push(n);
        }
        for n in &names {
            match modifier {
                "private" => class_rc.set_method_private(n.clone()),
                "protected" => class_rc.set_method_protected(n.clone()),
                _ => class_rc.set_method_public(n),
            }
        }
        match flat.len() {
            1 => Ok(Object::Symbol(Rc::new(names[0].clone()))),
            _ => Ok(Object::Array(Rc::new(std::cell::RefCell::new(
                names
                    .into_iter()
                    .map(|n| Object::Symbol(Rc::new(n)))
                    .collect(),
            )))),
        }
    }
}
