use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::native_methods::is_valid_constant_name;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

/// Prefix for the class-variable keys under which a module records the
/// refinements created in its body by `refine`.
pub(crate) const REFINEMENT_KEY_PREFIX: &str = "__refine__";

/// Class-variable key holding a refinement's display label, `Target@Module`.
pub(crate) const REFINEMENT_LABEL_KEY: &str = "__refinement_label__";

impl VirtualMachine {
    pub(crate) fn call_module_methods(
        &mut self,
        module_rc: &Rc<Class>,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
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
                    let holder = Rc::new(Class::new_module(""));
                    holder.set_class_var(
                        REFINEMENT_LABEL_KEY,
                        Object::String(Rc::new(format!(
                            "{}@{}",
                            target.ruby_name(),
                            module_rc.inspect_name()
                        ))),
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
            let body = self.apply_block_as_class_body(&holder, &block, position);
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
            let path = match &arguments[0] {
                Object::String(s) => s.as_str().to_string(),
                other => {
                    return Err(method_argument_type_error(
                        "require", "String", other, position,
                    ));
                }
            };
            return self
                .call_native_function("require", vec![Object::String(Rc::new(path))], position)
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
            return self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some);
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
                "trap" => return self.install_signal_trap(arguments, position).map(Some),
                _ => {}
            }
        }

        if module_rc.name() == "Process" {
            match method_name {
                "pid" => return Ok(Some(Object::Int(std::process::id() as i64))),
                "ppid" => return Ok(Some(Object::Int(0))),
                "kill" => return self.send_signal(arguments, position).map(Some),
                // SAFETY: `geteuid` and `getuid` read process ids and touch
                // nothing else.
                "euid" => return Ok(Some(Object::Int(unsafe { libc::geteuid() } as i64))),
                "uid" => return Ok(Some(Object::Int(unsafe { libc::getuid() } as i64))),
                "last_status" => return Ok(Some(self.process_last_status())),
                // `Process.exit`, `.exit!`, and `.abort` end this process the
                // way the bare forms do.
                "exit" | "exit!" | "abort" => {
                    return self
                        .call_native_function(method_name, arguments.to_vec(), position)
                        .map(Some);
                }
                // `wait` and `waitpid` answer the child's process id, and
                // `wait2` pairs it with the status. All three record `$?`.
                "wait" | "waitpid" | "wait2" | "waitpid2" => {
                    let requested = match arguments.first() {
                        Some(Object::Int(pid)) => *pid as i32,
                        _ => -1,
                    };
                    let (pid, status) = self.wait_for_child(requested, position)?;
                    if matches!(method_name, "wait2" | "waitpid2") {
                        let pair = vec![Object::Int(pid as i64), status];
                        return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(pair)))));
                    }
                    return Ok(Some(Object::Int(pid as i64)));
                }
                "waitall" => {
                    let mut results = Vec::new();
                    while let Ok((pid, status)) = self.wait_for_child(-1, position) {
                        if pid <= 0 {
                            break;
                        }
                        results.push(Object::Array(Rc::new(std::cell::RefCell::new(vec![
                            Object::Int(pid as i64),
                            status,
                        ]))));
                    }
                    return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                        results,
                    )))));
                }
                _ => {}
            }
        }

        if (module_rc.name() == "GC" || module_rc.name() == "ObjectSpace") && method_name != "name"
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
                return Ok(Some(Object::String(Rc::new(name))));
            }
            "ancestors" => {
                let mut chain: Vec<Object> = Vec::new();
                let mut seen: Vec<*const crate::class::Class> = Vec::new();
                super::class_methods::push_module_ancestors(module_rc, &mut chain, &mut seen);
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(chain)))));
            }
            // `remove_method`, `undef_method`, and `alias_method` are shared
            // with classes: dispatch falls through to `call_class_methods`.
            // `autoload :CONST, "path"` registers a lazy loader. The path is
            // stored verbatim — `autoload?` returns it unchanged on hit.
            "autoload" => {
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
                    Some(Object::Symbol(s)) => (**s).clone(),
                    Some(Object::String(s)) => (**s).clone(),
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
                    Some(p) => Object::String(Rc::new(p)),
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
                        list.push(Object::Symbol(Rc::new(hook.to_string())));
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
}
