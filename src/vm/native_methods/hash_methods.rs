//! Native method implementations for the Hash class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::rc::Rc;

/// Sentinel key for storing the default proc on hashes created with Hash.new { ... }
const DEFAULT_PROC_KEY: &str = "__MX_DEFAULT_PROC__";
/// Sentinel key for storing original non-primitive key objects
const KEY_OBJECTS_KEY: &str = "__MX_KEY_OBJECTS__";

/// Check if a key is an internal sentinel key
fn is_internal_key(key: &str) -> bool {
    key == DEFAULT_PROC_KEY || key == "__MX_KWARGS__" || key == KEY_OBJECTS_KEY
}

/// Reconstruct the original key Object from its string key, using the sentinel
/// __MX_KEY_OBJECTS__ sub-map if present for non-primitive keys.
fn reconstruct_key(dict: &std::collections::HashMap<String, Object>, key_str: &str) -> Object {
    if let Some(Object::Dict(key_objs)) = dict.get(KEY_OBJECTS_KEY)
        && let Some(obj) = key_objs.borrow().get(key_str)
    {
        return obj.clone();
    }
    crate::vm::utils::dict_key_to_object(key_str)
}

impl VirtualMachine {
    /// Execute native methods for the Hash class.
    pub(crate) fn call_hash_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Dict(dict_rc) = receiver else {
            return Ok(None);
        };
        match method_name {
            // No-op stub: metorex doesn't distinguish identity from equality
            // for hash keys, so `compare_by_identity` just returns the receiver.
            "compare_by_identity" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(receiver.clone()))
            }
            "compare_by_identity?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(false)))
            }
            "keys" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let dict = dict_rc.borrow();
                let keys: Vec<Object> = dict
                    .keys()
                    .filter(|k| !is_internal_key(k))
                    .map(|k| reconstruct_key(&dict, k))
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(keys)))))
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
                let dict = dict_rc.borrow();
                let values: Vec<Object> = dict
                    .iter()
                    .filter(|(k, _)| !is_internal_key(k))
                    .map(|(_, v)| v.clone())
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(values)))))
            }
            "has_key?" | "key?" | "include?" | "member?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key_obj = &arguments[0];
                let key_str = crate::vm::utils::object_to_dict_key(key_obj).unwrap_or_default();
                let dict = dict_rc.borrow();
                Ok(Some(Object::Bool(dict.contains_key(&key_str))))
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
                let dict = dict_rc.borrow();
                let entries: Vec<Object> = dict
                    .iter()
                    .filter(|(k, _)| !is_internal_key(k))
                    .map(|(k, v)| {
                        Object::Array(Rc::new(RefCell::new(vec![
                            reconstruct_key(&dict, k),
                            v.clone(),
                        ])))
                    })
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(entries)))))
            }
            "delete" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let key_str =
                    crate::vm::utils::object_to_dict_key(&arguments[0]).unwrap_or_default();
                let mut dict = dict_rc.borrow_mut();
                let removed = dict.remove(&key_str).unwrap_or(Object::Nil);
                // Also remove from key objects sentinel if present
                if let Some(Object::Dict(key_objs)) = dict.get(KEY_OBJECTS_KEY) {
                    key_objs.borrow_mut().remove(&key_str);
                }
                Ok(Some(removed))
            }
            "default" => {
                // Return nil (we don't track custom defaults yet)
                Ok(Some(Object::Nil))
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
                let dict = dict_rc.borrow();
                let count = dict.keys().filter(|k| !is_internal_key(k)).count();
                Ok(Some(Object::Int(count as i64)))
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
                            Ok(Some(arguments[1].clone()))
                        } else if method_name == "fetch" {
                            Err(MetorexError::runtime_error(
                                format!("Key '{}' not found in hash", key_str),
                                position_to_location(position),
                            ))
                        } else {
                            Ok(Some(Object::Nil))
                        }
                    }
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
                let dict = dict_rc.borrow();
                let entries: Vec<(Object, Object)> = dict
                    .iter()
                    .filter(|(k, _)| !is_internal_key(k))
                    .map(|(k, v)| (reconstruct_key(&dict, k), v.clone()))
                    .collect();
                drop(dict);
                for (key, value) in entries {
                    let args = vec![key, value];
                    match self.execute_block_with_control_flow(&block, args)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        super::super::ControlFlow::Break { .. } => break,
                        super::super::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: super::super::utils::position_to_location(position),
                            });
                        }
                        super::super::ControlFlow::Exception {
                            exception,
                            position,
                        } => {
                            return Err(MetorexError::UncaughtException {
                                exception: exception.clone(),
                                location: super::super::utils::position_to_location(position),
                                message: super::super::utils::format_exception(&exception),
                            });
                        }
                    }
                }
                Ok(Some(receiver.clone()))
            }
            _ => Ok(None),
        }
    }
}
