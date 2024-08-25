// Pattern matching execution for the Metorex VM.
// This module handles match statements and pattern matching logic.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::utils::*;

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute a match statement for pattern matching.
    pub(crate) fn execute_match(
        &mut self,
        expression: &Expression,
        cases: &[crate::ast::MatchCase],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // Evaluate the value to match against
        let match_value = self.evaluate_expression(expression)?;

        // Try each case in order
        for case in cases {
            // Try to match the pattern
            let mut bindings: HashMap<String, Object> = HashMap::new();
            if self.match_pattern(&case.pattern, &match_value, &mut bindings, position)? {
                // Pattern matched! Now check guard if present
                if let Some(guard_expr) = &case.guard {
                    // Push a new scope for guard evaluation with bindings
                    self.environment_mut().push_scope();
                    for (name, value) in &bindings {
                        self.environment_mut().define(name.clone(), value.clone());
                    }

                    let guard_result = self.evaluate_expression(guard_expr)?;
                    self.environment_mut().pop_scope();

                    // If guard evaluates to false, skip this case
                    if !is_truthy(&guard_result) {
                        continue;
                    }
                }

                // Pattern and guard matched! Execute the body with bindings
                self.environment_mut().push_scope();
                for (name, value) in bindings {
                    self.environment_mut().define(name, value);
                }

                // Execute the body and track the last expression value
                let mut last_value = Object::Nil;
                for statement in &case.body {
                    // If it's an expression statement, track its value
                    if let Statement::Expression { expression, .. } = statement {
                        last_value = self.evaluate_expression(expression)?;
                        continue;
                    }

                    // Execute other statements
                    match self.execute_statement(statement)? {
                        ControlFlow::Next => {}
                        flow => {
                            self.environment_mut().pop_scope();
                            return Ok(flow);
                        }
                    }
                }

                self.environment_mut().pop_scope();

                // Return the last expression value using Return control flow
                // (but this is not a true return statement, just a value)
                return Ok(ControlFlow::Return {
                    value: last_value,
                    position,
                });
            }
        }

        // No pattern matched
        Err(MetorexError::runtime_error(
            format!("No pattern matched value: {}", match_value),
            position_to_location(position),
        ))
    }

    /// Match a pattern against a value and collect variable bindings.
    /// Returns true if the pattern matches, false otherwise.
    pub(crate) fn match_pattern(
        &self,
        pattern: &crate::ast::MatchPattern,
        value: &Object,
        bindings: &mut HashMap<String, Object>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        use crate::ast::MatchPattern;

        match pattern {
            // Literal patterns - exact equality match
            MatchPattern::IntLiteral(pattern_int) => match value {
                Object::Int(value_int) => Ok(pattern_int == value_int),
                _ => Ok(false),
            },
            MatchPattern::FloatLiteral(pattern_float) => match value {
                Object::Float(value_float) => {
                    // Use approximate equality for floats
                    Ok((pattern_float - value_float).abs() < f64::EPSILON)
                }
                _ => Ok(false),
            },
            MatchPattern::StringLiteral(pattern_string) => match value {
                Object::String(value_string) => Ok(pattern_string == value_string.as_ref()),
                _ => Ok(false),
            },
            MatchPattern::BoolLiteral(pattern_bool) => match value {
                Object::Bool(value_bool) => Ok(pattern_bool == value_bool),
                _ => Ok(false),
            },
            MatchPattern::NilLiteral => Ok(matches!(value, Object::Nil)),

            // Identifier pattern - binds the value to a variable
            MatchPattern::Identifier(name) => {
                bindings.insert(name.clone(), value.clone());
                Ok(true)
            }

            // Wildcard pattern - matches anything without binding
            MatchPattern::Wildcard => Ok(true),

            // Array pattern - destructure arrays
            MatchPattern::Array(patterns) => match value {
                Object::Array(array_rc) => {
                    let array = array_rc.borrow();
                    self.match_array_pattern(patterns, &array, bindings, position)
                }
                _ => Ok(false),
            },

            // Rest pattern - should only appear inside array patterns
            MatchPattern::Rest(_) => Err(MetorexError::runtime_error(
                "Rest pattern (...) can only be used inside array patterns".to_string(),
                position_to_location(position),
            )),

            // Object pattern - destructure dictionaries
            MatchPattern::Object(key_patterns) => match value {
                Object::Dict(dict_rc) => {
                    let dict = dict_rc.borrow();
                    self.match_object_pattern(key_patterns, &dict, bindings, position)
                }
                _ => Ok(false),
            },

            // Type pattern - not yet implemented
            MatchPattern::Type(_type_name) => Err(MetorexError::runtime_error(
                "Type patterns are not yet implemented".to_string(),
                position_to_location(position),
            )),
        }
    }

    /// Match an array pattern against an array value.
    pub(crate) fn match_array_pattern(
        &self,
        patterns: &[crate::ast::MatchPattern],
        array: &[Object],
        bindings: &mut HashMap<String, Object>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        use crate::ast::MatchPattern;

        // Find if there's a rest pattern and where
        let mut rest_index = None;
        for (i, pattern) in patterns.iter().enumerate() {
            if matches!(pattern, MatchPattern::Rest(_)) {
                if rest_index.is_some() {
                    return Err(MetorexError::runtime_error(
                        "Only one rest pattern (...) is allowed per array pattern".to_string(),
                        position_to_location(position),
                    ));
                }
                rest_index = Some(i);
            }
        }

        if let Some(rest_idx) = rest_index {
            // Array pattern with rest
            let patterns_before = &patterns[..rest_idx];
            let patterns_after = &patterns[rest_idx + 1..];
            let min_length = patterns_before.len() + patterns_after.len();

            // Array must have at least min_length elements
            if array.len() < min_length {
                return Ok(false);
            }

            // Match patterns before rest
            for (i, pattern) in patterns_before.iter().enumerate() {
                if !self.match_pattern(pattern, &array[i], bindings, position)? {
                    return Ok(false);
                }
            }

            // Match patterns after rest
            let rest_start = rest_idx;
            let rest_end = array.len() - patterns_after.len();
            for (i, pattern) in patterns_after.iter().enumerate() {
                if !self.match_pattern(pattern, &array[rest_end + i], bindings, position)? {
                    return Ok(false);
                }
            }

            // Bind rest elements
            if let MatchPattern::Rest(rest_name) = &patterns[rest_idx] {
                let rest_elements: Vec<Object> = array[rest_start..rest_end].to_vec();
                bindings.insert(
                    rest_name.clone(),
                    Object::Array(Rc::new(RefCell::new(rest_elements))),
                );
            }

            Ok(true)
        } else {
            // Array pattern without rest - exact length match required
            if patterns.len() != array.len() {
                return Ok(false);
            }

            // Match each pattern against corresponding element
            for (pattern, element) in patterns.iter().zip(array.iter()) {
                if !self.match_pattern(pattern, element, bindings, position)? {
                    return Ok(false);
                }
            }

            Ok(true)
        }
    }

    /// Match an object/dictionary pattern against a dictionary value.
    pub(crate) fn match_object_pattern(
        &self,
        key_patterns: &[(String, crate::ast::MatchPattern)],
        dict: &HashMap<String, Object>,
        bindings: &mut HashMap<String, Object>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        // Each key must exist in the dictionary and match its pattern
        for (key, pattern) in key_patterns {
            match dict.get(key) {
                Some(value) => {
                    if !self.match_pattern(pattern, value, bindings, position)? {
                        return Ok(false);
                    }
                }
                None => return Ok(false), // Key not found in dictionary
            }
        }

        Ok(true)
    }
}
