//! Native (built-in) method implementations for the virtual machine.
//!
//! This module contains the implementations of all built-in methods for
//! standard classes like Object, String, and Array.

use super::VirtualMachine;
use super::errors::*;
use super::utils::position_to_location;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::cell::RefCell;
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
        if let Object::Block(block) = receiver
            && method_name == "call"
        {
            return Ok(Some(block.call(self, arguments.to_vec(), position)?));
        }

        match class.name() {
            "Object" => match method_name {
                "to_s" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    Ok(Some(Object::string(receiver.to_string())))
                }
                "class" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    Ok(Some(Object::Class(self.builtins().class_of(receiver))))
                }
                "respond_to?" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let method_query = match &arguments[0] {
                        Object::String(name) => name.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                other,
                                position,
                            ));
                        }
                    };
                    Ok(Some(Object::Bool(
                        self.lookup_method(receiver, &method_query).is_some(),
                    )))
                }
                _ => Ok(None),
            },
            "String" => match method_name {
                "length" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        Ok(Some(Object::Int(string_value.chars().count() as i64)))
                    } else {
                        Ok(None)
                    }
                }
                "upcase" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        Ok(Some(Object::string(string_value.to_uppercase())))
                    } else {
                        Ok(None)
                    }
                }
                "downcase" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        Ok(Some(Object::string(string_value.to_lowercase())))
                    } else {
                        Ok(None)
                    }
                }
                "+" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let (Object::String(lhs), Object::String(rhs)) = (receiver, &arguments[0]) {
                        let mut combined = lhs.as_ref().clone();
                        combined.push_str(rhs);
                        Ok(Some(Object::string(combined)))
                    } else {
                        Err(method_argument_type_error(
                            method_name,
                            "String",
                            &arguments[0],
                            position,
                        ))
                    }
                }
                "trim" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        Ok(Some(Object::string(string_value.trim().to_string())))
                    } else {
                        Ok(None)
                    }
                }
                "reverse" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        let reversed: String = string_value.chars().rev().collect();
                        Ok(Some(Object::string(reversed)))
                    } else {
                        Ok(None)
                    }
                }
                "chars" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        let chars: Vec<Object> = string_value
                            .chars()
                            .map(|c| Object::string(c.to_string()))
                            .collect();
                        Ok(Some(Object::Array(Rc::new(RefCell::new(chars)))))
                    } else {
                        Ok(None)
                    }
                }
                "bytes" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        let bytes: Vec<Object> = string_value
                            .bytes()
                            .map(|b| Object::Int(b as i64))
                            .collect();
                        Ok(Some(Object::Array(Rc::new(RefCell::new(bytes)))))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            },
            "Array" => match method_name {
                "length" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Array(array_rc) = receiver {
                        Ok(Some(Object::Int(array_rc.borrow().len() as i64)))
                    } else {
                        Ok(None)
                    }
                }
                "push" | "append" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Array(array_rc) = receiver {
                        array_rc.borrow_mut().push(arguments[0].clone());
                        Ok(Some(receiver.clone()))
                    } else {
                        Ok(None)
                    }
                }
                "pop" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Array(array_rc) = receiver {
                        Ok(Some(array_rc.borrow_mut().pop().unwrap_or(Object::Nil)))
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
                "each" => {
                    // each takes a block parameter
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Array(array_rc) = receiver {
                        let block = match &arguments[0] {
                            Object::Block(block) => block.clone(),
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "Block",
                                    &arguments[0],
                                    position,
                                ));
                            }
                        };

                        let array = array_rc.borrow();
                        for element in array.iter() {
                            let args = vec![element.clone()];
                            match self.execute_block_with_control_flow(&block, args)? {
                                super::ControlFlow::Next | super::ControlFlow::Continue { .. } => {
                                    continue;
                                }
                                super::ControlFlow::Break { .. } => break,
                                super::ControlFlow::Return { value: _, position } => {
                                    return Err(super::errors::loop_control_error(
                                        "return", position,
                                    ));
                                }
                                super::ControlFlow::Exception {
                                    exception,
                                    position,
                                } => {
                                    return Err(MetorexError::runtime_error(
                                        format!(
                                            "Uncaught exception: {}",
                                            super::utils::format_exception(&exception)
                                        ),
                                        super::utils::position_to_location(position),
                                    ));
                                }
                            }
                        }
                        Ok(Some(receiver.clone()))
                    } else {
                        Ok(None)
                    }
                }
                "map" => {
                    // map takes a block parameter
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Array(array_rc) = receiver {
                        let block = match &arguments[0] {
                            Object::Block(block) => block.clone(),
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "Block",
                                    &arguments[0],
                                    position,
                                ));
                            }
                        };

                        let array = array_rc.borrow();
                        let mut results = Vec::new();
                        for element in array.iter() {
                            let args = vec![element.clone()];
                            let value = self.execute_block_body(&block, args)?;
                            results.push(value);
                        }
                        Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            },
            "Hash" => match method_name {
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
                "has_key?" => {
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
            },
            "Float" => match method_name {
                "round" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Float(float_value) = receiver {
                        let precision = match &arguments[0] {
                            Object::Int(p) => *p,
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "Integer",
                                    &arguments[0],
                                    position,
                                ));
                            }
                        };

                        if precision < 0 {
                            return Err(MetorexError::runtime_error(
                                format!(
                                    "Float.round precision must be non-negative, got {}",
                                    precision
                                ),
                                position_to_location(position),
                            ));
                        }

                        // Round to the specified number of decimal places
                        let multiplier = 10_f64.powi(precision as i32);
                        let rounded = (float_value * multiplier).round() / multiplier;
                        Ok(Some(Object::Float(rounded)))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            },
            "Range" => match method_name {
                "each" => {
                    // each takes a block parameter
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Range {
                        start,
                        end,
                        exclusive,
                    } = receiver
                    {
                        let block = match &arguments[0] {
                            Object::Block(block) => block.clone(),
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "Block",
                                    &arguments[0],
                                    position,
                                ));
                            }
                        };

                        // Only support integer ranges for now
                        match (start.as_ref(), end.as_ref()) {
                            (Object::Int(start_val), Object::Int(end_val)) => {
                                let end_inclusive =
                                    if *exclusive { *end_val - 1 } else { *end_val };

                                for i in *start_val..=end_inclusive {
                                    let args = vec![Object::Int(i)];
                                    match self.execute_block_with_control_flow(&block, args)? {
                                        super::ControlFlow::Next
                                        | super::ControlFlow::Continue { .. } => continue,
                                        super::ControlFlow::Break { .. } => break,
                                        super::ControlFlow::Return { value: _, position } => {
                                            return Err(super::errors::loop_control_error(
                                                "return", position,
                                            ));
                                        }
                                        super::ControlFlow::Exception {
                                            exception,
                                            position,
                                        } => {
                                            return Err(MetorexError::runtime_error(
                                                format!(
                                                    "Uncaught exception: {}",
                                                    super::utils::format_exception(&exception)
                                                ),
                                                super::utils::position_to_location(position),
                                            ));
                                        }
                                    }
                                }
                                Ok(Some(receiver.clone()))
                            }
                            _ => Err(MetorexError::runtime_error(
                                "Range.each only supports integer ranges".to_string(),
                                position_to_location(position),
                            )),
                        }
                    } else {
                        Ok(None)
                    }
                }
                "to_a" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Range {
                        start,
                        end,
                        exclusive,
                    } = receiver
                    {
                        match (start.as_ref(), end.as_ref()) {
                            (Object::Int(start_val), Object::Int(end_val)) => {
                                let end_inclusive =
                                    if *exclusive { *end_val - 1 } else { *end_val };

                                let elements: Vec<Object> =
                                    (*start_val..=end_inclusive).map(Object::Int).collect();
                                Ok(Some(Object::Array(Rc::new(RefCell::new(elements)))))
                            }
                            _ => Err(MetorexError::runtime_error(
                                "Range.to_a only supports integer ranges".to_string(),
                                position_to_location(position),
                            )),
                        }
                    } else {
                        Ok(None)
                    }
                }
                "include?" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Range {
                        start,
                        end,
                        exclusive,
                    } = receiver
                    {
                        match (start.as_ref(), end.as_ref(), &arguments[0]) {
                            (
                                Object::Int(start_val),
                                Object::Int(end_val),
                                Object::Int(test_val),
                            ) => {
                                let in_range = if *exclusive {
                                    *test_val >= *start_val && *test_val < *end_val
                                } else {
                                    *test_val >= *start_val && *test_val <= *end_val
                                };
                                Ok(Some(Object::Bool(in_range)))
                            }
                            _ => Err(MetorexError::runtime_error(
                                "Range.include? only supports integer ranges".to_string(),
                                position_to_location(position),
                            )),
                        }
                    } else {
                        Ok(None)
                    }
                }
                "map" => {
                    // map takes a block parameter
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Range {
                        start,
                        end,
                        exclusive,
                    } = receiver
                    {
                        let block = match &arguments[0] {
                            Object::Block(block) => block.clone(),
                            _ => {
                                return Err(method_argument_type_error(
                                    method_name,
                                    "Block",
                                    &arguments[0],
                                    position,
                                ));
                            }
                        };

                        // Only support integer ranges for now
                        match (start.as_ref(), end.as_ref()) {
                            (Object::Int(start_val), Object::Int(end_val)) => {
                                let end_inclusive =
                                    if *exclusive { *end_val - 1 } else { *end_val };

                                let mut results = Vec::new();
                                for i in *start_val..=end_inclusive {
                                    let args = vec![Object::Int(i)];
                                    let value = self.execute_block_body(&block, args)?;
                                    results.push(value);
                                }
                                Ok(Some(Object::Array(Rc::new(RefCell::new(results)))))
                            }
                            _ => Err(MetorexError::runtime_error(
                                "Range.map only supports integer ranges".to_string(),
                                position_to_location(position),
                            )),
                        }
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}
