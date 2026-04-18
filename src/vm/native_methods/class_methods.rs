use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn call_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let non_instantiable = matches!(class_rc.name(), "TrueClass" | "FalseClass" | "NilClass");
        if non_instantiable && method_name == "allocate" {
            let exc = Object::exception(
                "TypeError",
                format!("allocator undefined for {}", class_rc.name()),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: format!("allocator undefined for {}", class_rc.name()),
            });
        }
        // Class.allocate and subclasses: uninitialized class instance. `new` and
        // `superclass` on it must raise TypeError (Ruby semantics).
        if class_rc.get_class_var("__uninitialized__").is_some()
            && matches!(method_name, "new" | "superclass")
        {
            let message = "uninitialized class".to_string();
            let exc = Object::exception("TypeError", message.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message,
            });
        }
        if method_name == "allocate" {
            if class_rc.name() == "Class" {
                let anon = Rc::new(Class::new("", None));
                anon.set_class_var("__uninitialized__", Object::Bool(true));
                return Ok(Some(Object::Class(anon)));
            }
            let inst = crate::object::Instance::new(Rc::clone(class_rc));
            return Ok(Some(Object::Instance(Rc::new(std::cell::RefCell::new(
                inst,
            )))));
        }
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
            return Ok(Some(Object::Nil));
        }
        if method_name == "constants" {
            let names: Vec<Object> = class_rc
                .class_var_names()
                .into_iter()
                .filter(|n| !n.starts_with("__"))
                .filter(|n| n.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
                .map(|n| Object::Symbol(Rc::new(n)))
                .collect();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(names)))));
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
            return Ok(Some(attached));
        }
        if non_instantiable && method_name == "new" {
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
        if method_name == "new" && class_rc.name() == "Class" {
            let superclass = match arguments.first() {
                Some(Object::Class(c)) => {
                    // Singleton (meta) classes can't be used as a superclass —
                    // MRI raises TypeError("can't make subclass of singleton
                    // class") in this case.
                    if c.get_class_var("__singleton__").is_some() {
                        return Err(MetorexError::type_error(
                            "can't make subclass of singleton class",
                            position_to_location(position),
                        ));
                    }
                    Some(Rc::clone(c))
                }
                Some(other) => {
                    return Err(MetorexError::type_error(
                        format!("superclass must be a Class (given {})", other.type_name()),
                        position_to_location(position),
                    ));
                }
                None => self.globals().get("Object").and_then(|o| {
                    if let Object::Class(c) = o {
                        Some(c)
                    } else {
                        None
                    }
                }),
            };
            let anon = Rc::new(Class::new("", superclass.clone()));
            // Extract the pending block before triggering the `inherited`
            // hook — the hook's own invoke_method would otherwise consume it.
            let pending = self.pending_block.take();
            // Ruby: `inherited` hook fires before the block runs, so a hook
            // that records `self` sees the parent first, then the block's
            // `self` push appends the subclass.
            if let Some(sc) = &superclass {
                self.trigger_inherited_hook(sc, Rc::clone(&anon), position)?;
            }
            if let Some(Object::Block(block)) = pending {
                self.apply_block_as_class_body(&anon, &block, position)?;
            }
            return Ok(Some(Object::Class(anon)));
        }
        if method_name == "new" && class_rc.name() == "Module" {
            let anon = Rc::new(Class::new("", None));
            if let Some(Object::Block(block)) = self.pending_block.take() {
                self.apply_block_as_class_body_with_self(
                    &anon,
                    &block,
                    position,
                    Object::Module(Rc::clone(&anon)),
                )?;
            }
            return Ok(Some(Object::Module(anon)));
        }
        if method_name == "new" && class_rc.name() == "Time"
            || (method_name == "now" && class_rc.name() == "Time")
        {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            return Ok(Some(Object::Float(secs)));
        }
        if method_name == "new" && class_rc.name() == "Set" {
            use crate::object::ObjectHash;
            let mut set = std::collections::HashSet::new();
            if arguments.len() == 1 {
                if let Object::Array(arr_rc) = &arguments[0] {
                    for item in arr_rc.borrow().iter() {
                        if let Some(hash) = ObjectHash::from_object(item) {
                            set.insert(hash);
                        } else {
                            return Err(MetorexError::runtime_error(
                                format!("Cannot add {} to set (not hashable)", item.type_name()),
                                position_to_location(position),
                            ));
                        }
                    }
                } else {
                    return Err(method_argument_type_error(
                        "Set.new",
                        "Array",
                        &arguments[0],
                        position,
                    ));
                }
            } else if arguments.len() > 1 {
                return Err(MetorexError::runtime_error(
                    format!("Set.new expects 0-1 arguments, got {}", arguments.len()),
                    position_to_location(position),
                ));
            }
            return Ok(Some(Object::Set(Rc::new(std::cell::RefCell::new(set)))));
        }
        match method_name {
            "new" => {
                return self
                    .invoke_callable(
                        Object::Class(Rc::clone(class_rc)),
                        arguments.to_vec(),
                        position,
                    )
                    .map(Some);
            }
            "name" => {
                let name = class_rc.ruby_name();
                if name.is_empty() {
                    return Ok(Some(Object::Nil));
                }
                return Ok(Some(Object::String(Rc::new(name))));
            }
            // Module#extend: mix the given module's instance methods into the
            // receiver's singleton class, so `klass.some_module_method` works.
            "extend" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "extend",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let module_rc = match &arguments[0] {
                    Object::Module(m) => Rc::clone(m),
                    Object::Class(c) => Rc::clone(c),
                    other => {
                        return Err(method_argument_type_error(
                            "extend", "Module", other, position,
                        ));
                    }
                };
                let target = Object::Class(Rc::clone(class_rc));
                let singleton = self.singleton_class_of(&target);
                singleton.add_mixin(module_rc);
                return Ok(Some(target));
            }
            // `private_class_method :name` / `public_class_method :name` —
            // flip the class-method visibility on the receiver's singleton
            // class. Visibility is otherwise only honoured for private calls;
            // the inherited hook (line inherited_spec.rb:43) ensures a
            // marked-private method still fires via `super`/hook invocation.
            "private_class_method" | "public_class_method" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let target_class = Object::Class(Rc::clone(class_rc));
                let singleton = self.singleton_class_of(&target_class);
                for arg in arguments {
                    let name = match arg {
                        Object::String(s) => s.as_ref().clone(),
                        Object::Symbol(s) => s.as_ref().clone(),
                        other => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String or Symbol",
                                other,
                                position,
                            ));
                        }
                    };
                    // Mirror the method onto the singleton class so
                    // `lookup_method` finds it there (where visibility lives).
                    // The `inherited` hook is inherited from Class's singleton
                    // table via the `__class__` convention — copy it across
                    // so we have something to toggle visibility on.
                    if singleton.find_method(&name).is_none() {
                        let key = format!("__class__{}", name);
                        if let Some(method) = class_rc.find_method(&key) {
                            singleton.define_method(&name, method);
                        }
                    }
                    if method_name == "private_class_method" {
                        singleton.set_method_private(&name);
                    } else {
                        singleton.set_method_public(&name);
                    }
                }
                return Ok(Some(Object::Class(Rc::clone(class_rc))));
            }
            // Module#remove_const: remove a constant from this module's table.
            "remove_const" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "remove_const",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_name = match &arguments[0] {
                    Object::Symbol(s) => s.as_ref().clone(),
                    Object::String(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "remove_const",
                            "Symbol or String",
                            other,
                            position,
                        ));
                    }
                };
                let removed = class_rc.remove_class_var(&const_name);
                return Ok(Some(removed.unwrap_or(Object::Nil)));
            }
            "private" | "public" => {
                return self
                    .apply_class_visibility_modifier(class_rc, method_name, arguments, position)
                    .map(Some);
            }
            "private_methods" => {
                let include_super = !matches!(arguments.first(), Some(Object::Bool(false)));
                let mut names: Vec<String> = class_rc.private_method_names();
                if include_super {
                    let mut current = class_rc.superclass();
                    while let Some(parent) = current {
                        names.extend(parent.private_method_names());
                        current = parent.superclass();
                    }
                }
                names.sort();
                names.dedup();
                let syms: Vec<Object> = names
                    .into_iter()
                    .map(|n| Object::Symbol(Rc::new(n)))
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(syms)))));
            }
            "superclass" => {
                return match class_rc.superclass() {
                    Some(parent) => Ok(Some(Object::Class(parent))),
                    None => Ok(Some(Object::Nil)),
                };
            }
            "ancestors" => {
                let mut chain: Vec<Object> = Vec::new();
                chain.push(Object::Class(Rc::clone(class_rc)));
                for mixin in class_rc.mixin_chain() {
                    chain.push(Object::Module(mixin));
                }
                let mut current = class_rc.superclass();
                while let Some(parent) = current {
                    chain.push(Object::Class(Rc::clone(&parent)));
                    for mixin in parent.mixin_chain() {
                        chain.push(Object::Module(mixin));
                    }
                    current = parent.superclass();
                }
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(chain)))));
            }
            "const_defined?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "const_defined?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_name = match &arguments[0] {
                    Object::Symbol(s) => s.as_ref().clone(),
                    Object::String(s) => s.as_ref().clone(),
                    _ => return Ok(Some(Object::Bool(false))),
                };
                let found = class_rc.get_class_var(&const_name).is_some()
                    || self.environment().get(&const_name).is_some()
                    || self.globals().get(&const_name).is_some();
                return Ok(Some(Object::Bool(found)));
            }
            "const_get" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "const_get",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let const_name = match &arguments[0] {
                    Object::Symbol(s) => s.as_ref().clone(),
                    Object::String(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "const_get",
                            "Symbol or String",
                            other,
                            position,
                        ));
                    }
                };
                if let Some(val) = class_rc.get_class_var(&const_name) {
                    return Ok(Some(val));
                }
                let msg = format!("uninitialized constant {}", const_name);
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            "class_eval" | "module_eval" => {
                let block = self.pending_block.take();
                match block {
                    Some(Object::Block(b)) => {
                        let result = self.execute_block_with_receiver(
                            &b,
                            Object::Class(Rc::clone(class_rc)),
                            vec![],
                            position,
                        )?;
                        return Ok(Some(result));
                    }
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "class_eval requires a block".to_string(),
                            position_to_location(position),
                        ));
                    }
                }
            }
            "define_method" => {
                if arguments.is_empty() {
                    return Err(method_argument_error("define_method", 1, 0, position));
                }
                let method_name_str = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "define_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let block = self
                    .pending_block
                    .take()
                    .or_else(|| arguments.get(1).cloned());
                let block = match block {
                    Some(Object::Block(b)) => b,
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "define_method requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                let mut regular_params: Vec<String> = Vec::new();
                let mut variadic_param: Option<(usize, String)> = None;
                let mut block_parameter: Option<String> = None;
                for (i, param) in block.parameters.iter().enumerate() {
                    if let Some(name) = param.strip_prefix('*') {
                        variadic_param = Some((i, name.to_string()));
                        regular_params.push(name.to_string());
                    } else if let Some(name) = param.strip_prefix('&') {
                        block_parameter = Some(name.to_string());
                    } else {
                        regular_params.push(param.clone());
                    }
                }
                let mut method =
                    Method::new(method_name_str.clone(), regular_params, block.body.clone());
                method.variadic_param = variadic_param;
                method.block_parameter = block_parameter;
                method.captured_vars = Some(if block.captured_vars.is_empty() {
                    self.environment().current_scope_var_refs()
                } else {
                    block.captured_vars.clone()
                });
                class_rc.define_method(&method_name_str, Rc::new(method));
                return Ok(Some(Object::Nil));
            }
            "remove_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "remove_method",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "remove_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                if !class_rc.remove_method(&name_str) {
                    return Err(MetorexError::runtime_error(
                        format!("method '{}' not defined in {}", name_str, class_rc.name()),
                        position_to_location(position),
                    ));
                }
                return Ok(Some(Object::Nil));
            }
            "undef_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "undef_method",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "undef_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let sentinel = Method::undefined(name_str.clone());
                class_rc.define_method(&name_str, Rc::new(sentinel));
                return Ok(Some(Object::Nil));
            }
            "alias_method" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "alias_method",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let new_name = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "alias_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let old_name = match &arguments[1] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "alias_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                if !class_rc.alias_method(&new_name, &old_name) {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "undefined method '{}' for class '{}'",
                            old_name,
                            class_rc.name()
                        ),
                        position_to_location(position),
                    ));
                }
                return Ok(Some(Object::Nil));
            }
            "module_function" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "module_function",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "module_function",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                if let Some(method) = class_rc.find_method(&name_str) {
                    class_rc.set_class_var(format!("__ext__{}", name_str), Object::Method(method));
                } else {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "undefined method '{}' for module '{}'",
                            name_str,
                            class_rc.name()
                        ),
                        position_to_location(position),
                    ));
                }
                return Ok(Some(Object::Nil));
            }
            // Module methods we treat as no-ops (Metorex doesn't track these
            // concepts, but class bodies that use them still need to load).
            "deprecate_constant" | "ruby2_keywords" => {
                return Ok(Some(Object::Nil));
            }
            _ => {}
        }
        Ok(None)
    }
}
