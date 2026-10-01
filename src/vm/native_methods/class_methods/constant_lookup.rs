// Finding a constant on a class, and the hook for one that is missing.

use super::*;

impl VirtualMachine {
    /// Where a bare `autoload` registers: the scope the call sits in. A class
    /// or module body is that scope, and inside a method body it is the
    /// module the method was written in.
    pub(crate) fn autoload_definee(&self) -> Option<Rc<crate::class::Class>> {
        if let Some(open) = self.def_scope_stack.last() {
            return Some(Rc::clone(open));
        }
        // A block carries the scope it was written in, which its own
        // `def_scope_stack` holds, so only a method body reads the nesting
        // the method captured.
        let running_a_method = self.call_stack.last().is_some_and(|frame| {
            frame.block_depth() == 0
                && matches!(
                    frame.kind(),
                    crate::vm::call_frame::FrameKind::Method { .. }
                )
        });
        if !running_a_method {
            return None;
        }
        self.method_nesting_stack
            .last()
            .and_then(|nesting| nesting.first())
            .map(Rc::clone)
    }

    /// Search `class_rc` for constant `name` the way `const_defined?` does:
    /// its own constant table and autoload registry, plus — when `inherit` —
    /// its mixins (transitively) and superclass chain with their mixins.
    /// `Object` additionally sees top-level constants (globals), and a
    /// module receiver falls back to `Object` as a last resort (Ruby scopes
    /// module constant lookup through Object). `object_fallback` controls
    /// that top-level visibility — it is on for a directly-named constant
    /// and off for the trailing segments of a scoped name (`A::B` must not
    /// find `B` at the top level). Returns `Some((owner, Some(value)))` for
    /// a bound constant, `Some((owner, None))` for a registered-but-unloaded
    /// autoload, `None` when absent. Never triggers autoload loads or
    /// const_missing.
    pub(crate) fn const_entry_on(
        &mut self,
        class_rc: &Rc<Class>,
        name: &str,
        inherit: bool,
        object_fallback: bool,
    ) -> Option<(Rc<Class>, Option<Object>)> {
        let mut queue: Vec<Rc<Class>> = vec![Rc::clone(class_rc)];
        let mut seen: Vec<*const Class> = Vec::new();
        let mut idx = 0;
        while idx < queue.len() {
            let current = Rc::clone(&queue[idx]);
            idx += 1;
            let ptr = Rc::as_ptr(&current);
            if seen.contains(&ptr) {
                continue;
            }
            seen.push(ptr);
            if let Some(v) = current.get_class_var(name) {
                return Some((current, Some(v)));
            }
            // A bound top-level constant beats a still-registered autoload
            // (an autoloaded file may have defined the constant in globals
            // without clearing Object's registration).
            if object_fallback
                && current.name() == "Object"
                && let Some(v) = self.globals().get(name)
            {
                return Some((current, Some(v)));
            }
            // Thread-aware, read-only autoload check: the loading thread
            // sees its own in-progress autoload as cleared, other threads
            // still see it as registered.
            {
                let cls = Rc::clone(&current);
                if self.autoload_pending(&cls, name) {
                    return Some((current, None));
                }
            }
            if inherit {
                for mixin in current.mixin_chain() {
                    queue.push(mixin);
                }
                if let Some(sc) = current.superclass() {
                    queue.push(sc);
                }
            }
        }
        // Module receivers (no superclass chain) see Object's constants.
        if inherit
            && object_fallback
            && class_rc.superclass().is_none()
            && class_rc.name() != "Object"
            && class_rc.name() != "BasicObject"
            && let Some(Object::Class(object_class)) = self.globals().get("Object")
            && !seen.contains(&Rc::as_ptr(&object_class))
        {
            return self.const_entry_on(&object_class, name, inherit, object_fallback);
        }
        None
    }

