// Constants registered against the file that defines them.

use super::*;

impl VirtualMachine {
    /// Registering a constant against the file that defines it.
    pub(crate) fn call_autoload_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // `autoload :CONST, "path"` — register the constant→path mapping.
        // `autoload?(:CONST, [inherit=true])` returns the registered path.
        if method_name == "autoload" {
            // `Kernel.autoload` registers where the caller sits, the same way
            // the bare form does, rather than on Kernel itself.
            if class_rc.name() == "Kernel"
                && let Some(definee) = self.autoload_definee()
                && !Rc::ptr_eq(&definee, class_rc)
            {
                return self
                    .call_class_methods(&definee, method_name, arguments, position)
                    .map(nested_answer);
            }
            let const_name = match arguments.first() {
                Some(Object::Symbol(s)) => s.as_str().to_string(),
                Some(Object::String(s)) => s.as_str().to_string(),
                _ => return Ok(Answered(Object::Nil)),
            };
            if !is_valid_constant_name(&const_name) {
                let msg = format!("autoload must be constant name: {}", const_name);
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
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
            let path = match arguments.get(1) {
                Some(Object::String(s)) => s.as_str().to_string(),
                Some(Object::Symbol(s)) => s.as_str().to_string(),
                Some(other) => {
                    let other_obj = other.clone();
                    if let Some((cls, method)) = self.lookup_method(&other_obj, "to_path") {
                        let result =
                            self.invoke_method(cls, method, other_obj, Vec::new(), position)?;
                        match result {
                            Object::String(s) => s.as_str().to_string(),
                            _ => {
                                let msg = "to_path must return a String".to_string();
                                let exc = Object::exception("TypeError", msg.clone());
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: msg,
                                });
                            }
                        }
                    } else {
                        let msg = format!(
                            "no implicit conversion of {} into String",
                            other.type_name()
                        );
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
                None => return Ok(Answered(Object::Nil)),
            };
            if path.is_empty() {
                let msg = "empty file name".to_string();
                let exc = Object::exception("ArgumentError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            let caller_file = self
                .get_current_file()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            class_rc.set_autoload_location(const_name.clone(), caller_file, position.line as i64);
            class_rc.set_autoload(const_name.clone(), path);
            self.trigger_const_added_hook(
                Object::Class(Rc::clone(class_rc)),
                &const_name,
                position,
            )?;
            return Ok(Answered(Object::Nil));
        }
        Ok(Unclaimed)
    }
    /// The path a constant was registered against, which `autoload?` answers.
    pub(crate) fn call_autoload_query_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        _position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        if method_name == "autoload?" {
            let const_name = match arguments.first() {
                Some(Object::Symbol(s)) => s.as_str().to_string(),
                Some(Object::String(s)) => s.as_str().to_string(),
                _ => return Ok(Answered(Object::Nil)),
            };
            let inherit = !matches!(arguments.get(1), Some(Object::Bool(false)));
            let class_for_autoload = Rc::clone(class_rc);
            let local_only_blocked = !inherit && class_rc.get_autoload(&const_name).is_none();
            let path = if local_only_blocked {
                None
            } else {
                self.effective_autoload(&class_for_autoload, &const_name)
            };
            return Ok(Answered(match path {
                Some(p) => Object::string(p),
                None => Object::Nil,
            }));
        }
        Ok(Unclaimed)
    }
}
