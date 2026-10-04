// Attribute accessors, and whether a name resolves to a method.

use super::*;

impl VirtualMachine {
    /// The accessors `attr_reader` and friends define, and whether a name
    /// resolves to a method at all.
    pub(crate) fn call_attribute_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        match method_name {
            // attr_reader/attr_writer/attr_accessor as runtime instance
            // methods on Module/Class. Define the accessors on the receiver,
            // apply the receiver's current visibility, and return the array
            // of newly-defined method names as symbols (Ruby 3.0+).
            "attr_reader" | "attr_writer" | "attr_accessor" | "attr" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                // `attr name, true|false` is the deprecated 2-arg boolean form:
                // the second arg controls writer creation, and only the first
                // arg is a name. Ruby warns under `$VERBOSE = true`.
                let want_reader = matches!(method_name, "attr_reader" | "attr_accessor" | "attr");
                let mut want_writer = matches!(method_name, "attr_writer" | "attr_accessor");
                let names_slice: &[Object] = if method_name == "attr"
                    && arguments.len() == 2
                    && matches!(&arguments[1], Object::Bool(_))
                {
                    if matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
                        self.emit_warning_to_stderr(
                            "warning: optional boolean argument is obsoleted",
                            position,
                        );
                    }
                    want_writer = matches!(&arguments[1], Object::Bool(true));
                    &arguments[..1]
                } else {
                    arguments
                };
                let mut names: Vec<String> = Vec::with_capacity(names_slice.len());
                for arg in names_slice {
                    let n = self.coerce_method_name(arg, method_name, position)?;
                    names.push(n);
                }
                let visibility = class_rc.current_visibility();
                let mut defined: Vec<Object> = Vec::new();
                let mut newly_defined_names: Vec<String> = Vec::new();
                for attr_name in &names {
                    if want_reader {
                        let getter_body = vec![crate::ast::Statement::Return {
                            value: Some(crate::ast::Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            }),
                            position,
                        }];
                        let getter =
                            crate::object::Method::new(attr_name.clone(), vec![], getter_body);
                        class_rc.define_method(attr_name, Rc::new(getter));
                        if visibility != "public" {
                            class_rc.set_method_private(attr_name.clone());
                        }
                        class_rc.declare_instance_var(attr_name);
                        defined.push(Object::symbol(attr_name.clone()));
                        newly_defined_names.push(attr_name.clone());
                    }
                    if want_writer {
                        let setter_body = vec![crate::ast::Statement::Assignment {
                            target: crate::ast::Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: crate::ast::Expression::Identifier {
                                name: crate::object::UNNAMED_PARAMETER.to_string(),
                                position,
                            },
                            position,
                        }];
                        let setter_name = format!("{}=", attr_name);
                        let setter = crate::object::Method::new(
                            setter_name.clone(),
                            vec![crate::object::UNNAMED_PARAMETER.to_string()],
                            setter_body,
                        );
                        class_rc.define_method(&setter_name, Rc::new(setter));
                        if visibility != "public" {
                            class_rc.set_method_private(setter_name.clone());
                        }
                        class_rc.declare_instance_var(attr_name);
                        defined.push(Object::symbol(setter_name.clone()));
                        newly_defined_names.push(setter_name);
                    }
                }
                // Fire `method_added` (or `singleton_method_added` when the
                // receiver is a singleton class) for each method we just
                // installed, so user-defined hooks observe attr_* the same
                // way they observe `def`.
                let is_singleton = class_rc.get_class_var("__singleton__").is_some();
                let hook_name = if is_singleton {
                    "singleton_method_added"
                } else {
                    "method_added"
                };
                for added in &newly_defined_names {
                    self.invoke_class_hook(class_rc, hook_name, added, position)?;
                }
                return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                    defined,
                )))));
            }
            // Module#method_defined?(name) — true when `name` resolves to a
            // public or protected instance method on the receiver, including
            // inherited methods. The optional second arg (default true) limits
            // the search to the receiver itself when false.
            "method_defined?"
            | "public_method_defined?"
            | "private_method_defined?"
            | "protected_method_defined?" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name = self.coerce_method_name(&arguments[0], method_name, position)?;
                let include_super = match arguments.get(1) {
                    Some(Object::Bool(b)) => *b,
                    _ => true,
                };
                let found = if include_super {
                    class_rc.find_method_with_owner(&name)
                } else {
                    class_rc
                        .find_own_method(&name)
                        .map(|method| (Rc::clone(class_rc), method))
                };
                // BasicObject's private methods, `initialize` among them,
                // answer natively, so every class below it reports them
                // from the list.
                if found.is_none()
                    && BASIC_OBJECT_PRIVATE_METHODS.contains(&name.as_str())
                    && (class_rc.name() == "BasicObject"
                        || (include_super
                            && crate::vm::method_invocation::descends_from(
                                class_rc,
                                "BasicObject",
                            )))
                {
                    return Ok(Answered(Object::Bool(
                        method_name == "private_method_defined?",
                    )));
                }
                // Kernel's own methods live in the native dispatch tables
                // rather than in its method map, so the private ones are
                // listed rather than looked up.
                if class_rc.name() == "Kernel"
                    && found.is_none()
                    && KERNEL_PRIVATE_FUNCTIONS.contains(&name.as_str())
                {
                    return Ok(Answered(Object::Bool(
                        method_name == "private_method_defined?",
                    )));
                }
                // Kernel carries its functions as module functions as well,
                // and those are public methods of Kernel itself.
                if found.is_none()
                    && class_rc.is_singleton_class()
                    && matches!(
                        class_rc.get_class_var("__attached__"),
                        Some(Object::Module(held) | Object::Class(held)) if held.ruby_name() == "Kernel"
                    )
                    && (KERNEL_PRIVATE_FUNCTIONS.contains(&name.as_str())
                        || crate::vm::native_methods::is_native_kernel_method(&name))
                {
                    return Ok(Answered(Object::Bool(matches!(
                        method_name,
                        "method_defined?" | "public_method_defined?"
                    ))));
                }
                // The public ones live there too, so Kernel reports them the
                // same way rather than answering that it has none.
                if matches!(class_rc.name(), "Kernel" | "Object")
                    && found.is_none()
                    && crate::vm::native_methods::is_native_kernel_method(&name)
                {
                    return Ok(Answered(Object::Bool(matches!(
                        method_name,
                        "method_defined?" | "public_method_defined?"
                    ))));
                }
                let answer = match found {
                    // A tombstone left by `undef_method` is not a definition.
                    Some((_, method)) if method.is_undefined => false,
                    None => false,
                    Some((owner, _)) => {
                        let is_private = owner.is_method_private(&name);
                        let is_protected = owner.is_method_protected(&name);
                        match method_name {
                            "method_defined?" => !is_private,
                            "public_method_defined?" => !is_private && !is_protected,
                            "private_method_defined?" => is_private,
                            "protected_method_defined?" => is_protected,
                            _ => unreachable!(),
                        }
                    }
                };
                return Ok(Answered(Object::Bool(answer)));
            }
            _ => {}
        }
        Ok(Unclaimed)
    }
}
