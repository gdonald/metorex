// Pointing one method name at another.

use super::*;

impl VirtualMachine {
    /// Point `new_name` at whatever `old_name` already names on `class_rc`.
    /// A method written in Ruby is copied; one answered natively has no entry
    /// to copy, so a stub records the name to dispatch under instead.
    pub(crate) fn install_alias(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        new_name: &str,
        old_name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let new_name = new_name.to_string();
        let old_name = old_name.to_string();
        if !class_rc.alias_method(&new_name, &old_name) {
            let mut found = false;
            if let Some(Object::Class(object_class)) = self.globals().get("Object")
                && let Some(method) = object_class.find_method(&old_name)
            {
                class_rc.define_method(&new_name, method);
                found = true;
            }
            // Kernel methods live in the native dispatch tables, so
            // there is no entry to copy. A stub carrying the name
            // keeps the alias present for later removal.
            if !found && is_native_kernel_method(&old_name) {
                let mut stub = Method::with_owner(
                    new_name.clone(),
                    vec!["args".to_string()],
                    vec![],
                    "Kernel".to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                stub.native_alias = Some(old_name.clone());
                class_rc.define_method(&new_name, Rc::new(stub));
                // A Kernel function is a private method, and a name given to
                // one is private the same way.
                if crate::vm::native_methods::is_kernel_private_function(&old_name) {
                    class_rc.set_method_private(new_name.clone());
                }
                found = true;
            }
            // A singleton class aliasing one of the attached object's
            // native methods has no entry to copy either, so the stub
            // records the name it was cut from and the call reaches
            // the native implementation through that.
            if !found
                && let Some(attached) = class_rc.get_class_var("__attached__")
                && self.responds_to(&attached, &old_name)
            {
                let mut stub = Method::with_owner(
                    new_name.clone(),
                    vec!["args".to_string()],
                    vec![],
                    class_rc.name().to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                stub.original_name = Some(old_name.clone());
                stub.native_alias = Some(old_name.clone());
                class_rc.define_method(&new_name, Rc::new(stub));
                found = true;
            }
            // A refinement's body names the class it refines, so a name
            // given there stands for one that class answers to, whether it
            // holds an entry for it or answers it natively.
            if !found
                && let Some(Object::Class(refined) | Object::Module(refined)) = class_rc
                    .get_class_var(crate::vm::native_methods::module_methods::REFINEMENT_TARGET_KEY)
            {
                if let Some(method) = refined.find_method(&old_name) {
                    class_rc.define_method(&new_name, method);
                    found = true;
                } else if let Some(probe) = sample_of_class(refined.name())
                    && self.responds_to(&probe, &old_name)
                {
                    let mut stub = Method::with_owner(
                        new_name.clone(),
                        vec!["args".to_string()],
                        vec![],
                        refined.name().to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    stub.original_name = Some(old_name.clone());
                    stub.native_alias = Some(old_name.clone());
                    class_rc.define_method(&new_name, Rc::new(stub));
                    found = true;
                }
            }
            // A builtin class answers many of its methods natively,
            // with no entry to copy. A stub carrying the name keeps
            // the alias reaching the native one.
            if !found
                && let Some(probe) = sample_of_class(class_rc.name())
                && self.responds_to(&probe, &old_name)
            {
                let mut stub = Method::with_owner(
                    new_name.clone(),
                    vec!["args".to_string()],
                    vec![],
                    class_rc.name().to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                stub.original_name = Some(old_name.clone());
                stub.native_alias = Some(old_name.clone());
                class_rc.define_method(&new_name, Rc::new(stub));
                found = true;
            }
            if !found {
                let msg = format!(
                    "undefined method '{}' for {} '{}'",
                    old_name,
                    class_rc.kind_name().to_lowercase(),
                    class_rc.name()
                );
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
        }

        Ok(())
    }
}

/// The ancestor above `class_rc` that holds `name` when `class_rc` holds it
/// too. Ruby refuses to read a class variable in that state: which of the two
/// the name stands for is no longer settled.
pub(crate) fn overtaking_ancestor(class_rc: &Rc<Class>, name: &str) -> Option<Rc<Class>> {
    class_rc.get_class_var(name)?;
    let mut cursor = class_rc.superclass();
    while let Some(current) = cursor {
        if current.get_class_var(name).is_some() {
            return Some(current);
        }
        cursor = current.superclass();
    }
    None
}

/// Every part of a path but the last. Trailing separators do not count as a
/// part, a path with no separator at all stands in the working directory, and
/// a run of separators at the front reads as the one root.
pub(crate) fn parent_of_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return if path.is_empty() {
            ".".to_string()
        } else {
            "/".to_string()
        };
    }
    let Some(cut) = trimmed.rfind('/') else {
        return ".".to_string();
    };
    let front = trimmed[..cut].trim_end_matches('/');
    if front.is_empty() {
        return "/".to_string();
    }
    if let Some(rest) = front.strip_prefix("//") {
        return format!("/{}", rest.trim_start_matches('/'));
    }
    front.to_string()
}
