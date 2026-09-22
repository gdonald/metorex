// The callbacks a class fires as methods and modules are added to it.

use super::*;

impl VirtualMachine {
    /// Invoke a `method_added` / `singleton_method_added` hook on `class_rc` if
    /// the user defined one. The method receives the new method's name as a
    /// symbol; errors raised by the hook propagate.
    ///
    /// For `singleton_method_added`, Ruby fires the hook on the *attached
    /// object* (the object whose singleton class gained the method), not on
    /// the singleton class itself — so when `class_rc` is a singleton class we
    /// pivot to the attached object before lookup.
    /// The class-level method `name` on `class_rc`: one stored under the
    /// `__class__` convention, one on a singleton class along the superclass
    /// chain, or one copied in by `extend`.
    pub(crate) fn class_method_of(
        &mut self,
        class_rc: &Rc<Class>,
        name: &str,
    ) -> Option<Rc<Method>> {
        if let Some(method) = class_rc.find_method(&format!("__class__{}", name)) {
            return Some(method);
        }
        if let Some(Object::Method(method)) = class_rc.get_class_var(&format!("__ext__{}", name)) {
            return Some(method);
        }
        let mut cursor = Some(Rc::clone(class_rc));
        while let Some(current) = cursor {
            if let Some(sc) = current.singleton_class_slot().clone()
                && let Some(method) = sc.find_method(name)
            {
                return Some(method);
            }
            cursor = current.superclass();
        }
        None
    }

