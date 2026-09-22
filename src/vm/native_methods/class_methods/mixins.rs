// Including and prepending modules.

use super::*;

impl VirtualMachine {
    /// Mixing a module into a class, and asking which modules a class holds.
    pub(crate) fn call_mixin_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // `Klass.include(Mod)` / `Klass.prepend(Mod)`: mix the module into
        // the class through the `append_features` dispatch path so user
        // overrides on the module's singleton class fire and the cyclic /
        // frozen checks run. `prepend` ordering is still approximated as a
        // regular include (sufficient for current fixture setup).
        // The hooks whose default implementation does nothing and returns
        // nil. `included` and friends with real behavior are handled above.
        if matches!(
            method_name,
            "method_added"
                | "method_removed"
                | "method_undefined"
                | "included"
                | "extended"
                | "prepended"
        ) && arguments.len() == 1
            && !has_user_defined_method(class_rc, method_name)
        {
            return Ok(Answered(Object::Nil));
        }
        // `Klass.include?(Mod)` — whether Mod appears in the ancestors,
        // excluding the receiver itself. A class argument is a TypeError.
        if method_name == "include?" && arguments.len() == 1 {
            let Object::Module(queried) = &arguments[0] else {
                return Err(method_argument_type_error(
                    method_name,
                    "Module",
                    &arguments[0],
                    position,
                ));
            };
            let mut chain: Vec<Object> = Vec::new();
            let mut seen: Vec<*const Class> = Vec::new();
            push_class_ancestors(class_rc, &mut chain, &mut seen);
            let found = chain.iter().any(|ancestor| match ancestor {
                Object::Class(c) | Object::Module(c) => {
                    Rc::ptr_eq(c, queried) && !Rc::ptr_eq(c, class_rc)
                }
                _ => false,
            });
            return Ok(Answered(Object::Bool(found)));
        }
        // Bare `include` / `prepend` inside a class or module body: Ruby
        // reports the missing argument rather than a missing method.
        if matches!(method_name, "include" | "prepend")
            && arguments.is_empty()
            && !has_user_defined_method(class_rc, method_name)
        {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                0,
                position,
            ));
        }
        // A refinement holds methods for one class only, and Ruby removed both
        // of these from it rather than leave a way to mix into it.
        if matches!(method_name, "include" | "prepend")
            && class_rc
                .get_class_var(crate::vm::native_methods::REFINEMENT_LABEL_KEY)
                .is_some()
        {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("Refinement#{} has been removed", method_name),
                position,
            ));
        }
        if matches!(method_name, "include" | "prepend") && !arguments.is_empty() {
            // Ruby applies the arguments in reverse, so the first module
            // listed ends up nearest the receiver in the ancestor chain.
            for arg in arguments.iter().rev() {
                if let Some(module_rc) =
                    self.resolve_include_argument(arg, method_name, position)?
                {
                    if method_name == "prepend" {
                        self.apply_module_prepend(class_rc, &module_rc, position)?;
                    } else {
                        self.apply_module_include(class_rc, &module_rc, position)?;
                    }
                }
            }
            return Ok(Answered(Object::Class(Rc::clone(class_rc))));
        }
        // `mod.append_features(target)` / `mod.prepend_features(target)`:
        // default behavior — add `mod` as a mixin on `target`, with the
        // standard cyclic/frozen checks. Defer to a user-defined override
        // (singleton method on the receiver) when one is present.
        if matches!(method_name, "append_features" | "prepend_features") && !arguments.is_empty() {
            let class_method_key = format!("__class__{}", method_name);
            if class_rc.find_method(&class_method_key).is_some() {
                return Ok(Deferred);
            }
            if let Some(sc) = class_rc.singleton_class_slot().clone()
                && sc.find_method(method_name).is_some()
            {
                return Ok(Deferred);
            }
            for arg in arguments {
                match arg {
                    Object::Module(t) | Object::Class(t) => {
                        if method_name == "prepend_features" {
                            self.default_prepend_features(t, class_rc, position)?;
                        } else {
                            self.default_append_features(t, class_rc, position)?;
                        }
                    }
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Module",
                            other,
                            position,
                        ));
                    }
                }
            }
            return Ok(Answered(Object::Class(Rc::clone(class_rc))));
        }
        // `Klass.subclasses` returns the direct subclasses (Class objects).
        if method_name == "subclasses" {
            let subs: Vec<Object> = class_rc
                .subclasses()
                .into_iter()
                .map(Object::Class)
                .collect();
            return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                subs,
            )))));
        }
        Ok(Unclaimed)
    }
}
