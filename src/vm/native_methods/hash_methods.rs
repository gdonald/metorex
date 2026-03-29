//! Native method implementations for the Hash class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the Hash class.
    pub(crate) fn call_hash_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "keys" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Dict(dict_rc) = receiver {
                    let dict = dict_rc.borrow();
                    let keys: Vec<Object> =
                        dict.keys().map(|k| Object::string(k.clone())).collect();
                    Ok(Some(Object::Array(Rc::new(RefCell::new(keys)))))
                } else {
                    Ok(None)
                }
            }
            "values" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Dict(dict_rc) = receiver {
                    let dict = dict_rc.borrow();
                    let values: Vec<Object> = dict.values().cloned().collect();
                    Ok(Some(Object::Array(Rc::new(RefCell::new(values)))))
                } else {
                    Ok(None)
                }
            }
            "has_key?" | "key?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Dict(dict_rc) = receiver {
                    let key_obj = &arguments[0];
                    // Convert key object to string representation (same as used for dict keys)
                    let key_str = match key_obj {
                        Object::String(s) => s.as_str().to_string(),
                        Object::Int(i) => i.to_string(),
                        Object::Float(f) => f.to_string(),
                        Object::Bool(b) => b.to_string(),
                        Object::Nil => "nil".to_string(),
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String, Integer, Float, Bool, or Nil",
                                key_obj,
                                position,
                            ));
                        }
                    };
                    let dict = dict_rc.borrow();
                    Ok(Some(Object::Bool(dict.contains_key(&key_str))))
                } else {
                    Ok(None)
                }
            }
            "entries" | "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Dict(dict_rc) = receiver {
                    let dict = dict_rc.borrow();
                    let entries: Vec<Object> = dict
                        .iter()
                        .map(|(k, v)| {
                            Object::Array(Rc::new(RefCell::new(vec![
                                Object::string(k.clone()),
                                v.clone(),
                            ])))
                        })
                        .collect();
                    Ok(Some(Object::Array(Rc::new(RefCell::new(entries)))))
                } else {
                    Ok(None)
                }
            }
            "length" | "size" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Dict(dict_rc) = receiver {
                    Ok(Some(Object::Int(dict_rc.borrow().len() as i64)))
                } else {
                    Ok(None)
                }
            }
            "[]" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(self.evaluate_index_operation(
                    receiver.clone(),
                    arguments[0].clone(),
                    position,
                )?))
            }
            "get" | "fetch" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "Method '{}' expected 1-2 argument(s) but received {}",
                            method_name,
                            arguments.len()
                        ),
                        position_to_location(position),
                    ));
                }
                if let Object::Dict(dict_rc) = receiver {
                    let key_str =
                        crate::vm::utils::object_to_dict_key(&arguments[0]).ok_or_else(|| {
                            method_argument_type_error(
                                method_name,
                                "String, Integer, Float, Bool, or Nil",
                                &arguments[0],
                                position,
                            )
                        })?;
                    let dict = dict_rc.borrow();
                    match dict.get(&key_str) {
                        Some(value) => Ok(Some(value.clone())),
                        None => {
                            if arguments.len() == 2 {
                                // Return the default value
                                Ok(Some(arguments[1].clone()))
                            } else if method_name == "fetch" {
                                // fetch without default raises an error
                                Err(MetorexError::runtime_error(
                                    format!("Key '{}' not found in hash", key_str),
                                    position_to_location(position),
                                ))
                            } else {
                                // get without default returns nil
                                Ok(Some(Object::Nil))
                            }
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            "merge" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Dict(dict_rc) = receiver {
                    let other = match &arguments[0] {
                        Object::Dict(other_rc) => other_rc,
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "Hash",
                                &arguments[0],
                                position,
                            ));
                        }
                    };
                    let mut merged = dict_rc.borrow().clone();
                    for (k, v) in other.borrow().iter() {
                        merged.insert(k.clone(), v.clone());
                    }
                    Ok(Some(Object::Dict(Rc::new(RefCell::new(merged)))))
                } else {
                    Ok(None)
                }
            }
            "each" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            "each requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                if let Object::Dict(dict_rc) = receiver {
                    let entries: Vec<(String, Object)> = dict_rc
                        .borrow()
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    for (key, value) in entries {
                        let args = vec![Object::string(key), value];
                        match self.execute_block_with_control_flow(&block, args)? {
                            super::super::ControlFlow::Next
                            | super::super::ControlFlow::Continue { .. } => {
                                continue;
                            }
                            super::super::ControlFlow::Break { .. } => break,
                            super::super::ControlFlow::Return { value: _, position } => {
                                return Err(super::super::errors::loop_control_error(
                                    "return", position,
                                ));
                            }
                            super::super::ControlFlow::Exception {
                                exception,
                                position,
                            } => {
                                return Err(MetorexError::runtime_error(
                                    format!(
                                        "Uncaught exception: {}",
                                        super::super::utils::format_exception(&exception)
                                    ),
                                    super::super::utils::position_to_location(position),
                                ));
                            }
                        }
                    }
                    Ok(Some(receiver.clone()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