    /// The name a message calls a module by: what its `name` answers, or
    /// what its `inspect` answers when it has no name. A module that
    /// defines neither is called by its own name.
    pub(crate) fn module_name_for_message(
        &mut self,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<String, MetorexError> {
        let module = if module_rc.is_module() {
            Object::Module(Rc::clone(module_rc))
        } else {
            Object::Class(Rc::clone(module_rc))
        };
        let own = |method: &str| {
            crate::vm::method_lookup::module_own_method(module_rc, method).is_some()
                || module_rc
                    .singleton_class_slot()
                    .as_ref()
                    .is_some_and(|singleton| singleton.find_own_method(method).is_some())
        };
        if !own("name") && !own("inspect") {
            return Ok(module_rc.ruby_name());
        }
        if let Object::String(named) =
            self.send_to_object(module.clone(), "name", vec![], position)?
            && !named.as_str().is_empty()
        {
            return Ok(named.as_str().to_string());
        }
        match self.send_to_object(module, "inspect", vec![], position)? {
            Object::String(inspected) => Ok(inspected.as_str().to_string()),
            _ => Ok(module_rc.ruby_name()),
        }
    }

    /// Refuse a private constant read through `scope` with the scope
    /// written out. A `const_missing` the scope defines answers instead;
    /// otherwise NameError names the class that owns the constant.
    pub(crate) fn private_constant_refused(
        &mut self,
        scope: &Rc<Class>,
        owner: &Rc<Class>,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if user_const_missing(scope).is_some() {
            return self.dispatch_const_missing(scope, name, position);
        }
        let owner_name = self.module_name_for_message(owner, position)?;
        let message = format!("private constant {owner_name}::{name} referenced");
        let exception = Object::exception("NameError", message.clone());
        if let Object::Exception(details) = &exception {
            let mut details = details.borrow_mut();
            details.name = Some(name.to_string());
            details.receiver = Some(Box::new(if owner.is_module() {
                Object::Module(Rc::clone(owner))
            } else {
                Object::Class(Rc::clone(owner))
            }));
        }
        Err(MetorexError::UncaughtException {
            exception,
            location: position_to_location(position),
            message,
        })
    }

    /// Dispatch `const_missing(name)` on `module_rc` — the user-defined hook
    /// (a `def self.const_missing` anywhere on the superclass chain, or a
    /// singleton-class method, e.g. an mspec mock) when present, otherwise
    /// the default behavior: raise NameError with the `name` attribute set.
    pub(crate) fn dispatch_const_missing(
        &mut self,
        module_rc: &Rc<Class>,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some((holder, method)) = user_const_missing(module_rc) {
            return self.invoke_method(
                holder,
                method,
                Object::Class(Rc::clone(module_rc)),
                vec![Object::symbol(name.to_string())],
                position,
            );
        }
        let owner = self.module_name_for_message(module_rc, position)?;
        let qualified = if owner.is_empty() || owner == "Object" {
            name.to_string()
        } else {
            format!("{}::{}", owner, name)
        };
        let msg = format!("uninitialized constant {}", qualified);
        let exc = Object::exception("NameError", msg.clone());
        if let Object::Exception(e) = &exc {
            let mut details = e.borrow_mut();
            details.name = Some(name.to_string());
            details.receiver = Some(Box::new(if module_rc.is_module() {
                Object::Module(Rc::clone(module_rc))
            } else {
                Object::Class(Rc::clone(module_rc))
            }));
        }
        Err(MetorexError::UncaughtException {
            exception: exc,
            location: position_to_location(position),
            message: msg,
        })
    }
}

/// The `const_missing` a class or module defines in Ruby, on itself or an
/// ancestor, with the class holding it.
fn user_const_missing(module_rc: &Rc<Class>) -> Option<(Rc<Class>, Rc<crate::object::Method>)> {
    let mut cursor = Some(Rc::clone(module_rc));
    while let Some(current) = cursor {
        if let Some(method) = current.find_method("__class__const_missing") {
            return (!method.is_undefined).then_some((current, method));
        }
        if let Some(singleton) = current.singleton_class_slot().clone()
            && let Some(method) = singleton.find_method("const_missing")
        {
            return (!method.is_undefined).then_some((singleton, method));
        }
        cursor = current.superclass();
    }
    None
}
