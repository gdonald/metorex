//! Native (built-in) function implementations for the virtual machine.
//!
//! This module contains implementations of global built-in functions like puts, print, etc.

use super::VirtualMachine;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

impl VirtualMachine {
    /// Call a native function by name.
    pub(crate) fn call_native_function(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match name {
            "private" | "public" => {
                self.pending_block.take();
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
            "protected"
            | "module_function"
            | "private_class_method"
            | "public_class_method"
            | "deprecate_constant"
            | "noop_with_block" => {
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
                Ok(Object::Nil)
            }
            "warn" => {
                for arg in &arguments {
                    let s = self.get_string_representation(arg, position)?;
                    eprintln!("{}", s);
                }
                Ok(Object::Nil)
            }
            "sprintf" => {
                if arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        "sprintf requires at least 1 argument".to_string(),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let fmt = arguments[0].clone();
                let rest: Vec<Object> = arguments.into_iter().skip(1).collect();
                let rest_obj = if rest.len() == 1 {
                    rest.into_iter().next().unwrap()
                } else {
                    Object::Array(std::rc::Rc::new(std::cell::RefCell::new(rest)))
                };
                self.evaluate_string_format(fmt, rest_obj, position)
            }
            "__method__" => {
                let name = self
                    .call_stack()
                    .last()
                    .map(|f| {
                        let n = f.name();
                        n.rsplit('#').next().unwrap_or(n).to_string()
                    })
                    .unwrap_or_default();
                Ok(Object::Symbol(std::rc::Rc::new(name)))
            }
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
            "binding_kernel" => Ok(Object::Nil),
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
            "rand" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0);
                if arguments.is_empty() {
                    Ok(Object::Float((nanos as f64 / u32::MAX as f64).abs()))
                } else if let Object::Int(n) = &arguments[0] {
                    if *n > 0 {
                        Ok(Object::Int((nanos as i64) % n))
                    } else {
                        Ok(Object::Int(0))
                    }
                } else {
                    Ok(Object::Int(0))
                }
            }
            "srand" => Ok(Object::Int(0)),
            "sleep" => Ok(Object::Int(0)),
            "puts" => {
                // puts prints each argument on a new line
                for arg in &arguments {
                    // Try to call to_s or inspect method if it exists on the object
                    let output = self.get_string_representation(arg, position)?;
                    println!("{}", output);
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
            "print" => {
                // print outputs arguments without trailing newline
                for arg in &arguments {
                    let output = self.get_string_representation(arg, position)?;
                    print!("{}", output);
                }
                use std::io::Write;
                std::io::stdout().flush().ok();
                Ok(Object::Nil)
            }
            "p" => {
                // p prints the inspect representation of each argument
                for arg in &arguments {
                    println!("{:?}", arg);
                }
                if arguments.len() == 1 {
                    Ok(arguments.into_iter().next().unwrap())
                } else {
                    Ok(Object::Nil)
                }
            }
            "gets" => {
                // gets reads a line from stdin
                if !arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        format!("gets() expects 0 arguments, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).map_err(|e| {
                    MetorexError::runtime_error(
                        format!("Failed to read from stdin: {}", e),
                        crate::vm::utils::position_to_location(position),
                    )
                })?;
                // Remove trailing newline (like Ruby's gets)
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                Ok(Object::string(input))
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
                // eval runs at top-level of its string: treat as non-method scope.
                // Refinements activated inside eval are lexical to the eval string.
                let saved_nesting = self.user_def_nesting;
                self.user_def_nesting = 0;
                self.push_refinement_scope();
                let prev_file = self.current_file.clone();
                if let Some(f) = &filename {
                    self.current_file = Some(std::path::PathBuf::from(f));
                }
                let result = self.execute_program(&statements);
                self.current_file = prev_file;
                self.pop_refinement_scope();
                self.user_def_nesting = saved_nesting;
                Ok(result?.unwrap_or(Object::Nil))
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
            "exit" => {
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
            _ => Err(MetorexError::runtime_error(
                format!("Unknown native function: {}", name),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }

    /// Get the string representation of an object by calling to_s or inspect if available.
    fn get_string_representation(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        // First try to_s, then inspect, then fall back to Display
        match obj {
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
                // Fall back to default Display
                Ok(format!("{}", obj))
            }
            _ => Ok(format!("{}", obj)),
        }
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
