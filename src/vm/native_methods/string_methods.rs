//! Native method implementations for the String class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use std::cell::RefCell;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the String class.
    pub(crate) fn call_string_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
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
            "each_char" => {
                // each_char takes a block parameter
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::String(string_value) = receiver {
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

                    for ch in string_value.chars() {
                        let char_str = Object::string(ch.to_string());
                        let args = vec![char_str];
                        self.execute_block_body(&block, args)?;
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
