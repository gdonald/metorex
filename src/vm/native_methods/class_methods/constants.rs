// Reading and writing the constants a module holds.

use super::*;

impl VirtualMachine {
    /// Reading, writing and removing the constants a module holds.
    pub(crate) fn call_constant_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        match method_name {
            "const_defined?" => {
                // const_defined?(name [, inherit=true]) — when inherit is
                // false, only check the receiver itself; otherwise also
                // search mixins and the superclass chain. The autoload
                // registry counts as defined (Ruby treats a registered
                // autoload as a constant entry). Scoped names
                // (`A::B`, `::Top`) resolve segment by segment; invalid
                // segments raise NameError. Never calls const_missing.
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        "const_defined?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_path =
                    self.coerce_method_name(&arguments[0], "const_defined?", position)?;
                let inherit = match arguments.get(1) {
                    None => true,
                    Some(v) => crate::vm::utils::is_truthy(v),
                };
                let mut rest: &str = &const_path;
                let mut current = Rc::clone(class_rc);
                if let Some(stripped) = rest.strip_prefix("::") {
                    rest = stripped;
                    current = match self.globals().get("Object") {
                        Some(Object::Class(c)) => c,
                        _ => return Ok(Answered(Object::Bool(false))),
                    };
                }
                let segments: Vec<&str> = rest.split("::").collect();
                for seg in &segments {
                    if !is_valid_constant_name(seg) {
                        let msg = format!("wrong constant name {}", const_path);
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
                for (i, seg) in segments.iter().enumerate() {
                    let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                    if i + 1 == segments.len() {
                        return Ok(Answered(Object::Bool(entry.is_some())));
                    }
                    // Intermediate segments must resolve to a class/module
                    // value; a registered-but-unloaded autoload can't be
                    // traversed without triggering the load.
                    match entry {
                        Some((_, Some(Object::Class(c)))) | Some((_, Some(Object::Module(c)))) => {
                            current = c;
                        }
                        _ => return Ok(Answered(Object::Bool(false))),
                    }
                }
                return Ok(Answered(Object::Bool(false)));
            }
            "const_source_location" => {
                // const_source_location(name [, inherit=true]) — same search
                // as const_get, but returns the recorded [file, line] of the
                // constant's definition, [] for constants without a Ruby
                // source (builtins), and nil when not found (never calls
                // const_missing).
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        "const_source_location",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let was_symbol = matches!(&arguments[0], Object::Symbol(_));
                let const_path =
                    self.coerce_method_name(&arguments[0], "const_source_location", position)?;
                let inherit = match arguments.get(1) {
                    None => true,
                    Some(v) => crate::vm::utils::is_truthy(v),
                };
                let wrong_name = |path: &str| {
                    let msg = format!("wrong constant name {}", path);
                    let exc = Object::exception("NameError", msg.clone());
                    MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    }
                };
                // A Symbol must be a simple name — scope separators raise.
                if was_symbol && const_path.contains("::") {
                    return Err(wrong_name(&const_path));
                }
                let mut rest: &str = &const_path;
                let mut current = Rc::clone(class_rc);
                if let Some(stripped) = rest.strip_prefix("::") {
                    rest = stripped;
                    current = match self.globals().get("Object") {
                        Some(Object::Class(c)) => c,
                        _ => return Err(wrong_name(&const_path)),
                    };
                }
                let segments: Vec<&str> = rest.split("::").collect();
                for seg in &segments {
                    if !is_valid_constant_name(seg) {
                        return Err(wrong_name(&const_path));
                    }
                }
                let loc_array = |loc: Option<(String, i64)>| {
                    let items = match loc {
                        // A constant the interpreter defines stands in no
                        // file of the program's, which Ruby reports as no
                        // location at all. One written in the core library's
                        // own Ruby source is the same to a reader.
                        Some((file, _))
                            if file.is_empty()
                                || file.starts_with(crate::vm::INTERNAL_FILE_PREFIX) =>
                        {
                            Vec::new()
                        }
                        Some((file, line)) => {
                            vec![Object::string(file), Object::Int(line)]
                        }
                        None => Vec::new(),
                    };
                    Object::Array(Rc::new(std::cell::RefCell::new(items)))
                };
                for (i, seg) in segments.iter().enumerate() {
                    if i + 1 == segments.len() {
                        // Thread-aware: if this autoload is currently loading
                        // on a different thread, report the autoload
                        // registration's location — the constant isn't
                        // really defined from that thread's view yet.
                        let thread = self
                            .thread_current_stack
                            .last()
                            .cloned()
                            .unwrap_or(Object::Nil);
                        let other_thread_loading =
                            self.autoload_loading.iter().any(|(cls, n, loader)| {
                                if !Rc::ptr_eq(cls, &current) || n != *seg {
                                    return false;
                                }
                                let same = match (loader, &thread) {
                                    (Object::Nil, Object::Nil) => true,
                                    (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
                                    _ => false,
                                };
                                !same
                            });
                        if other_thread_loading {
                            return Ok(Answered(loc_array(current.get_autoload_location(seg))));
                        }
                        let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                        return Ok(Answered(match entry {
                            Some((owner, Some(_))) => loc_array(
                                owner
                                    .get_const_location(seg)
                                    .or_else(|| owner.get_autoload_location(seg)),
                            ),
                            Some((owner, None)) => loc_array(owner.get_autoload_location(seg)),
                            // A still-registered autoload that the lookup
                            // treats as cleared (e.g. this thread is the one
                            // loading it) keeps reporting its registration
                            // location until the constant is defined.
                            None if current.get_autoload(seg).is_some() => {
                                loc_array(current.get_autoload_location(seg))
                            }
                            None => Object::Nil,
                        }));
                    }
                    // Intermediate segments resolve like const_get, firing
                    // registered autoloads along the way.
                    let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                    let resolved = match entry {
                        Some((_, Some(v))) => Some(v),
                        Some((owner, None)) => self.try_autoload_constant(&owner, seg)?,
                        None => self.try_autoload_constant(&current, seg)?,
                    };
                    match resolved {
                        Some(Object::Class(c)) | Some(Object::Module(c)) => current = c,
                        _ => return Ok(Answered(Object::Nil)),
                    }
                }
                return Ok(Answered(Object::Nil));
            }
            "const_get" => {
                // const_get(name [, inherit=true]) — search order mirrors
                // const_defined?: the receiver, then (when inherit) mixins
                // and the superclass chain, with Object exposing top-level
                // constants and modules falling back to Object for a
                // directly-named constant. Scoped names (`A::B`, `::Top`)
                // resolve segment by segment; a Symbol must be a simple
                // name. Registered autoloads fire; unresolvable names
                // dispatch const_missing (default: NameError with `name`).
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        "const_get",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let was_symbol = matches!(&arguments[0], Object::Symbol(_));
                let const_path = self.coerce_method_name(&arguments[0], "const_get", position)?;
                let inherit = match arguments.get(1) {
                    None => true,
                    Some(v) => crate::vm::utils::is_truthy(v),
                };
                let wrong_name = |path: &str| {
                    let msg = format!("wrong constant name {}", path);
                    let exc = Object::exception("NameError", msg.clone());
                    MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    }
                };
                if was_symbol && const_path.contains("::") {
                    return Err(wrong_name(&const_path));
                }
                let mut rest: &str = &const_path;
                let mut current = Rc::clone(class_rc);
                if let Some(stripped) = rest.strip_prefix("::") {
                    rest = stripped;
                    current = match self.globals().get("Object") {
                        Some(Object::Class(c)) => c,
                        _ => return Err(wrong_name(&const_path)),
                    };
                }
                let segments: Vec<&str> = rest.split("::").collect();
                for seg in &segments {
                    if !is_valid_constant_name(seg) {
                        return Err(wrong_name(&const_path));
                    }
                }
                let mut value = Object::Nil;
                for (i, seg) in segments.iter().enumerate() {
                    self.settle_pending_autoload(&current, seg);
                    let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                    let resolved = match entry {
                        Some((_, Some(v))) => Some(v),
                        // Registered autoload — fire the load on the owner.
                        Some((owner, None)) => self.try_autoload_constant(&owner, seg)?,
                        // No entry — a registered autoload whose file was
                        // already loaded without defining the constant can
                        // still be satisfied by a re-load (several autoloads
                        // may point at one path); `try_autoload_constant`
                        // owns that logic.
                        None => self.try_autoload_constant(&current, seg)?,
                    };
                    let resolved = match resolved {
                        Some(v) => v,
                        None => {
                            let missing = self.dispatch_const_missing(&current, seg, position)?;
                            if i + 1 == segments.len() {
                                return Ok(Answered(missing));
                            }
                            missing
                        }
                    };
                    if i + 1 == segments.len() {
                        self.warn_deprecated_constant(&current, seg, position);
                        value = resolved;
                    } else {
                        match resolved {
                            Object::Class(c) | Object::Module(c) => current = c,
                            other => {
                                let msg =
                                    format!("{} does not refer to class/module", other.type_name());
                                let exc = Object::exception("TypeError", msg.clone());
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: msg,
                                });
                            }
                        }
                    }
                }
                return Ok(Answered(value));
            }
            // Default `Module#const_added` — a no-op returning nil. User
            // hooks (`def self.const_added`) are dispatched before native
            // fallback, so this only fires for the base implementation.
            "const_added" => {
                // Native dispatch runs before the user method body in
                // `invoke_method`, so step aside when a user hook exists.
                if class_rc.find_method("__class__const_added").is_some() {
                    return Ok(Deferred);
                }
                let mut cursor = Some(Rc::clone(class_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && sc.find_method("const_added").is_some()
                    {
                        return Ok(Deferred);
                    }
                    cursor = current.superclass();
                }
                // A reopened `Module` or `Class` gives every class the hook as
                // an instance method, and that user body wins over this one.
                if self.user_const_added_hook_defined() {
                    return Ok(Deferred);
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "const_added",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                return Ok(Answered(Object::Nil));
            }
            // Default `Module#const_missing` — raise NameError with the
            // qualified constant path and the `name` attribute set. User
            // hooks step aside the same way const_added's do.
            "const_missing" => {
                if class_rc.find_method("__class__const_missing").is_some() {
                    return Ok(Deferred);
                }
                let mut cursor = Some(Rc::clone(class_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && sc.find_method("const_missing").is_some()
                    {
                        return Ok(Deferred);
                    }
                    cursor = current.superclass();
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "const_missing",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_name = match &arguments[0] {
                    Object::Symbol(s) => s.as_str().to_string(),
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "const_missing",
                            "Symbol or String",
                            other,
                            position,
                        ));
                    }
                };
                return self
                    .dispatch_const_missing(class_rc, &const_name, position)
                    .map(Answered);
            }
            "const_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "const_set",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // FrozenError fires before name validation or coercion.
                if class_rc.is_frozen() {
                    let kind = if class_rc.superclass().is_some() {
                        "Class"
                    } else {
                        "Module"
                    };
                    let msg = format!("can't modify frozen {}: {}", kind, class_rc.inspect_name());
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let const_name = self.coerce_method_name(&arguments[0], "const_set", position)?;
                if !is_valid_constant_name(&const_name) {
                    let msg = format!("wrong constant name {}", const_name);
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Overwriting a bound value warns; replacing a pending
                // autoload registration does not.
                if class_rc.get_class_var(&const_name).is_some() {
                    let msg = format!(
                        "warning: already initialized constant {}::{}",
                        class_rc.inspect_name(),
                        const_name
                    );
                    self.emit_warning_to_stderr(&msg, position);
                }
                // Setting the constant cancels any pending autoload for it
                // and clears any "loaded but unrealized" bookkeeping.
                class_rc.remove_autoload(&const_name);
                class_rc.clear_unrealized_autoload(&const_name);
                class_rc.set_class_var(&const_name, arguments[1].clone());
                let assign_file = self
                    .reported_current_file()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                class_rc.set_const_location(&const_name, assign_file, position.line as i64);
                // Object's constants are top-level constants — publish to
                // globals so bare references resolve.
                if class_rc.name() == "Object" {
                    self.globals_mut()
                        .set(const_name.clone(), arguments[1].clone());
                }
                // An anonymous module/class value takes the constant path as
                // its name, cascading into anonymous modules nested under it.
                if let Object::Class(v) | Object::Module(v) = &arguments[1] {
                    let qualified = if class_rc.name() == "Object" {
                        const_name.clone()
                    } else {
                        format!("{}::{}", class_rc.inspect_name(), const_name)
                    };
                    v.assign_name_recursive(&qualified);
                }
                self.trigger_const_added_hook(
                    Object::Class(Rc::clone(class_rc)),
                    &const_name,
                    position,
                )?;
                return Ok(Answered(arguments[1].clone()));
            }
            _ => {}
        }
        Ok(Unclaimed)
    }
}
