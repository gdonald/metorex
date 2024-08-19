//! Expression evaluation functions for the Metorex VM.
//!
//! This module contains the core logic for evaluating expressions including:
//! - Interpolated strings
//! - Array literals
//! - Dictionary literals
//! - Index operations (array/dictionary access)

use crate::ast::{Expression, InterpolationPart};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::core::VirtualMachine;
use super::errors::{index_out_of_bounds_error, undefined_dictionary_key_error};
use super::utils::{object_to_dict_key, position_to_location};

impl VirtualMachine {
    /// Evaluate string interpolation parts into a single owned string.
    pub(crate) fn evaluate_interpolated_string(
        &mut self,
        parts: &[InterpolationPart],
    ) -> Result<String, MetorexError> {
        let mut buffer = String::new();

        for part in parts {
            match part {
                InterpolationPart::Text(text) => buffer.push_str(text),
                InterpolationPart::Expression(expr) => {
                    let value = self.evaluate_expression(expr)?;
                    buffer.push_str(&value.to_string());
                }
            }
        }

        Ok(buffer)
    }

    /// Evaluate array literal expressions.
    pub(crate) fn evaluate_array_literal(
        &mut self,
        elements: &[Expression],
    ) -> Result<Object, MetorexError> {
        let mut evaluated = Vec::with_capacity(elements.len());
        for element in elements {
            evaluated.push(self.evaluate_expression(element)?);
        }
        Ok(Object::Array(Rc::new(RefCell::new(evaluated))))
    }

    /// Evaluate dictionary literal expressions.
    pub(crate) fn evaluate_dictionary_literal(
        &mut self,
        entries: &[(Expression, Expression)],
    ) -> Result<Object, MetorexError> {
        let mut map = HashMap::with_capacity(entries.len());

        for (key_expr, value_expr) in entries {
            let key_value = self.evaluate_expression(key_expr)?;
            let key_string = object_to_dict_key(&key_value).ok_or_else(|| {
                MetorexError::type_error(
                    format!(
                        "Dictionary keys must be String, Symbol, Integer, Float, Bool, or Nil, found {}",
                        key_value.type_name()
                    ),
                    position_to_location(key_expr.position()),
                )
            })?;

            let value = self.evaluate_expression(value_expr)?;
            map.insert(key_string, value);
        }

        Ok(Object::Dict(Rc::new(RefCell::new(map))))
    }

    /// Evaluate indexing operations on arrays and dictionaries.
    pub(crate) fn evaluate_index_operation(
        &self,
        collection: Object,
        key: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match collection {
            Object::Array(elements_rc) => match key {
                Object::Int(index) => {
                    let elements = elements_rc.borrow();
                    if index < 0 || (index as usize) >= elements.len() {
                        Err(index_out_of_bounds_error(index, elements.len(), position))
                    } else {
                        Ok(elements[index as usize].clone())
                    }
                }
                _ => Err(MetorexError::type_error(
                    format!("Array index must be an Integer, found {}", key.type_name()),
                    position_to_location(position),
                )),
            },
            Object::Dict(dict_rc) => {
                let key_string = object_to_dict_key(&key).ok_or_else(|| {
                    MetorexError::type_error(
                        format!(
                            "Dictionary index must be String, Symbol, Integer, Float, Bool, or Nil, found {}",
                            key.type_name()
                        ),
                        position_to_location(position),
                    )
                })?;

                let dict = dict_rc.borrow();
                dict.get(&key_string)
                    .cloned()
                    .ok_or_else(|| undefined_dictionary_key_error(&key_string, position))
            }

            other => Err(MetorexError::type_error(
                format!("Cannot index into type '{}'", other.type_name()),
                position_to_location(position),
            )),
        }
    }
}
