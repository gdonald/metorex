// Mixing a module into a class, and the hooks that fire when one is.

use super::*;

impl VirtualMachine {
    /// Execute include at statement level (outside class body).
    /// Top-level `include Mod` adds the module to Object's mixin chain
    /// (Ruby semantics for `include` at the main scope). When called from
    /// inside a block whose def-scope is non-empty (e.g. a lambda invoked
    /// from inside a module body), the innermost class/module receives the
    /// include instead — matching `module M; include X; end` behavior.
    /// Suppressed when running inside `load(path, true)` (wrapped load).
    pub(crate) fn execute_include(
        &mut self,
        module_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        if self.load_wrap_depth > 0 {
            return Ok(ControlFlow::Next);
        }
        let module = self
            .resolve_constant_with_autoload(module_name)?
            .or_else(|| self.resolve_qualified_constant(module_name))
            .or_else(|| self.environment().get(module_name))
            .or_else(|| self.globals().get(module_name));
        let module_rc = match module {
            Some(Object::Module(m)) => m,
            Some(_) => {
                return Err(MetorexError::runtime_error(
                    format!("'{}' is not a module", module_name),
                    position_to_location(position),
                ));
            }
            None => {
                return Err(MetorexError::runtime_error(
                    format!("Undefined module '{}'", module_name),
                    position_to_location(position),
                ));
            }
        };
        // Prefer the innermost lexical class/module if there is one; this
        // makes `include X` inside a lambda invoked from a module body
        // mix into that module rather than into Object.
        if let Some(target) = self.def_scope_stack.last().cloned() {
            self.apply_module_include(&target, &module_rc, position)?;
        } else if let Some(Object::Class(object_class)) = self.globals().get("Object") {
            object_class.add_mixin(module_rc);
        }
        Ok(ControlFlow::Next)
    }

