//! Native (built-in) function implementations for the virtual machine.
//!
//! This module contains implementations of global built-in functions like puts, print, etc.

use super::VirtualMachine;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// What a command wrote, as a String in the encoding this process reads
    /// and writes. Bytes that spell no text stand for themselves.
    pub(crate) fn command_output(&mut self, written: &[u8]) -> Object {
        let held = self.globals().get("__Encoding_default_external");
        let named = match held {
            Some(setting) => {
                match self.send_to_object(setting, "name", Vec::new(), Position::default()) {
                    Ok(Object::String(text)) => text.as_str().to_string(),
                    _ => "UTF-8".to_string(),
                }
            }
            None => "UTF-8".to_string(),
        };
        let made = match std::str::from_utf8(written) {
            Ok(text) => crate::object::StringValue::new(text),
            Err(_) => crate::object::StringValue::from_bytes(
                written.iter().map(|byte| *byte as char).collect::<String>(),
            ),
        };
        made.set_encoding(named);
        Object::String(std::rc::Rc::new(made))
    }

    /// Call a native function by name.
    pub(crate) fn call_native_function(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // The receiver the call was written with reaches the function invoked
        // here and no further, so a function that runs code of its own does
        // not hand it on to whatever that code calls.
        let invoked_with = self.kernel_function_receiver.take();
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
            "private_constant" | "public_constant" | "deprecate_constant" => {
                // Apply visibility marks to constants on the current `self`
                // module/class. Inside a `module M; ...; end` body, `self`
                // is the module being defined, so qualified accesses like
                // `M::PrivConst` from outside raise NameError /private
                // constant/. `public_constant` is the inverse, and
                // `deprecate_constant` says a read of the name warns.
                self.pending_block.take();
                let target = match self.environment().get("self") {
                    Some(Object::Class(c)) | Some(Object::Module(c)) => c,
                    _ => return Ok(Object::Nil),
                };
                for arg in &arguments {
                    let const_name = match arg {
                        Object::Symbol(s) => s.as_str().to_string(),
                        Object::String(s) => s.as_str().to_string(),
                        _ => continue,
                    };
                    match name {
                        "private_constant" => target.mark_private_constant(const_name),
                        "deprecate_constant" => target.mark_deprecated_constant(const_name),
                        _ => target.unmark_private_constant(&const_name),
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
                        names.push(Object::symbol(name));
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
            "noop_with_block" => {
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
            // `END { ... }` registers its body once, however many times the
            // line holding it is read.
            "__end_once__" => {
                let Some(block) = self.pending_block.take() else {
                    let message = "called without a block".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                let site = (
                    self.current_source_file.clone().unwrap_or_default(),
                    position.line,
                    position.column,
                );
                if self.opened_blocks.insert(site) {
                    self.at_exit_handlers.push(block.clone());
                }
                Ok(block)
            }
            "at_exit" => {
                let Some(block) = self.pending_block.take() else {
                    let message = "called without a block".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("ArgumentError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                };
                self.at_exit_handlers.push(block.clone());
                Ok(block)
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
            // `trap` is Kernel's name for `Signal.trap`.
            "trap" => self.install_signal_trap(&arguments, position),
            "sprintf" => {
                if arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        "sprintf requires at least 1 argument".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                // Ruby converts the format with `#to_str`, so a non-String
                // that answers it works and anything else raises TypeError.
                let fmt = match &arguments[0] {
                    held @ Object::String(_) => held.clone(),
                    other => Object::string(self.coerce_name_argument(other, position)?),
                };
                let rest: Vec<Object> = arguments.into_iter().skip(1).collect();
                self.format_with_values(&fmt, rest, position)
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
                    Object::symbol(chosen)
                }
                None => Object::Nil,
            }),
            // `caller` reports the same frames `caller_locations` does, each
            // rendered the way a backtrace entry reads.
            "caller" => {
                let Some(locations) = self.sliced_caller_locations(&arguments, position)? else {
                    return Ok(Object::Nil);
                };
                let entries: Vec<Object> = locations
                    .iter()
                    .map(|location| {
                        let read = |name: &str| match location {
                            Object::Instance(instance) => instance
                                .borrow()
                                .get_var(name)
                                .cloned()
                                .unwrap_or(Object::Nil),
                            _ => Object::Nil,
                        };
                        let path = match read("path") {
                            Object::String(path) => path.as_str().to_string(),
                            _ => String::new(),
                        };
                        let line = match read("lineno") {
                            Object::Int(line) => line,
                            _ => 0,
                        };
                        let label = match read("label") {
                            Object::String(label) => label.as_str().to_string(),
                            _ => String::new(),
                        };
                        Object::string(if label.is_empty() {
                            format!("{}:{}", path, line)
                        } else {
                            format!("{}:{}:in '{}'", path, line, label)
                        })
                    })
                    .collect();
                Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                    entries,
                ))))
            }
            // `caller_locations(start = 1, length = nil)` — walk the VM call
            // stack. Each frame stores the source position it was called
            // from, so level 1 (the caller of the current method) reads the
            // top frame's recorded location. Returns Location objects
            // responding to `lineno` and `path`.
            "caller_locations" => match self.sliced_caller_locations(&arguments, position)? {
                Some(locations) => Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                    locations,
                )))),
                None => Ok(Object::Nil),
            },
            // `binding` captures the frame that called it, not the receiver
            // it was sent to: the local variables in scope (as shared cells,
            // so an assignment through the binding is visible to both) and the
            // `self` in force there.
            "binding_kernel" => {
                // Only the locals in force where the call sits belong to a
                // binding. The builtins the root scope holds are constants
                // and methods, which `local_variables` does not name.
                let named = self.environment().binding_variable_names();
                let mut variables = std::collections::HashMap::new();
                let mut order: Vec<String> = Vec::new();
                for name in named {
                    // The root scope also holds the builtins, which are not
                    // locals of the program. A name the globals hold is one
                    // of those only while it still answers with the same
                    // object: a local of the same name shadows it, and that
                    // one is a local like any other.
                    if let Some(builtin) = self.globals().get(&name)
                        && self.environment().get(&name) == Some(builtin)
                    {
                        continue;
                    }
                    // A local is named the way Ruby lets one be named, which
                    // rules out the globals and constants sharing the scope.
                    if !name.starts_with(|held: char| held == '_' || held.is_lowercase()) {
                        continue;
                    }
                    if variables.contains_key(&name) {
                        continue;
                    }
                    if let Some(cell) = self.environment().get_ref(&name) {
                        order.push(name.clone());
                        variables.insert(name, cell);
                    }
                }
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
                let held = crate::object::Binding::with_receiver(variables, receiver);
                held.set_order(order);
                // The method the call sits in, which code run through the
                // binding names as its own.
                *held.method.borrow_mut() = self.enclosing_method_names();
                // The refinements in force here, which code run through the
                // binding sees.
                *held.refinements.borrow_mut() = self.snapshot_active_refinements();
                // The classes and modules open here, which a class opened by
                // code run through the binding is nested in.
                *held.nesting.borrow_mut() = match self.method_nesting_stack.last() {
                    Some(captured) => captured.clone(),
                    None => self.snapshot_lexical_nesting(),
                };
                // Where the call sits, which `source_location` reports.
                *held.source.borrow_mut() = Some((
                    self.current_file
                        .as_ref()
                        .map(|file| file.display().to_string())
                        .unwrap_or_default(),
                    position.line,
                ));
                Ok(Object::Binding(std::rc::Rc::new(held)))
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
                Ok(Object::symbol(method_name))
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
                    None => Object::Int(crate::vm::core::seed_from_clock() as i64),
                    Some(given) => self.coerce_to_seed(given, position)?,
                };
                let words = seed.as_big_integer().unwrap_or_default();
                let previous = std::mem::replace(&mut self.random_seed, seed);
                self.seed_random(&words);
                Ok(previous)
            }
            // Metorex does not hold the program up: a sleep answers at once.
            // A sleep inside `Timeout.timeout` that would run past the limit
            // is the one thing it reports on, since that is what the block
            // was given a limit for.
            "sleep" => {
                let wanted = match arguments.first() {
                    None | Some(Object::Nil) => None,
                    Some(held) => Some(self.float_value_of(held, position)?),
                };
                if let Some((deadline, class, message)) = self.timeout_limits.last().cloned() {
                    let left = deadline.saturating_duration_since(std::time::Instant::now());
                    if wanted.is_none_or(|seconds| seconds > left.as_secs_f64()) {
                        self.call_native_function("raise", vec![class, message], position)?;
                    }
                }
                // Sleeping hands the turn over, so whatever else the program
                // has to run gets one while this waits. With no length at all
                // the wait lasts until something wakes the thread.
                if wanted.is_none() {
                    self.sleep_until_woken(position)?;
                } else {
                    self.wait_for_other_threads(position);
                    self.raise_if_thread_killed(position)?;
                }
                Ok(Object::Int(wanted.unwrap_or(0.0) as i64))
            }
            // `Timeout.timeout` opens a limit around the block it runs, and
            // closes it however the block ends.
            "__timeout_open__" => {
                let seconds = self.float_value_of(&arguments[0], position)?;
                let deadline = std::time::Instant::now()
                    + std::time::Duration::from_secs_f64(seconds.max(0.0));
                self.timeout_limits
                    .push((deadline, arguments[1].clone(), arguments[2].clone()));
                Ok(Object::Nil)
            }
            "__timeout_close__" => {
                self.timeout_limits.pop();
                Ok(Object::Nil)
            }
            // The primitive behind the Math module: the function named by the
            // first argument, applied to the numbers that follow.
            "__math_function__" => self.apply_math_function(&arguments, position),
            "puts" => {
                // Ruby's `puts` hands its arguments to `$stdout.puts`, which
                // is what a program replacing that stream relies on.
                let target = self.globals().get("stdout").unwrap_or(Object::Nil);
                if !matches!(target, Object::Nil) && self.responds_to(&target, "puts") {
                    self.send_to_object(target, "puts", arguments, position)?;
                    return Ok(Object::Nil);
                }
                if arguments.is_empty() {
                    self.write_to_stdout("\n", position)?;
                }
                for arg in &arguments {
                    self.puts_object(arg, position)?;
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

                // A name may be written as a Symbol or as a String, which is
                // what `method("hello")` passes.
                let method_name = match &arguments[0] {
                    Object::Symbol(name) => name.as_str(),
                    Object::String(name) => name.as_str(),
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

                // A `def` outside any class or module writes the method on
                // Object, which is where a bare name at the top level is
                // answered from and what it reports as its owner.
                let written_on_object = match self.globals().get("Object") {
                    Some(object_class) => self.top_level_method(&object_class, &method_name),
                    None => None,
                };
                // At the top level the program runs against `main`, which
                // is where a bare `method(:name)` looks when the scope binds
                // no `self` of its own.
                let here = self
                    .environment()
                    .get("self")
                    .or_else(|| self.globals().get("__main__"));
                // Look up the method in the current environment
                if let Some(obj) = self.environment().get(&method_name) {
                    if let Object::Method(held) = &obj {
                        if held.owner_class.is_none()
                            && let Some(found) = written_on_object
                        {
                            return Ok(found);
                        }
                        return Ok(obj);
                    }
                    // A name the environment holds as something other than a
                    // method may still name one the receiver defines, which is
                    // what `def p(a); end` does to the builtin of that name.
                    let not_a_method = MetorexError::runtime_error(
                        format!("'{}' is not a method", method_name),
                        crate::vm::utils::position_to_location(position),
                    );
                    let Some(receiver) = here else {
                        return Err(not_a_method);
                    };
                    let name = Object::symbol(method_name.to_string());
                    self.send_to_object(receiver, "method", vec![name], position)
                        .map_err(|_| not_a_method)
                } else if let Some(receiver) = here {
                    // Inside an instance method a bare `method(:name)` means
                    // `self.method(:name)`, and the name is not a local.
                    let name = Object::symbol(method_name.to_string());
                    self.send_to_object(receiver.clone(), "method", vec![name], position)
                        .or_else(
                            |error| match self.top_level_method(&receiver, &method_name) {
                                Some(held) => Ok(held),
                                None => Err(error),
                            },
                        )
                } else if let Some(found) = written_on_object {
                    Ok(found)
                } else {
                    Err(MetorexError::runtime_error(
                        format!("undefined method '{}'", method_name),
                        crate::vm::utils::position_to_location(position),
                    ))
                }
            }
            // Bare `autoload` / `autoload?` register on the definee, which is
            // Object at the top level and the enclosing module inside one.
            "autoload" | "autoload?" => {
                // Inside a method the definee is the module the method was
                // written in, which a module's instance method reaches
                // through the nesting it captured.
                let owner = match self.autoload_definee() {
                    Some(enclosing) => enclosing,
                    None => match self.globals().get("Object") {
                        Some(Object::Class(object_class)) => object_class,
                        _ => {
                            return Err(MetorexError::runtime_error(
                                "Object is not defined",
                                crate::vm::utils::position_to_location(position),
                            ));
                        }
                    },
                };
                self.call_class_methods(&owner, name, &arguments, position)
                    .map(|result| result.unwrap_or(Object::Nil))
            }
            // Kernel#` — run the command through the shell, answering what it
            // wrote to stdout. Its stderr passes through to ours, and `$?`
            // reports how it ended.
            "`" => {
                let Some(argument) = arguments.first() else {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Exact(1),
                        0,
                        position,
                    ));
                };
                let command = self.coerce_command_argument(argument, position)?;
                // The shell reads bytes, so a command whose characters stand
                // for bytes is handed those rather than their text form.
                let written: std::ffi::OsString = match argument {
                    Object::String(text) if text.holds_bytes() => {
                        use std::os::unix::ffi::OsStringExt;
                        std::ffi::OsString::from_vec(
                            crate::vm::native_methods::string_methods::binary_bytes(text),
                        )
                    }
                    _ => std::ffi::OsString::from(command.clone()),
                };
                // Spawn rather than run to completion in one step, so the
                // child's process id is read before it is waited for and
                // `$?.pid` can report it.
                let spawned = std::process::Command::new("/bin/sh")
                    .arg("-c")
                    .arg(&written)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::inherit())
                    .spawn();
                let output = match spawned {
                    Ok(child) => {
                        let child_pid = child.id() as i64;
                        child.wait_with_output().map(|output| (output, child_pid))
                    }
                    Err(error) => Err(error),
                };
                let (output, child_pid) = match output {
                    Ok(pair) => pair,
                    Err(error) => {
                        let message =
                            format!("No such file or directory - {} ({})", command, error);
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("Errno::ENOENT", message.clone()),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        });
                    }
                };
                self.record_last_status(&output.status, Some(child_pid));
                // A command the shell could not find ends with status 127,
                // which Ruby reports as Errno::ENOENT from the spawn itself.
                if output.status.code() == Some(127) {
                    let message = format!("No such file or directory - {}", command);
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("Errno::ENOENT", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                Ok(self.command_output(&output.stdout))
            }
            // `chomp` and `chop` with no receiver rewrite `$_` in place, which
            // is the line `-n` read. `chomp` takes the separator from `$/`.
            "chomp" | "chop" => {
                let line = match self.globals().get("_") {
                    Some(Object::String(text)) => text.as_str().to_string(),
                    _ => String::new(),
                };
                let separator = match arguments.first() {
                    Some(Object::String(text)) => Some(text.as_str().to_string()),
                    _ => match self.globals().get("/") {
                        Some(Object::String(text)) => Some(text.as_str().to_string()),
                        _ => None,
                    },
                };
                let trimmed = if name == "chop" {
                    let mut trimmed = line.clone();
                    if trimmed.ends_with("\r\n") {
                        trimmed.truncate(trimmed.len() - 2);
                    } else {
                        trimmed.pop();
                    }
                    trimmed
                } else {
                    chomped(&line, separator.as_deref())
                };
                let result = Object::string(trimmed);
                self.globals_mut().set_variable("_", result.clone());
                Ok(result)
            }
            // `open(path, mode = "r", perm = nil, **options)` opens a file.
            // An argument answering `to_open` is asked to open itself, and
            // whatever it answers is what `open` hands back.
            "open" => {
                // Any conversion below invokes a method, which would consume
                // the block, so it is taken first and put back at the end.
                let pending = self.pending_block.take();
                let (positional, _keywords) =
                    super::native_methods::kernel_conversion::split_conversion_keywords(&arguments);
                let Some(target) = positional.first() else {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(1, 3),
                        0,
                        position,
                    ));
                };
                let target = &target.clone();
                if self.responds_to(target, "to_open") {
                    let Some((class, method)) = self.lookup_method(target, "to_open") else {
                        return Ok(Object::Nil);
                    };
                    let opened = self.invoke_method(
                        class,
                        method,
                        target.clone(),
                        arguments[1..].to_vec(),
                        position,
                    )?;
                    if let Some(Object::Block(block)) = pending {
                        return self.execute_block_callable(&block, vec![opened], position);
                    }
                    return Ok(opened);
                }
                // Only the file form is limited to path, mode, and
                // permissions; `to_open` takes whatever it is given.
                if positional.len() > 3 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(1, 3),
                        positional.len(),
                        position,
                    ));
                }
                let path = self.coerce_load_path(target, position)?;
                let mut open_arguments = vec![Object::string(path)];
                if let Some(mode) = positional.get(1)
                    && !matches!(mode, Object::Nil)
                {
                    open_arguments.push(mode.clone());
                }
                let Some(Object::Class(file_class)) = self.globals().get("File") else {
                    return Err(MetorexError::runtime_error(
                        "File is not defined",
                        crate::vm::utils::position_to_location(position),
                    ));
                };
                self.pending_block = pending;
                self.call_file_dir_methods(&file_class, "open", &open_arguments, position)
                    .map(|opened| opened.unwrap_or(Object::Nil))
            }
            "require" => {
                // require(name) loads and executes a file from $LOAD_PATH
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!("require() expects 1 argument, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }

                let require_name = self.coerce_load_path(&arguments[0], position)?;
                // A leading `~` names the home directory, as the shell has it.
                let require_name = self.expand_home_path(&require_name);

                // Search $LOAD_PATH for the file
                let search_dirs = self.load_path_directories();

                // A library metorex provides itself is already loaded, so a
                // require of it answers false rather than looking for a file.
                if BUILT_IN_FEATURES.contains(&require_name.trim_end_matches(".rb")) {
                    return Ok(Object::Bool(false));
                }
                // An absolute path, or one written relative to the working
                // directory, names the file outright rather than being
                // searched for.
                let mut found_path = None;
                let direct = std::path::PathBuf::from(&require_name);
                if direct.is_absolute()
                    || require_name.starts_with("./")
                    || require_name.starts_with("../")
                {
                    for candidate in [
                        std::path::PathBuf::from(format!("{}.rb", require_name)),
                        direct.clone(),
                    ] {
                        if candidate.is_file() {
                            found_path = Some(candidate);
                            break;
                        }
                    }
                }
                for dir in &search_dirs {
                    if found_path.is_some() {
                        break;
                    }
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
                        // A library metorex carries is used when the load path
                        // holds no file of that name.
                        if let Some(source) = crate::vm::stdlib::embedded_library(&require_name) {
                            let already = self.run_embedded_library(&require_name, source)?;
                            return Ok(Object::Bool(already));
                        }
                        // Raise a LoadError exception so Ruby-level rescue LoadError catches it.
                        let exc = crate::vm::errors::load_error(
                            format!("cannot load such file -- {}", require_name),
                            &require_name,
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

                // `$LOADED_FEATURES` is the source of truth: a spec that
                // restores it expects the next require to load the file again
                // and answer true.
                let canonical_str = canonical_path.to_string_lossy().into_owned();
                let was_already_loaded = match self.globals().get("\"") {
                    Some(Object::Array(features)) => features
                        .borrow()
                        .iter()
                        .any(|feature| matches!(feature, Object::String(name) if *name.as_str() == *canonical_str)),
                    _ => self.is_file_loaded(&canonical_path),
                };

                self.load_call_site = self
                    .current_source_file
                    .clone()
                    .or_else(|| {
                        self.current_file
                            .as_ref()
                            .map(|file| file.display().to_string())
                    })
                    .map(|file| (file, position));
                self.execute_file(&resolved).map_err(|e| {
                    crate::vm::errors::keep_exception(e, |message| {
                        MetorexError::runtime_error(
                            format!("require('{}') — {}", require_name, message),
                            crate::vm::utils::position_to_location(position),
                        )
                    })
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

                // Ruby resolves the path against the file the call was
                // written in, which is not the file being loaded when a
                // method defined elsewhere is what is running.
                let written_in = self
                    .current_source_file
                    .as_ref()
                    .map(std::path::PathBuf::from)
                    .filter(|path| path.is_file());
                let current_file = written_in.as_ref().or_else(|| self.get_current_file()).ok_or_else(|| {
                    MetorexError::runtime_error(
                        "require_relative cannot be used without a current file context (e.g., in REPL)"
                            .to_string(),
                        crate::vm::utils::position_to_location(position),
                    )
                })?;

                // Resolve the relative path
                let resolved_path = crate::file_loader::resolve_relative_path(
                    current_file,
                    &relative_path.as_str(),
                )
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

                // Find the actual file path with extension auto-detection. A
                // missing file is a LoadError, the way Ruby reports one.
                let actual_path = match crate::file_loader::find_file_path(&resolved_path) {
                    Ok(path) => path,
                    Err(_) => {
                        let feature = resolved_path.display().to_string();
                        let mut message = format!("cannot load such file -- {}", feature);
                        let suggestions = crate::file_loader::suggest_similar_files(&resolved_path);
                        if !suggestions.is_empty() {
                            message
                                .push_str(&format!(". Did you mean: {}?", suggestions.join(", ")));
                        }
                        return Err(MetorexError::UncaughtException {
                            exception: crate::vm::errors::load_error(message.clone(), &feature),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        });
                    }
                };

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
                self.load_call_site = self
                    .current_source_file
                    .clone()
                    .or_else(|| {
                        self.current_file
                            .as_ref()
                            .map(|file| file.display().to_string())
                    })
                    .map(|file| (file, position));
                self.execute_file(&resolved_path).map_err(|e| {
                    crate::vm::errors::keep_exception(e, |message| {
                        MetorexError::runtime_error(
                            format!("require_relative('{}') — {}", relative_path, message),
                            crate::vm::utils::position_to_location(position),
                        )
                    })
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
            // `printf(io, format, *args)` writes to that io, and
            // `printf(format, *args)` writes to `$stdout`.
            "printf" => {
                if arguments.is_empty() {
                    return Ok(Object::Nil);
                }
                let first_is_format = matches!(&arguments[0], Object::String(_))
                    || !self.responds_to(&arguments[0], "write");
                let (target, rest) = if first_is_format {
                    (None, arguments.as_slice())
                } else {
                    (Some(arguments[0].clone()), &arguments[1..])
                };
                let Some(format) = rest.first() else {
                    return Ok(Object::Nil);
                };
                let format = match format {
                    Object::String(text) => Object::String(std::rc::Rc::clone(text)),
                    other => Object::string(self.coerce_load_path(other, position)?),
                };
                let values = Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                    rest[1..].to_vec(),
                )));
                let rendered = self.evaluate_string_format(format, values, position)?;
                let text = match &rendered {
                    Object::String(text) => text.as_str().to_string(),
                    other => other.to_string(),
                };
                match target {
                    Some(target) => {
                        self.send_to_object(target, "write", vec![Object::string(text)], position)?;
                    }
                    None => self.write_to_stdout(&text, position)?,
                }
                Ok(Object::Nil)
            }
            // `pp` prints the same inspect form `p` does. Ruby breaks a wide
            // structure across lines; metorex writes it on one.
            "p" | "pp" => {
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
                // A value of the program's own stands in for source when it
                // says how to read itself as a String.
                let mut arguments = arguments;
                if !matches!(arguments[0], Object::String(_))
                    && self.responds_to(&arguments[0], "to_str")
                {
                    let spelled =
                        self.send_to_object(arguments[0].clone(), "to_str", vec![], position)?;
                    arguments[0] = spelled;
                }
                // Only a Binding says where code runs. A Proc names a scope
                // of its own but is not one.
                if let Some(held) = arguments.get(1)
                    && !matches!(held, Object::Binding(_) | Object::Nil)
                {
                    // Ruby names a callable by what it is rather than by its
                    // class, so a Proc reads as `proc` here.
                    let named = match held {
                        Object::Block(_) => "proc".to_string(),
                        other => self.builtins().class_of(other).name().to_string(),
                    };
                    let message = format!("wrong argument type {named} (expected binding)");
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                let (code, code_encoding) = match &arguments[0] {
                    // Source written in an encoding of its own is read back
                    // through that encoding before it is lexed. A magic
                    // comment names the encoding the source was written in,
                    // whatever the string carrying it is tagged with, so
                    // binary source that says UTF-8 is read as UTF-8.
                    Object::String(s) => {
                        let tagged = s.encoding_name();
                        let carried = crate::vm::native_methods::name_text(s);
                        match crate::lexer::named_source_encoding(&carried).map(|named| {
                            crate::vm::native_methods::string_methods::canonical_encoding_name(
                                &named,
                            )
                        }) {
                            Some(named) if named != tagged => (
                                crate::vm::native_methods::text_in_encoding(s, &named),
                                named,
                            ),
                            _ => (carried, tagged),
                        }
                    }
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
                // Code is written in the encoding a magic comment of its own
                // names, and otherwise in the one the string carrying it is
                // tagged with, which is what `__ENCODING__` answers inside.
                let previous_source_encoding = self.current_source_encoding.replace(
                    crate::lexer::named_source_encoding(&code)
                        .unwrap_or_else(|| code_encoding.clone()),
                );
                // Optional filename (arg 3) and lineno (arg 4) shape the
                // positions recorded for code inside the eval'd string
                // (`__LINE__`, const_source_location, backtraces).
                let filename = match arguments.get(2) {
                    Some(Object::String(s)) => Some(s.as_str().to_string()),
                    _ => None,
                };
                // The line the code is counted from, which a program may
                // name as anything, negative included: it shapes the numbers
                // a backtrace and a syntax error report.
                let named_lineno = match arguments.get(3) {
                    Some(Object::Int(n)) => *n,
                    _ => 1,
                };
                let lineno = named_lineno.max(1) as usize;
                let tokens = crate::lexer::Lexer::with_start_line(&code, lineno)
                    .with_source_encoding(Some(
                        crate::lexer::named_source_encoding(&code)
                            .unwrap_or_else(|| code_encoding.clone()),
                    ))
                    .tokenize();
                let statements = crate::parser::Parser::new(tokens)
                    .parse()
                    .map_err(|errors| {
                        // Ruby names the file in front of the message, so
                        // code eval'd on behalf of a template points at the
                        // template rather than at the eval.
                        let reported = errors
                            .iter()
                            .map(|e| e.to_string())
                            .collect::<Vec<_>>()
                            .join("; ");
                        let at = errors
                            .first()
                            .and_then(|held| held.location())
                            .map_or(0, |held| held.line.max(1).saturating_sub(lineno))
                            as i64
                            + named_lineno;
                        let message = match &filename {
                            Some(named) => format!("{named}:{at}: {reported}"),
                            None => format!("eval: parse error: {reported}"),
                        };
                        crate::vm::errors::syntax_error(message, filename.as_deref(), position)
                    })?;
                // A `frozen_string_literal` comment written after code names
                // nothing, and a verbose run says so.
                if crate::lexer::frozen_string_literal_after_a_token(&code)
                    && matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true)))
                {
                    self.emit_warning_to_stderr(
                        "warning: `frozen_string_literal' is ignored after any tokens",
                        position,
                    );
                }
                // Ruby reports every string it compiles, which is what a
                // trace reads the source back out of.
                self.fire_event(
                    "script_compiled",
                    position,
                    vec![("eval_script", Object::string(code.clone()))],
                )?;
                // A Binding argument re-establishes the frame it captured:
                // its locals (shared cells, so assignment through the eval is
                // visible to a later one) and the `self` in force there.
                let binding = match arguments.get(1) {
                    Some(Object::Binding(b)) => Some(std::rc::Rc::clone(b)),
                    _ => None,
                };
                // Code run through a binding sees the class the binding was
                // taken in, which is what a class variable written there
                // belongs to.
                let carried_cref = binding.as_ref().and_then(|held| match &held.receiver {
                    Some(Object::Class(class) | Object::Module(class)) => Some(Rc::clone(class)),
                    Some(Object::Instance(instance)) => Some(Rc::clone(&instance.borrow().class)),
                    _ => None,
                });
                let carried_cref_pushed = carried_cref.is_some();
                // A class opened in the eval'd code is nested where the
                // binding was taken, so `class Inside; end` run through a
                // binding taken in a class belongs to that class.
                // Only a binding taken in a class or module body nests what
                // the code opens. One taken in an instance method leaves the
                // eval at the top level, where `main` sits.
                // Code eval'd inside a method body opens what it defines
                // where that method was written, so `eval "class C; end"` in
                // a method of A makes A::C rather than a top-level C.
                let captured_nesting: Option<Vec<Rc<crate::class::Class>>> = match &binding {
                    Some(held) if !held.nesting.borrow().is_empty() => {
                        Some(held.nesting.borrow().iter().rev().map(Rc::clone).collect())
                    }
                    Some(_) => None,
                    None if !self.def_scope_stack.is_empty() => None,
                    None => self
                        .method_nesting_stack
                        .last()
                        .filter(|captured| !captured.is_empty())
                        .map(|captured| captured.iter().rev().map(Rc::clone).collect()),
                };
                let saved_def_scope = captured_nesting
                    .map(|opened| std::mem::replace(&mut self.def_scope_stack, opened));
                if let Some(cref) = carried_cref {
                    self.class_var_cref_stack.push(Some(cref));
                }
                if let Some(b) = &binding {
                    self.environment_mut().push_isolated_scope();
                    // The binding's own order is what `local_variables` inside
                    // the eval reports after the names the eval binds itself.
                    for name in b.keys() {
                        if let Some(cell) = b.get(&name) {
                            self.environment_mut().define_inherited(name, cell);
                        }
                    }
                    if let Some(receiver) = &b.receiver {
                        self.environment_mut()
                            .define("self".to_string(), receiver.clone());
                    }
                } else {
                    // Code eval'd without a binding runs in a scope of its
                    // own that sees the caller's locals. A local the code
                    // binds belongs to that scope and is gone once it has
                    // run, which is what Ruby does.
                    let carried: Vec<(String, std::rc::Rc<std::cell::RefCell<Object>>)> = self
                        .environment()
                        .binding_variable_names()
                        .into_iter()
                        .filter_map(|name| {
                            self.environment().get_ref(&name).map(|cell| (name, cell))
                        })
                        .collect();
                    // Code runs against whoever the eval was written with,
                    // which for `Kernel.eval` is Kernel itself and otherwise
                    // is the self in force where the call was made.
                    let here = invoked_with
                        .clone()
                        .or_else(|| self.environment().get("self"));
                    self.environment_mut().push_isolated_scope();
                    for (name, cell) in carried {
                        self.environment_mut().define_inherited(name, cell);
                    }
                    if let Some(receiver) = here {
                        self.environment_mut().define("self".to_string(), receiver);
                    }
                }
                // A name the code assigns to is a local of the scope the eval
                // opens, whether or not the line assigning it runs, which is
                // what leaves `a` behind as nil for a later look.
                let mut hoisted: Vec<String> = Vec::new();
                for name in crate::ast::scope_locals::collect_assigned_locals(&statements) {
                    if self.environment().assignment_introduces_a_local(&name) {
                        self.environment_mut().hoist(name.clone());
                        hoisted.push(name);
                    }
                }
                // eval runs at top-level of its string: treat as non-method scope.
                // Refinements activated inside eval are lexical to the eval string.
                let saved_nesting = self.user_def_nesting;
                self.user_def_nesting = 0;
                // Code run through a binding sees the refinements in force
                // where the binding was taken; anything else opens a scope of
                // its own, since a refinement used inside an eval is lexical
                // to the string.
                let carried_refinements: Vec<crate::vm::core::RefinementEntry> = binding
                    .as_ref()
                    .map(|held| {
                        held.refinements
                            .borrow()
                            .iter()
                            .map(|(module, classes)| crate::vm::core::RefinementEntry {
                                module: Rc::clone(module),
                                classes: classes.iter().cloned().collect(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                self.refinement_scopes.push(carried_refinements);
                let prev_file = self.current_file.clone();
                match &filename {
                    Some(f) => self.current_file = Some(std::path::PathBuf::from(f)),
                    // Ruby names the eval'd code after the place it was
                    // written, which is what `__FILE__` reports inside it.
                    // Code run through a binding is named for the eval that
                    // ran it rather than for where the binding was taken.
                    None => {
                        let written_in = prev_file
                            .as_ref()
                            .map(|file| file.display().to_string())
                            .unwrap_or_default();
                        self.current_file = Some(std::path::PathBuf::from(format!(
                            "{}{}:{})",
                            crate::vm::EVAL_FILE_PREFIX,
                            written_in,
                            position.line
                        )));
                    }
                }
                // The code was written where the eval stands, so it names
                // that place rather than the file the caller's method came
                // from.
                let prev_source_file = std::mem::replace(
                    &mut self.current_source_file,
                    self.current_file
                        .as_ref()
                        .map(|file| file.display().to_string()),
                );
                // The eval'd string runs in the caller's body, so it sees the
                // visibility state in force there. A toggle it sets belongs to
                // the eval and is restored afterwards.
                let enclosing = match self.environment().get("self") {
                    Some(Object::Class(class) | Object::Module(class)) => {
                        Some((Rc::clone(&class), class.current_visibility()))
                    }
                    _ => None,
                };
                // Code handed to `eval` is counted into the file it names when
                // the run was started with `eval` coverage on.
                if self.coverage_counts_eval()
                    && let Some(named) = self.current_source_file.clone()
                {
                    self.coverage_note_eval(&named, &statements);
                }
                // The eval is a place of its own in a backtrace, standing at
                // the line it was written on in the file that wrote it, so
                // code inside it still reports back to the program.
                // The frame reads as the one that ran the eval, which is the
                // name Ruby gives code inside one and what keeps `__method__`
                // there naming the method around it.
                let written_in = self
                    .call_stack()
                    .last()
                    .cloned()
                    .unwrap_or_else(|| crate::vm::CallFrame::boundary("<main>"));
                self.call_stack_push(
                    written_in
                        .with_location(Some(format!("{}:{}", position.line, position.column)))
                        .with_source_file(prev_source_file.clone()),
                );
                // Code run through a binding runs where the binding was
                // taken, so `__method__` names the method it was taken in.
                let named_method = binding
                    .as_ref()
                    .and_then(|held| held.method.borrow().clone());
                if let Some((callee, defined)) = &named_method {
                    self.call_stack_push(crate::vm::CallFrame::method(
                        callee.clone(),
                        None,
                        callee.clone(),
                        defined.clone(),
                    ));
                }
                // A block written on the eval belongs to the eval, so a
                // method the code calls is not handed it and a `yield` there
                // has nothing to run.
                let held_block = self.pending_block.take();
                // A `return` written in the code returns from the scope the
                // eval was written in rather than ending the eval, so it
                // carries on out rather than being answered here.
                let result = self.run_eval_statements(&statements);
                self.pending_block = held_block;
                if named_method.is_some() {
                    self.call_stack_pop();
                }
                self.call_stack_pop();
                if carried_cref_pushed {
                    self.class_var_cref_stack.pop();
                }
                if let Some(held) = saved_def_scope {
                    self.def_scope_stack = held;
                }
                if let Some(held) = &binding {
                    // A local the code named that the binding did not have is
                    // added to it, which is how `eval("x = 1", b)` leaves `x`
                    // behind for the next look through `b`.
                    for (name, cell) in self.environment().current_scope_var_refs() {
                        if !held.has(&name) {
                            held.set(&name, cell);
                        }
                    }
                    // A name the code only assigns to on a line that never
                    // ran is a local of the scope all the same, and the next
                    // look through the binding reads it as nil.
                    for name in &hoisted {
                        if !held.has(name)
                            && let Some(cell) = self.environment().get_ref(name)
                        {
                            held.set(name, cell);
                        }
                    }
                }
                self.environment_mut().pop_scope();
                if let Some((class, visibility)) = enclosing {
                    class.set_current_visibility(visibility);
                }
                self.current_file = prev_file;
                self.current_source_file = prev_source_file;
                self.current_source_encoding = previous_source_encoding;
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
                    .map(|name| Object::symbol(format!("${}", name)))
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
                            // A builtin the root scope seeded is not a local,
                            // but a local of the same name shadows it, and
                            // that one is reported like any other.
                            && !(self.seeded_global_names.contains(name)
                                && self.globals().get(name) == self.environment().get(name))
                            && !self
                                .environment()
                                .get(name)
                                .is_some_and(|value| self.name_is_a_definition(name, &value))
                    })
                    .collect();
                let mut seen = std::collections::HashSet::new();
                names.retain(|name| seen.insert(name.clone()));
                let names: Vec<Object> = names.into_iter().map(Object::symbol).collect();
                Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                    names,
                ))))
            }
            // `fail` is Ruby's other spelling of `raise`.
            // `fail` is Ruby's other spelling of `raise`. Both are reachable
            // as methods, so `send(:raise, ...)` and a singleton that makes
            // `raise` public find them here.
            "fail" | "raise" => {
                // A class, a message and a backtrace, with `cause:` allowed
                // alongside them.
                let counted = match arguments.last() {
                    Some(Object::Dict(pairs)) if pairs.borrow().contains_key("__MX_KWARGS__") => {
                        arguments.len() - 1
                    }
                    _ => arguments.len(),
                };
                if counted > 3 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(0, 3),
                        counted,
                        position,
                    ));
                }
                let exception = self.build_raise_exception(&arguments, position)?;
                let message = match &exception {
                    Object::Exception(_) => crate::vm::utils::format_exception(&exception),
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
                // Without a block, `loop` answers an enumerator that yields
                // forever, which is what `loop.size` and `loop.each` read.
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => block,
                    _ => {
                        let Some(enumerator_class) = self.globals().get("Enumerator") else {
                            let message = "uninitialized constant Enumerator".to_string();
                            return Err(MetorexError::UncaughtException {
                                exception: Object::exception("NameError", message.clone()),
                                location: crate::vm::utils::position_to_location(position),
                                message,
                            });
                        };
                        return self.send_to_object(
                            enumerator_class,
                            "endless",
                            Vec::new(),
                            position,
                        );
                    }
                };
                loop {
                    match block.call(self, Vec::new(), position) {
                        Ok(_) => {}
                        Err(MetorexError::BlockBreak { value, .. }) => return Ok(value),
                        // A StopIteration ends the loop, which answers the
                        // result the finished iterator carried.
                        Err(MetorexError::UncaughtException { exception, .. })
                            if self.exception_matches(
                                &exception,
                                &["StopIteration".to_string()],
                            )? =>
                        {
                            if let Object::Exception(details) = &exception {
                                let result = details.borrow().instance_vars.get("result").cloned();
                                if let Some(result) = result {
                                    return Ok(result);
                                }
                            }
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
                    let exception = Object::exception("UncaughtThrowError", message.clone());
                    if let Object::Exception(details) = &exception {
                        let mut details = details.borrow_mut();
                        details
                            .instance_vars
                            .insert(crate::vm::THROW_TAG_KEY.to_string(), tag);
                        details.instance_vars.insert(
                            crate::vm::THROW_VALUE_KEY.to_string(),
                            arguments.get(1).cloned().unwrap_or(Object::Nil),
                        );
                    }
                    return Err(MetorexError::UncaughtException {
                        exception,
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
                // `load(path, true)` runs the file inside a fresh anonymous
                // module, and `load(path, SomeModule)` inside that one.
                let wrapper: Option<Rc<crate::class::Class>> = match arguments.get(1) {
                    Some(Object::Bool(true)) => Some(Rc::new(crate::class::Class::new_module(""))),
                    Some(Object::Module(module) | Object::Class(module)) => Some(Rc::clone(module)),
                    _ => None,
                };
                let wrap = wrapper.is_some();
                self.load_wrap_module = wrapper;
                let path_str = self.coerce_load_path(&arguments[0], position)?;
                // A leading `~` names the home directory, as the shell has it.
                let path_str = self.expand_home_path(&path_str);
                let path = std::path::PathBuf::from(&path_str);
                let path = path.as_path();
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
                // A path written relative to the working directory names that
                // file outright and is never searched for in `$LOAD_PATH`.
                let anchored = path_str.starts_with("./")
                    || path_str.starts_with("../")
                    || path_str.starts_with('/');
                let result = if path.is_file() {
                    self.execute_file_recording(path, false).map_err(|error| {
                        // A file that cannot be read is one that cannot be
                        // loaded, which Ruby reports as a LoadError.
                        if error.message().contains("Failed to read file") {
                            let message = format!("cannot load such file -- {}", path_str);
                            return MetorexError::UncaughtException {
                                exception: Object::exception("LoadError", message.clone()),
                                location: crate::vm::utils::position_to_location(position),
                                message,
                            };
                        }
                        crate::vm::errors::keep_exception(error, |message| {
                            MetorexError::runtime_error(
                                format!("load('{}') — {}", path_str, message),
                                crate::vm::utils::position_to_location(position),
                            )
                        })
                    })
                } else if anchored {
                    let message = format!("cannot load such file -- {}", path_str);
                    Err(MetorexError::UncaughtException {
                        exception: Object::exception("LoadError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    })
                } else {
                    // Search $LOAD_PATH
                    let search_dirs = self.load_path_directories();
                    let mut found = None;
                    for dir in &search_dirs {
                        let candidate = std::path::PathBuf::from(dir).join(&path_str);
                        if candidate.exists() {
                            found = Some(candidate);
                            break;
                        }
                    }
                    match found {
                        Some(resolved) => {
                            self.execute_file_recording(&resolved, false).map_err(|e| {
                                crate::vm::errors::keep_exception(e, |message| {
                                    MetorexError::runtime_error(
                                        format!("load('{}') — {}", path_str, message),
                                        crate::vm::utils::position_to_location(position),
                                    )
                                })
                            })
                        }
                        None => {
                            let message = format!("cannot load such file -- {}", path_str);
                            Err(MetorexError::UncaughtException {
                                exception: Object::exception("LoadError", message.clone()),
                                location: crate::vm::utils::position_to_location(position),
                                message,
                            })
                        }
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
                let code = self.exit_status_argument(arguments.first(), position)?;
                if name == "exit!" {
                    std::process::exit(code as i32);
                }
                // `exit` raises SystemExit so `ensure` blocks and a rescue of
                // SystemExit still see it. An uncaught one ends the program
                // with this status.
                let exception = Object::Exception(std::rc::Rc::new(std::cell::RefCell::new(
                    crate::object::Exception {
                        exception_type: "SystemExit".to_string(),
                        message: "exit".to_string(),
                        backtrace: None,
                        location: None,
                        cause: None,
                        status: Some(code),
                        name: None,
                        receiver: None,
                        backtrace_array: None,
                        backtrace_sites: None,
                        backtrace_locations_array: None,
                        class: None,
                        instance_vars: indexmap::IndexMap::new(),
                        message_given: true,
                    },
                )));
                Err(MetorexError::UncaughtException {
                    exception,
                    location: crate::vm::utils::position_to_location(position),
                    message: "exit".to_string(),
                })
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
                        receiver: None,
                        backtrace_array: None,
                        backtrace_sites: None,
                        backtrace_locations_array: None,
                        class: None,
                        instance_vars: indexmap::IndexMap::new(),
                        message_given: true,
                    },
                )));
                Err(MetorexError::UncaughtException {
                    exception,
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
            // `exec` replaces this process with the command, so nothing after
            // it runs. A command the shell cannot find raises Errno::ENOENT.
            "exec" => {
                let Some(first) = arguments.first() else {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::AtLeast(1),
                        0,
                        position,
                    ));
                };
                let program = self.coerce_command_argument(first, position)?;
                let mut rest = Vec::new();
                for argument in arguments.iter().skip(1) {
                    rest.push(self.coerce_command_argument(argument, position)?);
                }
                use std::io::Write as _;
                let _ = std::io::stdout().flush();
                // A command with nothing for the shell to do is run directly,
                // so a missing program is reported as ENOENT rather than
                // becoming the shell's own "command not found" exit.
                let needs_shell = rest.is_empty()
                    && program.contains(|c: char| " \t\n|&;<>()$`\\\"'*?[]#~=%".contains(c));
                // A program that cannot be found is refused before anything
                // is started. Leaving it to the spawn reports differently
                // from one platform to the next: some hand back the error,
                // and some start a child that exits 127.
                if !needs_shell && findable_program(&program).is_none() {
                    let message = format!("No such file or directory - {}", program);
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("Errno::ENOENT", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                let status = if !rest.is_empty() {
                    std::process::Command::new(&program).args(&rest).status()
                } else if needs_shell {
                    std::process::Command::new("/bin/sh")
                        .arg("-c")
                        .arg(&program)
                        .status()
                } else {
                    std::process::Command::new(&program).status()
                };
                match status {
                    Ok(status) => std::process::exit(status.code().unwrap_or(0)),
                    Err(_) => {
                        let message = format!("No such file or directory - {}", program);
                        Err(MetorexError::UncaughtException {
                            exception: Object::exception("Errno::ENOENT", message.clone()),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        })
                    }
                }
            }
            // `spawn` starts a command and answers its process id without
            // waiting for it, which is what `Process.wait` is then given.
            "spawn" => {
                // A Hash in front names the environment the child runs with,
                // and one at the back names how it is run rather than what it
                // is run with.
                let mut given: &[Object] = &arguments;
                let mut environment = None;
                if let Some(Object::Dict(entries)) = given.first() {
                    environment = Some(entries.borrow().clone());
                    given = &given[1..];
                }
                let mut settings = None;
                if given.len() > 1
                    && let Some(Object::Dict(entries)) = given.last()
                {
                    settings = Some(entries.borrow().clone());
                    given = &given[..given.len() - 1];
                }
                let Some(command) = given.first() else {
                    return Err(MetorexError::runtime_error(
                        "spawn requires at least 1 argument".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                };
                let program = self.get_string_representation(command, position)?;
                let mut rest = Vec::new();
                for argument in &given[1..] {
                    rest.push(self.get_string_representation(argument, position)?);
                }
                let mut running = if rest.is_empty() {
                    let mut shell = std::process::Command::new("/bin/sh");
                    shell.arg("-c").arg(&program);
                    shell
                } else {
                    let mut named = std::process::Command::new(&program);
                    named.args(&rest);
                    named
                };
                if let Some(environment) = environment {
                    for (name, value) in environment.iter() {
                        let name = name.trim_start_matches(':');
                        match value {
                            Object::Nil => {
                                running.env_remove(name);
                            }
                            held => {
                                running.env(name, self.get_string_representation(held, position)?);
                            }
                        }
                    }
                }
                if let Some(settings) = &settings
                    && let Some(Object::String(directory)) = settings.get(":chdir")
                {
                    running.current_dir(directory.as_str().to_string());
                }
                // `pgroup: true` starts the child in a process group of its
                // own, which is what keeps a signal to this group from
                // reaching it.
                let own_group = matches!(
                    settings.as_ref().and_then(|held| held.get(":pgroup")),
                    Some(Object::Bool(true)) | Some(Object::Int(0))
                );
                if own_group {
                    use std::os::unix::process::CommandExt;
                    // SAFETY: the child calls `setpgid` on itself between the
                    // fork and the exec, which is what it is for.
                    unsafe {
                        running.pre_exec(|| {
                            libc::setpgid(0, 0);
                            Ok(())
                        });
                    }
                }
                match running.spawn() {
                    Ok(child) => Ok(Object::Int(i64::from(child.id()))),
                    Err(problem) => {
                        let message = format!("No such file or directory - {program} ({problem})");
                        Err(MetorexError::UncaughtException {
                            exception: Object::exception("Errno::ENOENT", message.clone()),
                            location: crate::vm::utils::position_to_location(position),
                            message,
                        })
                    }
                }
            }
            "system" => {
                let Some(command) = arguments.first() else {
                    return Err(MetorexError::runtime_error(
                        "system requires at least 1 argument".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                };
                let program = self.get_string_representation(command, position)?;
                // A trailing Hash names where the child's streams go and
                // whether a failure is raised rather than reported.
                let mut given = &arguments[1..];
                let mut options = None;
                if let Some(Object::Dict(entries)) = given.last() {
                    options = Some(entries.borrow().clone());
                    given = &given[..given.len() - 1];
                }
                let mut rest = Vec::new();
                for arg in given {
                    rest.push(self.get_string_representation(arg, position)?);
                }
                let mut raises = false;
                let mut redirects: Vec<(i32, String)> = Vec::new();
                if let Some(options) = &options {
                    for (name, target) in options.iter() {
                        match name.trim_start_matches(':') {
                            "exception" => raises = target.is_truthy(),
                            "out" => {
                                if let Object::String(path) = target {
                                    redirects.push((1, path.as_str().to_string()));
                                }
                            }
                            "err" => {
                                if let Object::String(path) = target {
                                    redirects.push((2, path.as_str().to_string()));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // One string holding a character the shell reads runs through
                // the shell, and anything else runs as the program it names.
                let mut reached = None;
                let words: Vec<String> = if !rest.is_empty() {
                    let mut held = vec![program.clone()];
                    held.extend(rest);
                    held
                } else if needs_a_shell(&program) {
                    // The shell is reached by its path and told its name is
                    // `sh`, which is the name `$0` answers inside it.
                    reached = Some("/bin/sh".to_string());
                    vec!["sh".to_string(), "-c".to_string(), program.clone()]
                } else {
                    program
                        .split_whitespace()
                        .map(|held| held.to_string())
                        .collect()
                };
                if words.is_empty() {
                    return Ok(Object::Nil);
                }
                let reached = reached.unwrap_or_else(|| words[0].clone());
                let (status, pid) = run_to_completion(&reached, &words, &redirects);
                self.record_last_status(&status, Some(pid));
                let code = status.code().unwrap_or(-1);
                if raises && code != 0 {
                    // A child that never reached the program reports it the
                    // way the operating system does.
                    if code == 127 {
                        let message = format!("No such file or directory - {program}");
                        return Err(crate::vm::errors::simple_exception(
                            "Errno::ENOENT",
                            &message,
                            position,
                        ));
                    }
                    let message = format!("Command failed with exit {code}: {}", words.join(" "));
                    return Err(crate::vm::errors::simple_exception(
                        "RuntimeError",
                        &message,
                        position,
                    ));
                }
                // A command that never ran at all is reported as nothing
                // rather than as a failure.
                if code == 127 {
                    return Ok(Object::Nil);
                }
                Ok(Object::Bool(status.success()))
            }
            // `fork` splits the process. The child answers nil, or runs the
            // block and exits with its status; the parent answers the child's
            // process id either way.
            "fork" => {
                use std::io::Write as _;
                let _ = std::io::stdout().flush();
                let _ = std::io::stderr().flush();
                let block = self.pending_block.take();
                // SAFETY: `fork` is called with no other threads running, and
                // the child does nothing but run the block and exit.
                let child = unsafe { libc::fork() };
                if child < 0 {
                    let message = "fork failed".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("Errno::EAGAIN", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    });
                }
                if child > 0 {
                    return Ok(Object::Int(child as i64));
                }
                // Only the thread that called `fork` survives into the child,
                // so every other one is marked finished there.
                for thread in std::mem::take(&mut self.pending_threads) {
                    if let Object::Instance(instance) = thread {
                        instance
                            .borrow_mut()
                            .set_var("__thread_value".to_string(), Object::Nil);
                    }
                }
                // In the child. Without a block, `fork` answers nil and the
                // caller carries on as the child.
                let Some(Object::Block(block)) = block else {
                    return Ok(Object::Nil);
                };
                let outcome = self.execute_block_callable(&block, Vec::new(), position);
                let _ = std::io::stdout().flush();
                let status = match outcome {
                    Ok(_) => 0,
                    Err(MetorexError::UncaughtException {
                        exception: Object::Exception(details),
                        ..
                    }) if details.borrow().is_system_exit() => {
                        details.borrow().status.unwrap_or(0) as i32
                    }
                    Err(error) => {
                        eprintln!("{}", error);
                        1
                    }
                };
                std::process::exit(status);
            }
            _ => Err(MetorexError::runtime_error(
                format!("Unknown native function: {}", name),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }

    /// Write one `puts` argument. An Array is written a line per element,
    /// however deeply nested, and an empty one writes a line of its own. A
    /// string that already ends in a newline is not given a second.
    fn puts_object(&mut self, value: &Object, position: Position) -> Result<(), MetorexError> {
        if let Object::Array(elements) = value {
            let elements = elements.borrow().clone();
            // An empty Array still writes a line, as `puts []` does.
            if elements.is_empty() {
                self.write_to_stdout("\n", position)?;
                return Ok(());
            }
            for element in &elements {
                self.puts_object(element, position)?;
            }
            return Ok(());
        }
        let output = self.get_string_representation(value, position)?;
        if output.ends_with('\n') {
            self.write_to_stdout(&output, position)?;
        } else {
            self.write_to_stdout(&format!("{}\n", output), position)?;
        }
        Ok(())
    }

    /// The VM call stack as Location objects, outermost call last, the way
    /// `caller_locations(0)` reports them.
    pub(crate) fn caller_location_objects(&mut self, position: Position) -> Vec<Object> {
        use crate::object::Instance;
        use std::cell::RefCell;
        use std::rc::Rc;
        let loc_class = self.backtrace_location_class();
        let current_file = self
            .reported_current_file()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let stack = self.call_stack();
        let mut locations = Vec::with_capacity(stack.len() + 1);
        // The call stack records where each frame was entered from, so the
        // line `caller_locations` itself sits on is not among them. Ruby
        // counts it as the innermost location, which is what level 0 names.
        let mut here = Instance::new(Rc::clone(&loc_class));
        here.set_var("lineno".to_string(), Object::Int(position.line as i64));
        here.set_var("path".to_string(), Object::string(current_file.clone()));
        let here_absolute = match self.absolute_path_for(&current_file) {
            Some(resolved) => Object::string(resolved),
            None => Object::Nil,
        };
        here.set_var("absolute_path".to_string(), here_absolute);
        let frames: Vec<_> = stack.iter().rev().collect();
        // Where each frame was called from, with a call made inside the core
        // library standing for the place that reached it: Ruby names the
        // program's own file rather than `<internal:...>`.
        let called_from: Vec<(String, i64)> = frame_call_sites(&frames, &current_file);
        // A frame's own name labels the location it is running at, and Ruby
        // names a block by the scope holding it: `block in <main>`.
        let label_at = |index: usize| -> String { frame_label_at(&frames, index) };
        here.set_var("label".to_string(), Object::string(label_at(0)));
        locations.push(Object::Instance(Rc::new(RefCell::new(here))));
        for (index, frame) in frames.iter().enumerate() {
            // A frame with no recorded call site was never called from
            // anywhere — the file body itself — so it is not a caller.
            if frame.location().is_none() {
                continue;
            }
            let (path, line) = called_from[index].clone();
            // A frame entered from nowhere in particular records no line,
            // and there is no call for a backtrace to name there.
            if line == 0 {
                continue;
            }
            let mut inst = Instance::new(Rc::clone(&loc_class));
            inst.set_var("lineno".to_string(), Object::Int(line));
            let absolute = match self.absolute_path_for(&path) {
                Some(resolved) => Object::string(resolved),
                None => Object::Nil,
            };
            inst.set_var("path".to_string(), Object::string(path));
            inst.set_var("absolute_path".to_string(), absolute);
            // A frame records where it was called from, so its location pairs
            // with the name of the frame below it: the one that made the call.
            // A frame records where it was called from, so its location
            // pairs with the name of the frame below it: the one that made
            // the call.
            inst.set_var("label".to_string(), Object::string(label_at(index + 1)));
            locations.push(Object::Instance(Rc::new(RefCell::new(inst))));
        }
        locations
    }

    /// The caller locations a `caller`/`caller_locations` argument list names.
    /// `(start, length)` drops `start` frames and keeps `length` of them; a
    /// Range says which frames to keep directly. Dropping more than there are
    /// answers None, which both report as nil.
    fn sliced_caller_locations(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Vec<Object>>, MetorexError> {
        let all = self.caller_location_objects(position);
        let Some((skip, length)) = self.caller_slice_bounds(arguments, all.len(), 1) else {
            return Ok(None);
        };
        let mut kept: Vec<Object> = all.into_iter().skip(skip).collect();
        if let Some(length) = length {
            kept.truncate(length);
        }
        Ok(Some(kept))
    }

    /// Where a slice of a backtrace starts and how long it is, from the
    /// arguments naming it. None when it starts past the end, which Ruby
    /// answers as nil rather than an empty list.
    pub(crate) fn caller_slice_bounds(
        &mut self,
        arguments: &[Object],
        total: usize,
        default_skip: usize,
    ) -> Option<(usize, Option<usize>)> {
        let (skip, length) = match arguments.first() {
            Some(Object::Range { .. }) if arguments.len() == 1 => {
                self.range_bounds_for(&arguments[0], total)?
            }
            Some(Object::Int(number)) => (
                (*number).max(0) as usize,
                match arguments.get(1) {
                    Some(Object::Int(limit)) => Some((*limit).max(0) as usize),
                    _ => None,
                },
            ),
            _ => (default_skip, None),
        };
        if skip > total {
            return None;
        }
        Some((skip, length))
    }

    /// The (skip, length) a Range argument names over `total` frames, or None
    /// when it starts past the end.
    fn range_bounds_for(&mut self, value: &Object, total: usize) -> Option<(usize, Option<usize>)> {
        let Object::Range {
            start,
            end,
            exclusive,
            ..
        } = value
        else {
            return None;
        };
        let resolve = |bound: &Object, default: i64| -> i64 {
            match bound {
                Object::Int(number) => *number,
                _ => default,
            }
        };
        let first = resolve(start, 0);
        let first = if first < 0 {
            (total as i64 + first).max(0)
        } else {
            first
        } as usize;
        if first > total {
            return None;
        }
        let last = match end.as_ref() {
            Object::Nil => total as i64 - 1,
            bound => {
                let last = resolve(bound, total as i64 - 1);
                let last = if last < 0 { total as i64 + last } else { last };
                if *exclusive { last - 1 } else { last }
            }
        };
        let length = (last - first as i64 + 1).max(0) as usize;
        Some((first, Some(length)))
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
    /// Coerce a `load` / `require` argument to a path: a String stands, and
    /// anything else goes through `to_path` and then `to_str`, each of which
    /// must answer a String.
    pub(crate) fn coerce_load_path(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(path) = argument {
            return Ok(path.as_str().to_string());
        }
        let refuse = |vm: &mut Self, value: &Object| {
            let message = format!(
                "no implicit conversion of {} into String",
                vm.builtins().class_of(value).name()
            );
            MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            }
        };
        let mut value = argument.clone();
        if self.responds_to(&value, "to_path") {
            value = self.invoke_named_conversion(&value, "to_path", position)?;
            if let Object::String(path) = &value {
                return Ok(path.as_str().to_string());
            }
        }
        if !self.responds_to(&value, "to_str") {
            return Err(refuse(self, &value));
        }
        let converted = self.invoke_named_conversion(&value, "to_str", position)?;
        match converted {
            Object::String(path) => Ok(path.as_str().to_string()),
            other => Err(refuse(self, &other)),
        }
    }

    /// Call a conversion method by name on a value.
    fn invoke_named_conversion(
        &mut self,
        value: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some((class, method)) = self.lookup_method(value, method_name) else {
            return Ok(Object::Nil);
        };
        self.invoke_method(class, method, value.clone(), Vec::new(), position)
    }

    /// The status `exit` was asked for: an Integer as it stands, true or
    /// false as 0 or 1, a Float truncated, and anything else through `to_int`.
    fn exit_status_argument(
        &mut self,
        argument: Option<&Object>,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let Some(argument) = argument else {
            return Ok(0);
        };
        match argument {
            Object::Int(status) => Ok(*status),
            Object::Bool(success) => Ok(if *success { 0 } else { 1 }),
            Object::Float(status) => Ok(status.trunc() as i64),
            other => {
                let source = self.builtins().class_of(other).name().to_string();
                let refuse = |message: String| MetorexError::UncaughtException {
                    exception: Object::exception("TypeError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                };
                let Some((class, method)) = self.lookup_method(other, "to_int") else {
                    return Err(refuse(if matches!(other, Object::Nil) {
                        "no implicit conversion from nil to integer".to_string()
                    } else {
                        format!("no implicit conversion of {} into Integer", source)
                    }));
                };
                let converted =
                    self.invoke_method(class, method, other.clone(), Vec::new(), position)?;
                match converted {
                    Object::Int(status) => Ok(status),
                    produced => Err(refuse(format!(
                        "can't convert {} to Integer ({}#to_int gives {})",
                        source,
                        source,
                        self.builtins().class_of(&produced).name()
                    ))),
                }
            }
        }
    }

    /// Coerce the argument to `` ` `` to a String: one is taken as is, and
    /// anything else has to answer `to_str`.
    fn coerce_command_argument(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = argument {
            return Ok(text.as_str().to_string());
        }
        let source = self.builtins().class_of(argument).name().to_string();
        let Some((class, method)) = self.lookup_method(argument, "to_str") else {
            let message = format!("no implicit conversion of {} into String", source);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let converted =
            self.invoke_method(class, method, argument.clone(), Vec::new(), position)?;
        match converted {
            Object::String(text) => Ok(text.as_str().to_string()),
            other => {
                let produced = self.builtins().class_of(&other).name().to_string();
                let message = format!(
                    "can't convert {} to String ({}#to_str gives {})",
                    source, source, produced
                );
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("TypeError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                })
            }
        }
    }

    fn coerce_abort_message(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = argument {
            return Ok(text.as_str().to_string());
        }
        if let Some((class, method)) = self.lookup_method(argument, "to_str")
            && !method.is_undefined
        {
            let converted =
                self.invoke_method(class, method, argument.clone(), vec![], position)?;
            if let Object::String(text) = converted {
                return Ok(text.as_str().to_string());
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
    pub(crate) fn get_string_representation(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        // First try to_s, then inspect, then fall back to Display
        match obj {
            // `:name.to_s` is the bare name; only `inspect` keeps the colon.
            Object::Symbol(name) => Ok(name.as_str().to_string()),
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

    /// The string `inspect` produces for an object, tagged with the encoding
    /// it is written in. Text that encoding has no room for is escaped rather
    /// than refused.
    pub(crate) fn inspected_object(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let rendered = self.inspect_object(obj, position)?;
        let Object::String(text) = &rendered else {
            return Ok(rendered);
        };
        let writing = self.inspect_result_encoding();
        let held = text.encoding_name();
        // Text in an encoding that spells ASCII its own way is never read as
        // ASCII, however small the numbers its bytes happen to be.
        let reads_as_ascii =
            crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(&held)
                && text.as_str().is_ascii();
        if held == writing || reads_as_ascii {
            return Ok(rendered);
        }
        Ok(crate::vm::native_methods::string_methods::escaped_text(
            text,
        ))
    }

    /// The string `inspect` produces for an object, preferring a method the
    /// object defines over the native rendering.
    pub(crate) fn get_inspect_representation(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        let rendered = self.inspect_object(obj, position)?;
        Ok(match &rendered {
            Object::String(text) => text.to_string(),
            other => format!("{}", other),
        })
    }

    /// What `inspect` answered for an object, as the object it answered. A
    /// method the object defines is preferred over the native rendering.
    fn inspect_object(&mut self, obj: &Object, position: Position) -> Result<Object, MetorexError> {
        if let Some((class, method)) = self.lookup_method(obj, "inspect")
            && !method.is_undefined
        {
            let result = self.invoke_method(class, method, obj.clone(), vec![], position)?;
            if matches!(result, Object::String(_)) {
                return Ok(result);
            }
            // Ruby asks whatever `inspect` answered for its own `to_s`, and
            // shows that object's default form when `to_s` is no String
            // either. Neither one is asked for `to_str`.
            if let Some((class, method)) = self.lookup_method(&result, "to_s")
                && !method.is_undefined
            {
                let spelled =
                    self.invoke_method(class, method, result.clone(), vec![], position)?;
                if matches!(spelled, Object::String(_)) {
                    return Ok(spelled);
                }
                return Ok(Object::string(format!("{}", spelled)));
            }
            return Ok(Object::string(format!("{}", result)));
        }
        let class = self.builtins().class_of(obj);
        if let Some(rendered @ Object::String(_)) =
            self.call_native_method(&class, obj, "inspect", &[], position)?
        {
            return Ok(rendered);
        }
        // The per-class tables answer for the classes they know. Everything
        // else falls to Object's own, which is where an instance of a class
        // the program wrote is written out with the variables it holds.
        if let Some(rendered @ Object::String(_)) =
            self.call_object_method(obj, "inspect", &[], position)?
        {
            return Ok(rendered);
        }
        Ok(Object::string(format!("{}", obj)))
    }

    /// The Integer `srand` seeds with, kept at its full width so the next
    /// call answers the same number back. A Float truncates and any other
    /// object must answer `#to_int`, as Ruby requires.
    fn coerce_to_seed(
        &mut self,
        given: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match given {
            Object::Int(_) | Object::BigInt(_) => Ok(given.clone()),
            Object::Float(seed) => Ok(Object::Int(*seed as i64)),
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
                    let tokens = crate::lexer::Lexer::new(&code.as_str()).tokenize();
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

    /// Seed the generator from a whole number, the way Ruby seeds one: a
    /// single word spreads out on its own, and a wider seed is mixed in word
    /// by word.
    pub(crate) fn seed_random(&mut self, seed: &num_bigint::BigInt) {
        let magnitude = if *seed < num_bigint::BigInt::from(0) {
            -seed.clone()
        } else {
            seed.clone()
        };
        let mut words: Vec<u32> = Vec::new();
        let mut held = magnitude;
        let step = num_bigint::BigInt::from(1u64 << 32);
        while held > num_bigint::BigInt::from(0) {
            let low = &held % &step;
            words.push(low.to_string().parse::<u64>().unwrap_or(0) as u32);
            held /= &step;
        }
        if words.is_empty() {
            words.push(0);
        }
        // A seed whose top word is one carries nothing that word does not
        // already say, which is the trim Ruby makes before mixing.
        if words.len() > 1 && words[words.len() - 1] == 1 {
            words.pop();
        }
        self.random_words = vec![0u32; MT_WORDS];
        self.random_at = MT_WORDS;
        if words.len() <= 1 {
            self.seed_random_word(words[0]);
            return;
        }
        self.seed_random_word(19650218);
        let mut at = 1usize;
        let mut from = 0usize;
        let mut count = MT_WORDS.max(words.len());
        while count > 0 {
            let previous = self.random_words[at - 1];
            self.random_words[at] = (self.random_words[at]
                ^ (previous ^ (previous >> 30)).wrapping_mul(1664525))
            .wrapping_add(words[from])
            .wrapping_add(from as u32);
            at += 1;
            from += 1;
            if at >= MT_WORDS {
                self.random_words[0] = self.random_words[MT_WORDS - 1];
                at = 1;
            }
            if from >= words.len() {
                from = 0;
            }
            count -= 1;
        }
        let mut count = MT_WORDS - 1;
        while count > 0 {
            let previous = self.random_words[at - 1];
            self.random_words[at] = (self.random_words[at]
                ^ (previous ^ (previous >> 30)).wrapping_mul(1566083941))
            .wrapping_sub(at as u32);
            at += 1;
            if at >= MT_WORDS {
                self.random_words[0] = self.random_words[MT_WORDS - 1];
                at = 1;
            }
            count -= 1;
        }
        self.random_words[0] = 0x8000_0000;
        self.random_at = MT_WORDS;
    }

    /// The spread a single-word seed makes across the whole state.
    fn seed_random_word(&mut self, seed: u32) {
        self.random_words = vec![0u32; MT_WORDS];
        self.random_words[0] = seed;
        for at in 1..MT_WORDS {
            let previous = self.random_words[at - 1];
            self.random_words[at] = (previous ^ (previous >> 30))
                .wrapping_mul(1812433253)
                .wrapping_add(at as u32);
        }
        self.random_at = MT_WORDS;
    }

    /// The next word the generator answers.
    fn next_random_word(&mut self) -> u32 {
        if self.random_words.len() != MT_WORDS {
            let seed = num_bigint::BigInt::from(crate::vm::core::seed_from_clock());
            self.seed_random(&seed);
        }
        if self.random_at >= MT_WORDS {
            for at in 0..MT_WORDS {
                let mixed = (self.random_words[at] & 0x8000_0000)
                    | (self.random_words[(at + 1) % MT_WORDS] & 0x7fff_ffff);
                let mut next = self.random_words[(at + MT_STEP) % MT_WORDS] ^ (mixed >> 1);
                if mixed & 1 == 1 {
                    next ^= 0x9908_b0df;
                }
                self.random_words[at] = next;
            }
            self.random_at = 0;
        }
        let mut held = self.random_words[self.random_at];
        self.random_at += 1;
        held ^= held >> 11;
        held ^= (held << 7) & 0x9d2c_5680;
        held ^= (held << 15) & 0xefc6_0000;
        held ^ (held >> 18)
    }

    /// Advance the generator and answer the next 64 bits.
    fn next_random_bits(&mut self) -> u64 {
        let high = self.next_random_word() as u64;
        let low = self.next_random_word() as u64;
        (high << 32) | low
    }

    /// The next draw as a Float in [0, 1), read from two words the way Ruby
    /// reads one.
    pub(crate) fn next_random_float(&mut self) -> f64 {
        let high = (self.next_random_word() >> 5) as f64;
        let low = (self.next_random_word() >> 6) as f64;
        (high * 67108864.0 + low) * (1.0 / 9007199254740992.0)
    }

    /// The next draw as an Integer in [0, bound) for a bound past the
    /// machine word. Words are drawn until the value is under the bound,
    /// which keeps every value in the range equally likely.
    pub(crate) fn next_random_big(&mut self, bound: &num_bigint::BigInt) -> num_bigint::BigInt {
        let width = bound.bits();
        let words = width.div_ceil(64) as usize;
        loop {
            let mut drawn = num_bigint::BigInt::from(0);
            for _ in 0..words {
                drawn = (drawn << 64) + num_bigint::BigInt::from(self.next_random_bits());
            }
            drawn >>= (words as u64 * 64) - width;
            if drawn < *bound {
                return drawn;
            }
        }
    }

    /// The next draw as an Integer in [0, bound). Ruby fills a mask wide
    /// enough for the bound a word at a time and draws again whenever the
    /// value lands past it, which is what keeps the sequence the same.
    pub(crate) fn next_random_int(&mut self, bound: i64) -> i64 {
        if bound <= 0 {
            return 0;
        }
        let top = bound as u64 - 1;
        if top == 0 {
            return 0;
        }
        let mut mask = 1u64;
        while mask < top {
            mask = (mask << 1) | 1;
        }
        loop {
            let mut value = 0u64;
            let mut landed = true;
            for place in (0..2).rev() {
                if (mask >> (place * 32)) & 0xffff_ffff == 0 {
                    continue;
                }
                value |= (self.next_random_word() as u64) << (place * 32);
                value &= mask;
                if top < value {
                    landed = false;
                    break;
                }
            }
            if landed {
                return value as i64;
            }
        }
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
            // A bound past the machine word is drawn word by word, since
            // there is no single draw wide enough to cover it.
            Object::BigInt(ref bound) => {
                let magnitude = if **bound < num_bigint::BigInt::from(0) {
                    -(**bound).clone()
                } else {
                    (**bound).clone()
                };
                if magnitude == num_bigint::BigInt::from(0) {
                    return Ok(Object::Float(self.next_random_float()));
                }
                Ok(Object::integer(self.next_random_big(&magnitude)))
            }
            Object::Range {
                ref start,
                ref end,
                exclusive,
                ..
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
            return self.random_across_width(start, end, exclusive, position);
        };
        if high < low || (exclusive && high == low) {
            return Ok(Object::Nil);
        }
        if high == low {
            return Ok(Object::Float(low));
        }
        Ok(Object::Float(low + self.next_random_float() * (high - low)))
    }

    /// A range whose ends are neither Integers nor Floats. Ruby measures the
    /// width between them with `-` and adds a number of that size back onto
    /// the start, so any type that subtracts and adds can bound a draw.
    fn random_across_width(
        &mut self,
        start: &Object,
        end: &Object,
        exclusive: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let width = self
            .apply_named_method(end, "-", vec![start.clone()], position)
            .map_err(|_| bad_range_value(position))?;
        let drawn = match &width {
            Object::Float(measured) => Object::Float(self.next_random_float() * measured),
            _ => {
                let counted = match &width {
                    Object::Int(counted) => *counted,
                    _ => match self.apply_named_method(&width, "to_int", vec![], position) {
                        Ok(Object::Int(counted)) => counted,
                        _ => return Err(bad_range_value(position)),
                    },
                };
                let counted = if exclusive { counted } else { counted + 1 };
                if counted <= 0 {
                    return Err(bad_range_value(position));
                }
                Object::Int(self.next_random_int(counted))
            }
        };
        self.apply_named_method(start, "+", vec![drawn], position)
    }

    /// Call one method on a receiver by name, which native code needs when
    /// the operation belongs to whatever type the program handed over.
    fn apply_named_method(
        &mut self,
        receiver: &Object,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some((class, method)) = self.lookup_method(receiver, name) else {
            return Err(bad_range_value(position));
        };
        self.invoke_method(class, method, receiver.clone(), arguments, position)
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
    /// The `Thread::Backtrace::Location` class the prelude defines. Its
    /// instances carry a `path`, `lineno`, `label`, and `absolute_path`,
    /// which is what both `caller_locations` and
    /// `Exception#backtrace_locations` hand out.
    pub(crate) fn backtrace_location_class(&mut self) -> std::rc::Rc<crate::class::Class> {
        use std::rc::Rc;
        if let Some(Object::Class(thread)) = self.globals().get("Thread")
            && let Some(Object::Class(backtrace)) = thread.get_class_var("Backtrace")
            && let Some(Object::Class(location)) = backtrace.get_class_var("Location")
        {
            return location;
        }
        Rc::new(crate::class::Class::new(
            "Thread::Backtrace::Location",
            None,
        ))
    }

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
        // A stream reassigned to a File handle answers `write` natively, with
        // no entry in a method table to find.
        if let Object::Instance(_) = &target {
            let argument = Object::string(text.to_string());
            let class = self.builtins().class_of(&target);
            if self
                .call_native_method(&class, &target, "write", &[argument], position)?
                .is_some()
            {
                return Ok(());
            }
        }
        write_to_standard_stream(stream, text);
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
            1 => Ok(Object::symbol(names[0].clone())),
            _ => Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                names.into_iter().map(Object::symbol).collect(),
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
/// The low machine word of a seed, which is all the generator reads. A seed
/// wider than 64 bits still has to drive the same state word.
/// How many words the Mersenne Twister keeps, and how far a step reaches.
const MT_WORDS: usize = 624;
const MT_STEP: usize = 397;

#[allow(dead_code)]
fn seed_low_bits(seed: &Object) -> u64 {
    match seed {
        Object::BigInt(wide) => {
            let (sign, digits) = wide.to_u64_digits();
            let magnitude = digits.first().copied().unwrap_or(0);
            if matches!(sign, num_bigint::Sign::Minus) {
                (magnitude as i64).wrapping_neg() as u64
            } else {
                magnitude
            }
        }
        Object::Int(narrow) => *narrow as u64,
        _ => 0,
    }
}

fn global_name_from(named: &Object) -> String {
    let text = match named {
        Object::Symbol(name) | Object::String(name) => name.as_str().to_string(),
        other => other.to_string(),
    };
    text.strip_prefix('$').unwrap_or(&text).to_string()
}

/// A line with its trailing separator removed. With no separator given, a
/// trailing "\r\n", "\n", or "\r" goes, which is what `$/` names by default.
fn chomped(line: &str, separator: Option<&str>) -> String {
    match separator {
        Some(separator) if separator != "\n" => match line.strip_suffix(separator) {
            Some(rest) => rest.to_string(),
            None => line.to_string(),
        },
        _ => {
            for ending in ["\r\n", "\n", "\r"] {
                if let Some(rest) = line.strip_suffix(ending) {
                    return rest.to_string();
                }
            }
            line.to_string()
        }
    }
}

/// Libraries metorex provides itself, which `require` answers for without
/// looking for a file.
const BUILT_IN_FEATURES: &[&str] = &["stringio", "set", "enumerator", "prettyprint"];

impl VirtualMachine {
    /// The Math module's functions. Each takes its arguments as Floats, which
    /// is what `Float()` reads them as, and answers a Float.
    fn apply_math_function(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(Object::Symbol(name) | Object::String(name)) = arguments.first() else {
            return Err(MetorexError::runtime_error(
                "__math_function__ expects the function name first",
                crate::vm::utils::position_to_location(position),
            ));
        };
        let name = name.as_str().to_string();
        let mut values = Vec::new();
        for (index, argument) in arguments[1..].iter().enumerate() {
            // `ldexp` scales by a whole number of powers of two, so its second
            // argument is read the way `Integer()` reads one.
            if name == "ldexp" && index == 1 {
                let exponent = match argument {
                    Object::Float(number) => *number,
                    other => self
                        .integer_class_method("try_convert", other, position)
                        .ok()
                        .and_then(|converted| match converted {
                            Object::Int(number) => Some(number as f64),
                            _ => None,
                        })
                        .ok_or_else(|| {
                            let message = format!(
                                "can't convert {} into Integer",
                                self.builtins().class_of(other).name()
                            );
                            MetorexError::UncaughtException {
                                exception: Object::exception("TypeError", message.clone()),
                                location: crate::vm::utils::position_to_location(position),
                                message,
                            }
                        })?,
                };
                values.push(exponent);
                continue;
            }
            values.push(self.math_argument(argument, position)?);
        }
        let first = values.first().copied().unwrap_or_default();
        // NaN names no point in any domain, and every function answers it
        // rather than refusing it.
        if first.is_nan() && !matches!(name.as_str(), "frexp" | "lgamma" | "ldexp") {
            return Ok(Object::Float(first));
        }
        // A function with no answer at this point names the argument as out of
        // its domain, which is what Ruby reports for `Math.sqrt(-1)`.
        let out_of_domain = |name: &str| {
            // Ruby quotes the function's name here, except in `log1p`, which
            // reports it bare.
            let named = match name {
                "log1p" => name.to_string(),
                _ => format!("\"{}\"", name),
            };
            crate::vm::errors::simple_exception(
                "Math::DomainError",
                &format!("Numerical argument is out of domain - {}", named),
                position,
            )
        };
        let answer = match name.as_str() {
            "sqrt" => {
                if first < 0.0 {
                    return Err(out_of_domain("sqrt"));
                }
                first.sqrt()
            }
            "cbrt" => first.cbrt(),
            "sin" => first.sin(),
            "cos" => first.cos(),
            "tan" => first.tan(),
            "asin" => {
                if !(-1.0..=1.0).contains(&first) {
                    return Err(out_of_domain("asin"));
                }
                first.asin()
            }
            "acos" => {
                if !(-1.0..=1.0).contains(&first) {
                    return Err(out_of_domain("acos"));
                }
                first.acos()
            }
            "atan" => first.atan(),
            "atan2" => first.atan2(values.get(1).copied().unwrap_or_default()),
            "sinh" => first.sinh(),
            "cosh" => first.cosh(),
            "tanh" => first.tanh(),
            "asinh" => first.asinh(),
            "acosh" => {
                if first < 1.0 {
                    return Err(out_of_domain("acosh"));
                }
                first.acosh()
            }
            "atanh" => {
                if first.abs() > 1.0 {
                    return Err(out_of_domain("atanh"));
                }
                first.atanh()
            }
            "exp" => first.exp(),
            "log" => {
                if first < 0.0 {
                    return Err(out_of_domain("log"));
                }
                let logarithm = match arguments.get(1).and_then(exact_log2) {
                    Some(exact) => exact,
                    None => first.log2(),
                };
                match values.get(1) {
                    Some(base) => logarithm / base.log2(),
                    None => logarithm * std::f64::consts::LN_2,
                }
            }
            "log2" => {
                if first < 0.0 {
                    return Err(out_of_domain("log2"));
                }
                match arguments.get(1).and_then(exact_log2) {
                    Some(exact) => exact,
                    None => first.log2(),
                }
            }
            "log10" => {
                if first < 0.0 {
                    return Err(out_of_domain("log10"));
                }
                match arguments.get(1).and_then(exact_log2) {
                    Some(exact) => exact / (10f64).log2(),
                    None => first.log10(),
                }
            }
            "log1p" => {
                if first < -1.0 {
                    return Err(out_of_domain("log1p"));
                }
                first.ln_1p()
            }
            "expm1" => first.exp_m1(),
            "hypot" => first.hypot(values.get(1).copied().unwrap_or_default()),
            "erf" => error_function(first),
            "erfc" => 1.0 - error_function(first),
            "gamma" => {
                if first == 0.0 {
                    return Ok(Object::Float(f64::INFINITY * first.signum()));
                }
                if first.is_infinite() {
                    if first.is_sign_negative() {
                        return Err(out_of_domain("gamma"));
                    }
                    return Ok(Object::Float(f64::INFINITY));
                }
                if first < 0.0 && first.fract() == 0.0 {
                    return Err(out_of_domain("gamma"));
                }
                // A whole number's gamma is a factorial, and multiplying the
                // factors keeps every digit a double can hold, which the
                // series approximation does not.
                if first > 0.0 && first.fract() == 0.0 && first <= LARGEST_EXACT_FACTORIAL {
                    let mut product = 1.0;
                    let mut factor = 2.0;
                    while factor < first {
                        product *= factor;
                        factor += 1.0;
                    }
                    return Ok(Object::Float(product));
                }
                gamma_function(first)
            }
            // `ldexp` scales by a power of two, and `frexp` splits a number
            // into the pair that does so.
            "ldexp" => {
                let exponent = values.get(1).copied().unwrap_or_default();
                if exponent.is_nan() {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "float NaN out of range of integer",
                        position,
                    ));
                }
                return Ok(Object::Float(
                    crate::vm::native_methods::scale_by_power_of_two(first, exponent as i64),
                ));
            }
            "frexp" => {
                let (fraction, exponent) = split_float(first);
                return Ok(Object::array(vec![
                    Object::Float(fraction),
                    Object::Int(exponent),
                ]));
            }
            // `lgamma` answers the log of the gamma function's magnitude with
            // the sign it dropped.
            "lgamma" => {
                if first.is_infinite() && first < 0.0 {
                    return Err(out_of_domain("lgamma"));
                }
                // The gamma function grows without bound, so its logarithm
                // does too.
                if first.is_infinite() {
                    return Ok(Object::array(vec![
                        Object::Float(f64::INFINITY),
                        Object::Int(1),
                    ]));
                }
                // It has a pole at every whole number at or below zero, so
                // its magnitude there has no logarithm short of infinity. The
                // sign alternates from one pole to the next, and negative
                // zero approaches the pole at zero from the other side.
                if first <= 0.0 && first.fract() == 0.0 {
                    let sign = if first == 0.0 {
                        if first.is_sign_negative() { -1 } else { 1 }
                    } else if (first / 2.0).fract() != 0.0 {
                        1
                    } else {
                        -1
                    };
                    return Ok(Object::array(vec![
                        Object::Float(f64::INFINITY),
                        Object::Int(sign),
                    ]));
                }
                let value = gamma_function(first);
                return Ok(Object::array(vec![
                    Object::Float(value.abs().ln()),
                    Object::Int(if value < 0.0 { -1 } else { 1 }),
                ]));
            }
            other => {
                return Err(MetorexError::runtime_error(
                    format!("unknown math function: {}", other),
                    crate::vm::utils::position_to_location(position),
                ));
            }
        };
        Ok(Object::Float(answer))
    }

    /// A Math argument as a Float. Ruby reads it the way `Float()` does, so a
    /// String or an object that is not a number is refused.
    fn math_argument(&mut self, value: &Object, position: Position) -> Result<f64, MetorexError> {
        match value {
            Object::Int(number) => Ok(*number as f64),
            Object::BigInt(number) => Ok(crate::vm::operators::big_to_float(number)),
            Object::Float(number) => Ok(*number),
            other => {
                let message = format!(
                    "can't convert {} into Float",
                    match other {
                        Object::Nil => "nil".to_string(),
                        _ => self.builtins().class_of(other).name().to_string(),
                    }
                );
                let refuse = MetorexError::UncaughtException {
                    exception: Object::exception("TypeError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                };
                // Only a number answers `to_f` here: a String has one too, and
                // Ruby refuses it all the same.
                let numeric = matches!(other, Object::Instance(instance)
                    if crate::vm::method_invocation::descends_from(&instance.borrow().class, "Numeric"));
                if !numeric {
                    return Err(refuse);
                }
                match self.send_to_object(other.clone(), "to_f", vec![], position)? {
                    Object::Float(number) => Ok(number),
                    Object::Int(number) => Ok(number as f64),
                    _ => Err(refuse),
                }
            }
        }
    }
}

/// The error function, by the series that converges quickly for a small
/// argument and the continued-fraction form beyond it.
fn error_function(value: f64) -> f64 {
    if value.is_nan() {
        return value;
    }
    let magnitude = value.abs();
    if magnitude > 6.0 {
        return value.signum();
    }
    // Abramowitz and Stegun 7.1.26, which is accurate to about 1e-7.
    let t = 1.0 / (1.0 + 0.3275911 * magnitude);
    let polynomial = t
        * (0.254829592
            + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    let answer = 1.0 - polynomial * (-magnitude * magnitude).exp();
    answer * value.signum()
}

/// The largest whole number whose factorial a double still holds. Above it
/// the product overflows to infinity, which is what Ruby answers there too.
const LARGEST_EXACT_FACTORIAL: f64 = 171.0;

/// The gamma function, by the Lanczos approximation.
fn gamma_function(value: f64) -> f64 {
    const COEFFICIENTS: [f64; 9] = [
        0.999_999_999_999_81,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if value < 0.5 {
        // The reflection formula carries a negative argument to a positive one.
        return std::f64::consts::PI
            / ((std::f64::consts::PI * value).sin() * gamma_function(1.0 - value));
    }
    let value = value - 1.0;
    let mut series = COEFFICIENTS[0];
    for (index, coefficient) in COEFFICIENTS.iter().enumerate().skip(1) {
        series += coefficient / (value + index as f64);
    }
    let t = value + 7.5;
    (2.0 * std::f64::consts::PI).sqrt() * t.powf(value + 0.5) * (-t).exp() * series
}

/// A float split into the fraction and the power of two that rebuild it, which
/// is the pair `frexp` answers.
fn split_float(value: f64) -> (f64, i64) {
    if value == 0.0 || !value.is_finite() {
        return (value, 0);
    }
    let exponent = value.abs().log2().floor() as i64 + 1;
    let fraction = value / (2f64).powi(exponent as i32);
    (fraction, exponent)
}

/// The base-2 logarithm of an exact integer, which keeps the digits a Float
/// cannot hold: the bit length gives the whole part and the leading bits the
/// fraction, so `Math.log2(2 ** 10001)` is 10001.0 rather than an infinity.
fn exact_log2(value: &Object) -> Option<f64> {
    let value = match value {
        Object::BigInt(number) => (*number).clone(),
        _ => return None,
    };
    if *value <= num_bigint::BigInt::from(0) {
        return None;
    }
    let bits = value.bits() as i64;
    // Keep the leading bits as a Float and let the rest count as the exponent.
    let kept = 64.min(bits);
    let leading = crate::vm::operators::big_to_float(&(&*value >> (bits - kept) as u32));
    Some(leading.log2() + (bits - kept) as f64)
}

/// The ArgumentError Ruby raises for a range whose ends cannot bound a draw.
fn bad_range_value(position: Position) -> MetorexError {
    let message = "bad value for range".to_string();
    MetorexError::UncaughtException {
        exception: Object::exception("ArgumentError", message.clone()),
        location: crate::vm::utils::position_to_location(position),
        message,
    }
}

/// The name the frame at `index` reads in a backtrace. A block is named for
/// the scope it was written in and for how many blocks deep it sits there.
pub(crate) fn frame_label_at(frames: &[&crate::vm::CallFrame], index: usize) -> String {
    let Some(frame) = frames.get(index) else {
        return "<main>".to_string();
    };
    let name = frame.name().to_string();
    let depth = if name == "<block>" {
        frame.block_depth().max(1)
    } else {
        frame.block_depth()
    };
    let held = if name == "<block>" {
        frame.written_in().unwrap_or("<main>").to_string()
    } else {
        name
    };
    let held = backtrace_label(&held);
    match depth {
        0 => held,
        1 => format!("block in {held}"),
        counted => format!("block ({counted} levels) in {held}"),
    }
}

/// The name a backtrace entry reads. A method defined on one object alone is
/// named by itself: the singleton class holding it has no name a reader would
/// know, so only the method's own name is written.
pub(crate) fn backtrace_label(name: &str) -> String {
    // The body of a file that was required is named for being that, since
    // the file it belongs to is written alongside the label anyway.
    if name.starts_with("<file:") {
        return "<top (required)>".to_string();
    }
    if name.starts_with("#<Class:#<") {
        return match name.rfind('#') {
            Some(at) if at + 1 < name.len() => name[at + 1..].to_string(),
            _ => name.to_string(),
        };
    }
    // A method written in `class << Name` belongs to Name, and that is the
    // name a reader knows it by.
    if let Some(rest) = name.strip_prefix("#<Class:")
        && let Some(at) = rest.find(">.")
    {
        return format!("{}{}", &rest[..at], &rest[at + 1..]);
    }
    name.to_string()
}

/// Whether a command written as one string holds a character the shell reads,
/// which is what decides between running it through `sh` and running it as
/// the program it names.
fn needs_a_shell(command: &str) -> bool {
    const READ_BY_THE_SHELL: &[u8] = b"*?{}[]<>()~&|\\$;'`\"\n#";
    command
        .bytes()
        .any(|byte| READ_BY_THE_SHELL.contains(&byte))
}

/// Run a program to completion, answering how it ended and the process id it
/// ran under. A program that cannot be reached ends with status 127 and says
/// nothing, which is what the shell reports for one.
fn run_to_completion(
    reached: &str,
    words: &[String],
    redirects: &[(i32, String)],
) -> (std::process::ExitStatus, i64) {
    use std::os::unix::process::ExitStatusExt as _;
    let named = std::ffi::CString::new(reached).unwrap_or_default();
    let spelled: Vec<std::ffi::CString> = words
        .iter()
        .map(|held| std::ffi::CString::new(held.as_str()).unwrap_or_default())
        .collect();
    let opened: Vec<(i32, std::fs::File)> = redirects
        .iter()
        .filter_map(|(slot, path)| {
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)
                .ok()
                .map(|held| (*slot, held))
        })
        .collect();
    // SAFETY: the child does nothing but point its streams where it was told
    // and hand itself over to the program, so nothing the parent holds is
    // read or written there.
    let child = unsafe { libc::fork() };
    if child == 0 {
        use std::os::unix::io::AsRawFd as _;
        for (slot, file) in &opened {
            // SAFETY: both numbers name descriptors this process holds.
            unsafe { libc::dup2(file.as_raw_fd(), *slot) };
        }
        let mut pointers: Vec<*const libc::c_char> =
            spelled.iter().map(|held| held.as_ptr()).collect();
        pointers.push(std::ptr::null());
        // SAFETY: the argument list is null-terminated and outlives the call.
        unsafe {
            libc::execvp(named.as_ptr(), pointers.as_ptr());
            libc::_exit(127);
        }
    }
    if child < 0 {
        return (std::process::ExitStatus::from_raw(127 << 8), 0);
    }
    let mut held: libc::c_int = 0;
    // SAFETY: `child` is a process this one started.
    unsafe { libc::waitpid(child, &mut held, 0) };
    (std::process::ExitStatus::from_raw(held), child as i64)
}

impl crate::vm::core::VirtualMachine {
    /// A name written at the top level names a method of Object, since that
    /// is where a `def` outside any class or module is written. The top-level
    /// `self` is the Object class itself, so an ordinary send looks for a
    /// class method of that name and finds nothing.
    fn top_level_method(&mut self, receiver: &Object, name: &str) -> Option<Object> {
        let Object::Class(held) = receiver else {
            return None;
        };
        if held.name() != "Object" {
            return None;
        }
        let (owner, method) = held.find_method_with_owner(name)?;
        let mut bound = (*method).clone();
        bound.receiver = Some(Box::new(receiver.clone()));
        bound.owner = Some(owner.ruby_name());
        bound.owner_class = Some(owner);
        Some(Object::Method(std::rc::Rc::new(bound)))
    }
}

/// Write to one of the standard streams. A stream whose other end has gone
/// ends the program the way the signal would.
pub(crate) fn write_to_standard_stream(stream: &str, text: &str) {
    use std::io::Write as _;
    let sent = if stream == "stderr" {
        let mut held = std::io::stderr();
        held.write_all(text.as_bytes()).and_then(|()| held.flush())
    } else {
        let mut held = std::io::stdout();
        held.write_all(text.as_bytes()).and_then(|()| held.flush())
    };
    if let Err(trouble) = sent
        && trouble.kind() == std::io::ErrorKind::BrokenPipe
    {
        die_of_a_broken_pipe();
    }
}

/// End the program the way a signal would when the other end of the standard
/// stream has gone. Ruby lets SIGPIPE through for the standard streams, and a
/// program in a pipeline is told to stop that way.
fn die_of_a_broken_pipe() -> ! {
    // SAFETY: both calls name a signal this process may send itself, and the
    // disposition restored is the one every program starts with.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        libc::raise(libc::SIGPIPE);
    }
    std::process::exit(141)
}

/// Where each frame was called from, innermost first. A frame reached from
/// the core library's own Ruby source stands for the place that reached it,
/// so a backtrace names the program's file rather than `<internal:...>`.
fn frame_call_sites(frames: &[&crate::vm::CallFrame], current_file: &str) -> Vec<(String, i64)> {
    let mut sites: Vec<(String, i64)> = frames
        .iter()
        .map(|frame| match frame.location() {
            // Frame locations are "line:column" or "file:line:column".
            Some(written) => {
                let parts: Vec<&str> = written.rsplitn(3, ':').collect();
                let line = parts
                    .get(1)
                    .and_then(|held| held.parse::<i64>().ok())
                    .unwrap_or(0);
                let path = match parts.get(2) {
                    Some(path) if !path.is_empty() => (*path).to_string(),
                    // The frame records the file its call site sits in, which
                    // is where the location belongs.
                    _ => frame
                        .source_file()
                        .map(|file| file.to_string())
                        .unwrap_or_else(|| current_file.to_string()),
                };
                (path, line)
            }
            None => (
                frame
                    .source_file()
                    .map(|file| file.to_string())
                    .unwrap_or_else(|| current_file.to_string()),
                0,
            ),
        })
        .collect();
    // Walking outward, an internal call site takes the one below it, which is
    // the nearest place in the program itself.
    let mut carried: Option<(String, i64)> = None;
    for site in sites.iter_mut().rev() {
        if site.0.starts_with(crate::vm::INTERNAL_FILE_PREFIX) {
            if let Some(held) = &carried {
                *site = held.clone();
            }
        } else {
            carried = Some(site.clone());
        }
    }
    sites
}

/// Where a program named on the command line sits, which is the name itself
/// when it holds a slash and otherwise the first place on the path that holds
/// a file of that name the program may run.
fn findable_program(program: &str) -> Option<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt as _;
    let runnable = |path: &std::path::Path| {
        std::fs::metadata(path)
            .map(|held| held.is_file() && held.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    };
    if program.contains('/') {
        let named = std::path::PathBuf::from(program);
        return runnable(&named).then_some(named);
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|held| held.join(program))
        .find(|held| runnable(held))
}
