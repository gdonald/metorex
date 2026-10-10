// Running a `case` statement, whether it matches on values or on
// patterns.

use super::*;

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
        self.warn_duplicated_when_clauses(cases, position)?;

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

        // No branch matched and there is no `else`. Ruby's `case/when`
        // evaluates to nil; only `case/in` raises.
        let _ = match_value;
        Ok(ControlFlow::Value(Object::Nil))
    }

    /// Ruby names a `when` clause written twice with the same literal, since
    /// the second one can never be reached.
    pub(crate) fn warn_duplicated_when_clauses(
        &mut self,
        cases: &[crate::ast::MatchCase],
        position: Position,
    ) -> Result<(), MetorexError> {
        let mut seen: Vec<(String, usize)> = Vec::new();
        for case in cases {
            let Some(spelling) = literal_pattern_spelling(&case.pattern) else {
                continue;
            };
            let line = case.position.line;
            if let Some((_, first)) = seen.iter().find(|(held, _)| *held == spelling) {
                let message = format!(
                    "warning: 'when' clause on line {} duplicates 'when' clause on line {} and is ignored\n",
                    line, first
                );
                self.warn_through_warning_module(message, position)?;
                continue;
            }
            seen.push((spelling, line));
        }
        Ok(())
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
        // The value is asked for its elements once for the whole `case`.
        self.deconstructed_values.push((match_value.clone(), None));
        let held = self.run_in_clauses(cases, &match_value, position);
        self.deconstructed_values.pop();
        held
    }

    /// Try each `in` clause against the value in turn.
    pub(crate) fn run_in_clauses(
        &mut self,
        cases: &[crate::ast::MatchCase],
        match_value: &Object,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        let match_value = match_value.clone();
        for case in cases {
            let mut bindings: HashMap<String, Object> = HashMap::new();
            self.pattern_failure = None;
            if self.match_pattern(&case.pattern, &match_value, &mut bindings, position)? {
                // A pattern's names are locals of the scope the `case` was
                // written in, so the guard and the body read them there and
                // they are still readable once the `case` is over.
                self.apply_pattern_bindings(&bindings);
                if !self.evaluate_guard(case.guard.as_ref())? {
                    self.note_pattern_failure(PatternFailure::Detail(
                        "guard clause does not return true".to_string(),
                    ));
                    continue;
                }

                let result = (|| -> Result<ControlFlow, MetorexError> {
                    let mut last_value = Object::Nil;
                    for statement in &case.body {
                        if let Statement::Expression { expression, .. } = statement {
                            last_value = self.evaluate_expression(expression)?;
                            continue;
                        }
                        // A statement that answers a value, such as an
                        // assignment, goes on to the next one.
                        match self.execute_statement(statement)? {
                            ControlFlow::Next => {}
                            ControlFlow::Value(value) => last_value = value,
                            flow => {
                                return Ok(flow);
                            }
                        }
                    }

                    Ok(ControlFlow::Value(last_value))
                })();

                return result;
            }
            // A name the pattern reached before it failed is still a local of
            // the scope, holding nil, the way Ruby leaves one behind.
            self.apply_pattern_bindings(&bindings);
        }

        // A `case` with one `in` clause and no `else` has nowhere to go when
        // the clause fails, so Ruby reports what went wrong rather than the
        // value alone.
        if cases.len() == 1 {
            return Err(self.detailed_no_matching_pattern(&match_value, position));
        }
        Err(self.no_matching_pattern(&match_value, position))
    }

    /// Evaluate a guard expression with pattern bindings in a new scope.
    /// Returns true if the guard passes (is truthy) or if there's no guard.
    /// Returns false if the guard fails (is not truthy).
    /// Helper method to reduce code duplication between match statement and case expression.
    pub(crate) fn evaluate_guard_with_bindings(
        &mut self,
        guard_expr: Option<&Expression>,
        bindings: &HashMap<String, Object>,
    ) -> Result<bool, MetorexError> {
        if let Some(guard) = guard_expr {
            // Push a new scope for guard evaluation with bindings
            self.environment_mut().push_scope();
            self.apply_pattern_bindings(bindings);

            let guard_result = self.evaluate_expression(guard);
            self.environment_mut().pop_scope();

            // Return whether the guard is truthy
            Ok(is_truthy(&guard_result?))
        } else {
            // No guard means it always passes
            Ok(true)
        }
    }

    /// Whether an `in` clause's guard lets the match stand. The pattern's
    /// names are already locals of the scope, so the guard reads them there.
    pub(crate) fn evaluate_guard(
        &mut self,
        guard_expr: Option<&Expression>,
    ) -> Result<bool, MetorexError> {
        let Some(guard) = guard_expr else {
            return Ok(true);
        };
        let held = self.evaluate_expression(guard)?;
        Ok(is_truthy(&held))
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

                let result = self.evaluate_expression(&case.body);
                self.environment_mut().pop_scope();

                return result;
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

/// How a `when` pattern is written, for the literals that can be compared as
/// text. Anything else answers None, since two expressions that happen to
/// name the same value are not a duplicate as written.
pub(crate) fn literal_pattern_spelling(pattern: &crate::ast::MatchPattern) -> Option<String> {
    use crate::ast::{Expression, MatchPattern};
    match pattern {
        MatchPattern::IntLiteral(held) => Some(format!("i{held}")),
        MatchPattern::StringLiteral(held) => Some(format!("s{held}")),
        MatchPattern::SymbolLiteral(held) => Some(format!("y{held}")),
        MatchPattern::BoolLiteral(held) => Some(format!("b{held}")),
        MatchPattern::NilLiteral => Some("nil".to_string()),
        MatchPattern::Expression(held) => match held.as_ref() {
            Expression::IntLiteral { value, .. } => Some(format!("i{value}")),
            Expression::StringLiteral { value, .. } => Some(format!("s{value}")),
            Expression::Symbol { value, .. } => Some(format!("y{value}")),
            Expression::BoolLiteral { value, .. } => Some(format!("b{value}")),
            _ => None,
        },
        _ => None,
    }
}

/// Whether two names stand for the very same object, which is what tells the
/// value a `case` is matching from one nested inside a pattern.
pub(crate) fn names_the_same_value(left: &Object, right: &Object) -> bool {
    match (left, right) {
        (Object::Array(one), Object::Array(other)) => Rc::ptr_eq(one, other),
        (Object::Dict(one), Object::Dict(other)) => Rc::ptr_eq(one, other),
        (Object::Instance(one), Object::Instance(other)) => Rc::ptr_eq(one, other),
        (Object::String(one), Object::String(other)) => Rc::ptr_eq(one, other),
        _ => left == right,
    }
}
