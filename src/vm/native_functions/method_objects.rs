// A method held as an object, and a constant registered against a file.

use super::*;

impl VirtualMachine {
    pub(crate) fn method_object(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // method(:name) returns a Method object for the given method name
        if arguments.len() != 1 {
            return Err(MetorexError::runtime_error(
                format!("method() expects 1 argument, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }

        // A name may be written as a Symbol or as a String, which is
        // what `method("hello")` passes.
        let method_name = match &arguments[0] {
            Object::Symbol(name) => name.as_str(),
            Object::String(name) => name.as_str(),
            _ => {
                return Err(MetorexError::runtime_error(
                    format!(
                        "method() expects a Symbol argument, got {}",
                        arguments[0].type_name()
                    ),
                    crate::vm::utils::position_to_location(position),
                ));
            }
        };

        // A `def` outside any class or module writes the method on
        // Object, which is where a bare name at the top level is
        // answered from and what it reports as its owner.
        let written_on_object = match self.globals().get("Object") {
            Some(object_class) => self.top_level_method(&object_class, &method_name),
            None => None,
        };
        // At the top level the program runs against `main`, which
        // is where a bare `method(:name)` looks when the scope binds
        // no `self` of its own.
        let here = self.eval_self(position).ok();
        // Look up the method in the current environment
        if let Some(obj) = self.environment().get(&method_name) {
            if let Object::Method(held) = &obj {
                if held.owner_class.is_none() {
                    if let Some(found) = written_on_object {
                        return Ok(bound_to(found, here.as_ref()));
                    }
                    // A load wrapped in a module writes what it
                    // defines there, which is what the receiver
                    // answers with rather than an unowned copy.
                    if let Some(receiver) = here.clone() {
                        let name = Object::symbol(method_name.to_string());
                        if let Ok(found) =
                            self.send_to_object(receiver, "method", vec![name], position)
                        {
                            return Ok(found);
                        }
                    }
                }
                return Ok(bound_to(obj, here.as_ref()));
            }
            // A name the environment holds as something other than a
            // method may still name one the receiver defines, which is
            // what `def p(a); end` does to the builtin of that name.
            let not_a_method = MetorexError::runtime_error(
                format!("'{}' is not a method", method_name),
                crate::vm::utils::position_to_location(position),
            );
            let Some(receiver) = here else {
                return Err(not_a_method);
            };
            let name = Object::symbol(method_name.to_string());
            self.send_to_object(receiver, "method", vec![name], position)
                .map_err(|_| not_a_method)
        } else if let Some(receiver) = here {
            // Inside an instance method a bare `method(:name)` means
            // `self.method(:name)`, and the name is not a local.
            let name = Object::symbol(method_name.to_string());
            self.send_to_object(receiver.clone(), "method", vec![name], position)
                .or_else(
                    |error| match self.top_level_method(&receiver, &method_name) {
                        Some(held) => Ok(held),
                        None => Err(error),
                    },
                )
        } else if let Some(found) = written_on_object {
            Ok(bound_to(found, here.as_ref()))
        } else {
            Err(MetorexError::runtime_error(
                format!("undefined method '{}'", method_name),
                crate::vm::utils::position_to_location(position),
            ))
        }
    }

    /// Bare `autoload` / `autoload?` register on the definee, which is
    /// Object at the top level and the enclosing module inside one.
    pub(crate) fn register_autoload(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Inside a method the definee is the module the method was
        // written in, which a module's instance method reaches
        // through the nesting it captured.
        let owner = match self.autoload_definee() {
            Some(enclosing) => enclosing,
            None => match self.globals().get("Object") {
                Some(Object::Class(object_class)) => object_class,
                _ => {
                    return Err(MetorexError::runtime_error(
                        "Object is not defined",
                        crate::vm::utils::position_to_location(position),
                    ));
                }
            },
        };
        self.call_class_methods(&owner, name, &arguments, position)
            .map(|result| result.unwrap_or(Object::Nil))
    }
}

/// A Method that carries no receiver, bound to `receiver`, which is the
/// object a top-level `method(:name)` was asked of.
fn bound_to(found: Object, receiver: Option<&Object>) -> Object {
    match (&found, receiver) {
        (Object::Method(method), Some(receiver)) if method.receiver.is_none() => {
            let mut bound = (**method).clone();
            bound.receiver = Some(Box::new(receiver.clone()));
            Object::Method(std::rc::Rc::new(bound))
        }
        _ => found,
    }
}
