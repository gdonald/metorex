// The class an object alone has, and the modules mixed into it.

use super::*;

impl VirtualMachine {
    /// The class an object alone has, and the modules mixed into it.
    pub(crate) fn call_object_singleton_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "singleton_class" => {
                self.warn_chilled_string(receiver, position);
                if let Some(sole) = match receiver {
                    Object::Nil => Some("NilClass"),
                    Object::Bool(true) => Some("TrueClass"),
                    Object::Bool(false) => Some("FalseClass"),
                    _ => None,
                } && let Some(class @ Object::Class(_)) = self.globals().get(sole)
                {
                    return Ok(Some(class));
                }
                // One of the interned strings `String#-@` hands back stands
                // for every use of that text, so it has no singleton class.
                if refuses_a_singleton(receiver) {
                    let msg = "can't define singleton".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let singleton = self.singleton_class_of(receiver);
                // A frozen object's singleton class is frozen too, so no
                // method can be added to it afterwards. Only an instance
                // carries per-object frozen state; the immediates share one
                // singleton class per type, which must stay writable.
                if matches!(receiver, Object::Instance(_)) && self.object_is_frozen(receiver) {
                    singleton.freeze();
                }
                Ok(Some(Object::Class(singleton)))
            }
            // Object#extend(mod) — add `mod` as a mixin on the receiver's
            // singleton class so the module's instance methods become callable
            // on this specific object. (Class/Module already have their own
            // dedicated `extend` in class_methods.rs that hits earlier.)
            "extend" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        "extend",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                for arg in arguments {
                    // Ruby takes a module here and rejects a class.
                    let module_rc = match arg {
                        Object::Module(m) => std::rc::Rc::clone(m),
                        other => {
                            return Err(method_argument_type_error(
                                "extend", "Module", other, position,
                            ));
                        }
                    };
                    self.apply_module_extend(receiver, &module_rc, position)?;
                }
                Ok(Some(receiver.clone()))
            }
            "singleton_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "no implicit conversion into String".to_string(),
                            position_to_location(position),
                        ));
                    }
                };
                // Only the singleton layer counts: a method the object's
                // class defines is not a singleton method of the object.
                if let Some((owner, found)) = self.singleton_layer_method(receiver, &method) {
                    let mut bound = (*found).clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    if bound.owner_class.is_none() {
                        bound.owner_class = Some(owner);
                    }
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
                let msg = format!("undefined singleton method '{}' for {}", method, receiver);
                let exc = Object::exception("NameError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            _ => Ok(None),
        }
    }
}
