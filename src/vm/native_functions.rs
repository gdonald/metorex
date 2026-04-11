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
            "private"
            | "public"
            | "protected"
            | "module_function"
            | "private_class_method"
            | "public_class_method"
            | "freeze" => {
                // Visibility modifiers and Object#freeze — no-op stubs. Accept any args.
                Ok(Object::Nil)
            }
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
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!("eval() expects 1 argument, got {}", arguments.len()),
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
                let tokens = crate::lexer::Lexer::new(&code).tokenize();
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
                let result = self.execute_program(&statements)?;
                Ok(result.unwrap_or(Object::Nil))
            }
            "load" => {
                if arguments.len() != 1 {
                    return Err(MetorexError::runtime_error(
                        format!("load() expects 1 argument, got {}", arguments.len()),
                        crate::vm::utils::position_to_location(position),
                    ));
                }
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
                // load always executes the file (no deduplication)
                // Try the path directly first, then search $LOAD_PATH
                if path.exists() {
                    self.execute_file(path).map_err(|e| {
                        MetorexError::runtime_error(
                            format!("load('{}') — {}", path_str, e.message()),
                            crate::vm::utils::position_to_location(position),
                        )
                    })?;
                    Ok(Object::Bool(true))
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
                    let resolved = found.ok_or_else(|| {
                        MetorexError::runtime_error(
                            format!("cannot load such file -- {}", path_str),
                            crate::vm::utils::position_to_location(position),
                        )
                    })?;
                    self.execute_file(&resolved).map_err(|e| {
                        MetorexError::runtime_error(
                            format!("load('{}') — {}", path_str, e.message()),
                            crate::vm::utils::position_to_location(position),
                        )
                    })?;
                    Ok(Object::Bool(true))
                }
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
}