    /// Copy an instance method to the module object as a module function:
    /// the copy is a public module-level method and the original becomes a
    /// private instance method.
    pub(crate) fn copy_to_module_function(
        &mut self,
        module_rc: &Rc<Class>,
        name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let method = module_rc
            .find_method(name)
            .or_else(|| {
                // A refinement's body names the class it refines, so a name
                // aliased there is the one that class answers to rather than
                // one the refinement holds of its own.
                let Some(Object::Class(refined) | Object::Module(refined)) = module_rc
                    .get_class_var(
                        crate::vm::native_methods::module_methods::REFINEMENT_TARGET_KEY,
                    )
                else {
                    return None;
                };
                refined.find_method(name)
            })
            .or_else(|| {
                // Kernel methods live on Object, which a module does not
                // inherit from; `module_function :require` copies from there.
                match self.globals().get("Object") {
                    Some(Object::Class(object_class)) => object_class.find_method(name),
                    _ => None,
                }
            });
        let method = match method {
            Some(method) => method,
            // A refinement aliasing one of the refined class's own names
            // reaches a method that answers natively rather than out of a
            // table, so a stub stands for it the way Kernel's do.
            None if module_rc
                .get_class_var(crate::vm::native_methods::module_methods::REFINEMENT_TARGET_KEY)
                .is_some() =>
            {
                let mut stub = Method::with_owner(
                    name.to_string(),
                    vec!["args".to_string()],
                    vec![],
                    module_rc.name().to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                // The name it stands for is what the refined class answers
                // to, which is what the call reaches through.
                stub.original_name = Some(name.to_string());
                Rc::new(stub)
            }
            // Kernel's own methods are native rather than table entries; a
            // stub reaches the same implementation when invoked.
            None if is_native_kernel_method(name) => {
                let mut stub = Method::with_owner(
                    name.to_string(),
                    vec!["args".to_string()],
                    vec![],
                    "Kernel".to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                Rc::new(stub)
            }
            None => {
                return Err(MetorexError::runtime_error(
                    format!(
                        "undefined method '{}' for module '{}'",
                        name,
                        module_rc.name()
                    ),
                    position_to_location(position),
                ));
            }
        };
        module_rc.define_method(format!("__class__{}", name), Rc::clone(&method));
        if module_rc.find_own_method(name).is_some() {
            module_rc.set_method_private(name.to_string());
        }
        self.invoke_class_hook(module_rc, "singleton_method_added", name, position)?;
        Ok(())
    }

    /// A hook that was undefined still reaches a user-defined
    /// `method_missing`. Answers None when nothing defines one, so the caller
    /// raises the NoMethodError itself.
    pub(crate) fn undefined_hook_via_method_missing(
        &mut self,
        receiver: &Object,
        hook: &str,
        argument: &Object,
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        let (owner, handler) = self.lookup_method(receiver, "method_missing")?;
        if handler.is_undefined {
            return None;
        }
        let arguments = vec![Object::symbol(hook.to_string()), argument.clone()];
        Some(self.invoke_method(owner, handler, receiver.clone(), arguments, position))
    }

    /// The class or module a singleton class is attached to, when it is one.
    pub(crate) fn attached_class_of(&self, class_rc: &Rc<Class>) -> Option<Rc<Class>> {
        class_rc.get_class_var("__singleton__")?;
        match class_rc.get_class_var("__attached__") {
            Some(Object::Class(attached) | Object::Module(attached)) => Some(attached),
            _ => None,
        }
    }

    pub(crate) fn invoke_class_hook(
        &mut self,
        class_rc: &Rc<Class>,
        hook: &str,
        added_name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let arg = Object::symbol(added_name.to_string());

        // A hook that was undefined raises when it would have been called,
        // the same as any other call to a method that is not there. A
        // singleton hook lives on the singleton class, which is where
        // `class << obj; undef_method ...; end` puts the marker.
        let undefined_marker = class_rc
            .find_method(hook)
            .filter(|existing| existing.is_undefined)
            .or_else(|| {
                class_rc
                    .singleton_class_slot()
                    .clone()
                    .and_then(|singleton| singleton.find_method(hook))
                    .filter(|existing| existing.is_undefined)
            });
        if undefined_marker.is_some() {
            let attached = class_rc
                .get_class_var("__attached__")
                .unwrap_or(Object::Class(Rc::clone(class_rc)));
            // An undefined method still reaches `method_missing`, which is
            // where the default implementation raises.
            if let Some(handled) =
                self.undefined_hook_via_method_missing(&attached, hook, &arg, position)
            {
                handled?;
                return Ok(());
            }
            let message = format!("undefined method '{}' for {}", hook, attached);
            return Err(MetorexError::UncaughtException {
                exception: crate::vm::errors::no_method_error(
                    &message,
                    hook,
                    &attached,
                    std::slice::from_ref(&arg),
                ),
                location: position_to_location(position),
                message,
            });
        }

        if hook.starts_with("singleton_method_")
            && class_rc.get_class_var("__singleton__").is_some()
            && let Some(attached) = class_rc.get_class_var("__attached__")
        {
            let attached_class = match &attached {
                Object::Class(c) | Object::Module(c) => Some(Rc::clone(c)),
                _ => None,
            };
            if let Some(target_class) = attached_class {
                // The hook may sit on the attached class's singleton class or
                // under the `__class__` name `def self.hook` stores it as.
                if let Some(sc) = target_class.singleton_class_slot().clone()
                    && let Some(method) = sc.find_method(hook)
                {
                    self.invoke_method(sc, method, attached.clone(), vec![arg], position)?;
                    return Ok(());
                }
                if let Some(method) = self.class_method_of(&target_class, hook) {
                    self.invoke_method(
                        target_class,
                        method,
                        attached.clone(),
                        vec![arg],
                        position,
                    )?;
                }
                return Ok(());
            }
            // The attached object is an ordinary one, so the hook is looked
            // up on it the way any other method call on it would be.
            if let Some((owner, method)) = self.lookup_method(&attached, hook)
                && !method.is_undefined
            {
                self.invoke_method(owner, method, attached.clone(), vec![arg], position)?;
            }
            return Ok(());
        }

        let class_method_name = format!("__class__{}", hook);
        let receiver = Object::Class(Rc::clone(class_rc));

        if let Some(method) = class_rc.find_method(&class_method_name) {
            self.invoke_method(Rc::clone(class_rc), method, receiver, vec![arg], position)?;
            return Ok(());
        }
        if let Some(sc) = class_rc.singleton_class_slot().clone()
            && let Some(method) = sc.find_method(hook)
        {
            self.invoke_method(sc, method, receiver, vec![arg], position)?;
        }
        Ok(())
    }
}
