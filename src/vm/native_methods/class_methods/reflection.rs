// What a class reports about itself and its constants.

use super::*;

impl VirtualMachine {
    /// `Class#initialize`, the constants a module holds, and the object a
    /// singleton class is attached to.
    pub(crate) fn call_reflection_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // Class#initialize: private; already-initialized classes raise
        // TypeError, and passing `Class` itself as the superclass argument also
        // raises TypeError (MRI rejects `Class` as a superclass regardless of
        // whether the receiver was freshly allocated).
        if method_name == "initialize" {
            if let Some(Object::Class(c)) = arguments.first()
                && c.name() == "Class"
            {
                let msg = "already initialized class".to_string();
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            if class_rc.get_class_var("__uninitialized__").is_none() {
                let msg = "already initialized class".to_string();
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            class_rc.remove_class_var("__uninitialized__");
            return Ok(Answered(Object::Nil));
        }
        if method_name == "constants" {
            // Collect one class/module's visible constants: its constant
            // table (public, uppercase-leading), plus registered autoloads
            // and names whose autoload fired without defining the constant
            // (MRI keeps those in `#constants` even though `const_defined?`
            // and `autoload?` both report nothing).
            let collect_from = |cls: &Rc<Class>, names: &mut Vec<String>| {
                for n in cls.class_var_names() {
                    if n.starts_with("__")
                        || !n.chars().next().is_some_and(|c| c.is_uppercase())
                        || cls.is_private_constant(&n)
                    {
                        continue;
                    }
                    if !names.contains(&n) {
                        names.push(n);
                    }
                }
                for n in cls
                    .autoload_names()
                    .into_iter()
                    .chain(cls.unrealized_autoload_names())
                {
                    if !names.contains(&n) {
                        names.push(n);
                    }
                }
            };
            // `Module.constants` with no argument is special-cased by Ruby
            // to the constants reachable at the call site — for our model,
            // the top-level constants (globals plus Object's table).
            if class_rc.name() == "Module" && arguments.is_empty() {
                let mut names: Vec<String> = self
                    .globals()
                    .iter()
                    .map(|(n, _)| n.clone())
                    .filter(|n| {
                        !n.starts_with("__")
                            && !n.contains("::")
                            && n.chars().next().is_some_and(|c| c.is_uppercase())
                    })
                    .collect();
                if let Some(Object::Class(object_class)) = self.globals().get("Object") {
                    collect_from(&object_class, &mut names);
                }
                let names: Vec<Object> = names.into_iter().map(Object::symbol).collect();
                return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                    names,
                )))));
            }
            // constants(inherit = true): with inherit, include constants
            // from mixins (transitively) and the superclass chain — but not
            // Object's, which are top-level constants.
            let inherit = match arguments.first() {
                None => true,
                Some(v) => crate::vm::utils::is_truthy(v),
            };
            let mut names: Vec<String> = Vec::new();
            collect_from(class_rc, &mut names);
            // Every top-level constant is a constant of Object, including the
            // built-in class names, which live in the global registry rather
            // than in Object's own table.
            if class_rc.name() == "Object" {
                for (name, _) in self.globals().iter() {
                    if name.starts_with("__")
                        || name.contains("::")
                        || !name.chars().next().is_some_and(|c| c.is_uppercase())
                        || names.contains(name)
                    {
                        continue;
                    }
                    names.push(name.clone());
                }
            }
            if inherit {
                let mut queue: Vec<Rc<Class>> = class_rc.prepend_chain();
                queue.extend(class_rc.mixin_chain());
                let mut cursor = class_rc.superclass();
                while let Some(sc) = cursor {
                    if matches!(sc.name(), "Object" | "BasicObject") {
                        break;
                    }
                    queue.push(Rc::clone(&sc));
                    cursor = sc.superclass();
                }
                let mut seen: Vec<*const Class> = vec![Rc::as_ptr(class_rc)];
                let mut idx = 0;
                while idx < queue.len() {
                    let current = Rc::clone(&queue[idx]);
                    idx += 1;
                    let ptr = Rc::as_ptr(&current);
                    if seen.contains(&ptr) {
                        continue;
                    }
                    seen.push(ptr);
                    collect_from(&current, &mut names);
                    for prepended in current.prepend_chain() {
                        queue.push(prepended);
                    }
                    for mixin in current.mixin_chain() {
                        queue.push(mixin);
                    }
                }
            }
            let names: Vec<Object> = names.into_iter().map(Object::symbol).collect();
            return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                names,
            )))));
        }
        if method_name == "attached_object" {
            let is_singleton = class_rc.get_class_var("__singleton__").is_some();
            if !is_singleton {
                let msg = format!("'{}' is not a singleton class", class_rc.name());
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            let attached = class_rc
                .get_class_var("__attached__")
                .unwrap_or(Object::Nil);
            // Singleton classes of nil / true / false exist but their attached
            // object can't be obtained directly — MRI raises TypeError here.
            let tag = match &attached {
                Object::Nil => Some("NilClass"),
                Object::Bool(true) => Some("TrueClass"),
                Object::Bool(false) => Some("FalseClass"),
                _ => None,
            };
            if let Some(name) = tag {
                let msg = format!("'{}' is not a singleton class", name);
                let exc = Object::exception("TypeError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            return Ok(Answered(attached));
        }
        if lacks_an_allocator(class_rc) && method_name == "new" {
            let exc = Object::exception(
                "NoMethodError",
                format!("undefined method 'new' for {}:Class", class_rc.name()),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: format!("undefined method 'new' for {}:Class", class_rc.name()),
            });
        }
        Ok(Unclaimed)
    }
}
