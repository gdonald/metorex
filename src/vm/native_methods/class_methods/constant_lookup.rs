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
        let mut found: Option<(Rc<Class>, Rc<crate::object::Method>)> = None;
        let mut cursor = Some(Rc::clone(module_rc));
        while let Some(current) = cursor {
            if let Some(m) = current.find_method("__class__const_missing") {
                found = Some((current, m));
                break;
            }
            if let Some(sc) = current.singleton_class_slot().clone()
                && let Some(m) = sc.find_method("const_missing")
            {
                found = Some((sc, m));
                break;
            }
            cursor = current.superclass();
        }
        if let Some((holder, method)) = found
            && !method.is_undefined
        {
            return self.invoke_method(
                holder,
                method,
                Object::Class(Rc::clone(module_rc)),
                vec![Object::symbol(name.to_string())],
                position,
            );
        }
        let owner = module_rc.ruby_name();
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
