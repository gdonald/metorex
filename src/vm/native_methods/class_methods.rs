use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::native_methods::is_valid_constant_name;
use crate::vm::utils::{is_truthy, position_to_location};
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn call_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Some(result) = self.call_warning_methods(class_rc, method_name, arguments) {
            return Ok(Some(result));
        }
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
                let names: Vec<Object> = names
                    .into_iter()
                    .map(|n| Object::Symbol(Rc::new(n)))
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(names)))));
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
            if inherit {
                let mut queue: Vec<Rc<Class>> = class_rc.mixin_chain();
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
                    for mixin in current.mixin_chain() {
                        queue.push(mixin);
                    }
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
        if class_rc.name() == "Kernel"
            && let Some(result) = self.call_kernel_conversion(method_name, arguments, position)?
        {
            return Ok(Some(result));
        }
        if class_rc.name() == "Kernel" && method_name == "abort" {
            return self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some);
        }
        // `Kernel.block_given?` reports on the frame that called it, the same
        // as the bare form.
        if class_rc.name() == "Kernel" && method_name == "block_given?" {
            return Ok(Some(Object::Bool(matches!(
                self.environment().get("block_given?"),
                Some(Object::Bool(true))
            ))));
        }
        if class_rc.name() == "Kernel" && method_name == "binding" {
            return self
                .call_native_function("binding_kernel", arguments.to_vec(), position)
                .map(Some);
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
            let anon = Rc::new(Class::new_module(""));
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
                let msg = format!(
                    "can't modify frozen {}: {}",
                    class_rc.kind_name(),
                    class_rc.name()
                );
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
            class_rc.set_autoload(const_name.clone(), path);
            self.trigger_const_added_hook(
                Object::Class(Rc::clone(class_rc)),
                &const_name,
                position,
            )?;
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
            return Ok(Some(Object::Nil));
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
            return Ok(Some(Object::Bool(found)));
        }
        // Bare `include` / `prepend` inside a class or module body: Ruby
        // reports the missing argument rather than a missing method.
        if matches!(method_name, "include" | "prepend")
            && arguments.is_empty()
            && !has_user_defined_method(class_rc, method_name)
        {
            return Err(method_argument_error(method_name, 1, 0, position));
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
        // Regexp.escape / Regexp.quote: the string with every regex
        // metacharacter escaped, so it matches itself literally.
        if class_rc.name() == "Regexp"
            && matches!(method_name, "escape" | "quote")
            && arguments.len() == 1
        {
            let source = self.coerce_name_argument(&arguments[0], position)?;
            return Ok(Some(Object::String(Rc::new(regex::escape(&source)))));
        }
        // Module.used_refinements: the refinements `using` has brought into
        // the current scope.
        if method_name == "used_refinements" && class_rc.name() == "Module" {
            let refinements = self.active_refinements();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                refinements,
            )))));
        }
        if method_name == "nesting" && class_rc.name() == "Module" {
            // Inside a method body the answer is the nesting captured where
            // the method was defined; elsewhere it is the scopes open here.
            let scopes = match self.method_nesting_stack.last() {
                Some(captured) => captured.clone(),
                None => self.snapshot_lexical_nesting(),
            };
            let nesting: Vec<Object> = scopes
                .into_iter()
                .map(|scope| {
                    if scope.is_module() {
                        Object::Module(scope)
                    } else {
                        Object::Class(scope)
                    }
                })
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
        // serialized "concurrent" specs (which set a shared flag between
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
            // `Proc.new { ... }` yields the block itself — there is no
            // separate instance to build.
            "new" if Rc::ptr_eq(class_rc, &self.builtins().proc_class) => {
                if let Some(block) = self.pending_block.take() {
                    return Ok(Some(block));
                }
                let msg = "tried to create Proc object without a block";
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", msg.to_string()),
                    location: position_to_location(position),
                    message: msg.to_string(),
                });
            }
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
                // A singleton class has no name of its own, even though it
                // displays as `#<Class:Something>`.
                let name = class_rc.ruby_name();
                if name.is_empty() || class_rc.is_singleton_class() {
                    return Ok(Some(Object::Nil));
                }
                return Ok(Some(Object::String(Rc::new(name))));
            }
            // Module#instance_method / Class#instance_method: returns the
            // bound `Method` object so `parameters` and friends work on it.
            "instance_method" | "public_instance_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = self.coerce_method_name(&arguments[0], method_name, position)?;
                if let Some((owner, method)) = class_rc.find_method_with_owner(&name_str) {
                    // `public_instance_method` only hands out public methods.
                    if method.is_undefined
                        || (method_name == "public_instance_method"
                            && (owner.is_method_restricted(&name_str)
                                || self.method_is_restricted(
                                    &Object::Class(Rc::clone(class_rc)),
                                    &name_str,
                                )))
                    {
                        return Err(undefined_instance_method_error(
                            &name_str, class_rc, position,
                        ));
                    }
                    if method.owner_class.is_some() {
                        return Ok(Some(Object::Method(method)));
                    }
                    let mut unbound = (*method).clone();
                    unbound.owner = Some(owner.name().to_string());
                    unbound.owner_class = Some(owner);
                    return Ok(Some(Object::Method(Rc::new(unbound))));
                }
                // Synthesize a stub for well-known Module-private mixin
                // hooks so `Module.instance_method(:append_features)` works
                // for spec patterns that bind/call them.
                if class_rc.name() == "Module" && MODULE_PRIVATE_HOOKS.contains(&name_str.as_str())
                {
                    let stub = Method::with_owner(
                        name_str.clone(),
                        vec!["target".to_string()],
                        vec![],
                        "Module".to_string(),
                    );
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                // The rest of Module's own methods are implemented natively,
                // so hand out a stub carrying the right parameter list.
                if matches!(class_rc.name(), "Module" | "Class")
                    && let Some(stub) = native_module_method_stub(&name_str)
                {
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                // Kernel methods are implemented natively rather than living
                // in Object's method table. A body-less stub reaches the same
                // native implementation when invoked, so `Object` can hand out
                // an UnboundMethod for them.
                if matches!(class_rc.name(), "Object" | "Kernel")
                    && is_native_kernel_method(&name_str)
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        "Kernel".to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    return Ok(Some(Object::Method(Rc::new(stub))));
                }
                return Err(undefined_instance_method_error(
                    &name_str, class_rc, position,
                ));
            }
            // Module#undefined_instance_methods: the names this class itself
            // has undefined with `undef_method`, not those its ancestors did.
            "undefined_instance_methods" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let undefined: Vec<Object> = class_rc
                    .method_names()
                    .into_iter()
                    .filter(|name| {
                        class_rc
                            .find_own_method(name)
                            .is_some_and(|method| method.is_undefined)
                    })
                    .map(|name| Object::Symbol(Rc::new(name)))
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    undefined,
                )))));
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
                // A `private`/`public` naming an inherited method marks the
                // visibility here without defining anything, and Ruby counts
                // that name among this class's own methods.
                let object_class = match self.globals().get("Object") {
                    Some(Object::Class(object_class)) => Some(object_class),
                    _ => None,
                };
                for name in class_rc.visibility_marked_names() {
                    let resolves = class_rc.find_method(&name).is_some()
                        || object_class
                            .as_ref()
                            .is_some_and(|oc| oc.find_method(&name).is_some());
                    if !method_list.contains(&name) && resolves {
                        method_list.push(name);
                    }
                }
                if include_super {
                    // The ancestor walk already covers mixins of mixins and
                    // each superclass's mixins.
                    let mut chain: Vec<Object> = Vec::new();
                    let mut seen: Vec<*const Class> = Vec::new();
                    push_class_ancestors(class_rc, &mut chain, &mut seen);
                    for ancestor in &chain {
                        let (Object::Class(c) | Object::Module(c)) = ancestor else {
                            continue;
                        };
                        for n in c.method_names() {
                            if !method_list.contains(&n) {
                                method_list.push(n);
                            }
                        }
                    }
                }
                // For the `Module` / `Class` receiver, advertise the native
                // mutation methods we actually implement so mspec matchers
                // (e.g. `have_public_instance_method(:alias_method, false)`)
                // recognize them as public instance methods.
                if matches!(class_rc.name(), "Module" | "Class") {
                    for (n, _, _) in NATIVE_MODULE_METHODS {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                    }
                }
                // A method removed with `undef_method` stays in the table as
                // a tombstone so lookups stop at it; it is not an instance
                // method any more.
                method_list.retain(|n| {
                    class_rc
                        .find_method(n)
                        .is_none_or(|method| !method.is_undefined)
                });
                // A name's visibility comes from the nearest ancestor that
                // defines it: an ancestor further along may mark its own copy
                // private without that reaching the one in front.
                let mut priv_set: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                let mut protected_set: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                // Module-private mixin hooks: append_features and friends
                // are private instance methods on Module. Class is *also* a
                // Module subclass — but `append_features` is undefined on
                // Class (Ruby sets it to undef), so only surface them when
                // the receiver is Module itself.
                if class_rc.name() == "Module" {
                    for n in MODULE_PRIVATE_HOOKS
                        .iter()
                        .chain(MODULE_PRIVATE_DECLARATIONS.iter())
                    {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                        priv_set.insert((*n).to_string());
                    }
                }
                // Kernel's conversion functions are private instance methods
                // on Kernel, implemented natively rather than in its table.
                if class_rc.name() == "Kernel" {
                    for n in
                        crate::vm::native_methods::kernel_conversion::KERNEL_CONVERSION_FUNCTIONS
                            .iter()
                            .chain(KERNEL_PRIVATE_FUNCTIONS.iter())
                    {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                        priv_set.insert((*n).to_string());
                    }
                }
                let visibility_chain: Vec<Rc<Class>> = if include_super {
                    let mut chain: Vec<Object> = Vec::new();
                    let mut seen: Vec<*const Class> = Vec::new();
                    push_class_ancestors(class_rc, &mut chain, &mut seen);
                    chain
                        .iter()
                        .filter_map(|ancestor| match ancestor {
                            Object::Class(c) | Object::Module(c) => Some(Rc::clone(c)),
                            _ => None,
                        })
                        .collect()
                } else {
                    vec![Rc::clone(class_rc)]
                };
                for name in &method_list {
                    for ancestor in &visibility_chain {
                        if ancestor.has_public_override(name) {
                            break;
                        }
                        if ancestor.is_method_private(name) {
                            priv_set.insert(name.clone());
                            break;
                        }
                        if ancestor.is_method_protected(name) {
                            protected_set.insert(name.clone());
                            break;
                        }
                        if ancestor.find_own_method(name).is_some() {
                            break;
                        }
                    }
                }
                let filtered: Vec<Object> = method_list
                    .into_iter()
                    .filter(|n| !n.starts_with("__"))
                    .filter(|n| match method_name {
                        "private_instance_methods" => priv_set.contains(n),
                        "protected_instance_methods" => protected_set.contains(n),
                        "public_instance_methods" => {
                            !priv_set.contains(n) && !protected_set.contains(n)
                        }
                        // Ruby's `instance_methods` covers public and
                        // protected alike.
                        "instance_methods" => !priv_set.contains(n),
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
                let name = self.coerce_method_name(&arguments[0], method_name, position)?;
                let include_super = match arguments.get(1) {
                    Some(Object::Bool(b)) => *b,
                    _ => true,
                };
                let found = if include_super {
                    class_rc.find_method_with_owner(&name)
                } else {
                    class_rc
                        .find_own_method(&name)
                        .map(|method| (Rc::clone(class_rc), method))
                };
                let answer = match found {
                    // A tombstone left by `undef_method` is not a definition.
                    Some((_, method)) if method.is_undefined => false,
                    None => false,
                    Some((owner, _)) => {
                        let is_private = owner.is_method_private(&name);
                        let is_protected = owner.is_method_protected(&name);
                        match method_name {
                            "method_defined?" => !is_private,
                            "public_method_defined?" => !is_private && !is_protected,
                            "private_method_defined?" => is_private,
                            "protected_method_defined?" => is_protected,
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
                // Ruby takes a module here and rejects a class.
                let module_rc = match &arguments[0] {
                    Object::Module(m) => Rc::clone(m),
                    other => {
                        return Err(method_argument_type_error(
                            "extend", "Module", other, position,
                        ));
                    }
                };
                let target = Object::Class(Rc::clone(class_rc));
                self.apply_module_extend(&target, &module_rc, position)?;
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
                // A lone array argument names several methods at once.
                let named: Vec<Object> = match arguments {
                    [Object::Array(names)] => names.borrow().clone(),
                    other => other.to_vec(),
                };
                let target_class = Object::Class(Rc::clone(class_rc));
                let singleton = self.singleton_class_of(&target_class);
                for argument in &named {
                    let name = self.coerce_method_name(argument, method_name, position)?;
                    // Mirror the method onto the singleton class so
                    // `lookup_method` finds it there (where visibility lives).
                    // The `inherited` hook is inherited from Class's singleton
                    // table via the `__class__` convention — copy it across
                    // so we have something to toggle visibility on.
                    if singleton.find_method(&name).is_none() {
                        match self.class_method_of(class_rc, &name) {
                            Some(method) => singleton.define_method(&name, method),
                            None => {
                                let msg = format!(
                                    "undefined method '{}' for {} '{}'",
                                    name,
                                    class_rc.kind_name().to_lowercase(),
                                    class_rc.ruby_name()
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
                    if method_name == "private_class_method" {
                        singleton.set_method_private(&name);
                    } else {
                        singleton.set_method_public(&name);
                    }
                }
                return Ok(Some(Object::Class(Rc::clone(class_rc))));
            }
            // Module#set_temporary_name: a display name for an anonymous
            // module, cleared by passing nil. A permanent name cannot be
            // replaced, and the name may not look like a constant path.
            "set_temporary_name" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if class_rc.has_permanent_name() {
                    let msg = "can't change permanent name".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("RuntimeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let receiver = if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                };
                if matches!(arguments[0], Object::Nil) {
                    class_rc.set_temporary_name(None);
                    return Ok(Some(receiver));
                }
                let name = self.coerce_name_argument(&arguments[0], position)?;
                let complaint = if name.is_empty() {
                    Some("empty class/module name")
                } else if looks_like_constant_path(&name) {
                    Some("the temporary name must not be a constant path to avoid confusion")
                } else {
                    None
                };
                if let Some(msg) = complaint {
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", msg.to_string()),
                        location: position_to_location(position),
                        message: msg.to_string(),
                    });
                }
                class_rc.set_temporary_name(Some(name));
                return Ok(Some(receiver));
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
                let const_name = self.coerce_name_argument(&arguments[0], position)?;
                if !is_valid_constant_name(&const_name) {
                    let msg = format!("wrong constant name {}", const_name);
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Only a constant of the receiver itself can be removed, and
                // a pending autoload counts as one.
                if class_rc.get_class_var(&const_name).is_none()
                    && class_rc.get_autoload(&const_name).is_none()
                    && !class_rc.unrealized_autoload_names().contains(&const_name)
                    && !(class_rc.name() == "Object" && self.globals().contains(&const_name))
                {
                    let msg = format!(
                        "constant {}::{} not defined",
                        class_rc.ruby_name(),
                        const_name
                    );
                    let exc = Object::exception("NameError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Drop the resolved constant (if any), any pending autoload
                // registration, and any "loaded but unrealized" bookkeeping.
                // Without removing all three, the name would still surface
                // in `#constants` because the constants list aggregates
                // class_vars + autoloads + unrealized autoloads.
                self.warn_deprecated_constant(class_rc, &const_name, position);
                let mut removed = class_rc.remove_class_var(&const_name);
                // Object's constants are top-level constants — drop the
                // globals binding too so bare references stop resolving.
                if class_rc.name() == "Object" {
                    let from_globals = self.globals_mut().remove(&const_name);
                    if removed.is_none() {
                        removed = from_globals;
                    }
                }
                let removed_autoload = class_rc.remove_autoload(&const_name);
                class_rc.clear_unrealized_autoload(&const_name);
                // Drop the recorded source location so a subsequent
                // `autoload` for the same name surfaces *its* location via
                // `const_source_location` instead of the stale class-def
                // location from before the removal.
                class_rc.remove_const_location(&const_name);
                // Removing a constant that was only registered for autoload
                // answers with nil: it never held a value.
                let _ = removed_autoload;
                return Ok(Some(removed.unwrap_or(Object::Nil)));
            }
            "private" | "public" | "protected" => {
                return self
                    .apply_class_visibility_modifier(class_rc, method_name, arguments, position)
                    .map(Some);
            }
            "private_methods" => {
                let include_super = !matches!(arguments.first(), Some(Object::Bool(false)));
                let mut names: Vec<String> = class_rc.private_method_names();
                // Classes and modules inherit Module's private instance
                // methods. `extend_object` and the `*_features` pair are
                // undefined on Class, so the module dispatch path adds those.
                names.extend(
                    [
                        "extended",
                        "included",
                        "prepended",
                        "const_added",
                        "method_added",
                    ]
                    .map(String::from),
                );
                names.extend(MODULE_PRIVATE_DECLARATIONS.iter().map(|n| (*n).to_string()));
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
            // Module#singleton_class?: whether this is the class of exactly
            // one object, as `class << obj` opens.
            "singleton_class?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                return Ok(Some(Object::Bool(class_rc.is_singleton_class())));
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
            // The ancestors that are modules rather than classes, with the
            // receiver itself left out.
            "included_modules" => {
                let mut chain: Vec<Object> = Vec::new();
                let mut seen: Vec<*const Class> = Vec::new();
                push_class_ancestors(class_rc, &mut chain, &mut seen);
                let modules: Vec<Object> = chain
                    .into_iter()
                    .filter(|ancestor| match ancestor {
                        Object::Module(m) => !Rc::ptr_eq(m, class_rc),
                        _ => false,
                    })
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    modules,
                )))));
            }
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
                        _ => return Ok(Some(Object::Bool(false))),
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
                        return Ok(Some(Object::Bool(entry.is_some())));
                    }
                    // Intermediate segments must resolve to a class/module
                    // value; a registered-but-unloaded autoload can't be
                    // traversed without triggering the load.
                    match entry {
                        Some((_, Some(Object::Class(c)))) | Some((_, Some(Object::Module(c)))) => {
                            current = c;
                        }
                        _ => return Ok(Some(Object::Bool(false))),
                    }
                }
                return Ok(Some(Object::Bool(false)));
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
                        Some((file, line)) => {
                            vec![Object::String(Rc::new(file)), Object::Int(line)]
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
                            return Ok(Some(loc_array(current.get_autoload_location(seg))));
                        }
                        let entry = self.const_entry_on(&current, seg, inherit, i == 0);
                        return Ok(Some(match entry {
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
                        _ => return Ok(Some(Object::Nil)),
                    }
                }
                return Ok(Some(Object::Nil));
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
                                return Ok(Some(missing));
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
                return Ok(Some(value));
            }
            // Default `Module#const_added` — a no-op returning nil. User
            // hooks (`def self.const_added`) are dispatched before native
            // fallback, so this only fires for the base implementation.
            "const_added" => {
                // Native dispatch runs before the user method body in
                // `invoke_method`, so step aside when a user hook exists.
                if class_rc.find_method("__class__const_added").is_some() {
                    return Ok(None);
                }
                let mut cursor = Some(Rc::clone(class_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && sc.find_method("const_added").is_some()
                    {
                        return Ok(None);
                    }
                    cursor = current.superclass();
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "const_added",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                return Ok(Some(Object::Nil));
            }
            // Default `Module#const_missing` — raise NameError with the
            // qualified constant path and the `name` attribute set. User
            // hooks step aside the same way const_added's do.
            "const_missing" => {
                if class_rc.find_method("__class__const_missing").is_some() {
                    return Ok(None);
                }
                let mut cursor = Some(Rc::clone(class_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && sc.find_method("const_missing").is_some()
                    {
                        return Ok(None);
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
                    Object::Symbol(s) => s.as_ref().clone(),
                    Object::String(s) => s.as_ref().clone(),
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
                    .map(Some);
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
                    .get_current_file()
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
                return self
                    .module_define_method(class_rc, arguments, position)
                    .map(Some);
            }
            "remove_method" => {
                // Ruby accepts any number of names, including none, and
                // answers with the receiver.
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    names.push(self.coerce_method_name(argument, method_name, position)?);
                }
                if class_rc.is_frozen() && !names.is_empty() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.ruby_name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                for name in names {
                    if !class_rc.remove_method(&name) {
                        let msg =
                            format!("method '{}' not defined in {}", name, class_rc.ruby_name());
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    self.invoke_class_hook(class_rc, "method_removed", &name, position)?;
                }
                return Ok(Some(if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                }));
            }
            "undef_method" => {
                // Like `remove_method`, this takes any number of names and
                // answers with the receiver. The method must exist somewhere
                // in the ancestry, though it need not be the receiver's own.
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    names.push(self.coerce_method_name(argument, method_name, position)?);
                }
                if class_rc.is_frozen() && !names.is_empty() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.ruby_name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                for name in names {
                    // Kernel methods live in the native dispatch tables rather
                    // than in a class's method map, so they count as present.
                    if class_rc
                        .find_method(&name)
                        .is_none_or(|method| method.is_undefined)
                        && !is_native_kernel_method(&name)
                    {
                        let msg = format!(
                            "undefined method '{}' for {} '{}'",
                            name,
                            class_rc.kind_name().to_lowercase(),
                            undef_target_name(class_rc)
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        if let Object::Exception(cell) = &exc {
                            cell.borrow_mut().name = Some(name.clone());
                        }
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    let sentinel = Method::undefined(name.clone());
                    class_rc.define_method(&name, Rc::new(sentinel));
                    self.invoke_class_hook(class_rc, "method_undefined", &name, position)?;
                }
                return Ok(Some(if class_rc.is_module() {
                    Object::Module(Rc::clone(class_rc))
                } else {
                    Object::Class(Rc::clone(class_rc))
                }));
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
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.name()
                    );
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
                self.invoke_class_hook(class_rc, "method_added", &new_name, position)?;
                return Ok(Some(Object::Symbol(Rc::new(new_name))));
            }
            "module_function" => {
                // Ruby undefines `module_function` on Class, so a rebound
                // call with a class receiver is a TypeError.
                if !class_rc.is_module() {
                    let msg = "module_function must be called for modules".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // With no arguments it is a toggle: every method defined
                // afterwards in the body becomes a module function.
                if arguments.is_empty() {
                    class_rc.set_current_visibility(MODULE_FUNCTION_VISIBILITY);
                    return Ok(Some(Object::Nil));
                }
                let mut names = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    let name = self.coerce_method_name(argument, method_name, position)?;
                    self.copy_to_module_function(class_rc, &name, position)?;
                    names.push(Object::Symbol(Rc::new(name)));
                }
                return Ok(Some(match names.len() {
                    1 => names.remove(0),
                    _ => Object::Array(Rc::new(std::cell::RefCell::new(names))),
                }));
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
                if class_rc.is_frozen() {
                    let msg = format!(
                        "can't modify frozen {}: {}",
                        class_rc.kind_name(),
                        class_rc.name()
                    );
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
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
            // Module#remove_class_variable: only a variable defined directly
            // on the receiver can be removed, and its value comes back.
            "remove_class_variable" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key = self.coerce_class_variable_name(&arguments[0], position)?;
                match class_rc.remove_class_var(&key) {
                    Some(value) => return Ok(Some(value)),
                    None => {
                        let msg = format!(
                            "class variable @@{} not defined for {}",
                            key,
                            class_rc.ruby_name()
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
            "class_variables" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        "class_variables",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // `class_variables(inherit = true)` — when inherit is false,
                // only this class/module's own class variables are reported.
                let inherit = arguments.first().map(is_truthy).unwrap_or(true);
                let names = if inherit {
                    class_rc.inherited_class_variable_names()
                } else {
                    class_rc.own_class_variable_names()
                };
                let symbols: Vec<Object> = names
                    .into_iter()
                    .map(|n| Object::Symbol(Rc::new(format!("@@{}", n))))
                    .collect();
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    symbols,
                )))));
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

    /// Coerce a name argument to a String: Strings and Symbols are used
    /// directly, anything else goes through `to_str`. Raises TypeError when
    /// that conversion is missing or returns a non-String.
    pub(crate) fn coerce_name_argument(
        &mut self,
        arg: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match arg {
            Object::Symbol(s) => Ok((**s).clone()),
            Object::String(s) => Ok((**s).clone()),
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
                    Object::String(s) => Ok((*s).clone()),
                    other => {
                        let msg = format!("can't convert {} to String", other.type_name());
                        let exc = Object::exception("TypeError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                }
            }
        }
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
        let name = self.coerce_name_argument(arg, position)?;
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
                vec![Object::Symbol(Rc::new(name.to_string()))],
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
            e.borrow_mut().name = Some(name.to_string());
        }
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
    /// The class-level method `name` on `class_rc`: one stored under the
    /// `__class__` convention, one on a singleton class along the superclass
    /// chain, or one copied in by `extend`.
    fn class_method_of(&mut self, class_rc: &Rc<Class>, name: &str) -> Option<Rc<Method>> {
        if let Some(method) = class_rc.find_method(&format!("__class__{}", name)) {
            return Some(method);
        }
        if let Some(Object::Method(method)) = class_rc.get_class_var(&format!("__ext__{}", name)) {
            return Some(method);
        }
        let mut cursor = Some(Rc::clone(class_rc));
        while let Some(current) = cursor {
            if let Some(sc) = current.singleton_class_slot().clone()
                && let Some(method) = sc.find_method(name)
            {
                return Some(method);
            }
            cursor = current.superclass();
        }
        None
    }

    /// Copy an instance method to the module object as a module function:
    /// the copy is a public module-level method and the original becomes a
    /// private instance method.
    pub(crate) fn copy_to_module_function(
        &mut self,
        module_rc: &Rc<Class>,
        name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let method = module_rc.find_method(name).or_else(|| {
            // Kernel methods live on Object, which a module does not
            // inherit from; `module_function :require` copies from there.
            match self.globals().get("Object") {
                Some(Object::Class(object_class)) => object_class.find_method(name),
                _ => None,
            }
        });
        let method = match method {
            Some(method) => method,
            // Kernel's own methods are native rather than table entries; a
            // stub reaches the same implementation when invoked.
            None if is_native_kernel_method(name) => {
                let mut stub = Method::with_owner(
                    name.to_string(),
                    vec!["args".to_string()],
                    vec![],
                    "Kernel".to_string(),
                );
                stub.variadic_param = Some((0, "args".to_string()));
                Rc::new(stub)
            }
            None => {
                return Err(MetorexError::runtime_error(
                    format!(
                        "undefined method '{}' for module '{}'",
                        name,
                        module_rc.name()
                    ),
                    position_to_location(position),
                ));
            }
        };
        module_rc.define_method(format!("__class__{}", name), Rc::clone(&method));
        if module_rc.find_own_method(name).is_some() {
            module_rc.set_method_private(name.to_string());
        }
        self.invoke_class_hook(module_rc, "singleton_method_added", name, position)?;
        Ok(())
    }

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

/// The visibility state a bare `module_function` sets in a module body. Every
/// method defined afterwards is copied to the module object and made private
/// as an instance method.
pub(crate) const MODULE_FUNCTION_VISIBILITY: &str = "module_function";

/// Kernel's process-control functions, which Ruby exposes as private instance
/// methods on Kernel and as public singleton methods on the module.
pub(super) const KERNEL_PRIVATE_FUNCTIONS: &[&str] = &[
    "abort",
    "binding",
    "block_given?",
    "catch",
    "fail",
    "gets",
    "global_variables",
    "initialize_clone",
    "initialize_copy",
    "initialize_dup",
    "throw",
];

/// The hooks Module defines as private instance methods with a no-op default
/// implementation. Each takes one argument and returns nil unless the module
/// overrides it.
pub(super) const MODULE_PRIVATE_HOOKS: &[&str] = &[
    "append_features",
    "prepend_features",
    "extend_object",
    "extended",
    "included",
    "prepended",
    "const_added",
    "method_added",
    "method_removed",
    "method_undefined",
];

/// Module's private instance methods beyond the hooks: the declarations a
/// class or module body calls without a receiver. `alias_method` and
/// `define_method` are deliberately absent, being public in Ruby.
pub(super) const MODULE_PRIVATE_DECLARATIONS: &[&str] = &[
    MODULE_FUNCTION_VISIBILITY,
    "private",
    "public",
    "protected",
    "remove_const",
];

/// The native Module and Class instance methods metorex implements, with the
/// parameters each takes and whether the last one is variadic.
/// `Module#instance_methods` advertises the names, and `Object#method` builds
/// a callable stub from the parameter list.
pub(super) const NATIVE_MODULE_METHODS: &[(&str, &[&str], bool)] = &[
    ("alias_method", &["new_name", "old_name"], false),
    ("attr", &["names"], true),
    ("attr_accessor", &["names"], true),
    ("attr_reader", &["names"], true),
    ("attr_writer", &["names"], true),
    ("constants", &["inherit"], true),
    ("define_method", &["name", "body"], true),
    ("include", &["modules"], true),
    ("method_defined?", &["name", "inherit"], true),
    ("prepend", &["modules"], true),
    ("instance_method", &["name"], false),
    ("remove_method", &["names"], true),
    ("undef_method", &["names"], true),
    ("public_instance_method", &["name"], false),
    ("protected_instance_methods", &["include_super"], true),
    ("instance_methods", &["include_super"], true),
    ("public_instance_methods", &["include_super"], true),
    ("private_instance_methods", &["include_super"], true),
    ("module_function", &["names"], true),
    ("name", &[], false),
];

/// The Kernel methods `call_object_method` implements natively, with the
/// parameter list each one takes, so `obj.method(:name)` can hand out a stub
/// whose `arity` matches Ruby's. A trailing `true` marks the last parameter
/// variadic.
pub(super) const NATIVE_KERNEL_METHODS: &[(&str, &[&str], bool)] = &[
    ("class", &[], false),
    ("clone", &["options"], true),
    ("dup", &[], false),
    ("eql?", &["other"], false),
    ("equal?", &["other"], false),
    ("extend", &["modules"], true),
    ("freeze", &[], false),
    ("frozen?", &[], false),
    ("hash", &[], false),
    ("inspect", &[], false),
    ("instance_of?", &["klass"], false),
    ("instance_variable_get", &["name"], false),
    ("instance_variable_set", &["name", "value"], false),
    ("instance_variables", &[], false),
    ("is_a?", &["klass"], false),
    ("itself", &[], false),
    ("kind_of?", &["klass"], false),
    ("method", &["name"], false),
    ("methods", &["regular"], true),
    ("nil?", &[], false),
    ("object_id", &[], false),
    ("public_send", &["arguments"], true),
    ("require", &["path"], false),
    ("require_relative", &["path"], false),
    ("respond_to?", &["arguments"], true),
    ("respond_to_missing?", &["name", "include_private"], false),
    ("send", &["arguments"], true),
    ("tap", &[], false),
    ("to_s", &[], false),
    ("__id__", &[], false),
    ("__send__", &["arguments"], true),
];

/// A body-less stub for one of the natively implemented Kernel methods.
pub(super) fn native_kernel_method_stub(name: &str) -> Option<Method> {
    let (_, parameters, variadic) = NATIVE_KERNEL_METHODS
        .iter()
        .find(|(entry, _, _)| *entry == name)?;
    let mut stub = Method::with_owner(
        name.to_string(),
        parameters.iter().map(|p| (*p).to_string()).collect(),
        vec![],
        "Kernel".to_string(),
    );
    if *variadic {
        let last = parameters.len().saturating_sub(1);
        stub.variadic_param = Some((last, parameters[last].to_string()));
    }
    Some(stub)
}

/// The NameError `Module#instance_method` raises for a name that is not
/// defined, or has been removed with `undef_method`. Ruby exposes the missing
/// name through `NameError#name`.
fn undefined_instance_method_error(
    name: &str,
    class_rc: &Rc<Class>,
    position: Position,
) -> MetorexError {
    let msg = format!("undefined method '{}' for {}", name, class_rc.name());
    let exc = Object::exception("NameError", msg.clone());
    if let Object::Exception(cell) = &exc {
        cell.borrow_mut().name = Some(name.to_string());
    }
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message: msg,
    }
}

/// A body-less stub for one of the natively implemented Module methods,
/// carrying its parameter list so `arity` and `bind` behave. Invoking it
/// reaches the same native implementation.
pub(super) fn native_module_method_stub(name: &str) -> Option<Method> {
    let (_, parameters, variadic) = NATIVE_MODULE_METHODS
        .iter()
        .find(|(entry, _, _)| *entry == name)?;
    let mut stub = Method::with_owner(
        name.to_string(),
        parameters.iter().map(|p| (*p).to_string()).collect(),
        vec![],
        "Module".to_string(),
    );
    if *variadic {
        let last = parameters.len().saturating_sub(1);
        stub.variadic_param = Some((last, parameters[last].to_string()));
    }
    Some(stub)
}

/// How `undef_method` names its receiver. The metaclass of a class or module
/// is reported as that class, while any other singleton class is reported by
/// its own display.
fn undef_target_name(class_rc: &Rc<Class>) -> String {
    if class_rc.is_singleton_class()
        && let Some(Object::Class(attached) | Object::Module(attached)) =
            class_rc.get_class_var("__attached__")
    {
        return attached.inspect_name();
    }
    class_rc.inspect_name()
}

/// Whether `name` reads as a constant path, which `set_temporary_name`
/// rejects: a `::`-separated chain whose every segment is a constant name,
/// with an optional leading `::`.
fn looks_like_constant_path(name: &str) -> bool {
    let path = name.strip_prefix("::").unwrap_or(name);
    !path.is_empty() && path.split("::").all(is_valid_constant_name)
}

/// Whether the receiver answers `name` through a method of its own: an
/// instance method, a `def self.name` class method, or a singleton method.
/// Used to let a user-defined accessor win over a native handler.
fn has_user_defined_method(class_rc: &Rc<Class>, name: &str) -> bool {
    class_rc.find_method(name).is_some()
        || class_rc
            .find_method(&format!("__class__{}", name))
            .is_some()
        || class_rc
            .singleton_class_slot()
            .as_ref()
            .is_some_and(|sc| sc.find_method(name).is_some())
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

/// Kernel methods that `call_object_method` implements natively, so a
/// body-less stub can stand in for them in `Object.instance_method`.
pub(crate) fn is_native_kernel_method(name: &str) -> bool {
    matches!(
        name,
        "class"
            | "clone"
            | "dup"
            | "eql?"
            | "equal?"
            | "extend"
            | "freeze"
            | "frozen?"
            | "hash"
            | "inspect"
            | "instance_of?"
            | "instance_variable_get"
            | "instance_variable_set"
            | "instance_variables"
            | "is_a?"
            | "itself"
            | "kind_of?"
            | "method"
            | "methods"
            | "nil?"
            | "object_id"
            | "public_send"
            | "require"
            | "require_relative"
            | "respond_to?"
            | "respond_to_missing?"
            | "send"
            | "tap"
            | "to_s"
            | "__id__"
            | "__send__"
    )
}