    /// Execute `extend Mod` at statement level. Outside a class body the
    /// receiver is the current `self`, so the module's methods become
    /// singleton methods of that object, the way `main.extend Mod` works.
    pub(crate) fn execute_extend(
        &mut self,
        _module_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        if let Some(Object::Module(module_rc)) = self.globals().get(_module_name) {
            // At the top level there is no `self` to extend, and metorex
            // already installs top-level `def`s on Object, so the module's
            // methods go there too.
            let target = self
                .environment()
                .get("self")
                .or_else(|| self.globals().get("Object"));
            if let Some(target) = target {
                let singleton = self.singleton_class_of(&target);
                singleton.add_mixin(std::rc::Rc::clone(&module_rc));
                if matches!(target, Object::Class(_)) {
                    // Top-level extend makes the methods callable without a
                    // receiver, which means Object's instances answer them.
                    if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                        object_class.add_mixin(module_rc);
                    }
                }
                return Ok(ControlFlow::Next);
            }
        }
        Err(MetorexError::runtime_error(
            "extend can only be used inside a class definition",
            position_to_location(position),
        ))
    }

    /// Validate an argument to `include`/`prepend`. Ruby accepts a module
    /// (never a class) and rejects a refinement. An instance of a Module
    /// subclass is accepted; metorex has no mixin chain behind such an
    /// object, so `None` means "accepted, nothing to mix in".
    pub(crate) fn resolve_include_argument(
        &mut self,
        argument: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<Option<Rc<Class>>, MetorexError> {
        if let Object::Module(module_rc) | Object::Class(module_rc) = argument
            && module_rc
                .get_class_var(crate::vm::REFINEMENT_LABEL_KEY)
                .is_some()
        {
            let msg = format!("Cannot {} refinement", method_name);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", msg.clone()),
                location: position_to_location(position),
                message: msg,
            });
        }
        match argument {
            Object::Module(module_rc) => Ok(Some(Rc::clone(module_rc))),
            Object::Instance(_) if self.is_module_subclass_instance(argument) => Ok(None),
            other => Err(method_argument_type_error(
                method_name,
                "Module",
                other,
                position,
            )),
        }
    }

    /// Whether `value` is an instance of a class that descends from Module.
    pub(crate) fn is_module_subclass_instance(&mut self, value: &Object) -> bool {
        let Some(Object::Class(module_class)) = self.globals().get("Module") else {
            return false;
        };
        let value_class = self.builtins().class_of(value);
        self.builtins().is_subclass_of(&value_class, &module_class)
    }

    /// Mix `module_rc` into `target` with full Ruby semantics: dispatch to a
    /// user-defined `append_features` on the module's singleton class if one
    /// exists; otherwise enforce the default checks (FrozenError when target
    /// is frozen, ArgumentError on a cyclic include) and add the mixin.
    pub(crate) fn apply_module_include(
        &mut self,
        target: &Rc<Class>,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        self.apply_module_mixin(target, module_rc, "append_features", "included", position)
    }

    /// Prepend `module_rc` to `target`, ahead of the target's own methods.
    pub(crate) fn apply_module_prepend(
        &mut self,
        target: &Rc<Class>,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        self.apply_module_mixin(target, module_rc, "prepend_features", "prepended", position)
    }

    /// Mix `module_rc` into `target`: dispatch to a user-defined features
    /// hook if the module has one, otherwise enforce the default checks
    /// (FrozenError when the target is frozen, ArgumentError on a cyclic
    /// mixin) and add the mixin, then fire the notification hook.
    fn apply_module_mixin(
        &mut self,
        target: &Rc<Class>,
        module_rc: &Rc<Class>,
        features_hook: &str,
        notify_hook: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        // Ruby calls the features hook, then the notification hook, whether
        // or not the module overrode the first one.
        let target_argument = Object::Class(Rc::clone(target));
        if !self.invoke_module_hook(module_rc, features_hook, &target_argument, position)? {
            if features_hook == "prepend_features" {
                self.default_prepend_features(target, module_rc, position)?;
            } else {
                self.default_append_features(target, module_rc, position)?;
            }
        }
        self.invoke_module_hook(module_rc, notify_hook, &target_argument, position)?;
        Ok(())
    }

    /// Call `module_rc`'s own definition of `hook` with `argument`, reporting
    /// whether one was found. `def self.hook` lands in the module's table
    /// under the `__class__` prefix; `class << mod` puts it on the singleton.
    fn invoke_module_hook(
        &mut self,
        module_rc: &Rc<Class>,
        hook: &str,
        argument: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let receiver = Object::Module(Rc::clone(module_rc));
        if let Some(method) = module_rc.find_method(&format!("__class__{}", hook)) {
            self.invoke_method(
                Rc::clone(module_rc),
                method,
                receiver,
                vec![argument.clone()],
                position,
            )?;
            return Ok(true);
        }
        if let Some(sc) = module_rc.singleton_class_slot().clone()
            && let Some(method) = sc.find_method(hook)
        {
            self.invoke_method(sc, method, receiver, vec![argument.clone()], position)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Default `Module#append_features(target)` behavior: validate the target
    /// (frozen check, cyclic include check) then add `module_rc` to its
    /// mixin chain.
    pub(crate) fn default_append_features(
        &mut self,
        target: &Rc<Class>,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if target.is_frozen() {
            let msg = format!("can't modify frozen Module: {}", target.name());
            let exc = Object::exception("FrozenError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        if Rc::ptr_eq(target, module_rc) || module_includes(module_rc, target) {
            let msg = "cyclic include detected".to_string();
            let exc = Object::exception("ArgumentError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        // Ruby ignores an include of a module the target already inherits,
        // leaving it at its original place in the ancestor chain.
        if module_includes(target, module_rc) {
            return Ok(());
        }
        target.add_mixin(Rc::clone(module_rc));
        Ok(())
    }

    /// `prepend`'s counterpart to `default_append_features`. The module lands
    /// ahead of the target's own methods. Unlike `include`, a module the
    /// target merely inherits is still prepended here, since the front of the
    /// chain is a different place from where it already sits.
    pub(crate) fn default_prepend_features(
        &mut self,
        target: &Rc<Class>,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if target.is_frozen() {
            let msg = format!("can't modify frozen Module: {}", target.name());
            let exc = Object::exception("FrozenError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        if Rc::ptr_eq(target, module_rc) || module_includes(module_rc, target) {
            let msg = "cyclic prepend detected".to_string();
            let exc = Object::exception("ArgumentError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        if target.has_prepend(module_rc) {
            return Ok(());
        }
        target.add_prepend(Rc::clone(module_rc));
        Ok(())
    }
}
