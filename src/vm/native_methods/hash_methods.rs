//! Native method implementations for the Hash class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
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
            _ => Ok(None),
        }
    }
}
