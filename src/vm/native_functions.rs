//! Native (built-in) function implementations for the virtual machine.
//!
//! This module contains implementations of global built-in functions like puts, print, etc.

use super::VirtualMachine;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// Call a native function by name.
    pub(crate) fn call_native_function(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match name {
            // A receiverless `private` / `public` / `protected` applies to the
            // enclosing class or module; at the top level it applies to Object.
            "private" | "public" | "protected" => {
                self.pending_block.take();
                if let Some(class) = self.current_definee() {
                    return self
                        .apply_class_visibility_modifier(&class, name, &arguments, position);
                }
                self.apply_visibility_modifier(name, arguments, position)
            }
            "private_constant" | "public_constant" => {
                // Apply visibility marks to constants on the current `self`
                // module/class. Inside a `module M; ...; end` body, `self`
                // is the module being defined, so qualified accesses like
                // `M::PrivConst` from outside raise NameError /private
                // constant/. `public_constant` is the inverse.
                self.pending_block.take();
                let target = match self.environment().get("self") {
                    Some(Object::Class(c)) | Some(Object::Module(c)) => c,
                    _ => return Ok(Object::Nil),
                };
                let make_private = name == "private_constant";
                for arg in &arguments {
                    let const_name = match arg {
                        Object::Symbol(s) => s.as_str().to_string(),
                        Object::String(s) => s.as_str().to_string(),
                        _ => continue,
                    };
                    if make_private {
                        target.mark_private_constant(const_name);
                    } else {
                        target.unmark_private_constant(&const_name);
                    }
                }
                Ok(Object::Nil)
            }
            // A receiverless `module_function` toggles the module-function
            // state on the enclosing module.
            "module_function" => {
                self.pending_block.take();
                let current_self = self
                    .current_definee()
                    .map(Object::Module)
                    .unwrap_or(Object::Nil);
                if let Object::Class(class) | Object::Module(class) = &current_self {
                    if arguments.is_empty() {
                        class.set_current_visibility(
                            crate::vm::native_methods::MODULE_FUNCTION_VISIBILITY,
                        );
                        return Ok(Object::Nil);
                    }
                    let class = Rc::clone(class);
                    let mut names = Vec::with_capacity(arguments.len());
                    for argument in &arguments {
                        let name =
                            self.coerce_method_name(argument, "module_function", position)?;
                        self.copy_to_module_function(&class, &name, position)?;
                        names.push(Object::Symbol(Rc::new(name)));
                    }
                    return Ok(match names.len() {
                        1 => names.remove(0),
                        _ => Object::Array(Rc::new(std::cell::RefCell::new(names))),
                    });
                }
                Ok(Object::Nil)
            }
            // A receiverless `private_class_method` / `public_class_method`
            // inside a class or module body applies to that class.
            "private_class_method" | "public_class_method" => {
                self.pending_block.take();
                let current_self = self
                    .current_definee()
                    .map(Object::Module)
                    .unwrap_or(Object::Nil);
                if let Object::Class(class) | Object::Module(class) = &current_self {
                    let class = Rc::clone(class);
                    if let Some(result) =
                        self.call_class_methods(&class, name, &arguments, position)?
                    {
                        return Ok(result);
                    }
                }
                Ok(Object::Nil)
            }
            "deprecate_constant" | "noop_with_block" => {
                // Visibility modifiers and Object#freeze — no-op stubs. Accept any args.
                self.pending_block.take();
                Ok(Object::Nil)
            }
            // A receiverless `freeze` freezes `self` — inside a class or
            // module body that is the class object itself.
            "freeze" => {
                self.pending_block.take();
                let current_self = self.environment().get("self").unwrap_or(Object::Nil);
                match &current_self {
                    Object::Class(class) | Object::Module(class) => class.freeze(),
                    Object::Instance(instance) => instance.borrow_mut().frozen = true,
                    _ => {}
                }
                Ok(current_self)
            }
            // Kernel#lambda — the block becomes a lambda-style Proc. A proc
            // handed over as `&expr` is not a literal block, so Ruby rejects
            // it unless it is already a lambda.
            "lambda" => {
                let from_ampersand = self.pending_block_from_ampersand;
                match self.pending_block.take() {
                    Some(Object::Block(block)) => {
                        if block.is_lambda {
                            return Ok(Object::Block(block));
                        }
                        if from_ampersand {
                            let msg = "the lambda method requires a literal block";
                            return Err(MetorexError::UncaughtException {
                                exception: Object::exception("ArgumentError", msg.to_string()),
                                location: crate::vm::utils::position_to_location(position),
                                message: msg.to_string(),
                            });
                        }
                        let mut as_lambda = (*block).clone();
                        as_lambda.is_lambda = true;
                        Ok(Object::Block(std::rc::Rc::new(as_lambda)))
                    }
                    Some(other) => Ok(other),
                    None => {
                        let msg = "tried to create Proc object without a block";
                        Err(MetorexError::UncaughtException {
                            exception: Object::exception("ArgumentError", msg.to_string()),
                            location: crate::vm::utils::position_to_location(position),
                            message: msg.to_string(),
                        })
                    }
                }
            }
            // Kernel#proc — the block itself is the Proc.
            "proc" => {
                if let Some(block) = self.pending_block.take() {
                    return Ok(block);
                }
                let msg = "tried to create Proc object without a block";
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", msg.to_string()),
                    location: crate::vm::utils::position_to_location(position),
                    message: msg.to_string(),
                })
            }
            "at_exit" => {
                // Discard the block; we never invoke at_exit handlers.
                self.pending_block.take();
                Ok(Object::Nil)
            }
            "using" => {
                if arguments.len() != 1 {
                    let exc = Object::exception(
                        "ArgumentError",
                        format!(
                            "wrong number of arguments (given {}, expected 1)",
                            arguments.len()
                        ),
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: crate::vm::utils::position_to_location(position),
                        message: "wrong number of arguments for using".to_string(),
                    });
                }
                let module = match &arguments[0] {
                    Object::Module(m) => std::rc::Rc::clone(m),
                    other => {
                        let exc = Object::exception(
                            "TypeError",
                            format!(
                                "wrong argument type {} (expected Module)",
                                other.type_name()
                            ),
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: crate::vm::utils::position_to_location(position),
                            message: "wrong argument type for using".to_string(),
                        });
                    }
                };
                // `using` is forbidden inside a method body.
                if self.inside_user_method() {
                    let exc = Object::exception(
                        "RuntimeError",
                        "Module#using is not permitted in methods".to_string(),
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: crate::vm::utils::position_to_location(position),
                        message: "using in method".to_string(),
                    });
                }
                self.activate_refinement(module);
                // Ruby answers with the enclosing class or module, or `main`
                // at the top level.
                Ok(match self.current_definee() {
                    Some(definee) if definee.is_module() => Object::Module(definee),
                    Some(definee) => Object::Class(definee),
                    None => self.environment().get("self").unwrap_or(Object::Nil),
                })
            }
            "warn" => self.kernel_warn(arguments, position),
            "sprintf" => {
                if arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        "sprintf requires at least 1 argument".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                // Ruby converts the format with `#to_str`, so a non-String
                // that answers it works and anything else raises TypeError.
                let fmt = Object::string(self.coerce_name_argument(&arguments[0], position)?);
                let rest: Vec<Object> = arguments.into_iter().skip(1).collect();
                let rest_obj = if rest.len() == 1 {
                    rest.into_iter().next().unwrap()
                } else {
                    Object::Array(std::rc::Rc::new(std::cell::RefCell::new(rest)))
                };
                self.evaluate_string_format(fmt, rest_obj, position)
            }
            // `__method__` names the method as it was defined, `__callee__`
            // as it was called. They differ inside an aliased method. Both
            // look through block frames to the method that encloses them and
            // stop at a class body or file scope, where neither has an answer.
            "__method__" | "__callee__" => Ok(match self.enclosing_method_names() {
                Some((callee, defined)) => {
                    let chosen = if name == "__callee__" {
                        callee
                    } else {
                        defined
                    };
                    Object::Symbol(std::rc::Rc::new(chosen))
                }
                None => Object::Nil,
            }),
            "caller" => Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                Vec::new(),
            )))),
            // `caller_locations(start = 1, length = nil)` — walk the VM call
            // stack. Each frame stores the source position it was called
            // from, so level 1 (the caller of the current method) reads the
            // top frame's recorded location. Returns Location objects
            // responding to `lineno` and `path`.
            "caller_locations" => {
                use crate::ast::{Expression, Statement};
                use crate::object::{Instance, Method};
                use std::cell::RefCell;
                use std::rc::Rc;
                let start = match arguments.first() {
                    Some(Object::Int(n)) => (*n).max(1) as usize,
                    _ => 1,
                };
                let length = match arguments.get(1) {
                    Some(Object::Int(n)) => Some((*n).max(0) as usize),
                    _ => None,
                };
                let loc_class = match self.globals().get("__Backtrace_Location_class") {
                    Some(Object::Class(c)) => c,
                    _ => {
                        let cls = Rc::new(crate::class::Class::new(
                            "Thread::Backtrace::Location",
                            None,
                        ));
                        for attr in ["lineno", "path"] {
                            let body = vec![Statement::Return {
                                value: Some(Expression::InstanceVariable {
                                    name: attr.to_string(),
                                    position: crate::lexer::Position::default(),
                                }),
                                position: crate::lexer::Position::default(),
                            }];
                            cls.define_method(
                                attr,
                                Rc::new(Method::new(attr.to_string(), vec![], body)),
                            );
                        }
                        self.globals_mut()
                            .set("__Backtrace_Location_class", Object::Class(Rc::clone(&cls)));
                        cls
                    }
                };
                let stack = self.call_stack();
                let mut locations: Vec<Object> = Vec::new();
                let mut level = start;
                loop {
                    if let Some(len) = length
                        && locations.len() >= len
                    {
                        break;
                    }
                    if level > stack.len() {
                        break;
                    }
                    let frame = &stack[stack.len() - level];
                    // Frame locations are "line:column" or "file:line:column".
                    let (path, line) = match frame.location() {
                        Some(loc) => {
                            let parts: Vec<&str> = loc.rsplitn(3, ':').collect();
                            let line = parts
                                .get(1)
                                .and_then(|s| s.parse::<i64>().ok())
                                .unwrap_or(0);
                            let path = parts.get(2).map(|s| s.to_string()).unwrap_or_default();
                            (path, line)
                        }
                        None => (String::new(), 0),
                    };
                    let mut inst = Instance::new(Rc::clone(&loc_class));
                    inst.set_var("lineno".to_string(), Object::Int(line));
                    inst.set_var("path".to_string(), Object::String(Rc::new(path)));
                    locations.push(Object::Instance(Rc::new(RefCell::new(inst))));
                    level += 1;
                }
                Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                    locations,
                ))))
            }
            // `binding` captures the frame that called it, not the receiver
            // it was sent to: the local variables in scope (as shared cells,
            // so an assignment through the binding is visible to both) and the
            // `self` in force there.
            "binding_kernel" => {
                let variables = self.environment().current_scope_var_refs();
                // At file scope there is no `self` binding; Ruby's top-level
                // self is `main`, which is what TOPLEVEL_BINDING holds.
                let receiver = self
                    .environment()
                    .get("self")
                    .or_else(|| match self.globals().get("TOPLEVEL_BINDING") {
                        Some(Object::Binding(b)) => b.receiver.clone(),
                        _ => None,
                    })
                    .unwrap_or(Object::Nil);
                Ok(Object::Binding(std::rc::Rc::new(
                    crate::object::Binding::with_receiver(variables, receiver),
                )))
            }
            "top_level_to_s" => Ok(Object::string("main".to_string())),
            "define_method" => {
                use crate::object::Method;
                use std::rc::Rc;
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!("define_method expects 1 argument, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let method_name = match &arguments[0] {
                    Object::Symbol(s) => s.as_str().to_string(),
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "define_method expects a Symbol or String, got {}",
                                other.type_name()
                            ),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "define_method requires a block".to_string(),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };
                let params: Vec<String> = block.parameters.clone();
                let body: Vec<crate::ast::Statement> = block.body.clone();
                let mut method = Method::new(method_name.clone(), params, body);
                method.captured_vars = Some(block.captured_vars().clone());
                // Optional block params (`|a, b = 1|`) become the method's
                // default parameters.
                for (orig_idx, expr) in block.parameter_defaults.iter() {
                    let reg_idx = block.parameters[..*orig_idx]
                        .iter()
                        .filter(|p| !p.starts_with('&'))
                        .count();
                    method.default_parameters.push((reg_idx, expr.clone()));
                }
                let method_rc = Rc::new(method);
                // Install on current self if it's a Class/Module (e.g. inside class_eval),
                // otherwise on global Object (top-level `define_method` semantics).
                let target = match self.environment().get("self") {
                    Some(Object::Class(c)) | Some(Object::Module(c)) => Some(c),
                    _ => match self.globals().get("Object") {
                        Some(Object::Class(c)) => Some(c),
                        _ => None,
                    },
                };
                if let Some(class) = target {
                    class.define_method(method_name.clone(), method_rc);
                }
                Ok(Object::Symbol(Rc::new(method_name)))
            }
            // Kernel#rand — a Float in [0, 1) with no argument, an Integer
            // below the given bound, or a value drawn from a Range.
            "rand" => {
                if arguments.len() > 1 {
                    return Err(MetorexError::runtime_error(
                        format!("rand() expects 0-1 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let Some(limit) = arguments.first().cloned() else {
                    return Ok(Object::Float(self.next_random_float()));
                };
                self.random_below(limit, position)
            }
            // Kernel#trace_var — run a hook whenever the named global is
            // assigned. The hook comes from a block, a Proc argument, or a
            // String of code to evaluate.
            "trace_var" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    let message = format!(
                        "wrong number of arguments (given {}, expected 1..2)",
                        arguments.len()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                let name = global_name_from(&arguments[0]);
                let hook = match arguments.get(1) {
                    Some(hook) => hook.clone(),
                    None => match self.pending_block.take() {
                        Some(block) => block,
                        None => {
                            let message = "tracing requires a block or a proc".to_string();
                            return Err(MetorexError::UncaughtException {
                                exception: Object::exception("ArgumentError", message.clone()),
                                location: crate::vm::utils::position_to_location(position),
                                message,
                            });
                        }
                    },
                };
                self.traced_globals.entry(name).or_default().push(hook);
                Ok(Object::Nil)
            }
            // Kernel#untrace_var — drop the hooks on a global. With a second
            // argument only that hook goes.
            "untrace_var" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    let message = format!(
                        "wrong number of arguments (given {}, expected 1..2)",
                        arguments.len()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                let name = global_name_from(&arguments[0]);
                match arguments.get(1) {
                    Some(hook) => {
                        if let Some(hooks) = self.traced_globals.get_mut(&name) {
                            hooks.retain(|existing| !existing.equals(hook));
                        }
                    }
                    None => {
                        self.traced_globals.remove(&name);
                    }
                }
                Ok(Object::Nil)
            }
            // Kernel#srand — reseed the generator and answer the seed it had.
            // With no argument it picks one, so successive calls differ.
            "srand" => {
                if arguments.len() > 1 {
                    return Err(MetorexError::runtime_error(
                        format!("srand() expects 0-1 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let seed = match arguments.first() {
                    None => crate::vm::core::seed_from_clock() as i64,
                    Some(given) => self.coerce_to_seed(given, position)?,
                };
                let previous = self.random_seed;
                self.random_seed = seed;
                // The generator takes the seed as-is; a seed of zero still has
                // to produce a usable sequence, so it is mixed rather than
                // used directly.
                self.random_state = (seed as u64) ^ 0x9E3779B97F4A7C15;
                Ok(Object::Int(previous))
            }
            "sleep" => Ok(Object::Int(0)),
            "puts" => {
                if arguments.is_empty() {
                    self.write_to_stdout("\n", position)?;
                }
                for arg in &arguments {
                    // Try to call to_s or inspect method if it exists on the object
                    let output = self.get_string_representation(arg, position)?;
                    self.write_to_stdout(&format!("{}\n", output), position)?;
                }
                Ok(Object::Nil)
            }
            "method" => {
                // method(:name) returns a Method object for the given method name
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!("method() expects 1 argument, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }

                let method_name = match &arguments[0] {
                    Object::Symbol(name) => name.as_str(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "method() expects a Symbol argument, got {}",
                                arguments[0].type_name()
                            ),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };

                // Look up the method in the current environment
                if let Some(obj) = self.environment().get(method_name) {
                    match obj {
                        Object::Method(_) => Ok(obj),
                        _ => Err(MetorexError::runtime_error(
                            format!("'{}' is not a method", method_name),
                            crate::vm::utils::position_to_location(position),
                        )),
                    }
                } else if let Some(receiver) = self.environment().get("self") {
                    // Inside an instance method a bare `method(:name)` means
                    // `self.method(:name)`, and the name is not a local.
                    let name = Object::Symbol(std::rc::Rc::new(method_name.to_string()));
                    self.send_to_object(receiver, "method", vec![name], position)
                } else {
                    Err(MetorexError::runtime_error(
                        format!("undefined method '{}'", method_name),
                        crate::vm::utils::position_to_location(position),
                    ))
                }
            }
            "require" => {
                // require(name) loads and executes a file from $LOAD_PATH
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!("require() expects 1 argument, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }

                let require_name = match &arguments[0] {
                    Object::String(path) => path.as_ref().clone(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "require() expects a String argument, got {}",
                                arguments[0].type_name()
                            ),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };

                // Search $LOAD_PATH for the file
                let load_path = self.globals().get(":").unwrap_or(Object::Nil);
                let search_dirs: Vec<String> = match &load_path {
                    Object::Array(arr) => arr
                        .borrow()
                        .iter()
                        .filter_map(|obj| match obj {
                            Object::String(s) => Some(s.as_ref().clone()),
                            _ => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                };

                let mut found_path = None;
                for dir in &search_dirs {
                    let base = std::path::PathBuf::from(dir);
                    // Prefer `.rb` file over a directory of the same name.
                    let candidates = [
                        base.join(format!("{}.rb", require_name)),
                        base.join(&require_name),
                    ];
                    for candidate in &candidates {
                        if candidate.is_file() {
                            found_path = Some(candidate.clone());
                            break;
                        }
                    }
                    if found_path.is_some() {
                        break;
                    }
                }

                let resolved = match found_path {
                    Some(p) => p,
                    None => {
                        // Raise a LoadError exception so Ruby-level rescue LoadError catches it.
                        let exc = Object::exception(
                            "LoadError",
                            format!("cannot load such file -- {}", require_name),
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: exc.clone(),
                            location: crate::vm::utils::position_to_location(position),
                            message: format!("{}", exc),
                        });
                    }
                };

                let canonical_path = resolved.canonicalize().map_err(|e| {
                    MetorexError::runtime_error(
                        format!(
                            "Failed to canonicalize path '{}': {}",
                            resolved.display(),
                            e
                        ),
                        crate::vm::utils::position_to_location(position),
                    )
                })?;

                let was_already_loaded = self.is_file_loaded(&canonical_path);

                self.execute_file(&resolved).map_err(|e| {
                    MetorexError::runtime_error(
                        format!("require('{}') — {}", require_name, e.message()),
                        crate::vm::utils::position_to_location(position),
                    )
                })?;

                Ok(Object::Bool(!was_already_loaded))
            }
            "require_relative" => {
                // require_relative(path) loads and executes a file relative to the current file
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "require_relative() expects 1 argument, got {}",
                            arguments.len()
                        ),
                        crate::vm::utils::position_to_location(position),
                    ));
                }

                let relative_path = match &arguments[0] {
                    Object::String(path) => path.as_ref(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "require_relative() expects a String argument, got {}",
                                arguments[0].type_name()
                            ),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };

                // Get current file path
                let current_file = self.get_current_file().ok_or_else(|| {
                    MetorexError::runtime_error(
                        "require_relative cannot be used without a current file context (e.g., in REPL)"
                            .to_string(),
                        crate::vm::utils::position_to_location(position),
                    )
                })?;

                // Resolve the relative path
                let resolved_path =
                    crate::file_loader::resolve_relative_path(current_file, relative_path)
                        .map_err(|e| {
                            MetorexError::runtime_error(
                                format!(
                                    "require_relative('{}') — cannot resolve path: {}",
                                    relative_path,
                                    e.message()
                                ),
                                crate::vm::utils::position_to_location(position),
                            )
                        })?;

                // Find the actual file path with extension auto-detection
                let actual_path =
                    crate::file_loader::find_file_path(&resolved_path).map_err(|e| {
                        MetorexError::runtime_error(
                            format!("require_relative('{}') — {}", relative_path, e.message()),
                            crate::vm::utils::position_to_location(position),
                        )
                    })?;

                // Canonicalize to get the absolute path for deduplication checking
                let canonical_path = actual_path.canonicalize().map_err(|e| {
                    MetorexError::runtime_error(
                        format!(
                            "Failed to canonicalize path '{}': {}",
                            actual_path.display(),
                            e
                        ),
                        crate::vm::utils::position_to_location(position),
                    )
                })?;

                // Check if file was already loaded BEFORE executing
                let was_already_loaded = self.is_file_loaded(&canonical_path);

                // Execute the file (it will handle its own deduplication)
                self.execute_file(&resolved_path).map_err(|e| {
                    MetorexError::runtime_error(
                        format!("require_relative('{}') — {}", relative_path, e.message()),
                        crate::vm::utils::position_to_location(position),
                    )
                })?;

                // Return true if newly loaded, false if already loaded (Ruby behavior)
                Ok(Object::Bool(!was_already_loaded))
            }
            // Kernel#print writes its arguments with no separator. With no
            // arguments it writes `$_`, the last line `gets` read.
            "print" => {
                if arguments.is_empty() {
                    let last_line = self.globals().get("_").unwrap_or(Object::Nil);
                    if !matches!(last_line, Object::Nil) {
                        let output = self.get_string_representation(&last_line, position)?;
                        self.write_to_stdout(&output, position)?;
                    }
                    return Ok(Object::Nil);
                }
                for arg in &arguments {
                    let output = self.get_string_representation(arg, position)?;
                    self.write_to_stdout(&output, position)?;
                }
                Ok(Object::Nil)
            }
            // Kernel#p writes each argument's `inspect` on its own line and
            // answers with the argument, the argument list, or nil for none.
            "p" => {
                for argument in &arguments {
                    let rendered = self.get_inspect_representation(argument, position)?;
                    self.write_to_stdout(&format!("{}\n", rendered), position)?;
                }
                match arguments.len() {
                    0 => Ok(Object::Nil),
                    1 => Ok(arguments.into_iter().next().unwrap()),
                    _ => Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                        arguments,
                    )))),
                }
            }
            // Kernel#readline — `gets` that refuses to answer nil: at end of
            // input it raises EOFError.
            "readline" => {
                if !arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        format!("readline() expects 0 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                match self.read_raw_line_from_stdin(position)? {
                    Some(line) => Ok(Object::string(line)),
                    None => {
                        let message = "end of file reached".to_string();
                        Err(MetorexError::UncaughtException {
                            exception: Object::exception("EOFError", message.clone()),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        })
                    }
                }
            }
            // Kernel#readlines — every remaining line, as an Array.
            "readlines" => {
                if !arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        format!("readlines() expects 0 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let mut lines = Vec::new();
                while let Some(line) = self.read_raw_line_from_stdin(position)? {
                    lines.push(Object::string(line));
                }
                Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                    lines,
                ))))
            }
            // `gets` is ARGF's, so a stand-in installed on ARGF answers here.
            "gets" => {
                if !arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        format!("gets() expects 0 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let argf = self.globals().get("ARGF").unwrap_or(Object::Nil);
                if let Some((class, method)) = self.lookup_method(&argf, "gets")
                    && !method.is_undefined
                {
                    return self.invoke_method(class, method, argf, vec![], position);
                }
                self.read_line_from_stdin(position)
            }
            "assert" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(MetorexError::runtime_error(
                        format!("assert() expects 1-2 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                if arguments[0].is_truthy() {
                    Ok(Object::Bool(true))
                } else {
                    let msg = if arguments.len() == 2 {
                        self.get_string_representation(&arguments[1], position)?
                    } else {
                        "Assertion failed".to_string()
                    };
                    Err(MetorexError::runtime_error(
                        msg,
                        crate::vm::utils::position_to_location(position),
                    ))
                }
            }
            "assert_equal" => {
                if arguments.len() < 2 || arguments.len() > 3 {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "assert_equal() expects 2-3 arguments, got {}",
                            arguments.len()
                        ),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                if arguments[0].equals(&arguments[1]) {
                    Ok(Object::Bool(true))
                } else {
                    let msg = if arguments.len() == 3 {
                        self.get_string_representation(&arguments[2], position)?
                    } else {
                        format!(
                            "Expected {}, got {}",
                            self.get_string_representation(&arguments[0], position)?,
                            self.get_string_representation(&arguments[1], position)?
                        )
                    };
                    Err(MetorexError::runtime_error(
                        msg,
                        crate::vm::utils::position_to_location(position),
                    ))
                }
            }
            "assert_raises" => {
                // assert_raises expects a block that should raise an error
                if !arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "assert_raises() expects 0 arguments (with a block), got {}",
                            arguments.len()
                        ),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "assert_raises requires a block",
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };
                match self.execute_block_body(&block, vec![]) {
                    Err(_) => Ok(Object::Bool(true)),
                    Ok(_) => Err(MetorexError::runtime_error(
                        "Expected block to raise an error, but it did not",
                        crate::vm::utils::position_to_location(position),
                    )),
                }
            }
            "parse" => {
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!("parse() expects 1 argument, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let code = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "parse() expects a String argument, got {}",
                                other.type_name()
                            ),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };
                let tokens = crate::lexer::Lexer::new(&code).tokenize();
                let statements = crate::parser::Parser::new(tokens)
                    .parse()
                    .map_err(|errors| {
                        MetorexError::runtime_error(
                            format!(
                                "parse: parse error: {}",
                                errors
                                    .iter()
                                    .map(|e| e.to_string())
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            ),
                            crate::vm::utils::position_to_location(position),
                        )
                    })?;
                use crate::vm::native_methods::ast_methods;
                Ok(ast_methods::serialize_statements(&statements))
            }
            "eval" => {
                if arguments.is_empty() || arguments.len() > 4 {
                    return Err(MetorexError::runtime_error(
                        format!("eval() expects 1-4 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let code = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "eval() expects a String argument, got {}",
                                other.type_name()
                            ),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };
                // Optional filename (arg 3) and lineno (arg 4) shape the
                // positions recorded for code inside the eval'd string
                // (`__LINE__`, const_source_location, backtraces).
                let filename = match arguments.get(2) {
                    Some(Object::String(s)) => Some(s.as_str().to_string()),
                    _ => None,
                };
                let lineno = match arguments.get(3) {
                    Some(Object::Int(n)) => (*n).max(1) as usize,
                    _ => 1,
                };
                let tokens = crate::lexer::Lexer::with_start_line(&code, lineno).tokenize();
                let statements = crate::parser::Parser::new(tokens)
                    .parse()
                    .map_err(|errors| {
                        MetorexError::runtime_error(
                            format!(
                                "eval: parse error: {}",
                                errors
                                    .iter()
                                    .map(|e| e.to_string())
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            ),
                            crate::vm::utils::position_to_location(position),
                        )
                    })?;
                // A Binding argument re-establishes the frame it captured:
                // its locals (shared cells, so assignment through the eval is
                // visible to a later one) and the `self` in force there.
                let binding = match arguments.get(1) {
                    Some(Object::Binding(b)) => Some(std::rc::Rc::clone(b)),
                    _ => None,
                };
                if let Some(b) = &binding {
                    self.environment_mut().push_isolated_scope();
                    for (name, cell) in &b.variables {
                        self.environment_mut()
                            .define_shared(name.clone(), std::rc::Rc::clone(cell));
                    }
                    if let Some(receiver) = &b.receiver {
                        self.environment_mut()
                            .define("self".to_string(), receiver.clone());
                    }
                }
                // eval runs at top-level of its string: treat as non-method scope.
                // Refinements activated inside eval are lexical to the eval string.
                let saved_nesting = self.user_def_nesting;
                self.user_def_nesting = 0;
                self.push_refinement_scope();
                let prev_file = self.current_file.clone();
                if let Some(f) = &filename {
                    self.current_file = Some(std::path::PathBuf::from(f));
                }
                // The eval'd string runs in the caller's body, so it sees the
                // visibility state in force there. A toggle it sets belongs to
                // the eval and is restored afterwards.
                let enclosing = match self.environment().get("self") {
                    Some(Object::Class(class) | Object::Module(class)) => {
                        Some((Rc::clone(&class), class.current_visibility()))
                    }
                    _ => None,
                };
                let result = self.execute_program(&statements);
                if binding.is_some() {
                    self.environment_mut().pop_scope();
                }
                if let Some((class, visibility)) = enclosing {
                    class.set_current_visibility(visibility);
                }
                self.current_file = prev_file;
                self.pop_refinement_scope();
                self.user_def_nesting = saved_nesting;
                Ok(result?.unwrap_or(Object::Nil))
            }
            // `catch(tag) { |tag| ... }` runs the block, answering a matching
            // `throw`'s value or, absent one, the block's own value. Called
            // with no tag it makes a fresh object and yields that.
            // `global_variables` names every global variable, sigil included.
            "global_variables" => {
                if !arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "global_variables() expects 0 arguments, got {}",
                            arguments.len()
                        ),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let names: Vec<Object> = self
                    .globals()
                    .variable_names()
                    .map(|name| Object::Symbol(std::rc::Rc::new(format!("${}", name))))
                    .collect();
                Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                    names,
                ))))
            }
            // Kernel#local_variables — the names bound in the current scope
            // chain, as Symbols. `self` is bound like a variable internally
            // but is not a local, and a name rebound in an inner scope is
            // reported once.
            "local_variables" => {
                if !arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "local_variables() expects 0 arguments, got {}",
                            arguments.len()
                        ),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let mut names: Vec<String> = self
                    .environment()
                    .local_variable_names()
                    .into_iter()
                    .filter(|name| {
                        name != "self"
                            && !self.seeded_global_names.contains(name)
                            && !self.environment().resolves_to_root_binding(name)
                            && !self
                                .environment()
                                .get(name)
                                .is_some_and(|value| self.name_is_a_definition(name, &value))
                    })
                    .collect();
                names.sort();
                names.dedup();
                let names: Vec<Object> = names
                    .into_iter()
                    .map(|name| Object::Symbol(std::rc::Rc::new(name)))
                    .collect();
                Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                    names,
                ))))
            }
            // `fail` is Ruby's other spelling of `raise`.
            // `fail` is Ruby's other spelling of `raise`. Both are reachable
            // as methods, so `send(:raise, ...)` and a singleton that makes
            // `raise` public find them here.
            "fail" | "raise" => {
                if arguments.len() > 2 {
                    return Err(MetorexError::runtime_error(
                        format!("{}() expects 0-2 arguments, got {}", name, arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let exception = self.build_raise_exception(&arguments, position)?;
                let message = match &exception {
                    Object::Exception(cell) => cell.borrow().message.clone(),
                    _ => String::new(),
                };
                Err(MetorexError::UncaughtException {
                    exception,
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
            // Kernel#loop — run the block until `break` or StopIteration.
            // `break value` is the loop's value; a StopIteration (or a
            // subclass) ends the loop and yields the iterator's result. Every
            // other exception propagates.
            "loop" => {
                if !arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        format!("loop() expects 0 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => block,
                    _ => {
                        let message = "no block given (yield)".to_string();
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("LocalJumpError", message.clone()),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        });
                    }
                };
                loop {
                    match block.call(self, Vec::new(), position) {
                        Ok(_) => {}
                        Err(MetorexError::BlockBreak { value, .. }) => return Ok(value),
                        Err(MetorexError::UncaughtException { exception, .. })
                            if self.exception_matches(
                                &exception,
                                &["StopIteration".to_string()],
                            )? =>
                        {
                            let _ = exception;
                            return Ok(Object::Nil);
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
            "catch" => {
                if arguments.len() > 1 {
                    return Err(MetorexError::runtime_error(
                        format!("catch() expects 0-1 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    _ => {
                        let message = "no block given (yield)".to_string();
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("LocalJumpError", message.clone()),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        });
                    }
                };
                let tag = match arguments.first() {
                    Some(tag) => tag.clone(),
                    None => match self.globals().get("Object") {
                        Some(Object::Class(object_class)) => Object::instance(object_class),
                        _ => Object::Nil,
                    },
                };
                self.catch_tags.push(tag.clone());
                let block_arguments = if block.binding_parameters().is_empty() {
                    Vec::new()
                } else {
                    vec![tag.clone()]
                };
                let result = block.call(self, block_arguments, position);
                self.catch_tags.pop();
                match result {
                    Err(MetorexError::Throw {
                        tag: thrown, value, ..
                    }) if throw_tags_match(&tag, &thrown) => Ok(value),
                    other => other,
                }
            }
            // `throw tag, value` unwinds to the matching `catch`. Ruby raises
            // UncaughtThrowError when no live catch holds the tag, rather than
            // unwinding out of the program.
            "throw" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    let message = format!(
                        "wrong number of arguments (given {}, expected 1..2)",
                        arguments.len()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                let tag = arguments[0].clone();
                if !self
                    .catch_tags
                    .iter()
                    .any(|live| throw_tags_match(live, &tag))
                {
                    let message = format!(
                        "uncaught throw {}",
                        crate::vm::native_methods::array_methods::inspect_element(&tag)
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("UncaughtThrowError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                Err(MetorexError::Throw {
                    tag,
                    value: arguments.get(1).cloned().unwrap_or(Object::Nil),
                    location: crate::vm::utils::position_to_location(position),
                })
            }
            "load" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(MetorexError::runtime_error(
                        format!("load() expects 1-2 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let wrap = matches!(arguments.get(1), Some(Object::Bool(true)));
                let path_str = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "load() expects a String argument, got {}",
                                arguments[0].type_name()
                            ),
                            crate::vm::utils::position_to_location(position),
                        ));
                    }
                };
                let path = std::path::Path::new(&path_str);
                if wrap {
                    self.load_wrap_depth += 1;
                }
                // Each load() has its own refinement scope and a fresh
                // user-method-nesting counter for its top-level statements.
                self.push_refinement_scope();
                let saved_nesting = self.user_def_nesting;
                self.user_def_nesting = 0;
                // load always executes the file (no deduplication)
                // Try the path directly first, then search $LOAD_PATH
                let result = if path.exists() {
                    self.execute_file(path).map_err(|e| {
                        MetorexError::runtime_error(
                            format!("load('{}') — {}", path_str, e.message()),
                            crate::vm::utils::position_to_location(position),
                        )
                    })
                } else {
                    // Search $LOAD_PATH
                    let load_path = self.globals().get(":").unwrap_or(Object::Nil);
                    let search_dirs: Vec<String> = match &load_path {
                        Object::Array(arr) => arr
                            .borrow()
                            .iter()
                            .filter_map(|obj| match obj {
                                Object::String(s) => Some(s.as_ref().clone()),
                                _ => None,
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    let mut found = None;
                    for dir in &search_dirs {
                        let candidate = std::path::PathBuf::from(dir).join(&path_str);
                        if candidate.exists() {
                            found = Some(candidate);
                            break;
                        }
                    }
                    match found {
                        Some(resolved) => self.execute_file(&resolved).map_err(|e| {
                            MetorexError::runtime_error(
                                format!("load('{}') — {}", path_str, e.message()),
                                crate::vm::utils::position_to_location(position),
                            )
                        }),
                        None => Err(MetorexError::runtime_error(
                            format!("cannot load such file -- {}", path_str),
                            crate::vm::utils::position_to_location(position),
                        )),
                    }
                };
                if wrap {
                    self.load_wrap_depth -= 1;
                }
                self.pop_refinement_scope();
                self.user_def_nesting = saved_nesting;
                result?;
                Ok(Object::Bool(true))
            }
            "exit" | "exit!" => {
                let code = if arguments.is_empty() {
                    0
                } else if let Object::Int(n) = &arguments[0] {
                    *n as i32
                } else if let Object::Bool(b) = &arguments[0] {
                    if *b { 0 } else { 1 }
                } else {
                    0
                };
                std::process::exit(code);
            }
            "abort" => {
                let message = match arguments.first() {
                    Some(argument) => {
                        let text = self.coerce_abort_message(argument, position)?;
                        self.emit_warning_to_stderr(&text, position);
                        text
                    }
                    None => "SystemExit".to_string(),
                };
                let exception = Object::Exception(std::rc::Rc::new(std::cell::RefCell::new(
                    crate::object::Exception {
                        exception_type: "SystemExit".to_string(),
                        message: message.clone(),
                        backtrace: None,
                        location: None,
                        cause: None,
                        status: Some(1),
                        name: None,
                    },
                )));
                Err(MetorexError::UncaughtException {
                    exception,
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
            "system" => {
                let Some(command) = arguments.first() else {
                    return Err(MetorexError::runtime_error(
                        "system requires at least 1 argument".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                };
                let program = self.get_string_representation(command, position)?;
                let mut rest = Vec::new();
                for arg in arguments.iter().skip(1) {
                    rest.push(self.get_string_representation(arg, position)?);
                }
                let status = if rest.is_empty() {
                    std::process::Command::new("/bin/sh")
                        .arg("-c")
                        .arg(&program)
                        .status()
                } else {
                    std::process::Command::new(&program).args(&rest).status()
                };
                Ok(match status {
                    Ok(status) => Object::Bool(status.success()),
                    Err(_) => Object::Nil,
                })
            }
            "fork" => {
                let message = "fork() function is unimplemented on this machine".to_string();
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("NotImplementedError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
            _ => Err(MetorexError::runtime_error(
                format!("Unknown native function: {}", name),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }

    /// Read one line from stdin, without its line ending.
    pub(crate) fn read_line_from_stdin(
        &mut self,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match self.read_raw_line_from_stdin(position)? {
            Some(line) => Ok(Object::string(line)),
            None => Ok(Object::Nil),
        }
    }

    /// One line of stdin with its terminator trimmed, or `None` at end of
    /// input. `gets` answers nil there and `readline` raises EOFError.
    fn read_raw_line_from_stdin(
        &mut self,
        position: Position,
    ) -> Result<Option<String>, MetorexError> {
        let mut input = String::new();
        let read = std::io::stdin().read_line(&mut input).map_err(|error| {
            MetorexError::runtime_error(
                format!("Failed to read from stdin: {}", error),
                crate::vm::utils::position_to_location(position),
            )
        })?;
        if read == 0 {
            return Ok(None);
        }
        if input.ends_with('\n') {
            input.pop();
            if input.ends_with('\r') {
                input.pop();
            }
        }
        Ok(Some(input))
    }

    /// Coerce `abort`'s argument to a String the way Ruby does: a String is
    /// taken as is, anything else must answer `to_str`, and a receiver without
    /// one raises TypeError.
    fn coerce_abort_message(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = argument {
            return Ok((**text).clone());
        }
        if let Some((class, method)) = self.lookup_method(argument, "to_str")
            && !method.is_undefined
        {
            let converted =
                self.invoke_method(class, method, argument.clone(), vec![], position)?;
            if let Object::String(text) = converted {
                return Ok((*text).clone());
            }
        }
        let source_class = self.builtins().class_of(argument).name().to_string();
        let message = format!("no implicit conversion of {} into String", source_class);
        Err(MetorexError::UncaughtException {
            exception: Object::exception("TypeError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }

    /// Get the string representation of an object by calling to_s or inspect if available.
    fn get_string_representation(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        // First try to_s, then inspect, then fall back to Display
        match obj {
            // `:name.to_s` is the bare name; only `inspect` keeps the colon.
            Object::Symbol(name) => Ok((**name).clone()),
            Object::Instance(_) => {
                // Try to_s first
                if let Some((class, method)) = self.lookup_method(obj, "to_s") {
                    let result =
                        self.invoke_method(class, method, obj.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok(s.to_string());
                    }
                }
                // Try inspect as fallback
                if let Some((class, method)) = self.lookup_method(obj, "inspect") {
                    let result =
                        self.invoke_method(class, method, obj.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok(s.to_string());
                    }
                }
                // Classes whose `to_s` is native, such as Rational, have no
                // entry in any method map for the lookups above to find.
                let class = self.builtins().class_of(obj);
                if let Some(Object::String(rendered)) =
                    self.call_native_method(&class, obj, "to_s", &[], position)?
                {
                    return Ok(rendered.to_string());
                }
                // Fall back to default Display
                Ok(format!("{}", obj))
            }
            _ => Ok(format!("{}", obj)),
        }
    }

    /// The string `inspect` produces for an object, preferring a method the
    /// object defines over the native rendering.
    pub(crate) fn get_inspect_representation(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Some((class, method)) = self.lookup_method(obj, "inspect")
            && !method.is_undefined
        {
            let result = self.invoke_method(class, method, obj.clone(), vec![], position)?;
            if let Object::String(text) = result {
                return Ok(text.to_string());
            }
        }
        let class = self.builtins().class_of(obj);
        if let Some(Object::String(rendered)) =
            self.call_native_method(&class, obj, "inspect", &[], position)?
        {
            return Ok(rendered.to_string());
        }
        Ok(format!("{}", obj))
    }

    /// The Integer `srand` seeds with. A Float truncates and any other object
    /// must answer `#to_int`, as Ruby requires.
    fn coerce_to_seed(&mut self, given: &Object, position: Position) -> Result<i64, MetorexError> {
        match given {
            Object::Int(seed) => Ok(*seed),
            Object::Float(seed) => Ok(*seed as i64),
            other => {
                let Some((class, method)) = self.lookup_method(other, "to_int") else {
                    let message = format!(
                        "no implicit conversion of {} into Integer",
                        self.builtins().class_of(other).name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                let converted =
                    self.invoke_method(class, method, other.clone(), vec![], position)?;
                self.coerce_to_seed(&converted, position)
            }
        }
    }

    /// Run the hooks `trace_var` registered for `name`, with the value just
    /// assigned. A String hook is evaluated as code, the way Ruby's is.
    pub(crate) fn fire_global_trace(
        &mut self,
        name: &str,
        value: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let Some(hooks) = self.traced_globals.get(name).cloned() else {
            return Ok(());
        };
        for hook in hooks {
            match hook {
                Object::String(code) => {
                    let tokens = crate::lexer::Lexer::new(&code).tokenize();
                    let statements =
                        crate::parser::Parser::new(tokens)
                            .parse()
                            .map_err(|errors| {
                                MetorexError::runtime_error(
                                    format!(
                                        "trace_var: parse error: {}",
                                        errors
                                            .iter()
                                            .map(|error| error.to_string())
                                            .collect::<Vec<_>>()
                                            .join("; ")
                                    ),
                                    crate::vm::utils::position_to_location(position),
                                )
                            })?;
                    for statement in &statements {
                        self.execute_statement(statement)?;
                    }
                }
                callable => {
                    self.invoke_callable(callable, vec![value.clone()], position)?;
                }
            }
        }
        Ok(())
    }

    /// Advance the generator and answer the next 64 bits. A SplitMix64 step,
    /// which needs no state beyond the seed itself.
    fn next_random_bits(&mut self) -> u64 {
        self.random_state = self.random_state.wrapping_add(0x9E3779B97F4A7C15);
        let mut mixed = self.random_state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D049BB133111EB);
        mixed ^ (mixed >> 31)
    }

    /// The next draw as a Float in [0, 1).
    pub(crate) fn next_random_float(&mut self) -> f64 {
        // 53 bits is the whole mantissa, so every representable value in the
        // interval is reachable and none round up to 1.0.
        (self.next_random_bits() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// The next draw as an Integer in [0, bound).
    fn next_random_int(&mut self, bound: i64) -> i64 {
        if bound <= 0 {
            return 0;
        }
        (self.next_random_bits() % bound as u64) as i64
    }

    /// `Kernel#rand(limit)` for every argument shape Ruby accepts.
    fn random_below(&mut self, limit: Object, position: Position) -> Result<Object, MetorexError> {
        match limit {
            // Ruby ignores the sign, and a bound of zero means "no bound",
            // which draws a Float instead.
            Object::Int(bound) => match bound.unsigned_abs() {
                0 => Ok(Object::Float(self.next_random_float())),
                magnitude => Ok(Object::Int(self.next_random_int(magnitude as i64))),
            },
            // A Float bound truncates. `rand(0.999)` truncates to zero, so it
            // draws a Float the way `rand(0)` does.
            Object::Float(bound) => match bound.abs().trunc() as i64 {
                0 => Ok(Object::Float(self.next_random_float())),
                magnitude => Ok(Object::Int(self.next_random_int(magnitude))),
            },
            Object::Range {
                ref start,
                ref end,
                exclusive,
            } => self.random_in_range(start, end, exclusive, position),
            other => {
                // Anything else is asked for an Integer bound.
                let Some((class, method)) = self.lookup_method(&other, "to_int") else {
                    let message = format!(
                        "no implicit conversion of {} into Integer",
                        self.builtins().class_of(&other).name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                let converted = self.invoke_method(class, method, other, vec![], position)?;
                self.random_below(converted, position)
            }
        }
    }

    /// `Kernel#rand(range)`. An all-Integer range draws an Integer; a Float on
    /// either side draws a Float. A backwards range answers nil.
    fn random_in_range(
        &mut self,
        start: &Object,
        end: &Object,
        exclusive: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let (Object::Int(low), Object::Int(high)) = (start, end) {
            let span = if exclusive {
                high - low
            } else {
                high - low + 1
            };
            if span <= 0 {
                return Ok(Object::Nil);
            }
            return Ok(Object::Int(low + self.next_random_int(span)));
        }
        let (Some(low), Some(high)) = (numeric_value(start), numeric_value(end)) else {
            let message = "bad value for range";
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.to_string()),
                location: crate::vm::utils::position_to_location(position),
                message: message.to_string(),
            });
        };
        if high < low || (exclusive && high == low) {
            return Ok(Object::Nil);
        }
        if high == low {
            return Ok(Object::Float(low));
        }
        Ok(Object::Float(low + self.next_random_float() * (high - low)))
    }

    /// Write `text` where `$stdout` points. The default is the process's own
    /// stdout; when a program (or a spec harness) assigns an object with its
    /// own `write`, the text goes there instead.
    pub(crate) fn write_to_stdout(
        &mut self,
        text: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        self.write_to_stream("stdout", text, position)
    }

    /// The `$stderr` counterpart of `write_to_stdout`.
    pub(crate) fn write_to_stderr(
        &mut self,
        text: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        self.write_to_stream("stderr", text, position)
    }

    fn write_to_stream(
        &mut self,
        stream: &str,
        text: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let target = self.globals().get(stream).unwrap_or(Object::Nil);
        if let Some((class, method)) = self.lookup_method(&target, "write")
            && !method.is_undefined
        {
            let argument = Object::string(text.to_string());
            self.invoke_method(class, method, target, vec![argument], position)?;
            return Ok(());
        }
        use std::io::Write;
        if stream == "stderr" {
            eprint!("{}", text);
            let _ = std::io::stderr().flush();
        } else {
            print!("{}", text);
            let _ = std::io::stdout().flush();
        }
        Ok(())
    }

    /// Apply `private` / `public` visibility modifier to top-level methods.
    /// At top level, method definitions target the Object class.
    pub(crate) fn apply_visibility_modifier(
        &mut self,
        modifier: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // No arguments: no-op (in Ruby this toggles subsequent-definition visibility).
        if arguments.is_empty() {
            return Ok(Object::Nil);
        }

        // A single array argument is unpacked.
        let flat: Vec<Object> = if arguments.len() == 1 {
            if let Object::Array(arr) = &arguments[0] {
                arr.borrow().clone()
            } else {
                arguments.clone()
            }
        } else {
            arguments.clone()
        };

        let Some(Object::Class(object_class)) = self.globals().get("Object") else {
            return Ok(Object::Nil);
        };

        let mut names: Vec<String> = Vec::with_capacity(flat.len());
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
                        location: crate::vm::utils::position_to_location(position),
                        message: format!("{} is not a symbol nor a string", arg),
                    });
                }
            };
            if object_class.find_method(&n).is_none() {
                let msg = format!("undefined method '{}' for class 'Object'", n);
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: crate::vm::utils::position_to_location(position),
                    message: msg,
                });
            }
            names.push(n);
        }

        for n in &names {
            if modifier == "private" {
                object_class.set_method_private(n.clone());
            } else {
                object_class.set_method_public(n);
            }
        }

        // Return first symbol argument (Ruby returns single sym or array for multi).
        match flat.len() {
            1 => Ok(Object::Symbol(std::rc::Rc::new(names[0].clone()))),
            _ => Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                names
                    .into_iter()
                    .map(|n| Object::Symbol(std::rc::Rc::new(n)))
                    .collect(),
            )))),
        }
    }
}

/// Whether a thrown tag names the same object a `catch` is holding. Ruby
/// matches by identity, so two equal Strings are different tags while a
/// Symbol is only ever itself.
fn throw_tags_match(live: &Object, thrown: &Object) -> bool {
    use std::rc::Rc;
    match (live, thrown) {
        (Object::String(a), Object::String(b)) => Rc::ptr_eq(a, b),
        (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
        (Object::Array(a), Object::Array(b)) => Rc::ptr_eq(a, b),
        (Object::Dict(a), Object::Dict(b)) => Rc::ptr_eq(a, b),
        (Object::Class(a), Object::Class(b)) => Rc::ptr_eq(a, b),
        (Object::Module(a), Object::Module(b)) => Rc::ptr_eq(a, b),
        (Object::Symbol(a), Object::Symbol(b)) => a == b,
        (Object::Int(a), Object::Int(b)) => a == b,
        (Object::Bool(a), Object::Bool(b)) => a == b,
        (Object::Nil, Object::Nil) => true,
        _ => false,
    }
}

/// The numeric value of a Range endpoint, when it has one.
fn numeric_value(object: &Object) -> Option<f64> {
    match object {
        Object::Int(value) => Some(*value as f64),
        Object::Float(value) => Some(*value),
        _ => None,
    }
}

/// The global's name without its `$`, however it was named.
fn global_name_from(named: &Object) -> String {
    let text = match named {
        Object::Symbol(name) | Object::String(name) => (**name).clone(),
        other => other.to_string(),
    };
    text.strip_prefix('$').unwrap_or(&text).to_string()
}
