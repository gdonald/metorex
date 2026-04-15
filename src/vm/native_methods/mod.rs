//! Native (built-in) method implementations for the virtual machine.
//!
//! This module contains the implementations of all built-in methods for
//! standard classes like Object, String, and Array.

mod array_methods;
pub(crate) mod ast_methods;
mod exception_methods;
mod float_methods;
mod hash_methods;
mod int_methods;
mod object_methods;
mod range_methods;
mod set_methods;
mod string_methods;

use super::VirtualMachine;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    /// Attempt to execute a native (built-in) method implementation.
    ///
    /// Returns `Ok(Some(result))` if a native method was found and executed successfully,
    /// `Ok(None)` if no native method exists (allowing fallback to user-defined methods),
    /// or `Err` if the method call failed.
    pub(crate) fn call_native_method(
        &mut self,
        class: &Class,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Special handling for Binding objects
        if let Object::Binding(binding) = receiver
            && method_name == "receiver"
        {
            return Ok(Some(binding.receiver.clone().unwrap_or(Object::Nil)));
        }

        // Special handling for Block/Lambda objects
        if let Object::Block(block) = receiver {
            match method_name {
                "call" | "[]" => {
                    return Ok(Some(block.call(self, arguments.to_vec(), position)?));
                }
                "binding" => {
                    use crate::object::Binding;
                    // Create a Binding object from the block's captured variables
                    let binding = Binding::new(block.captured_vars().clone());
                    return Ok(Some(Object::Binding(Rc::new(binding))));
                }
                _ => {}
            }
        }

        // Special handling for Class objects
        if let Object::Class(class_rc) = receiver {
            let non_instantiable =
                matches!(class_rc.name(), "TrueClass" | "FalseClass" | "NilClass");
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
            // Dir.pwd / Dir.getwd — current working directory.
            if class_rc.name() == "Dir" && (method_name == "pwd" || method_name == "getwd") {
                let cwd = std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                return Ok(Some(Object::string(cwd)));
            }
            // Dir.exist? / exists?
            if class_rc.name() == "Dir" && (method_name == "exist?" || method_name == "exists?") {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                return Ok(Some(Object::Bool(std::path::Path::new(&path).is_dir())));
            }
            // Dir.mkdir / Dir.delete — wrap stdlib calls.
            if class_rc.name() == "Dir"
                && (method_name == "mkdir" || method_name == "delete" || method_name == "rmdir")
            {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String",
                            other,
                            position,
                        ));
                    }
                };
                let _ = if method_name == "mkdir" {
                    std::fs::create_dir_all(&path)
                } else {
                    std::fs::remove_dir(&path)
                };
                return Ok(Some(Object::Int(0)));
            }
            // Dir["pattern"] / Dir.glob("pattern")
            if class_rc.name() == "Dir" && (method_name == "[]" || method_name == "glob") {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let mut results: Vec<Object> = Vec::new();
                for arg in arguments {
                    let pattern = match arg {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                other,
                                position,
                            ));
                        }
                    };
                    if let Ok(paths) = glob::glob(&pattern) {
                        for entry in paths.flatten() {
                            results.push(Object::string(entry.to_string_lossy().to_string()));
                        }
                    }
                }
                return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                    results,
                )))));
            }
            match method_name {
                "now" | "new" if class_rc.name() == "Time" => {
                    let secs = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs_f64();
                    return Ok(Some(Object::Float(secs)));
                }
                "new" if class_rc.name() == "Set" => {
                    // Set.new creates a new set, optionally from an array
                    use crate::object::ObjectHash;
                    let mut set = std::collections::HashSet::new();
                    if arguments.len() == 1 {
                        if let Object::Array(arr_rc) = &arguments[0] {
                            for item in arr_rc.borrow().iter() {
                                if let Some(hash) = ObjectHash::from_object(item) {
                                    set.insert(hash);
                                } else {
                                    return Err(MetorexError::runtime_error(
                                        format!(
                                            "Cannot add {} to set (not hashable)",
                                            item.type_name()
                                        ),
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
                "new" => {
                    // Delegate to invoke_callable which handles instance creation and initialize
                    return self
                        .invoke_callable(
                            Object::Class(Rc::clone(class_rc)),
                            arguments.to_vec(),
                            position,
                        )
                        .map(Some);
                }
                "name" => {
                    return Ok(Some(Object::String(Rc::new(class_rc.name().to_string()))));
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
                    // Check if the constant exists as a class variable or in globals
                    let found = class_rc.get_class_var(&const_name).is_some()
                        || self.environment().get(&const_name).is_some()
                        || self.globals().get(&const_name).is_some();
                    return Ok(Some(Object::Bool(found)));
                }
                // File class methods
                "read" if class_rc.name() == "File" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error("read", 1, arguments.len(), position));
                    }
                    let path = match &arguments[0] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "read", "String", other, position,
                            ));
                        }
                    };
                    let contents = std::fs::read_to_string(&path).map_err(|e| {
                        MetorexError::runtime_error(
                            format!("Failed to read file '{}': {}", path, e),
                            position_to_location(position),
                        )
                    })?;
                    return Ok(Some(Object::string(contents)));
                }
                "write" if class_rc.name() == "File" => {
                    if arguments.len() != 2 {
                        return Err(method_argument_error("write", 2, arguments.len(), position));
                    }
                    let path = match &arguments[0] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "write", "String", other, position,
                            ));
                        }
                    };
                    let content = match &arguments[1] {
                        Object::String(s) => s.as_str().to_string(),
                        other => format!("{}", other),
                    };
                    std::fs::write(&path, &content).map_err(|e| {
                        MetorexError::runtime_error(
                            format!("Failed to write file '{}': {}", path, e),
                            position_to_location(position),
                        )
                    })?;
                    return Ok(Some(Object::Int(content.len() as i64)));
                }
                "exist?" | "exists?" if class_rc.name() == "File" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            "exist?",
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let path = match &arguments[0] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "exist?", "String", other, position,
                            ));
                        }
                    };
                    return Ok(Some(Object::Bool(std::path::Path::new(&path).exists())));
                }
                "realpath" | "realdirpath" if class_rc.name() == "File" => {
                    if arguments.is_empty() || arguments.len() > 2 {
                        return Err(method_argument_error(
                            "realpath",
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let path_str = match &arguments[0] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "realpath", "String", other, position,
                            ));
                        }
                    };
                    let base = if arguments.len() == 2 {
                        match &arguments[1] {
                            Object::String(s) => s.as_str().to_string(),
                            other => {
                                return Err(method_argument_type_error(
                                    "realpath", "String", other, position,
                                ));
                            }
                        }
                    } else {
                        std::env::current_dir()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    };
                    let base_path = std::path::PathBuf::from(&base);
                    let expanded = base_path.join(&path_str);
                    match expanded.canonicalize() {
                        Ok(p) => {
                            return Ok(Some(Object::string(p.to_string_lossy().to_string())));
                        }
                        Err(e) => {
                            let exc = if e.kind() == std::io::ErrorKind::NotFound {
                                Object::exception(
                                    "Errno::ENOENT",
                                    format!("No such file or directory @ realpath - {}", path_str),
                                )
                            } else {
                                Object::exception(
                                    "Errno::ENOTDIR",
                                    format!("Not a directory @ realpath - {}", path_str),
                                )
                            };
                            return Err(MetorexError::UncaughtException {
                                exception: exc.clone(),
                                location: position_to_location(position),
                                message: format!("{}", exc),
                            });
                        }
                    }
                }
                "directory?" if class_rc.name() == "File" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            "directory?",
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let path = match &arguments[0] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "directory?",
                                "String",
                                other,
                                position,
                            ));
                        }
                    };
                    return Ok(Some(Object::Bool(std::path::Path::new(&path).is_dir())));
                }
                "file?" if class_rc.name() == "File" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error("file?", 1, arguments.len(), position));
                    }
                    let path = match &arguments[0] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "file?", "String", other, position,
                            ));
                        }
                    };
                    return Ok(Some(Object::Bool(std::path::Path::new(&path).is_file())));
                }
                "expand_path" if class_rc.name() == "File" => {
                    if arguments.is_empty() || arguments.len() > 2 {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "wrong number of arguments (given {}, expected 1..2)",
                                arguments.len()
                            ),
                            position_to_location(position),
                        ));
                    }
                    let path_str = match &arguments[0] {
                        Object::String(s) => s.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                "expand_path",
                                "String",
                                other,
                                position,
                            ));
                        }
                    };
                    let base = if arguments.len() == 2 {
                        match &arguments[1] {
                            Object::String(s) => s.as_str().to_string(),
                            other => {
                                return Err(method_argument_type_error(
                                    "expand_path",
                                    "String",
                                    other,
                                    position,
                                ));
                            }
                        }
                    } else {
                        std::env::current_dir()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    };
                    let base_path = std::path::PathBuf::from(&base);
                    let expanded = base_path.join(&path_str);
                    let result = match expanded.canonicalize() {
                        Ok(p) => p.to_string_lossy().to_string(),
                        Err(_) => {
                            // Path doesn't exist yet; normalize manually
                            let mut components = Vec::new();
                            for comp in expanded.components() {
                                match comp {
                                    std::path::Component::ParentDir => {
                                        components.pop();
                                    }
                                    std::path::Component::CurDir => {}
                                    _ => components.push(comp),
                                }
                            }
                            let normalized: std::path::PathBuf = components.iter().collect();
                            normalized.to_string_lossy().to_string()
                        }
                    };
                    return Ok(Some(Object::string(result)));
                }
                "class_eval" | "module_eval" => {
                    // Execute a block in the context of this class
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
                    // Parse out variadic (*args) and block (&block) from parameter names
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
                    // Capture closure: prefer existing captured_vars, otherwise snap current scope
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
                        // Store as __ext__ so it's callable on the class/module receiver
                        class_rc
                            .set_class_var(format!("__ext__{}", name_str), Object::Method(method));
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
                _ => {}
            }
        }

        // Module receivers: support the same methods as Class (remove_method, etc.)
        if let Object::Module(module_rc) = receiver {
            // Kernel.load — delegates to the VM's file loader.
            if module_rc.name() == "Kernel" && method_name == "load" {
                if arguments.len() != 1 {
                    return Err(method_argument_error("load", 1, arguments.len(), position));
                }
                let path = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            "load", "String", other, position,
                        ));
                    }
                };
                self.execute_file(std::path::Path::new(&path))?;
                return Ok(Some(Object::Bool(true)));
            }
            // Signal module stubs: trap is a no-op; the block is discarded.
            if module_rc.name() == "Signal" && method_name == "trap" {
                self.pending_block.take();
                return Ok(Some(Object::Nil));
            }
            // Process.pid returns the actual OS pid; the rest are stubs.
            if module_rc.name() == "Process" {
                match method_name {
                    "pid" => return Ok(Some(Object::Int(std::process::id() as i64))),
                    "ppid" => return Ok(Some(Object::Int(0))),
                    "kill" | "wait" | "waitall" | "exit" | "exit!" | "abort" => {
                        return Ok(Some(Object::Nil));
                    }
                    _ => {}
                }
            }
            // GC.* / ObjectSpace.* — no-op stubs returning nil for everything
            // except `name` (which falls through to the generic name handler).
            if (module_rc.name() == "GC" || module_rc.name() == "ObjectSpace")
                && method_name != "name"
            {
                return Ok(Some(Object::Nil));
            }
            match method_name {
                "name" => {
                    return Ok(Some(Object::String(Rc::new(module_rc.name().to_string()))));
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
                    if !module_rc.remove_method(&name_str) {
                        return Err(MetorexError::runtime_error(
                            format!("method '{}' not defined in {}", name_str, module_rc.name()),
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
                    module_rc.define_method(&name_str, Rc::new(sentinel));
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
                    if !module_rc.alias_method(&new_name, &old_name) {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "undefined method '{}' for module '{}'",
                                old_name,
                                module_rc.name()
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
                    if let Some(method) = module_rc.find_method(&name_str) {
                        module_rc
                            .set_class_var(format!("__ext__{}", name_str), Object::Method(method));
                    } else {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "undefined method '{}' for module '{}'",
                                name_str,
                                module_rc.name()
                            ),
                            position_to_location(position),
                        ));
                    }
                    return Ok(Some(Object::Nil));
                }
                _ => {}
            }
        }

        // Special handling for Method objects
        if let Object::Method(method_obj) = receiver {
            match method_name {
                "name" => {
                    return Ok(Some(Object::String(Rc::new(method_obj.name.clone()))));
                }
                "owner" => {
                    let owner_name = method_obj.owner.as_deref().unwrap_or("main");
                    return Ok(Some(Object::String(Rc::new(owner_name.to_string()))));
                }
                "source_location" => {
                    if let Some(loc) = &method_obj.source_location {
                        return Ok(Some(Object::String(Rc::new(loc.to_string()))));
                    } else {
                        return Ok(Some(Object::String(Rc::new("unknown".to_string()))));
                    }
                }
                "parameters" => {
                    let params: Vec<Object> = method_obj
                        .parameters
                        .iter()
                        .map(|p| Object::String(Rc::new(p.clone())))
                        .collect();
                    return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                        params,
                    )))));
                }
                "body" => {
                    return Ok(Some(ast_methods::serialize_statements(&method_obj.body)));
                }
                "arity" => {
                    return Ok(Some(Object::Int(method_obj.parameters.len() as i64)));
                }
                _ => {}
            }
        }

        // Special handling for Block objects (AST inspection)
        if let Object::Block(block_obj) = receiver {
            match method_name {
                "statements" => {
                    return Ok(Some(ast_methods::serialize_statements(&block_obj.body)));
                }
                "arity" => {
                    return Ok(Some(Object::Int(block_obj.parameters.len() as i64)));
                }
                _ => {}
            }
        }

        // Dispatch to the appropriate class-specific method implementation
        match class.name() {
            "Object" => self.call_object_method(receiver, method_name, arguments, position),
            "String" => self.call_string_method(receiver, method_name, arguments, position),
            "Integer" => self.call_int_method(receiver, method_name, arguments, position),
            "Array" => self.call_array_method(receiver, method_name, arguments, position),
            "Hash" => self.call_hash_method(receiver, method_name, arguments, position),
            "Float" => self.call_float_method(receiver, method_name, arguments, position),
            "Range" => self.call_range_method(receiver, method_name, arguments, position),
            "Set" => self.call_set_method(receiver, method_name, arguments, position),
            "Exception" => self.call_exception_method(receiver, method_name, arguments, position),
            _ => Ok(None),
        }
    }

    /// Apply `private` / `public` visibility modifier as a Class-level method.
    pub(crate) fn apply_class_visibility_modifier(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        modifier: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() {
            return Ok(Object::Nil);
        }
        let flat: Vec<Object> = if arguments.len() == 1 {
            if let Object::Array(arr) = &arguments[0] {
                arr.borrow().clone()
            } else {
                arguments.to_vec()
            }
        } else {
            arguments.to_vec()
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
                        location: position_to_location(position),
                        message: format!("{} is not a symbol nor a string", arg),
                    });
                }
            };
            if class_rc.find_method(&n).is_none() {
                let msg = format!("undefined method '{}' for class '{}'", n, class_rc.name());
                let exc = Object::exception("NameError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                });
            }
            names.push(n);
        }
        for n in &names {
            if modifier == "private" {
                class_rc.set_method_private(n.clone());
            } else {
                class_rc.set_method_public(n);
            }
        }
        match flat.len() {
            1 => Ok(Object::Symbol(Rc::new(names[0].clone()))),
            _ => Ok(Object::Array(Rc::new(std::cell::RefCell::new(
                names
                    .into_iter()
                    .map(|n| Object::Symbol(Rc::new(n)))
                    .collect(),
            )))),
        }
    }
}
