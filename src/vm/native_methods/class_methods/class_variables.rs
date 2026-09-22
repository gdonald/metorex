// Class variables, and the flags a method carries.

use super::*;

impl VirtualMachine {
    /// The class variables a module holds, and the flags a method carries.
    pub(crate) fn call_class_variable_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        match method_name {
            "class_variable_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "class_variable_set",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                if class_rc.is_frozen() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let key = self.coerce_class_variable_name(
                    &arguments[0],
                    &Object::Class(Rc::clone(class_rc)),
                    position,
                )?;
                // A write reaches the ancestor furthest up the chain that
                // already holds the name, which is the one every class below
                // it reads.
                crate::vm::core::VirtualMachine::class_var_owner(class_rc, &key)
                    .set_class_var(key, arguments[1].clone());
                return Ok(Answered(arguments[1].clone()));
            }
            "class_variable_get" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "class_variable_get",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(
                    &arguments[0],
                    &Object::Class(Rc::clone(class_rc)),
                    position,
                )?;
                // A class variable an ancestor defines after a class below
                // it already had one is ambiguous, so reading it is refused.
                if let Some(overtaken) = overtaking_ancestor(class_rc, &key) {
                    let msg = format!(
                        "class variable @@{} of {} is overtaken by {}",
                        key,
                        class_rc.inspect_name(),
                        overtaken.inspect_name()
                    );
                    return Err(MetorexError::runtime_error(
                        msg,
                        position_to_location(position),
                    ));
                }
                match class_rc.lookup_class_var(&key) {
                    Some(value) => return Ok(Answered(value)),
                    None => {
                        let msg = format!(
                            "uninitialized class variable @@{} in {}",
                            key,
                            class_rc.name()
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
            }
            // Module#remove_class_variable: only a variable defined directly
            // on the receiver can be removed, and its value comes back.
            "remove_class_variable" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(
                    &arguments[0],
                    &Object::Class(Rc::clone(class_rc)),
                    position,
                )?;
                match class_rc.remove_class_var(&key) {
                    Some(value) => return Ok(Answered(value)),
                    None => {
                        let msg = format!(
                            "class variable @@{} not defined for {}",
                            key,
                            class_rc.ruby_name()
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
            }
            "class_variable_defined?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "class_variable_defined?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(
                    &arguments[0],
                    &Object::Class(Rc::clone(class_rc)),
                    position,
                )?;
                return Ok(Answered(Object::Bool(
                    class_rc.lookup_class_var(&key).is_some(),
                )));
            }
            "class_variables" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        "class_variables",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // `class_variables(inherit = true)` — when inherit is false,
                // only this class/module's own class variables are reported.
                let inherit = arguments.first().map(is_truthy).unwrap_or(true);
                let names = if inherit {
                    class_rc.inherited_class_variable_names()
                } else {
                    class_rc.own_class_variable_names()
                };
                let symbols: Vec<Object> = names
                    .into_iter()
                    .map(|n| Object::symbol(format!("@@{}", n)))
                    .collect();
                return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                    symbols,
                )))));
            }
            // Module methods we treat as no-ops (Metorex doesn't track these
            // concepts, but class bodies that use them still need to load).
            "deprecate_constant" => {
                return Ok(Answered(Object::Nil));
            }
            // `ruby2_keywords :name` only applies to a method whose last
            // parameter is a bare `*args` splat. Anything else keeps its
            // signature and gets a warning; a name with no method raises.
            "ruby2_keywords" => {
                for argument in arguments {
                    if !matches!(argument, Object::Symbol(_) | Object::String(_)) {
                        let shown =
                            self.send_to_object(argument.clone(), "inspect", vec![], position)?;
                        let message = format!("{} is not a symbol nor a string", shown);
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    }
                    let name = self.coerce_name_argument(argument, position)?;
                    let Some(method) = class_rc.find_method(&name) else {
                        let message = format!(
                            "undefined method '{}' for class '{}'",
                            name,
                            class_rc.ruby_name()
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("NameError", message.clone()),
                            location: position_to_location(position),
                            message,
                        });
                    };
                    let takes_bare_splat = method
                        .variadic_param
                        .as_ref()
                        .is_some_and(|(index, _)| *index + 1 == method.parameters.len());
                    let takes_keywords = !method.keyword_parameters.is_empty()
                        || method.keyword_rest_parameter.is_some();
                    if !takes_bare_splat || takes_keywords {
                        self.emit_warning_to_stderr(
                            &format!(
                                "Skipping set of ruby2_keywords flag for {} (method accepts keywords or method does not accept argument splat)",
                                name
                            ),
                            position,
                        );
                        continue;
                    }
                    // The flag is shared with every copy of the method, so an
                    // alias made before or after this call carries it too.
                    method.ruby2_keywords.set(true);
                }
                return Ok(Answered(Object::Nil));
            }
            _ => {}
        }
        Ok(Unclaimed)
    }
}
