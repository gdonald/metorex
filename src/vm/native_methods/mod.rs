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
        // Special handling for Block/Lambda objects
        if let Object::Block(block) = receiver {
            match method_name {
                "call" => {
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
            match method_name {
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
                "superclass" => {
                    return match class_rc.superclass() {
                        Some(parent) => Ok(Some(Object::Class(parent))),
                        None => Ok(Some(Object::Nil)),
                    };
                }
                "ancestors" => {
                    let mut chain = vec![Object::Class(Rc::clone(class_rc))];
                    let mut current = class_rc.superclass();
                    while let Some(parent) = current {
                        chain.push(Object::Class(Rc::clone(&parent)));
                        current = parent.superclass();
                    }
                    return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(chain)))));
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
                    let mut method = Method::new(
                        method_name_str.clone(),
                        block.parameters.clone(),
                        block.body.clone(),
                    );
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
}
