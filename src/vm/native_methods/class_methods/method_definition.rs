// Defining and removing methods on a module.

use super::*;

impl VirtualMachine {
    /// Defining, removing, aliasing and undefining the methods of a module.
    pub(crate) fn call_method_definition_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        match method_name {
            "class_eval" | "module_eval" => {
                let result = self.class_eval_with_args(
                    class_rc,
                    Object::Class(Rc::clone(class_rc)),
                    arguments,
                    position,
                )?;
                return Ok(Answered(result));
            }
            "class_exec" | "module_exec" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    _ => return Err(local_jump_error(method_name, position)),
                };
                let result = self.class_exec_block(
                    class_rc,
                    Object::Class(Rc::clone(class_rc)),
                    &block,
                    arguments.to_vec(),
                    position,
                )?;
                return Ok(Answered(result));
            }
            "define_method" => {
                return self
                    .module_define_method(class_rc, arguments, position)
                    .map(Answered);
            }
            "remove_method" => {
                // Ruby accepts any number of names, including none, and
                // answers with the receiver.
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    names.push(self.coerce_method_name(argument, method_name, position)?);
                }
                if class_rc.is_frozen() && !names.is_empty() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.ruby_name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                for name in names {
                    // A method put on a class by `def Klass.name` lives under
                    // the `__class__` name on the class itself, so a
                    // `class << Klass` body has to reach it there.
                    let removed = class_rc.remove_method(&name)
                        || self.attached_class_of(class_rc).is_some_and(|attached| {
                            attached.remove_method(&format!("__class__{}", name))
                        });
                    // A name the interpreter answers natively has no entry
                    // in any method table, so removing it leaves a tombstone
                    // that the lookup reads as undefined.
                    let removed = removed
                        || (crate::vm::native_methods::is_native_kernel_method(&name)
                            || name == "method_missing")
                            && {
                                let sentinel = Method::undefined(name.clone());
                                class_rc.define_method(&name, Rc::new(sentinel));
                                true
                            };
                    if !removed {
                        let msg =
                            format!("method '{}' not defined in {}", name, class_rc.ruby_name());
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    let hook = if class_rc.get_class_var("__singleton__").is_some() {
                        "singleton_method_removed"
                    } else {
                        "method_removed"
                    };
                    self.invoke_class_hook(class_rc, hook, &name, position)?;
                }
                return Ok(Answered(if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                }));
            }
            "undef_method" => {
                // Like `remove_method`, this takes any number of names and
                // answers with the receiver. The method must exist somewhere
                // in the ancestry, though it need not be the receiver's own.
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    names.push(self.coerce_method_name(argument, method_name, position)?);
                }
                if class_rc.is_frozen() && !names.is_empty() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.ruby_name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                for name in names {
                    // A method put on a class by `def Klass.name` lives under
                    // the `__class__` name on the class itself, so a
                    // `class << Klass` body has to reach it there.
                    // `new` and `allocate` are answered natively for every
                    // class, so a singleton class holds them too.
                    let class_method_owner = self.attached_class_of(class_rc).filter(|attached| {
                        matches!(name.as_str(), "new" | "allocate")
                            || attached
                                .find_method(&format!("__class__{}", name))
                                .is_some_and(|method| !method.is_undefined)
                    });
                    // Kernel methods live in the native dispatch tables rather
                    // than in a class's method map, so they count as present.
                    if class_method_owner.is_none()
                        && class_rc
                            .find_method(&name)
                            .is_none_or(|method| method.is_undefined)
                        && !is_native_kernel_method(&name)
                        // The default hooks are native no-ops rather than
                        // table entries, so they count as present too.
                        && !MODULE_PRIVATE_HOOKS.contains(&name.as_str())
                        && !BASIC_OBJECT_PRIVATE_METHODS.contains(&name.as_str())
                        // An exception answers its own methods natively, so a
                        // class holding exceptions counts them as present.
                        && !answers_exception_method(self, class_rc, &name)
                    {
                        let msg = format!(
                            "undefined method '{}' for {} '{}'",
                            name,
                            class_rc.kind_name().to_lowercase(),
                            undef_target_name(class_rc)
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        if let Object::Exception(cell) = &exc {
                            cell.borrow_mut().name = Some(name.clone());
                        }
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    let sentinel = Method::undefined(name.clone());
                    match &class_method_owner {
                        // The sentinel has to sit where the lookup will find
                        // it, which for a class method is the `__class__` name
                        // on the attached class.
                        Some(attached) => {
                            attached.define_method(format!("__class__{}", name), Rc::new(sentinel))
                        }
                        None => class_rc.define_method(&name, Rc::new(sentinel)),
                    }
                    let hook = if class_rc.get_class_var("__singleton__").is_some() {
                        "singleton_method_undefined"
                    } else {
                        "method_undefined"
                    };
                    self.invoke_class_hook(class_rc, hook, &name, position)?;
                }
                return Ok(Answered(if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                }));
            }
            "alias_method" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "alias_method",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let new_name = self.coerce_method_name(&arguments[0], "alias_method", position)?;
                let old_name = self.coerce_method_name(&arguments[1], "alias_method", position)?;
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
                self.install_alias(class_rc, &new_name, &old_name, position)?;
                if matches!(
                    new_name.as_str(),
                    "initialize"
                        | "initialize_copy"
                        | "initialize_clone"
                        | "initialize_dup"
                        | "respond_to_missing?"
                ) {
                    class_rc.set_method_private(new_name.clone());
                }
                self.invoke_class_hook(class_rc, "method_added", &new_name, position)?;
                return Ok(Answered(Object::symbol(new_name)));
            }
            "module_function" => {
                // Ruby undefines `module_function` on Class, so a rebound
                // call with a class receiver is a TypeError.
                if !class_rc.is_module() {
                    let msg = "module_function must be called for modules".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // With no arguments it is a toggle: every method defined
                // afterwards in the body becomes a module function.
                if arguments.is_empty() {
                    class_rc.set_current_visibility(MODULE_FUNCTION_VISIBILITY);
                    return Ok(Answered(Object::Nil));
                }
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    let name = self.coerce_method_name(argument, method_name, position)?;
                    self.copy_to_module_function(class_rc, &name, position)?;
                    names.push(Object::symbol(name));
                }
                return Ok(Answered(match names.len() {
                    1 => names.remove(0),
                    _ => Object::Array(Rc::new(std::cell::RefCell::new(names))),
                }));
            }
            _ => {}
        }
        Ok(Unclaimed)
    }
}
