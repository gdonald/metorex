// Visibility and ancestry questions.

use super::*;

impl VirtualMachine {
    /// Visibility, ancestry, and the other questions a module answers about
    /// itself.
    pub(crate) fn call_module_query_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        match method_name {
            // Module#extend: mix the given module's instance methods into the
            // receiver's singleton class, so `klass.some_module_method` works.
            "extend" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "extend",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Ruby takes a module here and rejects a class.
                let module_rc = match &arguments[0] {
                    Object::Module(m) => Rc::clone(m),
                    other => {
                        return Err(method_argument_type_error(
                            "extend", "Module", other, position,
                        ));
                    }
                };
                let target = Object::Class(Rc::clone(class_rc));
                self.apply_module_extend(&target, &module_rc, position)?;
                return Ok(Answered(target));
            }
            // `private_class_method :name` / `public_class_method :name` —
            // flip the class-method visibility on the receiver's singleton
            // class. Visibility is otherwise only honoured for private calls;
            // the inherited hook (line inherited_spec.rb:43) ensures a
            // marked-private method still fires via `super`/hook invocation.
            "private_class_method" | "public_class_method" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                // A lone array argument names several methods at once.
                let named: Vec<Object> = match arguments {
                    [Object::Array(names)] => names.borrow().clone(),
                    other => other.to_vec(),
                };
                let target_class = Object::Class(Rc::clone(class_rc));
                let singleton = self.singleton_class_of(&target_class);
                for argument in &named {
                    let name = self.coerce_method_name(argument, method_name, position)?;
                    // Mirror the method onto the singleton class so
                    // `lookup_method` finds it there (where visibility lives).
                    // The `inherited` hook is inherited from Class's singleton
                    // table via the `__class__` convention, so copy it across
                    // and we have something to toggle visibility on.
                    if singleton.find_method(&name).is_none() {
                        match self.class_method_of(class_rc, &name) {
                            Some(method) => singleton.define_method(&name, method),
                            // `new` and `allocate` are answered by the runtime
                            // rather than a method table, so there is nothing
                            // to mirror. The marking on the singleton is what
                            // dispatch consults.
                            None if matches!(name.as_str(), "new" | "allocate") => {}
                            None => {
                                let msg = format!(
                                    "undefined method '{}' for {} '{}'",
                                    name,
                                    class_rc.kind_name().to_lowercase(),
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
                    if method_name == "private_class_method" {
                        singleton.set_method_private(&name);
                    } else {
                        singleton.set_method_public(&name);
                    }
                }
                return Ok(Answered(Object::Class(Rc::clone(class_rc))));
            }
            // Module#set_temporary_name: a display name for an anonymous
            // module, cleared by passing nil. A permanent name cannot be
            // replaced, and the name may not look like a constant path.
            "set_temporary_name" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if class_rc.has_permanent_name() {
                    let msg = "can't change permanent name".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("RuntimeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let receiver = if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                };
                if matches!(arguments[0], Object::Nil) {
                    class_rc.set_temporary_name(None);
                    return Ok(Answered(receiver));
                }
                let name = self.coerce_name_argument(&arguments[0], position)?;
                let complaint = if name.is_empty() {
                    Some("empty class/module name")
                } else if looks_like_constant_path(&name) {
                    Some("the temporary name must not be a constant path to avoid confusion")
                } else {
                    None
                };
                if let Some(msg) = complaint {
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", msg.to_string()),
                        location: position_to_location(position),
                        message: msg.to_string(),
                    });
                }
                class_rc.set_temporary_name(Some(name));
                return Ok(Answered(receiver));
            }
            // Module#remove_const: remove a constant from this module's table.
            "remove_const" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "remove_const",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_name = self.coerce_name_argument(&arguments[0], position)?;
                if !is_valid_constant_name(&const_name) {
                    let msg = format!("wrong constant name {}", const_name);
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Only a constant of the receiver itself can be removed, and
                // a pending autoload counts as one.
                if class_rc.get_class_var(&const_name).is_none()
                    && class_rc.get_autoload(&const_name).is_none()
                    && !class_rc.unrealized_autoload_names().contains(&const_name)
                    && !(class_rc.name() == "Object" && self.globals().contains(&const_name))
                {
                    let msg = format!(
                        "constant {}::{} not defined",
                        class_rc.ruby_name(),
                        const_name
                    );
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Drop the resolved constant (if any), any pending autoload
                // registration, and any "loaded but unrealized" bookkeeping.
                // Without removing all three, the name would still surface
                // in `#constants` because the constants list aggregates
                // class_vars + autoloads + unrealized autoloads.
                self.warn_deprecated_constant(class_rc, &const_name, position);
                let mut removed = class_rc.remove_class_var(&const_name);
                // Object's constants are top-level constants — drop the
                // globals binding too so bare references stop resolving.
                if class_rc.name() == "Object" {
                    let from_globals = self.globals_mut().remove(&const_name);
                    if removed.is_none() {
                        removed = from_globals;
                    }
                }
                let removed_autoload = class_rc.remove_autoload(&const_name);
                class_rc.clear_unrealized_autoload(&const_name);
                // Drop the recorded source location so a subsequent
                // `autoload` for the same name surfaces *its* location via
                // `const_source_location` instead of the stale class-def
                // location from before the removal.
                class_rc.remove_const_location(&const_name);
                // Removing a constant that was only registered for autoload
                // answers with nil: it never held a value.
                let _ = removed_autoload;
                return Ok(Answered(removed.unwrap_or(Object::Nil)));
            }
            "private" | "public" | "protected" => {
                return self
                    .apply_class_visibility_modifier(class_rc, method_name, arguments, position)
                    .map(Answered);
            }
            "private_methods" => {
                let include_super = !matches!(
                    arguments.first(),
                    Some(Object::Bool(false)) | Some(Object::Nil)
                );
                // What is private *on the class object* lives in its
                // singleton chain. Its own private instance methods are its
                // instances' business, so they only surface once ancestors do.
                let mut names: Vec<String> = self
                    .private_method_names_for(&Object::Class(Rc::clone(class_rc)), include_super);
                if include_super {
                    names.extend(class_rc.private_method_names());
                }
                // Classes and modules inherit Module's private instance
                // methods. `extend_object` and the `*_features` pair are
                // undefined on Class, so the module dispatch path adds those.
                names.extend(
                    [
                        "extended",
                        "included",
                        "prepended",
                        "const_added",
                        "method_added",
                    ]
                    .map(String::from),
                );
                names.extend(MODULE_PRIVATE_DECLARATIONS.iter().map(|n| (*n).to_string()));
                if include_super {
                    let mut current = class_rc.superclass();
                    while let Some(parent) = current {
                        names.extend(parent.private_method_names());
                        current = parent.superclass();
                    }
                }
                names.sort();
                names.dedup();
                let syms: Vec<Object> = names.into_iter().map(Object::symbol).collect();
                return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                    syms,
                )))));
            }
            // Module#singleton_class?: whether this is the class of exactly
            // one object, as `class << obj` opens.
            "singleton_class?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                return Ok(Answered(Object::Bool(class_rc.is_singleton_class())));
            }
            "superclass" => {
                return match class_rc.superclass() {
                    Some(parent) => Ok(Answered(Object::Class(parent))),
                    None => Ok(Answered(Object::Nil)),
                };
            }
            "ancestors" => {
                let mut chain: Vec<Object> = Vec::new();
                let mut seen: Vec<*const Class> = Vec::new();
                push_class_ancestors(class_rc, &mut chain, &mut seen);
                return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                    chain,
                )))));
            }
            // The ancestors that are modules rather than classes, with the
            // receiver itself left out.
            "included_modules" => {
                let mut chain: Vec<Object> = Vec::new();
                let mut seen: Vec<*const Class> = Vec::new();
                push_class_ancestors(class_rc, &mut chain, &mut seen);
                let modules: Vec<Object> = chain
                    .into_iter()
                    .filter(|ancestor| match ancestor {
                        Object::Module(m) => !Rc::ptr_eq(m, class_rc),
                        _ => false,
                    })
                    .collect();
                return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                    modules,
                )))));
            }
            _ => {}
        }
        Ok(Unclaimed)
    }
}
