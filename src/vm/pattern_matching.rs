// Pattern matching execution for the Metorex VM.
// This module handles match statements and pattern matching logic.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::utils::*;

use crate::ast::node::ExprMatchCase;
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
                if !self.evaluate_guard_with_bindings(case.guard.as_ref(), &bindings)? {
                    continue;
                }

                // Ruby case/when branches do NOT introduce a new scope — assignments
                // leak out, matching Ruby semantics. Pattern bindings are applied in-place.
                self.apply_pattern_bindings(&bindings);

                // Execute the body and capture the last expression's value so that
                // case-as-expression produces a value without triggering a real return.
                let mut last_value = Object::Nil;
                for statement in &case.body {
                    if let Statement::Expression { expression, .. } = statement {
                        last_value = self.evaluate_expression(expression)?;
                        continue;
                    }

                    match self.execute_statement(statement)? {
                        ControlFlow::Next => {}
                        ControlFlow::Value(v) => {
                            last_value = v;
                        }
                        flow => return Ok(flow),
                    }
                }

                return Ok(ControlFlow::Value(last_value));
            }
        }

        // No pattern matched — Ruby returns nil (no error for case/when).
        Err(MetorexError::runtime_error(
            format!("No pattern matched value: {}", match_value),
            position_to_location(position),
        ))
    }

    /// Execute a `case/in` statement (Ruby 2.7+ pattern matching).
    /// Raises `NoMatchingPatternError` if no arm matches and no else clause is present.
    pub(crate) fn execute_case_in(
        &mut self,
        expression: &Expression,
        cases: &[crate::ast::MatchCase],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        let match_value = self.evaluate_expression(expression)?;

        for case in cases {
            let mut bindings: HashMap<String, Object> = HashMap::new();
            if self.match_pattern(&case.pattern, &match_value, &mut bindings, position)? {
                if !self.evaluate_guard_with_bindings(case.guard.as_ref(), &bindings)? {
                    continue;
                }

                self.environment_mut().push_scope();
                self.apply_pattern_bindings(&bindings);

                let mut last_value = Object::Nil;
                for statement in &case.body {
                    if let Statement::Expression { expression, .. } = statement {
                        last_value = self.evaluate_expression(expression)?;
                        continue;
                    }
                    match self.execute_statement(statement)? {
                        ControlFlow::Next => {}
                        flow => {
                            self.environment_mut().pop_scope();
                            return Ok(flow);
                        }
                    }
                }

                self.environment_mut().pop_scope();
                return Ok(ControlFlow::Return {
                    value: last_value,
                    position,
                });
            }
        }

        Err(MetorexError::runtime_error(
            format!(
                "NoMatchingPatternError: {} (NoMatchingPatternError)",
                match_value
            ),
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
            MatchPattern::SymbolLiteral(pattern_name) => match value {
                Object::Symbol(value_name) => Ok(pattern_name == value_name.as_ref()),
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

            // Type pattern - match based on object type
            MatchPattern::Type(type_name) => {
                let actual_type = value.type_name();

                // Support both Ruby-style names (Integer, Hash) and internal names (Int, Dict)
                let matches = match type_name.as_str() {
                    // Direct type name matches
                    name if name == actual_type => true,

                    // Ruby-style aliases
                    "Integer" => matches!(value, Object::Int(_)),
                    "Float" => matches!(value, Object::Float(_)),
                    "String" => matches!(value, Object::String(_)),
                    "Array" => matches!(value, Object::Array(_)),
                    "Hash" | "Dict" => matches!(value, Object::Dict(_)),
                    "TrueClass" | "FalseClass" | "Boolean" => matches!(value, Object::Bool(_)),
                    "NilClass" => matches!(value, Object::Nil),
                    "Class" => matches!(value, Object::Class(_)),
                    "Method" => matches!(value, Object::Method(_)),
                    "Exception" => matches!(value, Object::Exception(_)),
                    "Set" => matches!(value, Object::Set(_)),
                    "Range" => matches!(value, Object::Range { .. }),

                    // Check for class instances
                    _ => {
                        if let Object::Instance(instance_rc) = value {
                            let instance = instance_rc.borrow();
                            instance.class_name() == *type_name
                        } else {
                            false
                        }
                    }
                };

                Ok(matches)
            }

            // Multiple pattern - OR matching (matches if any sub-pattern matches)
            // Used for: when 1, 2, 3 then "small"
            MatchPattern::Multiple(patterns) => {
                for pattern in patterns {
                    // Try each pattern - if any matches, the whole Multiple pattern matches
                    // Important: we need to preserve bindings only from the matching pattern
                    let mut temp_bindings = HashMap::new();
                    if self.match_pattern(pattern, value, &mut temp_bindings, position)? {
                        // This pattern matched - merge bindings and return true
                        bindings.extend(temp_bindings);
                        return Ok(true);
                    }
                }
                // None of the patterns matched
                Ok(false)
            }

            // Bind pattern: match inner pattern and bind the whole value to a name
            // Used in case/in: `in Integer => n`
            MatchPattern::Bind { pattern, name } => {
                if self.match_pattern(pattern, value, bindings, position)? {
                    bindings.insert(name.clone(), value.clone());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            // Range pattern: matches if value falls within start..end or start...end
            MatchPattern::Range {
                start,
                end,
                exclusive,
            } => {
                // Extract numeric value for comparison
                let val_num = match value {
                    Object::Int(n) => *n as f64,
                    Object::Float(f) => *f,
                    _ => return Ok(false),
                };

                // Extract start bound
                let start_num = match start.as_ref() {
                    MatchPattern::IntLiteral(n) => *n as f64,
                    MatchPattern::FloatLiteral(f) => *f,
                    _ => return Ok(false),
                };

                // Extract end bound
                let end_num = match end.as_ref() {
                    MatchPattern::IntLiteral(n) => *n as f64,
                    MatchPattern::FloatLiteral(f) => *f,
                    _ => return Ok(false),
                };

                let in_range = if *exclusive {
                    val_num >= start_num && val_num < end_num
                } else {
                    val_num >= start_num && val_num <= end_num
                };

                Ok(in_range)
            }
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

    /// Apply variable bindings from pattern matching to the current scope.
    /// Helper method to reduce code duplication between match statement and case expression.
    fn apply_pattern_bindings(&mut self, bindings: &HashMap<String, Object>) {
        for (name, value) in bindings {
            self.environment_mut().define(name.clone(), value.clone());
        }
    }

    /// Evaluate a guard expression with pattern bindings in a new scope.
    /// Returns true if the guard passes (is truthy) or if there's no guard.
    /// Returns false if the guard fails (is not truthy).
    /// Helper method to reduce code duplication between match statement and case expression.
    fn evaluate_guard_with_bindings(
        &mut self,
        guard_expr: Option<&Expression>,
        bindings: &HashMap<String, Object>,
    ) -> Result<bool, MetorexError> {
        if let Some(guard) = guard_expr {
            // Push a new scope for guard evaluation with bindings
            self.environment_mut().push_scope();
            self.apply_pattern_bindings(bindings);

            let guard_result = self.evaluate_expression(guard)?;
            self.environment_mut().pop_scope();

            // Return whether the guard is truthy
            Ok(is_truthy(&guard_result))
        } else {
            // No guard means it always passes
            Ok(true)
        }
    }

    /// Evaluate a case expression (pattern matching in expression context).
    /// Returns the value of the first matching case's body expression.
    pub(crate) fn evaluate_case_expression(
        &mut self,
        expression: &Expression,
        cases: &[ExprMatchCase],
        else_case: Option<&Expression>,
        _position: Position,
    ) -> Result<Object, MetorexError> {
        // Evaluate the value to match against
        let match_value = self.evaluate_expression(expression)?;

        // Try each case in order
        for case in cases {
            // Try to match the pattern
            let mut bindings: HashMap<String, Object> = HashMap::new();
            if self.match_pattern(&case.pattern, &match_value, &mut bindings, case.position)? {
                // Pattern matched! Now check guard if present
                if !self.evaluate_guard_with_bindings(case.guard.as_ref(), &bindings)? {
                    continue;
                }

                // Pattern and guard matched! Evaluate the body expression with bindings
                self.environment_mut().push_scope();
                self.apply_pattern_bindings(&bindings);

                let result = self.evaluate_expression(&case.body)?;
                self.environment_mut().pop_scope();

                return Ok(result);
            }
        }

        // No pattern matched - evaluate else case if present, otherwise return nil
        if let Some(else_expr) = else_case {
            self.evaluate_expression(else_expr)
        } else {
            Ok(Object::Nil)
        }
    }
}
