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
        if arguments.is_empty() {
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
            if class_rc.find_method(&n).is_none() {
                // Fall back to Object's method table — some specs toggle
                // visibility on Kernel for methods defined at the top level
                // (which live on Object in Ruby semantics).
                let on_object = matches!(
                    self.globals().get("Object"),
                    Some(Object::Class(oc)) if oc.find_method(&n).is_some()
                );
                if !on_object {
                    let msg = format!("undefined method '{}' for class '{}'", n, class_rc.name());
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
            }
            names.push(n);
        }
        for n in &names {
            if modifier == "private" {
                class_rc.set_method_private(n.clone());
            } else {
                class_rc.set_method_public(n);
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
