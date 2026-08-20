use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn apply_class_visibility_modifier(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        modifier: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // With no arguments the modifier is a toggle: the methods defined
        // after it in this body take the given visibility.
        if arguments.is_empty() {
            class_rc.set_current_visibility(modifier);
            return Ok(Object::Nil);
        }
        let flat: Vec<Object> = if arguments.len() == 1 {
            if let Object::Array(arr) = &arguments[0] {
                arr.borrow().clone()
            } else {
                arguments.to_vec()
            }
        } else {
            arguments.to_vec()
        };
        let mut names: Vec<String> = Vec::with_capacity(flat.len());
        let mut to_mark: Vec<String> = Vec::with_capacity(flat.len());
        for arg in &flat {
            let n = match arg {
                Object::Symbol(s) => s.as_str().to_string(),
                Object::String(s) => s.as_str().to_string(),
                _ => {
                    let exc = Object::exception(
                        "TypeError",
                        format!("{} is not a symbol nor a string", arg),
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: format!("{} is not a symbol nor a string", arg),
                    });
                }
            };
            // A visibility declaration for a method the receiver only
            // inherits gives the receiver its own entry, and that new entry
            // fires `method_added`. Kernel is reached through the Object
            // fallback, since a method defined at the top level lives on
            // Object rather than anywhere in Kernel's ancestry.
            // Ruby records the new visibility on the receiver without
            // copying the method down, so a later redefinition in the
            // ancestor is what the receiver goes on calling. Naming a method
            // that already has the visibility being asked for records
            // nothing at all.
            // An entry of the receiver's own, whether a definition or an
            // earlier visibility declaration, is what a new declaration
            // updates. Without one, declaring the visibility a method
            // already carries records nothing.
            let has_own_entry =
                class_rc.find_own_method(&n).is_some() || class_rc.has_visibility_marking(&n);
            let already_set = !has_own_entry && self.inherits_visibility(class_rc, &n, modifier);
            if !already_set {
                to_mark.push(n.clone());
            }
            if class_rc.find_method(&n).is_none() {
                let on_object = matches!(
                    self.globals().get("Object"),
                    Some(Object::Class(object_class)) if object_class.find_method(&n).is_some()
                );
                // Kernel methods such as `puts` and `abort` are registered as
                // global native functions rather than entries in Object's
                // method table, so a visibility declaration naming one still
                // has a method to talk about.
                let on_kernel = matches!(self.globals().get(&n), Some(Object::NativeFunction(_)));
                // A singleton class of a class carries the class-level
                // methods, which are native rather than table entries. `new`
                // is the one specs redeclare, to make a class uninstantiable.
                let on_metaclass = n == "new"
                    && matches!(
                        class_rc.get_class_var("__attached__"),
                        Some(Object::Class(_))
                    );
                if !on_object && !on_kernel && !on_metaclass {
                    let msg = format!("undefined method '{}' for class '{}'", n, class_rc.name());
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
            }
            names.push(n);
        }
        for n in &to_mark {
            // Recording a visibility for a method the receiver only inherits
            // gives it an entry of its own, and that entry fires the hook.
            let is_new_entry =
                class_rc.find_own_method(n).is_none() && !class_rc.has_visibility_marking(n);
            match modifier {
                "private" => class_rc.set_method_private(n.clone()),
                "protected" => class_rc.set_method_protected(n.clone()),
                _ => class_rc.set_method_public(n),
            }
            if is_new_entry {
                self.invoke_class_hook(class_rc, "method_added", n, position)?;
            }
        }
        match flat.len() {
            1 => Ok(Object::Symbol(Rc::new(names[0].clone()))),
            _ => Ok(Object::Array(Rc::new(std::cell::RefCell::new(
                names
                    .into_iter()
                    .map(|n| Object::Symbol(Rc::new(n)))
                    .collect(),
            )))),
        }
    }
}
