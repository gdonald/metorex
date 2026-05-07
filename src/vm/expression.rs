//! Expression evaluation functions for the Metorex VM.
//!
//! This module contains the core logic for evaluating expressions including:
//! - Interpolated strings
//! - Array literals
//! - Dictionary literals
//! - Index operations (array/dictionary access)

use crate::ast::node::ElsifBranch;
use crate::ast::{Expression, InterpolationPart, Statement};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::core::VirtualMachine;
use super::errors::{index_out_of_bounds_error, loop_control_error};
use super::utils::{format_exception, is_truthy, object_to_dict_key, position_to_location};

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
        let mut key_objs: HashMap<String, Object> = HashMap::new();

        for (key_expr, value_expr) in entries {
            let key_value = self.evaluate_expression(key_expr)?;
            let key_string = object_to_dict_key(&key_value).unwrap_or_default();
            if !crate::vm::utils::is_primitive_key(&key_value) {
                key_objs.insert(key_string.clone(), key_value.clone());
            }

            let value = self.evaluate_expression(value_expr)?;
            map.insert(key_string, value);
        }

        if !key_objs.is_empty() {
            map.insert(
                "__MX_KEY_OBJECTS__".to_string(),
                Object::Dict(Rc::new(RefCell::new(key_objs))),
            );
        }

        Ok(Object::Dict(Rc::new(RefCell::new(map))))
    }

    /// Evaluate indexing operations on arrays and dictionaries.
    pub(crate) fn evaluate_index_operation(
        &mut self,
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
                if let Some(value) = dict.get(&key_string) {
                    Ok(value.clone())
                } else if let Some(Object::Block(default_proc)) = dict.get("__MX_DEFAULT_PROC__") {
                    // Auto-vivify: call the default block with (hash, key)
                    // The block typically does h[k] = default_value, which sets
                    // the value directly in the hash.
                    let block = default_proc.clone();
                    drop(dict);
                    let hash_obj = Object::Dict(Rc::clone(&dict_rc));
                    let block_result =
                        self.execute_block_body(&block, vec![hash_obj, key.clone()])?;
                    // The block may have set the key directly (h[k] = val), or
                    // returned a value. Check the hash first, fall back to block result.
                    let stored = dict_rc.borrow().get(&key_string).cloned();
                    if let Some(value) = stored {
                        Ok(value)
                    } else {
                        // Block didn't set the key — store the return value
                        dict_rc
                            .borrow_mut()
                            .insert(key_string, block_result.clone());
                        Ok(block_result)
                    }
                } else {
                    Ok(Object::Nil)
                }
            }

            Object::String(s) => match key {
                Object::Int(i) => {
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as i64;
                    let idx = if i < 0 { len + i } else { i };
                    if idx < 0 || idx >= len {
                        Ok(Object::Nil)
                    } else {
                        Ok(Object::String(Rc::new(chars[idx as usize].to_string())))
                    }
                }
                Object::Range {
                    start,
                    end,
                    exclusive,
                } => {
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as i64;
                    let s_idx = match start.as_ref() {
                        Object::Int(n) => {
                            let i = if *n < 0 { len + n } else { *n };
                            i.max(0) as usize
                        }
                        _ => 0,
                    };
                    let e_idx = match end.as_ref() {
                        Object::Int(n) => {
                            let i = if *n < 0 { len + n } else { *n };
                            if exclusive {
                                i.max(0) as usize
                            } else {
                                (i + 1).max(0) as usize
                            }
                        }
                        _ => chars.len(),
                    };
                    let sliced: String = chars
                        .get(s_idx..e_idx.min(chars.len()))
                        .unwrap_or(&[])
                        .iter()
                        .collect();
                    Ok(Object::String(Rc::new(sliced)))
                }
                _ => Err(MetorexError::type_error(
                    format!(
                        "String index must be Integer or Range, found {}",
                        key.type_name()
                    ),
                    position_to_location(position),
                )),
            },

            Object::Class(_) | Object::Module(_) | Object::Instance(_) => {
                // Dispatch to the class's `[]` method (for Dir[], custom [] methods, etc.)
                let class = self.builtins().class_of(&collection);
                match self.call_native_method(
                    &class,
                    &collection,
                    "[]",
                    std::slice::from_ref(&key),
                    position,
                )? {
                    Some(val) => Ok(val),
                    None => Err(MetorexError::type_error(
                        format!("Cannot index into type '{}'", collection.type_name()),
                        position_to_location(position),
                    )),
                }
            }
            other => Err(MetorexError::type_error(
                format!("Cannot index into type '{}'", other.type_name()),
                position_to_location(position),
            )),
        }
    }

    /// Evaluate an if expression, returning the value of the matching branch.
    pub(crate) fn evaluate_if_expression(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        elsif_branches: &[ElsifBranch],
        else_branch: &Option<Vec<Statement>>,
    ) -> Result<Object, MetorexError> {
        if is_truthy(&self.evaluate_expression(condition)?) {
            return self.evaluate_branch_value(then_branch);
        }
        for elsif in elsif_branches {
            if is_truthy(&self.evaluate_expression(&elsif.condition)?) {
                return self.evaluate_branch_value(&elsif.body);
            }
        }
        if let Some(else_stmts) = else_branch {
            return self.evaluate_branch_value(else_stmts);
        }
        Ok(Object::Nil)
    }

    /// Evaluate an unless expression, returning the value of the matching branch.
    pub(crate) fn evaluate_unless_expression(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: &Option<Vec<Statement>>,
    ) -> Result<Object, MetorexError> {
        if !is_truthy(&self.evaluate_expression(condition)?) {
            return self.evaluate_branch_value(then_branch);
        }
        if let Some(else_stmts) = else_branch {
            return self.evaluate_branch_value(else_stmts);
        }
        Ok(Object::Nil)
    }

    /// Execute a list of statements and return the value of the last expression statement.
    fn evaluate_branch_value(&mut self, stmts: &[Statement]) -> Result<Object, MetorexError> {
        use super::ControlFlow;
        self.environment_mut().push_scope();
        let result = (|| -> Result<Object, MetorexError> {
            let mut last_value = Object::Nil;
            for stmt in stmts {
                if let Statement::Expression { expression, .. } = stmt {
                    last_value = self.evaluate_expression(expression)?;
                    continue;
                }
                match self.execute_statement(stmt)? {
                    ControlFlow::Next => {}
                    ControlFlow::Value(v) => {
                        last_value = v;
                    }
                    ControlFlow::Return { value, position } => {
                        // Bubble out to enclosing method, not the if branch.
                        return Err(MetorexError::NonLocalReturn {
                            value,
                            location: position_to_location(position),
                        });
                    }
                    ControlFlow::Break { value, position } => {
                        // `break` inside an if/elsif/else branch must
                        // unwind to the enclosing iterator/block, not be
                        // silently swallowed by the if-as-expression
                        // wrapper. BlockBreak is the signal that
                        // execute_block_callable / each-style natives
                        // recognise.
                        return Err(MetorexError::BlockBreak {
                            value,
                            location: position_to_location(position),
                        });
                    }
                    ControlFlow::Continue { position } => {
                        return Err(loop_control_error("continue", position));
                    }
                    ControlFlow::Exception {
                        exception,
                        position,
                    } => {
                        return Err(MetorexError::UncaughtException {
                            exception: exception.clone(),
                            location: position_to_location(position),
                            message: format_exception(&exception),
                        });
                    }
                }
            }
            Ok(last_value)
        })();
        self.environment_mut().pop_scope();
        result
    }
}
