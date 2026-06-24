use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::native_methods::is_valid_constant_name;
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
            let mut names: Vec<String> = class_rc
                .class_var_names()
                .into_iter()
                .filter(|n| !n.starts_with("__"))
                .filter(|n| n.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
                .collect();
            // Autoload-registered names participate in the constants table from
            // the moment `autoload` is called, even before the file is loaded.
            for n in class_rc.autoload_names() {
                if !names.contains(&n) {
                    names.push(n);
                }
            }
            // Names whose autoload fired but didn't actually define the
            // constant: MRI keeps these in `#constants` for a while, even
            // though `const_defined?` and `autoload?` both report nothing.
            for n in class_rc.unrealized_autoload_names() {
                if !names.contains(&n) {
                    names.push(n);
                }
            }
            let names: Vec<Object> = names
                .into_iter()
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
            if let Some(sc) = &superclass {
                sc.add_subclass(&anon);
            }
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
        // `autoload :CONST, "path"` — register the constant→path mapping.
        // `autoload?(:CONST, [inherit=true])` returns the registered path.
        if method_name == "autoload" {
            let const_name = match arguments.first() {
                Some(Object::Symbol(s)) => (**s).clone(),
                Some(Object::String(s)) => (**s).clone(),
                _ => return Ok(Some(Object::Nil)),
            };
            if !is_valid_constant_name(&const_name) {
                let msg = format!("autoload must be constant name: {}", const_name);
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            if class_rc.is_frozen() {
                let msg = format!("can't modify frozen Class: {}", class_rc.name());
                let exc = Object::exception("FrozenError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            let path = match arguments.get(1) {
                Some(Object::String(s)) => (**s).clone(),
                Some(Object::Symbol(s)) => (**s).clone(),
                Some(other) => {
                    let other_obj = other.clone();
                    if let Some((cls, method)) = self.lookup_method(&other_obj, "to_path") {
                        let result =
                            self.invoke_method(cls, method, other_obj, Vec::new(), position)?;
                        match result {
                            Object::String(s) => (*s).clone(),
                            _ => {
                                let msg = "to_path must return a String".to_string();
                                let exc = Object::exception("TypeError", msg.clone());
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: msg,
                                });
                            }
                        }
                    } else {
                        let msg = format!(
                            "no implicit conversion of {} into String",
                            other.type_name()
                        );
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
                None => return Ok(Some(Object::Nil)),
            };
            if path.is_empty() {
                let msg = "empty file name".to_string();
                let exc = Object::exception("ArgumentError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            let caller_file = self
                .get_current_file()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            class_rc.set_autoload_location(const_name.clone(), caller_file, position.line as i64);
            class_rc.set_autoload(const_name, path);
            return Ok(Some(Object::Nil));
        }
        if method_name == "autoload?" {
            let const_name = match arguments.first() {
                Some(Object::Symbol(s)) => (**s).clone(),
                Some(Object::String(s)) => (**s).clone(),
                _ => return Ok(Some(Object::Nil)),
            };
            let inherit = !matches!(arguments.get(1), Some(Object::Bool(false)));
            let class_for_autoload = Rc::clone(class_rc);
            let local_only_blocked = !inherit && class_rc.get_autoload(&const_name).is_none();
            let path = if local_only_blocked {
                None
            } else {
                self.effective_autoload(&class_for_autoload, &const_name)
            };
            return Ok(Some(match path {
                Some(p) => Object::String(Rc::new(p)),
                None => Object::Nil,
            }));
        }
        // `Klass.include(Mod)` / `Klass.prepend(Mod)`: mix the module into
        // the class through the `append_features` dispatch path so user
        // overrides on the module's singleton class fire and the cyclic /
        // frozen checks run. `prepend` ordering is still approximated as a
        // regular include (sufficient for current fixture setup).
        if matches!(method_name, "include" | "prepend") && !arguments.is_empty() {
            for arg in arguments {
                match arg {
                    Object::Module(m) | Object::Class(m) => {
                        self.apply_module_include(class_rc, m, position)?;
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
            return Ok(Some(Object::Class(Rc::clone(class_rc))));
        }
        // `mod.append_features(target)` / `mod.prepend_features(target)`:
        // default behavior — add `mod` as a mixin on `target`, with the
        // standard cyclic/frozen checks. Defer to a user-defined override
        // (singleton method on the receiver) when one is present.
        if matches!(method_name, "append_features" | "prepend_features") && !arguments.is_empty() {
            let class_method_key = format!("__class__{}", method_name);
            if class_rc.find_method(&class_method_key).is_some() {
                return Ok(None);
            }
            if let Some(sc) = class_rc.singleton_class_slot().clone()
                && sc.find_method(method_name).is_some()
            {
                return Ok(None);
            }
            for arg in arguments {
                match arg {
                    Object::Module(t) | Object::Class(t) => {
                        self.default_append_features(t, class_rc, position)?;
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
            return Ok(Some(Object::Class(Rc::clone(class_rc))));
        }
        // `Klass.subclasses` returns the direct subclasses (Class objects).
        if method_name == "subclasses" {
            let subs: Vec<Object> = class_rc
                .subclasses()
                .into_iter()
                .map(Object::Class)
                .collect();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(subs)))));
        }
        // `Module.nesting` returns the modules/classes currently being defined
        // (innermost first). We approximate with the def_scope_stack snapshot.
        if method_name == "nesting" && class_rc.name() == "Module" {
            let nesting: Vec<Object> = self
                .def_scope_stack
                .iter()
                .rev()
                .map(|c| Object::Module(Rc::clone(c)))
                .collect();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                nesting,
            )))));
        }
        // Minimal File class methods used by mspec's `fixture` helper.
        if class_rc.name() == "File" {
            match method_name {
                "dirname" => {
                    if let Some(Object::String(s)) = arguments.first() {
                        let p = std::path::Path::new(s.as_str());
                        let dir = p
                            .parent()
                            .and_then(|d| d.to_str())
                            .unwrap_or(".")
                            .to_string();
                        let result = if dir.is_empty() { ".".to_string() } else { dir };
                        return Ok(Some(Object::String(Rc::new(result))));
                    }
                }
                "expand_path" | "realpath" | "absolute_path" => {
                    if let Some(Object::String(s)) = arguments.first() {
                        let expanded = std::fs::canonicalize(s.as_str())
                            .ok()
                            .and_then(|p| p.to_str().map(String::from))
                            .unwrap_or_else(|| s.as_str().to_string());
                        return Ok(Some(Object::String(Rc::new(expanded))));
                    }
                }
                "join" => {
                    let mut parts: Vec<String> = Vec::new();
                    for arg in arguments {
                        match arg {
                            Object::String(s) => parts.push((**s).clone()),
                            Object::Symbol(s) => parts.push((**s).clone()),
                            Object::Array(arr) => {
                                for item in arr.borrow().iter() {
                                    if let Object::String(s) = item {
                                        parts.push((**s).clone());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    return Ok(Some(Object::String(Rc::new(parts.join("/")))));
                }
                "respond_to?" => {
                    if let Some(name_arg) = arguments.first() {
                        let name_str = match name_arg {
                            Object::String(s) => (**s).clone(),
                            Object::Symbol(s) => (**s).clone(),
                            _ => return Ok(Some(Object::Bool(false))),
                        };
                        let known = matches!(
                            name_str.as_str(),
                            "dirname"
                                | "expand_path"
                                | "realpath"
                                | "absolute_path"
                                | "join"
                                | "respond_to?"
                        );
                        return Ok(Some(Object::Bool(known)));
                    }
                }
                _ => {}
            }
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
        // Queue.new / SizedQueue.new — synchronous FIFO stub. The instance
        // carries an Array under `__queue_items`; SizedQueue ignores its
        // capacity argument (we never block).
        // Mutex.new / ConditionVariable.new — single-threaded stubs (no shared
        // state needed; the synchronize/wait/broadcast methods are no-ops).
        if method_name == "new"
            && (class_rc.name() == "Mutex" || class_rc.name() == "ConditionVariable")
        {
            use crate::object::Instance;
            let instance = Instance::new(Rc::clone(class_rc));
            let inst_rc = Rc::new(std::cell::RefCell::new(instance));
            return Ok(Some(Object::Instance(inst_rc)));
        }
        if method_name == "new" && (class_rc.name() == "Queue" || class_rc.name() == "SizedQueue") {
            use crate::object::Instance;
            let instance = Instance::new(Rc::clone(class_rc));
            let inst_rc = Rc::new(std::cell::RefCell::new(instance));
            inst_rc.borrow_mut().set_var(
                "__queue_items".to_string(),
                Object::Array(Rc::new(std::cell::RefCell::new(Vec::new()))),
            );
            return Ok(Some(Object::Instance(inst_rc)));
        }
        // Thread.new captures the block; we run it lazily on `value` so that
        // serialised "concurrent" specs (which set a shared flag between
        // construction and value-collection) still observe the flag change.
        // Newly-constructed threads land on `pending_threads` so an empty
        // `Queue#pop` (which would block in real Ruby) can drain them and
        // make forward progress.
        if method_name == "new" && class_rc.name() == "Thread" {
            use crate::object::Instance;
            let block = self.pending_block.take().unwrap_or(Object::Nil);
            let instance = Instance::new(Rc::clone(class_rc));
            let inst_rc = Rc::new(std::cell::RefCell::new(instance));
            inst_rc
                .borrow_mut()
                .set_var("__thread_block".to_string(), block);
            let obj = Object::Instance(inst_rc);
            self.pending_threads.push(obj.clone());
            return Ok(Some(obj));
        }
        // Thread.pass / Thread.current / Thread.report_on_exception= — minimal
        // stubs sufficient for fixture and spec helpers.
        if class_rc.name() == "Thread" {
            match method_name {
                "pass" => return Ok(Some(Object::Nil)),
                // Thread.current returns the innermost Thread instance whose
                // block is being executed, or Nil at the top level. Used by
                // spec fixtures that thread-local-store via
                // `Thread.current[:k] = v` and read it back via the thread
                // instance after `.value`/`.join`. `Thread.main` is the same
                // shape but conceptually the program's root thread; we don't
                // distinguish, so it returns the same value.
                "current" | "main" => {
                    return Ok(Some(
                        self.thread_current_stack
                            .last()
                            .cloned()
                            .unwrap_or(Object::Nil),
                    ));
                }
                "report_on_exception" | "report_on_exception=" => {
                    return Ok(Some(Object::Bool(true)));
                }
                "respond_to?" => {
                    if let Some(arg) = arguments.first() {
                        let n = match arg {
                            Object::String(s) => (**s).clone(),
                            Object::Symbol(s) => (**s).clone(),
                            _ => return Ok(Some(Object::Bool(false))),
                        };
                        let known = matches!(
                            n.as_str(),
                            "report_on_exception=" | "pass" | "current" | "new"
                        );
                        return Ok(Some(Object::Bool(known)));
                    }
                }
                _ => {}
            }
        }
        // Array.new — `new(size)`, `new(size, default)`, `new(size) { |i| ... }`.
        // Without arguments, returns an empty array. The block form invokes
        // the block with each index 0..size-1 and uses the result.
        if method_name == "new" && class_rc.name() == "Array" {
            let size = match arguments.first() {
                None => 0_i64,
                Some(Object::Int(n)) => *n,
                Some(other) => {
                    return Err(method_argument_type_error(
                        "Array.new",
                        "Integer",
                        other,
                        position,
                    ));
                }
            };
            if size < 0 {
                let msg = "negative array size".to_string();
                let exc = Object::exception("ArgumentError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            let block = self.pending_block.take();
            let mut elements: Vec<Object> = Vec::with_capacity(size as usize);
            if let Some(Object::Block(b)) = block {
                // The block may take 0 args (`Array.new(10) { rand }`) or 1
                // (`Array.new(10) { |i| ... }`). Match metorex's strict
                // arity check by only passing the index when the block
                // declares a positional parameter for it.
                let pass_index = b
                    .parameters
                    .iter()
                    .any(|p| !p.starts_with('&') && !p.starts_with('*'));
                for i in 0..size {
                    let args = if pass_index {
                        vec![Object::Int(i)]
                    } else {
                        vec![]
                    };
                    let v = self.execute_block_callable(&b, args, position)?;
                    elements.push(v);
                }
            } else {
                let default = arguments.get(1).cloned().unwrap_or(Object::Nil);
                for _ in 0..size {
                    elements.push(default.clone());
                }
            }
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                elements,
            )))));
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
            // Module#instance_method / Class#instance_method: returns the
            // bound `Method` object so `parameters` and friends work on it.
            "instance_method" | "public_instance_method" => {
                let name_str = match arguments.first() {
                    Some(Object::String(s)) => (**s).clone(),
                    Some(Object::Symbol(s)) => (**s).clone(),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            arguments.first().unwrap_or(&Object::Nil),
                            position,
                        ));
                    }
                };
                if let Some(method) = class_rc.find_method(&name_str) {
                    return Ok(Some(Object::Method(method)));
                }
                // Synthesize a stub for well-known Module-private mixin
                // hooks so `Module.instance_method(:append_features)` works
                // for spec patterns that bind/call them.
                if class_rc.name() == "Module"
                    && matches!(
                        name_str.as_str(),
                        "append_features"
                            | "prepend_features"
                            | "extend_object"
                            | "included"
                            | "extended"
                    )
                {
                    let stub = Method::with_owner(
                        name_str.clone(),
                        vec!["target".to_string()],
                        vec![],
                        "Module".to_string(),
                    );
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                let msg = format!("undefined method '{}' for {}", name_str, class_rc.name());
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            // Module#instance_methods / public_/private_/protected_ variants.
            // The `false` argument restricts to methods defined directly on
            // this class (excluding inherited and mixin methods).
            "instance_methods"
            | "public_instance_methods"
            | "private_instance_methods"
            | "protected_instance_methods" => {
                let include_super = match arguments.first() {
                    Some(Object::Bool(b)) => *b,
                    _ => true,
                };
                let mut method_list: Vec<String> = class_rc.method_names();
                if include_super {
                    // Walk mixins on this class first, then the superclass
                    // chain (recursively pulling in each ancestor's mixins).
                    for mixin in class_rc.mixin_chain() {
                        for n in mixin.method_names() {
                            if !method_list.contains(&n) {
                                method_list.push(n);
                            }
                        }
                    }
                    let mut current = class_rc.superclass();
                    while let Some(sc) = current {
                        for n in sc.method_names() {
                            if !method_list.contains(&n) {
                                method_list.push(n);
                            }
                        }
                        for mixin in sc.mixin_chain() {
                            for n in mixin.method_names() {
                                if !method_list.contains(&n) {
                                    method_list.push(n);
                                }
                            }
                        }
                        current = sc.superclass();
                    }
                }
                // For the `Module` / `Class` receiver, advertise the native
                // mutation methods we actually implement so mspec matchers
                // (e.g. `have_public_instance_method(:alias_method, false)`)
                // recognise them as public instance methods.
                if matches!(class_rc.name(), "Module" | "Class") {
                    for n in [
                        "alias_method",
                        "attr",
                        "attr_accessor",
                        "attr_reader",
                        "attr_writer",
                        "define_method",
                        "include",
                        "method_defined?",
                        "prepend",
                        "instance_method",
                        "instance_methods",
                        "public_instance_methods",
                        "private_instance_methods",
                        "module_function",
                        "name",
                    ] {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push(n.to_string());
                        }
                    }
                }
                let mut priv_set: std::collections::HashSet<String> =
                    class_rc.private_method_names().into_iter().collect();
                // Module-private mixin hooks: append_features and friends
                // are private instance methods on Module. Class is *also* a
                // Module subclass — but `append_features` is undefined on
                // Class (Ruby sets it to undef), so only surface them when
                // the receiver is Module itself.
                if class_rc.name() == "Module" {
                    for n in [
                        "append_features",
                        "prepend_features",
                        "extend_object",
                        "extended",
                        "included",
                    ] {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push(n.to_string());
                        }
                        priv_set.insert(n.to_string());
                    }
                }
                if include_super {
                    for mixin in class_rc.mixin_chain() {
                        priv_set.extend(mixin.private_method_names());
                    }
                    let mut current = class_rc.superclass();
                    while let Some(sc) = current {
                        priv_set.extend(sc.private_method_names());
                        for mixin in sc.mixin_chain() {
                            priv_set.extend(mixin.private_method_names());
                        }
                        current = sc.superclass();
                    }
                }
                let filtered: Vec<Object> = method_list
                    .into_iter()
                    .filter(|n| !n.starts_with("__"))
                    .filter(|n| match method_name {
                        "private_instance_methods" => priv_set.contains(n),
                        "protected_instance_methods" => false, // not tracked yet
                        "public_instance_methods" | "instance_methods" => !priv_set.contains(n),
                        _ => true,
                    })
                    .map(|n| Object::Symbol(Rc::new(n)))
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    filtered,
                )))));
            }
            // attr_reader/attr_writer/attr_accessor as runtime instance
            // methods on Module/Class. Define the accessors on the receiver,
            // apply the receiver's current visibility, and return the array
            // of newly-defined method names as symbols (Ruby 3.0+).
            "attr_reader" | "attr_writer" | "attr_accessor" | "attr" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                // `attr name, true|false` is the deprecated 2-arg boolean form:
                // the second arg controls writer creation, and only the first
                // arg is a name. Ruby warns under `$VERBOSE = true`.
                let want_reader = matches!(method_name, "attr_reader" | "attr_accessor" | "attr");
                let mut want_writer = matches!(method_name, "attr_writer" | "attr_accessor");
                let names_slice: &[Object] = if method_name == "attr"
                    && arguments.len() == 2
                    && matches!(&arguments[1], Object::Bool(_))
                {
                    if matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
                        self.emit_warning_to_stderr(
                            "warning: optional boolean argument is obsoleted",
                            position,
                        );
                    }
                    want_writer = matches!(&arguments[1], Object::Bool(true));
                    &arguments[..1]
                } else {
                    arguments
                };
                let mut names: Vec<String> = Vec::with_capacity(names_slice.len());
                for arg in names_slice {
                    let n = self.coerce_method_name(arg, method_name, position)?;
                    names.push(n);
                }
                let visibility = class_rc.current_visibility();
                let mut defined: Vec<Object> = Vec::new();
                let mut newly_defined_names: Vec<String> = Vec::new();
                for attr_name in &names {
                    if want_reader {
                        let getter_body = vec![crate::ast::Statement::Return {
                            value: Some(crate::ast::Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            }),
                            position,
                        }];
                        let getter =
                            crate::object::Method::new(attr_name.clone(), vec![], getter_body);
                        class_rc.define_method(attr_name, Rc::new(getter));
                        if visibility != "public" {
                            class_rc.set_method_private(attr_name.clone());
                        }
                        class_rc.declare_instance_var(attr_name);
                        defined.push(Object::Symbol(Rc::new(attr_name.clone())));
                        newly_defined_names.push(attr_name.clone());
                    }
                    if want_writer {
                        let setter_body = vec![crate::ast::Statement::Assignment {
                            target: crate::ast::Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: crate::ast::Expression::Identifier {
                                name: "value".to_string(),
                                position,
                            },
                            position,
                        }];
                        let setter_name = format!("{}=", attr_name);
                        let setter = crate::object::Method::new(
                            setter_name.clone(),
                            vec!["value".to_string()],
                            setter_body,
                        );
                        class_rc.define_method(&setter_name, Rc::new(setter));
                        if visibility != "public" {
                            class_rc.set_method_private(setter_name.clone());
                        }
                        class_rc.declare_instance_var(attr_name);
                        defined.push(Object::Symbol(Rc::new(setter_name.clone())));
                        newly_defined_names.push(setter_name);
                    }
                }
                // Fire `method_added` (or `singleton_method_added` when the
                // receiver is a singleton class) for each method we just
                // installed, so user-defined hooks observe attr_* the same
                // way they observe `def`.
                let is_singleton = class_rc.get_class_var("__singleton__").is_some();
                let hook_name = if is_singleton {
                    "singleton_method_added"
                } else {
                    "method_added"
                };
                for added in &newly_defined_names {
                    self.invoke_class_hook(class_rc, hook_name, added, position)?;
                }
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    defined,
                )))));
            }
            // Module#method_defined?(name) — true when `name` resolves to a
            // public or protected instance method on the receiver, including
            // inherited methods. The optional second arg (default true) limits
            // the search to the receiver itself when false.
            "method_defined?"
            | "public_method_defined?"
            | "private_method_defined?"
            | "protected_method_defined?" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name = match &arguments[0] {
                    Object::String(s) => (**s).clone(),
                    Object::Symbol(s) => (**s).clone(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let include_super = match arguments.get(1) {
                    Some(Object::Bool(b)) => *b,
                    _ => true,
                };
                let mut found_class: Option<Rc<Class>> = None;
                if class_rc.find_method(&name).is_some() {
                    found_class = Some(Rc::clone(class_rc));
                } else if include_super {
                    'outer: for mixin in class_rc.mixin_chain() {
                        if mixin.find_method(&name).is_some() {
                            found_class = Some(mixin);
                            break 'outer;
                        }
                    }
                    if found_class.is_none() {
                        let mut current = class_rc.superclass();
                        while let Some(sc) = current {
                            if sc.find_method(&name).is_some() {
                                found_class = Some(sc);
                                break;
                            }
                            for mixin in sc.mixin_chain() {
                                if mixin.find_method(&name).is_some() {
                                    found_class = Some(mixin);
                                    break;
                                }
                            }
                            if found_class.is_some() {
                                break;
                            }
                            current = sc.superclass();
                        }
                    }
                }
                let answer = match found_class {
                    None => false,
                    Some(cls) => {
                        let is_private = cls.is_method_private(&name);
                        match method_name {
                            "method_defined?" | "public_method_defined?" => !is_private,
                            "private_method_defined?" => is_private,
                            // protected isn't tracked separately — say false.
                            "protected_method_defined?" => false,
                            _ => unreachable!(),
                        }
                    }
                };
                return Ok(Some(Object::Bool(answer)));
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
                // Drop the resolved constant (if any), any pending autoload
                // registration, and any "loaded but unrealized" bookkeeping.
                // Without removing all three, the name would still surface
                // in `#constants` because the constants list aggregates
                // class_vars + autoloads + unrealized autoloads.
                let removed = class_rc.remove_class_var(&const_name);
                let removed_autoload = class_rc.remove_autoload(&const_name);
                class_rc.clear_unrealized_autoload(&const_name);
                // Drop the recorded source location so a subsequent
                // `autoload` for the same name surfaces *its* location via
                // `const_source_location` instead of the stale class-def
                // location from before the removal.
                class_rc.remove_const_location(&const_name);
                let result = removed.unwrap_or_else(|| {
                    removed_autoload
                        .map(|p| Object::String(Rc::new(p)))
                        .unwrap_or(Object::Nil)
                });
                return Ok(Some(result));
            }
            "private" | "public" | "protected" => {
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
                let mut seen: Vec<*const Class> = Vec::new();
                push_class_ancestors(class_rc, &mut chain, &mut seen);
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(chain)))));
            }
            "const_defined?" => {
                // const_defined?(name [, inherit=true]) — when inherit is
                // false, only check the receiver itself; otherwise also
                // search ancestors and outer scope. The autoload registry
                // counts as defined (Ruby treats a registered autoload as
                // a constant entry).
                if arguments.is_empty() || arguments.len() > 2 {
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
                let inherit = !matches!(arguments.get(1), Some(Object::Bool(false)));
                // Thread-aware autoload check via `effective_autoload`:
                // the loading thread sees the autoload as cleared, other
                // threads still see it as registered. Mirrors `defined?`
                // / `autoload?` behaviour during a load.
                let local_autoload = if class_rc.get_autoload(&const_name).is_some() {
                    let cls = Rc::clone(class_rc);
                    self.effective_autoload(&cls, &const_name).is_some()
                } else {
                    false
                };
                let found_local = class_rc.get_class_var(&const_name).is_some() || local_autoload;
                let found = if inherit {
                    found_local
                        || class_rc.lookup_autoload(&const_name).is_some()
                        || self.environment().get(&const_name).is_some()
                        || self.globals().get(&const_name).is_some()
                } else {
                    found_local
                };
                return Ok(Some(Object::Bool(found)));
            }
            "const_source_location" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "const_source_location",
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
                            "const_source_location",
                            "Symbol or String",
                            other,
                            position,
                        ));
                    }
                };
                // Thread-aware: if this autoload is currently loading on
                // a different thread, return the autoload registration's
                // location (the constant isn't really defined from this
                // thread's view yet). Otherwise prefer the constant's
                // own definition site, falling back to the autoload
                // registration when only an autoload is registered.
                let current = self
                    .thread_current_stack
                    .last()
                    .cloned()
                    .unwrap_or(Object::Nil);
                let other_thread_loading = self.autoload_loading.iter().any(|(cls, n, loader)| {
                    if !Rc::ptr_eq(cls, class_rc) || n != &const_name {
                        return false;
                    }
                    let same = match (loader, &current) {
                        (Object::Nil, Object::Nil) => true,
                        (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
                        _ => false,
                    };
                    !same
                });
                let chosen = if other_thread_loading {
                    class_rc.get_autoload_location(&const_name)
                } else {
                    class_rc
                        .get_const_location(&const_name)
                        .or_else(|| class_rc.get_autoload_location(&const_name))
                };
                let loc_obj = match chosen {
                    Some((file, line)) => Object::Array(Rc::new(std::cell::RefCell::new(vec![
                        Object::String(Rc::new(file)),
                        Object::Int(line),
                    ]))),
                    None => Object::Nil,
                };
                return Ok(Some(loc_obj));
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
                // const_get triggers autoload like a `Mod::Const` reference
                // would. Without firing it here, callers like
                // `Mod.const_get(:Foo)` would NameError even when `:Foo`
                // is a registered autoload.
                if let Some(val) = self.try_autoload_constant(class_rc, &const_name)? {
                    return Ok(Some(val));
                }
                // Top-level constants live on globals rather than on Object's
                // class_vars in our model. Ruby semantics treat `Object` as
                // the home of top-level constants, so `Object.const_get(:Foo)`
                // should resolve a top-level `Foo`. Walk superclasses too so
                // any class inheriting from Object inherits this view.
                let mut current: Option<Rc<crate::class::Class>> = Some(Rc::clone(class_rc));
                while let Some(cls) = current {
                    if cls.name() == "Object"
                        && let Some(val) = self.globals().get(&const_name)
                    {
                        return Ok(Some(val));
                    }
                    current = cls.superclass();
                }
                let msg = format!("uninitialized constant {}", const_name);
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
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
                let const_name = match &arguments[0] {
                    Object::Symbol(s) => s.as_ref().clone(),
                    Object::String(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "const_set",
                            "Symbol or String",
                            other,
                            position,
                        ));
                    }
                };
                if !is_valid_constant_name(&const_name) {
                    let msg = format!("wrong constant name {}", const_name);
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Setting the constant cancels any pending autoload for it
                // and clears any "loaded but unrealized" bookkeeping.
                class_rc.remove_autoload(&const_name);
                class_rc.clear_unrealized_autoload(&const_name);
                class_rc.set_class_var(&const_name, arguments[1].clone());
                return Ok(Some(arguments[1].clone()));
            }
            "class_eval" | "module_eval" => {
                let result = self.class_eval_with_args(
                    class_rc,
                    Object::Class(Rc::clone(class_rc)),
                    arguments,
                    position,
                )?;
                return Ok(Some(result));
            }
            "class_exec" | "module_exec" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    _ => return Err(local_jump_error(method_name, position)),
                };
                let result = self.class_exec_block(
                    class_rc,
                    Object::Class(Rc::clone(class_rc)),
                    &block,
                    arguments.to_vec(),
                    position,
                )?;
                return Ok(Some(result));
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
                let new_name = self.coerce_method_name(&arguments[0], "alias_method", position)?;
                let old_name = self.coerce_method_name(&arguments[1], "alias_method", position)?;
                if class_rc.is_frozen() {
                    let msg = format!("can't modify frozen Class: {}", class_rc.name());
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                if !class_rc.alias_method(&new_name, &old_name) {
                    let mut found = false;
                    if let Some(Object::Class(object_class)) = self.globals().get("Object")
                        && let Some(method) = object_class.find_method(&old_name)
                    {
                        class_rc.define_method(&new_name, method);
                        found = true;
                    }
                    if !found {
                        let msg = format!(
                            "undefined method '{}' for class '{}'",
                            old_name,
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
                if matches!(
                    new_name.as_str(),
                    "initialize"
                        | "initialize_copy"
                        | "initialize_clone"
                        | "initialize_dup"
                        | "respond_to_missing?"
                ) {
                    class_rc.set_method_private(new_name.clone());
                }
                return Ok(Some(Object::Symbol(Rc::new(new_name))));
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
            "class_variable_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "class_variable_set",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(&arguments[0], position)?;
                class_rc.set_class_var(key, arguments[1].clone());
                return Ok(Some(arguments[1].clone()));
            }
            "class_variable_get" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "class_variable_get",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(&arguments[0], position)?;
                match class_rc.lookup_class_var(&key) {
                    Some(value) => return Ok(Some(value)),
                    None => {
                        let msg = format!(
                            "uninitialized class variable @@{} in {}",
                            key,
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
            }
            "class_variable_defined?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "class_variable_defined?",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(&arguments[0], position)?;
                return Ok(Some(Object::Bool(
                    class_rc.lookup_class_var(&key).is_some(),
                )));
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

    /// Coerce a class-variable name argument to its storage key (the name with
    /// the leading `@@` removed). Strings and Symbols are used directly; any
    /// other object is converted via `to_str`. Raises TypeError when that
    /// conversion is missing or returns a non-String, and NameError when the
    /// resulting name is not a valid class variable name.
    pub(crate) fn coerce_class_variable_name(
        &mut self,
        arg: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let name = match arg {
            Object::Symbol(s) => (**s).clone(),
            Object::String(s) => (**s).clone(),
            other => {
                let other_obj = other.clone();
                let converted =
                    if let Some((cls, method)) = self.lookup_method(&other_obj, "to_str") {
                        self.invoke_method(cls, method, other_obj, Vec::new(), position)?
                    } else {
                        let msg = format!(
                            "no implicit conversion of {} into String",
                            other.type_name()
                        );
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    };
                match converted {
                    Object::String(s) => (*s).clone(),
                    other => {
                        let msg = format!("can't convert {} to String", other.type_name());
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                }
            }
        };
        if let Some(rest) = name.strip_prefix("@@")
            && is_valid_class_variable_ident(rest)
        {
            return Ok(rest.to_string());
        }
        let msg = format!("`{}' is not allowed as a class variable name", name);
        let exc = Object::exception("NameError", msg.clone());
        Err(MetorexError::UncaughtException {
            exception: exc,
            location: position_to_location(position),
            message: msg,
        })
    }

    /// Invoke a `method_added` / `singleton_method_added` hook on `class_rc` if
    /// the user defined one. The method receives the new method's name as a
    /// symbol; errors raised by the hook propagate.
    ///
    /// For `singleton_method_added`, Ruby fires the hook on the *attached
    /// object* (the object whose singleton class gained the method), not on
    /// the singleton class itself — so when `class_rc` is a singleton class we
    /// pivot to the attached object before lookup.
    pub(crate) fn invoke_class_hook(
        &mut self,
        class_rc: &Rc<Class>,
        hook: &str,
        added_name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let arg = Object::Symbol(Rc::new(added_name.to_string()));

        if hook == "singleton_method_added"
            && class_rc.get_class_var("__singleton__").is_some()
            && let Some(attached) = class_rc.get_class_var("__attached__")
        {
            let attached_class = match &attached {
                Object::Class(c) | Object::Module(c) => Some(Rc::clone(c)),
                _ => None,
            };
            if let Some(target_class) = attached_class
                && let Some(method) = target_class
                    .singleton_class_slot()
                    .clone()
                    .and_then(|sc| sc.find_method(hook))
            {
                let sc = target_class.singleton_class_slot().clone().unwrap();
                self.invoke_method(sc, method, attached.clone(), vec![arg], position)?;
                return Ok(());
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

/// Append the transitive ancestor chain of a module (including itself and all
/// modules it mixes in, recursively) onto `chain`. Uses pointer identity in
/// `seen` to skip modules that have already been added, matching Ruby's
/// dedup-on-first-sighting semantics.
pub(super) fn push_module_ancestors(
    module: &Rc<Class>,
    chain: &mut Vec<Object>,
    seen: &mut Vec<*const Class>,
) {
    let ptr = Rc::as_ptr(module);
    if seen.contains(&ptr) {
        return;
    }
    seen.push(ptr);
    chain.push(Object::Module(Rc::clone(module)));
    for mixin in module.mixin_chain() {
        push_module_ancestors(&mixin, chain, seen);
    }
}

/// Append the full ancestor chain of a class (class itself, its mixins
/// recursively, then each superclass with its own mixins) onto `chain`.
pub(super) fn push_class_ancestors(
    class: &Rc<Class>,
    chain: &mut Vec<Object>,
    seen: &mut Vec<*const Class>,
) {
    let ptr = Rc::as_ptr(class);
    if !seen.contains(&ptr) {
        seen.push(ptr);
        chain.push(Object::Class(Rc::clone(class)));
    }
    for mixin in class.mixin_chain() {
        push_module_ancestors(&mixin, chain, seen);
    }
    let mut current = class.superclass();
    while let Some(parent) = current {
        let pptr = Rc::as_ptr(&parent);
        if !seen.contains(&pptr) {
            seen.push(pptr);
            chain.push(Object::Class(Rc::clone(&parent)));
        }
        for mixin in parent.mixin_chain() {
            push_module_ancestors(&mixin, chain, seen);
        }
        current = parent.superclass();
    }
}

/// Validate the identifier portion of a class variable name (the part after
/// `@@`): it must start with a letter or underscore and contain only
/// alphanumerics and underscores.
fn is_valid_class_variable_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
