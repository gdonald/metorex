// Which module a call names, and the module methods answered here
// rather than in the prelude.

use super::*;

impl VirtualMachine {
    pub(crate) fn call_module_methods(
        &mut self,
        module_rc: &Rc<Class>,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // A refinement names the class it refines.
        if matches!(method_name, "target" | "refined_class")
            && let Some(found) = module_rc.get_class_var(REFINEMENT_TARGET_KEY)
        {
            return Ok(Some(found));
        }
        if method_name == "module_eval" || method_name == "class_eval" {
            let result = self.class_eval_with_args(
                module_rc,
                Object::Module(Rc::clone(module_rc)),
                arguments,
                position,
            )?;
            return Ok(Some(result));
        }

        if method_name == "module_exec" || method_name == "class_exec" {
            let block = match self.pending_block.take() {
                Some(Object::Block(b)) => b,
                _ => return Err(local_jump_error(method_name, position)),
            };
            let result = self.class_exec_block(
                module_rc,
                Object::Module(Rc::clone(module_rc)),
                &block,
                arguments.to_vec(),
                position,
            )?;
            return Ok(Some(result));
        }

        // Module#refinements: the refinement modules created by `refine` in
        // this module's own body, in definition order.
        if method_name == "refinements" && arguments.is_empty() {
            let refinements: Vec<Object> = module_rc
                .class_var_names()
                .into_iter()
                .filter(|key| key.starts_with(REFINEMENT_KEY_PREFIX))
                .filter_map(|key| match module_rc.get_class_var(&key) {
                    Some(Object::Class(holder) | Object::Module(holder)) => {
                        Some(Object::Module(holder))
                    }
                    _ => None,
                })
                .collect();
            return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                refinements,
            )))));
        }

        if method_name == "import_methods"
            && module_rc.get_class_var(REFINEMENT_TARGET_KEY).is_some()
        {
            return self
                .import_methods_into_refinement(module_rc, arguments, position)
                .map(Some);
        }

        if method_name == "refine" {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    "refine",
                    1,
                    arguments.len(),
                    position,
                ));
            }
            // Ruby refines a module as readily as a class, and reports
            // anything else with its own wording.
            let target = match &arguments[0] {
                Object::Class(target) | Object::Module(target) => Rc::clone(target),
                other => {
                    let message = format!(
                        "wrong argument type {} (expected Class or Module)",
                        self.builtins().class_of(other).name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
            };
            let refinement_key = format!(
                "{}{}@{:p}",
                REFINEMENT_KEY_PREFIX,
                target.name(),
                Rc::as_ptr(&target)
            );
            let holder = match module_rc.get_class_var(&refinement_key) {
                Some(Object::Class(existing) | Object::Module(existing)) => existing,
                _ => {
                    // The refinement is anonymous, so binding it to a
                    // constant names it. Its display comes from the label
                    // instead, which never changes.
                    let holder = Class::new_module("");
                    holder.set_class_var(REFINEMENT_TARGET_KEY, Object::Class(Rc::clone(&target)));
                    holder.set_class_var(
                        REFINEMENT_LABEL_KEY,
                        Object::string(format!(
                            "{}@{}",
                            target.ruby_name(),
                            module_rc.inspect_name()
                        )),
                    );
                    holder
                }
            };
            let Some(Object::Block(block)) = self.pending_block.take() else {
                let message = "no block given".to_string();
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: position_to_location(position),
                    message,
                });
            };
            // The refinement is registered before its body runs, so calls
            // inside the block see it, along with every sibling refinement
            // the same module has declared so far.
            module_rc.set_class_var(&refinement_key, Object::Module(Rc::clone(&holder)));
            self.push_refinement_scope();
            self.activate_refinement(Rc::clone(module_rc));
            let body = self.apply_refine_block(&holder, &block, position);
            self.pop_refinement_scope();
            body?;
            return Ok(Some(Object::Module(holder)));
        }

        // `Kernel.require(path)` — module-level dispatch that delegates to
        // the same implementation as the top-level `require` function.
        if module_rc.name() == "Kernel" && method_name == "require" {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    "require",
                    1,
                    arguments.len(),
                    position,
                ));
            }
            // A path may be named by anything that says how to read itself
            // as one, which the bare form takes too.
            let path = self.coerce_load_path(&arguments[0], position)?;
            return self
                .call_native_function("require", vec![Object::string(path)], position)
                .map(Some);
        }

        // `Kernel.\`` reaches the same command runner the bare form does.
        // `Kernel.chomp` and `Kernel.chop` reach the same `$_` rewriters the
        // bare forms do.
        if module_rc.name() == "Kernel"
            && matches!(
                method_name,
                "chomp"
                    | "chop"
                    | "eval"
                    | "exec"
                    | "exit"
                    | "exit!"
                    | "fork"
                    | "load"
                    | "open"
                    | "p"
                    | "pp"
                    | "printf"
                    | "sprintf"
            )
        {
            // Code run through `Kernel.eval` runs against Kernel, which is
            // the receiver the call was written with.
            let held = self
                .kernel_function_receiver
                .replace(Object::Module(Rc::clone(module_rc)));
            let answered = self.call_native_function(method_name, arguments.to_vec(), position);
            self.kernel_function_receiver = held;
            return answered.map(Some);
        }
        // `Kernel.format` is `sprintf` under its other name.
        if module_rc.name() == "Kernel" && method_name == "format" {
            return self
                .call_native_function("sprintf", arguments.to_vec(), position)
                .map(Some);
        }
        if module_rc.name() == "Kernel" && method_name == "`" {
            return self
                .call_native_function("`", arguments.to_vec(), position)
                .map(Some);
        }
        if module_rc.name() == "Signal" {
            match method_name {
                "list" => return Ok(Some(self.signal_list())),
                "signame" => return self.signal_name(arguments, position).map(Some),
                "trap" => return self.install_signal_trap(arguments, position).map(Some),
                _ => {}
            }
        }

        if module_rc.name() == "Coverage" {
            match method_name {
                "__coverage_start__" => {
                    let by_mode = matches!(arguments.first(), Some(Object::Bool(true)));
                    let modes = match arguments.get(1) {
                        Some(Object::Array(held)) => held
                            .borrow()
                            .iter()
                            .filter_map(|mode| match mode {
                                Object::Symbol(name) => Some(name.as_str().to_string()),
                                _ => None,
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    let eval_too = matches!(arguments.get(2), Some(Object::Bool(true)));
                    self.coverage_start(by_mode, modes, eval_too);
                    return Ok(Some(Object::Nil));
                }
                "__coverage_stop__" => {
                    self.coverage_stop();
                    return Ok(Some(Object::Nil));
                }
                "__coverage_result__" => {
                    let clear = matches!(arguments.first(), Some(Object::Bool(true)));
                    return self.coverage_report(clear).map(Some);
                }
                _ => {}
            }
        }

        if module_rc.name() == "Etc"
            && let Some(answered) = self.call_etc_methods(method_name, arguments, position)?
        {
            return Ok(Some(answered));
        }

        // The digest library asks for its answers here, so the algorithms
        // themselves stay in one place rather than being written in Ruby.
        if module_rc.name() == "Digest" && method_name == "__digest__" {
            return self.compute_digest(arguments, position).map(Some);
        }
        if module_rc.name() == "Digest" && method_name == "__pbkdf2__" {
            return self.compute_pbkdf2(arguments, position).map(Some);
        }
        if module_rc.name() == "Digest" && method_name == "__scrypt__" {
            return self.compute_scrypt(arguments, position).map(Some);
        }

        // The compressed stream formats are read and written here for the
        // same reason.
        if module_rc.name() == "Zlib" && method_name == "__stream__" {
            return self.zlib_stream(arguments, position).map(Some);
        }

        // The system log is written through the C library, which only the
        // interpreter can reach.
        if module_rc.name() == "Syslog" && method_name == "__write__" {
            return self.syslog_write(arguments, position).map(Some);
        }

        // The shape of a network address belongs to the operating system, so
        // the socket library reads and writes them here.
        if module_rc.name() == "Socket" && method_name == "__address__" {
            return self.socket_address(arguments, position).map(Some);
        }

        if module_rc.name() == "Process"
            && let Some(result) =
                self.call_process_module_methods(module_rc, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }
        // The two readings GC keeps a real account of. Metorex frees an
        // object when the last reference to it goes, so there is no collector
        // to time; `start` still counts as a collection having been asked for,
        // which is what makes the count climb.
        if module_rc.name() == "GC" {
            match method_name {
                "count" => {
                    return Ok(Some(Object::Int(self.gc_collection_count())));
                }
                "total_time" => {
                    return Ok(Some(Object::Int(self.gc_total_time())));
                }
                "start" | "garbage_collect" => {
                    self.record_gc_run();
                    return Ok(Some(Object::Nil));
                }
                _ => {}
            }
        }

        // Every object the running program can still reach, which is what
        // metorex has in place of a heap to walk: the names the program has
        // bound, and everything those names lead to.
        if module_rc.name() == "ObjectSpace" && method_name == "__live_objects__" {
            let mut found = Vec::new();
            let mut seen = std::collections::HashSet::new();
            let mut roots: Vec<Object> = Vec::new();
            if let Some(scope) = self.main_script_scope.clone() {
                roots.extend(scope.borrow().collect_all_vars().into_values());
            }
            roots.extend(
                self.environment()
                    .global_scope()
                    .borrow()
                    .collect_all_vars()
                    .into_values(),
            );
            for root in roots {
                gather_reachable(root, &mut seen, &mut found);
            }
            return Ok(Some(Object::array(found)));
        }

        // Every object still alive that is a kind of the module given, oldest
        // first: the instances the program made, then its classes and
        // modules. The singleton class of a singleton class is the
        // interpreter's own and is left out.
        if module_rc.name() == "ObjectSpace" && method_name == "__each_object__" {
            let wanted = arguments.first().cloned().unwrap_or(Object::Nil);
            let mut candidates: Vec<Object> = crate::object::live::live_instances()
                .into_iter()
                .map(Object::Instance)
                .collect();
            for class in crate::object::live::live_classes() {
                if class.is_singleton_class()
                    && matches!(
                        class.get_class_var("__attached__"),
                        Some(Object::Class(attached)) if attached.is_singleton_class()
                    )
                {
                    continue;
                }
                candidates.push(if class.is_module() {
                    Object::Module(class)
                } else {
                    Object::Class(class)
                });
            }
            if matches!(wanted, Object::Nil) {
                return Ok(Some(Object::array(candidates)));
            }
            let mut found = Vec::new();
            for candidate in candidates {
                let kind = self.send_to_object(
                    candidate.clone(),
                    "is_a?",
                    vec![wanted.clone()],
                    position,
                )?;
                if crate::vm::utils::is_truthy(&kind) {
                    found.push(candidate);
                }
            }
            return Ok(Some(Object::array(found)));
        }

        // Whether the place each object is made should be recorded, which
        // the tracing the object space offers is switched on and off by.
        if module_rc.name() == "ObjectSpace" && method_name == "__trace_allocations__" {
            self.tracing_allocations = arguments.first().is_some_and(crate::vm::utils::is_truthy);
            return Ok(Some(Object::Bool(self.tracing_allocations)));
        }

        if module_rc.name() == "OpenSSL" && method_name == "__rsa_generate__" {
            return self.rsa_generate(arguments, position).map(Some);
        }
        if module_rc.name() == "OpenSSL" && method_name == "__object__" {
            return Ok(Some(self.openssl_object_command(arguments)));
        }

        if module_rc.name() == "Fiddle" && method_name == "__dynamic_library__" {
            return self.dynamic_library_command(arguments, position).map(Some);
        }

        if module_rc.name() == "ObjectSpace" && method_name == "__allocation_tracing__" {
            return Ok(Some(self.allocation_tracing_command(arguments)));
        }

        // Where an object was made, as the file and the line it was written
        // on, and nil for one made before the tracing was switched on.
        if module_rc.name() == "ObjectSpace" && method_name == "__allocation_site__" {
            let held = match arguments.first() {
                Some(Object::String(text)) => text.created_at(),
                _ => None,
            };
            return Ok(Some(match held {
                Some(place) => Object::string(place),
                None => Object::Nil,
            }));
        }

        // What the object space can say about how much a program holds,
        // counted from what it has built rather than from a heap walk.
        if module_rc.name() == "ObjectSpace" && method_name == "__allocated__" {
            let counted = match arguments.first() {
                Some(Object::Class(held)) | Some(Object::Module(held)) => self
                    .allocation_counts
                    .get(&(Rc::as_ptr(held) as usize))
                    .copied()
                    .unwrap_or(0),
                _ => self.allocation_counts.values().sum(),
            };
            return Ok(Some(Object::Int(counted)));
        }

        // GC and ObjectSpace answer nil for the names metorex keeps no
        // account of. A name either module carries itself wins, which is how
        // the objspace library adds to them, and so does a name every object
        // answers, since those say something true about the module itself.
        // A method the program defined on Object or Kernel is one every
        // object answers too.
        if (module_rc.name() == "GC" || module_rc.name() == "ObjectSpace")
            && method_name != "name"
            && !ANSWERED_BY_EVERY_OBJECT.contains(&method_name)
            && module_rc.find_method(method_name).is_none()
            && crate::vm::method_lookup::module_level_method(module_rc, method_name).is_none()
            && self.class_method_of(module_rc, method_name).is_none()
            && self
                .lookup_method(&Object::Module(Rc::clone(module_rc)), method_name)
                .is_none_or(|(_, found)| found.body.is_empty() && found.captured_vars.is_none())
        {
            return Ok(Some(Object::Nil));
        }

        match method_name {
            "name" => {
                // Assigned names (an anonymous module bound to a constant)
                // count; a still-anonymous module's name is nil.
                let name = module_rc.ruby_name();
                if name.is_empty() {
                    return Ok(Some(Object::Nil));
                }
                // A module answers one name object, not a fresh string each
                // time it is asked.
                let slot = format!("__module_name_{:p}_{}", Rc::as_ptr(module_rc), name);
                return Ok(Some(self.memoized_text(&slot, &name)));
            }
            "ancestors" => {
                let mut chain: Vec<Object> = Vec::new();
                let mut seen: Vec<*const crate::class::Class> = Vec::new();
                crate::vm::native_methods::class_methods::push_module_ancestors(
                    module_rc, &mut chain, &mut seen,
                );
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(chain)))));
            }
            // `remove_method`, `undef_method`, and `alias_method` are shared
            // with classes: dispatch falls through to `call_class_methods`.
            // `autoload :CONST, "path"` registers a lazy loader. The path is
            // stored verbatim — `autoload?` returns it unchanged on hit.
            "autoload" => {
                // `Kernel.autoload` registers where the caller sits, the same
                // way the bare form does, rather than on Kernel itself.
                if module_rc.name() == "Kernel"
                    && let Some(definee) = self.autoload_definee()
                    && !Rc::ptr_eq(&definee, module_rc)
                {
                    return self.call_class_methods(&definee, method_name, arguments, position);
                }
                let const_name = match arguments.first() {
                    Some(Object::Symbol(s)) => s.as_str().to_string(),
                    Some(Object::String(s)) => s.as_str().to_string(),
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
                // FrozenError fires before any other validation in MRI, so the
                // constant slot stays untouched on a frozen module.
                if module_rc.is_frozen() {
                    let msg = format!("can't modify frozen Module: {}", module_rc.name());
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Coerce the filename: String/Symbol pass through; anything
                // else must respond to #to_path and return a String, else
                // TypeError. Empty strings raise ArgumentError.
                let path = match arguments.get(1) {
                    Some(Object::String(s)) => s.as_str().to_string(),
                    Some(Object::Symbol(s)) => s.as_str().to_string(),
                    Some(other) => {
                        let other_obj = other.clone();
                        if let Some((cls, method)) = self.lookup_method(&other_obj, "to_path") {
                            let result =
                                self.invoke_method(cls, method, other_obj, Vec::new(), position)?;
                            match result {
                                Object::String(s) => s.as_str().to_string(),
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
                module_rc.set_autoload_location(
                    const_name.clone(),
                    caller_file,
                    position.line as i64,
                );
                module_rc.set_autoload(const_name.clone(), path);
                self.trigger_const_added_hook(
                    Object::Module(Rc::clone(module_rc)),
                    &const_name,
                    position,
                )?;
                return Ok(Some(Object::Nil));
            }
            "autoload?" => {
                let const_name = match arguments.first() {
                    Some(Object::Symbol(s)) => s.as_str().to_string(),
                    Some(Object::String(s)) => s.as_str().to_string(),
                    _ => return Ok(Some(Object::Nil)),
                };
                // `autoload?(name, inherit=true)` — second arg disables ancestor lookup.
                let inherit = !matches!(arguments.get(1), Some(Object::Bool(false)));
                // `effective_autoload` is thread-aware (loading thread
                // sees nil, other threads see the path) and consults the
                // ancestor chain via `lookup_autoload`. For
                // `inherit=false` gate the call on a local-only check.
                let module_for_autoload = Rc::clone(module_rc);
                let local_only_blocked = !inherit && module_rc.get_autoload(&const_name).is_none();
                let path = if local_only_blocked {
                    None
                } else {
                    self.effective_autoload(&module_for_autoload, &const_name)
                };
                return Ok(Some(match path {
                    Some(p) => Object::string(p),
                    None => Object::Nil,
                }));
            }
            // `instance_method` / `public_instance_method` are shared with
            // the class dispatch path.
            // Module#include / Module#prepend: dispatch through
            // `append_features` so user overrides on the included module's
            // singleton class fire and the cyclic/frozen checks run.
            // `prepend` ordering is still approximated as a regular include
            // (sufficient for current fixture setup). We only intercept
            // calls *with* arguments so a zero-arg user-defined accessor
            // (e.g. `attr_reader :include` on MSpec) still wins.
            "include" | "prepend" if !arguments.is_empty() => {
                // Ruby applies the arguments in reverse, so the first module
                // listed ends up nearest the receiver in the ancestor chain.
                for arg in arguments.iter().rev() {
                    if let Some(mixin) =
                        self.resolve_include_argument(arg, method_name, position)?
                    {
                        if method_name == "prepend" {
                            self.apply_module_prepend(module_rc, &mixin, position)?;
                        } else {
                            self.apply_module_include(module_rc, &mixin, position)?;
                        }
                    }
                }
                return Ok(Some(Object::Module(Rc::clone(module_rc))));
            }
            // `mod.append_features(target)` / `mod.prepend_features(target)`:
            // the default behavior used by Module#include — add `mod` to
            // `target`'s mixin chain after the standard frozen / cyclic
            // checks. If the user has defined their own `append_features`
            // (as a singleton method on `mod`), defer to it instead so the
            // override actually runs.
            "append_features" | "prepend_features" if !arguments.is_empty() => {
                let class_method_key = format!("__class__{}", method_name);
                if module_rc.find_method(&class_method_key).is_some() {
                    return Ok(None);
                }
                if let Some(sc) = module_rc.singleton_class_slot().clone()
                    && sc.find_method(method_name).is_some()
                {
                    return Ok(None);
                }
                for arg in arguments {
                    match arg {
                        Object::Module(t) | Object::Class(t) => {
                            self.default_append_features(t, module_rc, position)?;
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
                return Ok(Some(Object::Module(Rc::clone(module_rc))));
            }
            // Module#extend_object: invoked by Module#extend; mix the module
            // into the argument's singleton class. We model this by adding a
            // mixin onto the argument if it is a module/class.
            // A module also answers with the mixin hooks that Class leaves
            // undefined; the shared names come from the class dispatch path.
            "private_methods" => {
                let Some(Object::Array(names)) =
                    self.call_class_methods(module_rc, method_name, arguments, position)?
                else {
                    return Ok(None);
                };
                {
                    let mut list = names.borrow_mut();
                    for hook in ["append_features", "prepend_features", "extend_object"] {
                        list.push(Object::symbol(hook.to_string()));
                    }
                    list.sort_by_key(|entry| entry.to_string());
                }
                return Ok(Some(Object::Array(names)));
            }
            "extend_object" if !arguments.is_empty() => {
                if module_rc.find_method("__class__extend_object").is_some() {
                    return Ok(None);
                }
                if let Some(sc) = module_rc.singleton_class_slot().clone()
                    && sc.find_method("extend_object").is_some()
                {
                    return Ok(None);
                }
                for arg in arguments {
                    self.default_extend_object(arg, module_rc, position)?;
                }
                return Ok(Some(Object::Module(Rc::clone(module_rc))));
            }
            // `module_function` is shared with the class dispatch path.
            _ => {}
        }

        // Fall through to receiver-agnostic dispatch
        let _ = receiver;
        Ok(None)
    }

    /// `Refinement#import_methods`: copy the methods each module defines
    /// itself into the refinement, as if they had been written in its body.
    /// Only methods written in Ruby can be copied, and a module that brings
    /// in others is warned about, since theirs are not copied.
    pub(crate) fn import_methods_into_refinement(
        &mut self,
        refinement: &Rc<Class>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut modules = Vec::with_capacity(arguments.len());
        for argument in arguments {
            let Object::Module(module) = argument else {
                let message = format!(
                    "wrong argument type {} (expected Module)",
                    crate::vm::native_methods::define_method::ruby_class_name(argument)
                );
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("TypeError", message.clone()),
                    location: position_to_location(position),
                    message,
                });
            };
            modules.push(Rc::clone(module));
        }
        for module in &modules {
            if !module.mixin_chain().is_empty() || !module.prepend_chain().is_empty() {
                let text = format!(
                    "{}{} has ancestors, but Refinement#import_methods doesn't import their methods\n",
                    self.warning_prefix(0, position),
                    module.inspect_name()
                );
                self.warn_through_warning_module(text, position)?;
            }
        }
        let refinements = self.snapshot_active_refinements();
        for module in &modules {
            for name in module.method_names() {
                if name.starts_with("__class__") {
                    continue;
                }
                let Some(method) = module.find_own_method(&name) else {
                    continue;
                };
                let written_in_ruby = method
                    .source_location
                    .as_ref()
                    .and_then(|written| written.filename.as_deref())
                    .is_some_and(|file| !crate::vm::stdlib::written_for_c(file))
                    && !method
                        .body
                        .first()
                        .is_some_and(|held| held.position().prelude);
                if !written_in_ruby {
                    let message = format!(
                        "Can't import method which is not defined with Ruby code: {}#{}",
                        module.inspect_name(),
                        name
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                let mut copied = (*method).clone();
                copied.owner = Some(refinement.name().to_string());
                copied.owner_class = Some(Rc::clone(refinement));
                copied.captured_refinements = refinements.clone();
                refinement.define_method(name.clone(), Rc::new(copied));
                if module.is_method_private(&name) {
                    refinement.set_method_private(name);
                } else if module.is_method_protected(&name) {
                    refinement.set_method_protected(name);
                }
            }
        }
        Ok(Object::Module(Rc::clone(refinement)))
    }
}
