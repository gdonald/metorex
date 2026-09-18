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
use crate::vm::MATCHEE_KEY;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Why a pattern did not cover a value. A `case` holding a single `in`
/// clause names this in the error it raises, the way Ruby does.
#[derive(Debug, Clone)]
pub(crate) enum PatternFailure {
    /// The wording that goes after the value in the message.
    Detail(String),
    /// A hash pattern named a key the hash does not hold, which Ruby reports
    /// as `NoMatchingPatternKeyError`.
    MissingKey { matchee: Object, key: String },
}

impl VirtualMachine {
    /// Record why the pattern being tried failed. The innermost pattern fails
    /// first, and its reason is the one Ruby reports, so a reason already
    /// recorded stands.
    fn note_pattern_failure(&mut self, failure: PatternFailure) {
        if self.pattern_failure.is_none() {
            self.pattern_failure = Some(failure);
        }
    }

    /// Record that `named` did not cover `value`, which is how Ruby words
    /// every failure of `===`.
    fn note_case_equal_failure(&mut self, named: &Object, value: &Object, position: Position) {
        if self.pattern_failure.is_some() {
            return;
        }
        let named = self.written_out(named, position);
        let value = self.written_out(value, position);
        self.note_pattern_failure(PatternFailure::Detail(format!(
            "{named} === {value} does not return true"
        )));
    }

    /// How a value reads in a message, which is what `inspect` answers.
    fn written_out(&mut self, value: &Object, position: Position) -> String {
        self.get_inspect_representation(value, position)
            .unwrap_or_else(|_| format!("{value}"))
    }

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
    fn warn_duplicated_when_clauses(
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
    fn run_in_clauses(
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
                        match self.execute_statement(statement)? {
                            ControlFlow::Next => {}
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

    /// What a value a single pattern does not cover raises, which names both
    /// the value and what the pattern found wrong with it.
    pub(crate) fn detailed_no_matching_pattern(
        &mut self,
        value: &Object,
        position: Position,
    ) -> MetorexError {
        let Some(failure) = self.pattern_failure.take() else {
            return self.no_matching_pattern(value, position);
        };
        let written = self.written_out(value, position);
        let (detail, missing) = match failure {
            PatternFailure::Detail(detail) => (detail, None),
            PatternFailure::MissingKey { matchee, key } => (
                format!("key not found: :{key}"),
                Some((matchee, Object::symbol(key))),
            ),
        };
        let class_name = match &missing {
            Some(_) => "NoMatchingPatternKeyError",
            None => "NoMatchingPatternError",
        };
        let raised = crate::vm::errors::simple_exception(
            class_name,
            &format!("{written}: {detail}"),
            position,
        );
        let Some((matchee, key)) = missing else {
            return raised;
        };
        let MetorexError::UncaughtException { exception, .. } = &raised else {
            return raised;
        };
        if let Object::Exception(details) = exception {
            let mut details = details.borrow_mut();
            details
                .instance_vars
                .insert(crate::vm::KEY_ERROR_KEY.to_string(), key);
            details
                .instance_vars
                .insert(MATCHEE_KEY.to_string(), matchee);
        }
        raised
    }

    /// What a value no pattern covers raises, which names the value itself.
    pub(crate) fn no_matching_pattern(
        &mut self,
        value: &Object,
        position: Position,
    ) -> MetorexError {
        let written = self
            .get_inspect_representation(value, position)
            .unwrap_or_else(|_| format!("{value}"));
        crate::vm::errors::simple_exception("NoMatchingPatternError", &written, position)
    }

    /// Match a pattern against a value and collect variable bindings.
    /// Returns true if the pattern matches, false otherwise.
    /// The value a constant named in a pattern stands for. The lexical scope
    /// answers first, and then the class the running method belongs to, which
    /// is where a class's own constants live.
    fn constant_for_pattern(&self, name: &str) -> Option<Object> {
        if let Some(found) = self.resolve_constant_in_scope(name) {
            return Some(found);
        }
        // `Socket::SOCK_DGRAM` names a constant the namespace reaches through
        // a module it includes, which no lookup by full path finds.
        if let Some((namespace, last)) = name.rsplit_once("::") {
            let holder = self.resolve_constant_in_scope(namespace)?;
            let (Object::Class(class_rc) | Object::Module(class_rc)) = holder else {
                return None;
            };
            let mut cursor = Some(class_rc);
            while let Some(current) = cursor {
                if let Some(found) = current.get_class_var(last) {
                    return Some(found);
                }
                for mixin in current.transitive_mixins() {
                    if let Some(found) = mixin.get_class_var(last) {
                        return Some(found);
                    }
                }
                cursor = current.superclass();
            }
            return None;
        }
        let holder = self.environment().get("self")?;
        let mut cursor = match &holder {
            Object::Class(class_rc) | Object::Module(class_rc) => Some(Rc::clone(class_rc)),
            Object::Instance(instance) => Some(Rc::clone(&instance.borrow().class)),
            _ => None,
        };
        while let Some(current) = cursor {
            if let Some(found) = current.get_class_var(name) {
                return Some(found);
            }
            for mixin in current.transitive_mixins() {
                if let Some(found) = mixin.get_class_var(name) {
                    return Some(found);
                }
            }
            cursor = current.superclass();
        }
        None
    }

    pub(crate) fn match_pattern(
        &mut self,
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

            // Identifier pattern - binds the value to a variable. The name is
            // readable by the rest of the same pattern, which is what lets
            // `in [n, ^n]` compare one element against another.
            MatchPattern::Identifier(name) => {
                bindings.insert(name.clone(), value.clone());
                self.bind_pattern_name(name, value.clone());
                Ok(true)
            }

            // Wildcard pattern - matches anything without binding
            MatchPattern::Wildcard => Ok(true),

            // Anything written where a pattern goes that stands for a value
            // of its own, which the case asks with `===`.
            // `when *values` asks each of the values in turn, so a list
            // written there stands for the choices it holds.
            MatchPattern::Expression(held)
                if matches!(held.as_ref(), crate::ast::Expression::Splat { .. }) =>
            {
                let crate::ast::Expression::Splat { expression, .. } = held.as_ref() else {
                    unreachable!("guarded by the match above")
                };
                let spread = self.evaluate_expression(expression)?;
                let choices = match spread {
                    Object::Array(held) => held.borrow().clone(),
                    other => vec![other],
                };
                for choice in choices {
                    let matched = self.evaluate_binary_operation(
                        &crate::ast::BinaryOp::CaseEqual,
                        choice,
                        value.clone(),
                        position,
                    )?;
                    if matched.is_truthy() {
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            MatchPattern::Expression(held) => {
                let pattern_value = self.evaluate_expression(held)?;
                let matched = self.pattern_case_equal(&pattern_value, value, position)?;
                if !matched {
                    self.note_case_equal_failure(&pattern_value, value, position);
                }
                Ok(matched)
            }

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
                // A constant that names something other than a class stands
                // for that value, so `when ROUND_FLOOR` compares against the
                // number rather than asking what class the value is.
                if let Some(named) = self.constant_for_pattern(type_name)
                    && !matches!(named, Object::Class(_) | Object::Module(_))
                {
                    return Ok(named == *value);
                }
                let actual_type = value.type_name();

                // Support both Ruby-style names (Integer, Hash) and internal names (Int, Dict)
                let matches = match type_name.as_str() {
                    // Direct type name matches
                    name if name == actual_type => true,

                    // Ruby-style aliases
                    // A number too wide for a machine word is an Integer
                    // just the same, so both spellings answer here.
                    "Integer" => matches!(value, Object::Int(_) | Object::BigInt(_)),
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
                    // Ruby reports the last choice a run of alternatives tried,
                    // so each one starts with nothing recorded against it.
                    self.pattern_failure = None;
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

            // `^name` compares against what the name holds rather than
            // binding anything.
            MatchPattern::Pinned(held) => {
                let pinned = self.evaluate_expression(held)?;
                let matched = self.evaluate_binary_operation(
                    &crate::ast::BinaryOp::CaseEqual,
                    pinned.clone(),
                    value.clone(),
                    position,
                )?;
                if !matched.is_truthy() {
                    self.note_case_equal_failure(&pinned, value, position);
                }
                Ok(matched.is_truthy())
            }

            MatchPattern::ArrayPattern {
                constant,
                prefix,
                rest,
                suffix,
            } => {
                if !self.pattern_constant_matches(constant.as_deref(), value, position)? {
                    return Ok(false);
                }
                let Some(held) = self.deconstructed(value, position)? else {
                    return Ok(false);
                };
                self.match_array_shape(prefix, rest, suffix, &held, bindings, position)
            }

            MatchPattern::FindPattern {
                constant,
                before,
                middle,
                after,
            } => {
                if !self.pattern_constant_matches(constant.as_deref(), value, position)? {
                    return Ok(false);
                }
                let Some(held) = self.deconstructed(value, position)? else {
                    return Ok(false);
                };
                // Ruby says the run was nowhere to be found rather than
                // naming the element a single placing stopped at, so what a
                // placing records is dropped once the search is over.
                let before_the_search = self.pattern_failure.clone();
                if middle.len() > held.len() {
                    self.note_find_pattern_failure(&held, position);
                    return Ok(false);
                }
                // The run is looked for at every place it could sit, from the
                // front, which is the first one Ruby answers with.
                for at in 0..=(held.len() - middle.len()) {
                    let mut tried = bindings.clone();
                    let mut matched = true;
                    for (offset, pattern) in middle.iter().enumerate() {
                        if !self.match_pattern(pattern, &held[at + offset], &mut tried, position)? {
                            matched = false;
                            break;
                        }
                    }
                    if !matched {
                        continue;
                    }
                    if let Some(name) = before {
                        tried.insert(name.clone(), Object::array(held[..at].to_vec()));
                    }
                    if let Some(name) = after {
                        tried.insert(
                            name.clone(),
                            Object::array(held[at + middle.len()..].to_vec()),
                        );
                    }
                    *bindings = tried;
                    return Ok(true);
                }
                self.pattern_failure = before_the_search;
                self.note_find_pattern_failure(&held, position);
                Ok(false)
            }

            MatchPattern::HashPattern {
                constant,
                entries,
                rest,
            } => {
                if !self.pattern_constant_matches(constant.as_deref(), value, position)? {
                    return Ok(false);
                }
                let wanted: Vec<String> = entries.iter().map(|(key, _)| key.clone()).collect();
                let Some(held) = self.deconstructed_keys(value, &wanted, rest, position)? else {
                    return Ok(false);
                };
                self.match_hash_shape(entries, rest, &held, bindings, position)
            }
        }
    }

    /// Whether the constant a pattern was written with covers the value, and
    /// true where the pattern named none.
    fn pattern_constant_matches(
        &mut self,
        constant: Option<&crate::ast::Expression>,
        value: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let Some(held) = constant else {
            return Ok(true);
        };
        let named = self.evaluate_expression(held)?;
        let matched = self.pattern_case_equal(&named, value, position)?;
        if !matched {
            self.note_case_equal_failure(&named, value, position);
        }
        Ok(matched)
    }

    /// Whether what a pattern names covers the value, asked the way Ruby
    /// asks: through `===`, and through a refinement where one is in force.
    fn pattern_case_equal(
        &mut self,
        named: &Object,
        value: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if self.refined_pattern_method(named, "===").is_some() {
            let held = self.send_pattern_method(named, "===", vec![value.clone()], position)?;
            return Ok(held.is_truthy());
        }
        let matched = self.evaluate_binary_operation(
            &crate::ast::BinaryOp::CaseEqual,
            named.clone(),
            value.clone(),
            position,
        )?;
        Ok(matched.is_truthy())
    }

    /// The elements an array pattern reads out of a value: the array itself,
    /// or what the value's own `deconstruct` answers. None where the value
    /// has no elements to give.
    fn deconstructed(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Option<Vec<Object>>, MetorexError> {
        // One `case` asks its own subject for its elements once, and every
        // clause in it reads what that answered.
        let is_the_subject = self
            .deconstructed_values
            .last()
            .is_some_and(|(subject, _)| names_the_same_value(subject, value));
        if is_the_subject && let Some((_, Some(held))) = self.deconstructed_values.last() {
            return Ok(Some(held.clone()));
        }
        if !self.answers_to_pattern_method(value, "deconstruct", position)? {
            let written = self.written_out(value, position);
            self.note_pattern_failure(PatternFailure::Detail(format!(
                "{written} does not respond to #deconstruct"
            )));
            return Ok(None);
        }
        let held = self.send_pattern_method(value, "deconstruct", Vec::new(), position)?;
        let elements = match crate::vm::native_methods::array_subclass_value(&held) {
            Some(Object::Array(elements)) => elements.borrow().clone(),
            _ => match &held {
                Object::Array(elements) => elements.borrow().clone(),
                other => {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!(
                            "deconstruct must return Array, got {}",
                            self.builtins().class_of(other).ruby_name()
                        ),
                        position,
                    ));
                }
            },
        };
        if is_the_subject && let Some((_, held)) = self.deconstructed_values.last_mut() {
            *held = Some(elements.clone());
        }
        Ok(Some(elements))
    }

    /// Whether a value answers the method a pattern is about to send it. The
    /// question is put to the value itself, which is what a program watching
    /// `respond_to?` sees.
    fn answers_to_pattern_method(
        &mut self,
        value: &Object,
        named: &str,
        position: Position,
    ) -> Result<bool, MetorexError> {
        // A refinement in force here answers for the value too, and it is not
        // one `respond_to?` knows about.
        if self.refined_pattern_method(value, named).is_some() {
            return Ok(true);
        }
        let held = self.send_to_object(
            value.clone(),
            "respond_to?",
            vec![Object::symbol(named.to_string())],
            position,
        )?;
        Ok(held.is_truthy())
    }

    /// The method a refinement in force here puts in place of the value's
    /// own, which is what a pattern reaches first.
    fn refined_pattern_method(
        &self,
        value: &Object,
        named: &str,
    ) -> Option<std::rc::Rc<crate::object::Method>> {
        let target = crate::vm::method_lookup::refinement_target_name(value, self)?;
        self.find_refined_method(&target, named)
    }

    /// Send a pattern's own method, reaching a refinement in force here
    /// before the value's own.
    fn send_pattern_method(
        &mut self,
        value: &Object,
        named: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some(method) = self.refined_pattern_method(value, named) {
            let class = self.builtins().class_of(value);
            return self.invoke_method(class, method, value.clone(), arguments, position);
        }
        self.send_to_object(value.clone(), named, arguments, position)
    }

    /// The keys a hash pattern reads out of a value: the hash itself, or what
    /// the value's own `deconstruct_keys` answers. A pattern that names every
    /// key it wants hands those names over, and one with a rest hands nil.
    fn deconstructed_keys(
        &mut self,
        value: &Object,
        wanted: &[String],
        rest: &crate::ast::HashPatternRest,
        position: Position,
    ) -> Result<Option<indexmap::IndexMap<String, Object>>, MetorexError> {
        if !self.answers_to_pattern_method(value, "deconstruct_keys", position)? {
            let written = self.written_out(value, position);
            self.note_pattern_failure(PatternFailure::Detail(format!(
                "{written} does not respond to #deconstruct_keys"
            )));
            return Ok(None);
        }
        // A pattern naming a rest may read any key, so it asks for all of
        // them; one that names its keys says which it wants.
        let named = match rest {
            // A pattern naming a rest of its own may read any key, so it asks
            // for all of them. Every other pattern says which it wants.
            crate::ast::HashPatternRest::Named(_) => Object::Nil,
            _ => Object::array(
                wanted
                    .iter()
                    .map(|key| Object::symbol(key.clone()))
                    .collect(),
            ),
        };
        let held = self.send_pattern_method(value, "deconstruct_keys", vec![named], position)?;
        match held {
            Object::Dict(entries) => Ok(Some(entries.borrow().clone())),
            other => Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!(
                    "deconstruct_keys must return Hash, got {}",
                    self.builtins().class_of(&other).ruby_name()
                ),
                position,
            )),
        }
    }

    /// Record that a run of elements a find pattern looks for sits nowhere
    /// in the value.
    fn note_find_pattern_failure(&mut self, held: &[Object], position: Position) {
        if self.pattern_failure.is_some() {
            return;
        }
        let written = self.written_out(&Object::array(held.to_vec()), position);
        self.note_pattern_failure(PatternFailure::Detail(format!(
            "{written} does not match to find pattern"
        )));
    }

    /// Record that a value holds the wrong number of elements for an array
    /// pattern. A pattern with a rest names the fewest it can take.
    fn note_length_mismatch(
        &mut self,
        held: &[Object],
        wanted: usize,
        takes_more: bool,
        position: Position,
    ) {
        if self.pattern_failure.is_some() {
            return;
        }
        let written = self.written_out(&Object::array(held.to_vec()), position);
        let given = held.len();
        let wanted = match takes_more {
            true => format!("{wanted}+"),
            false => format!("{wanted}"),
        };
        self.note_pattern_failure(PatternFailure::Detail(format!(
            "{written} length mismatch (given {given}, expected {wanted})"
        )));
    }

    /// Match the elements against the patterns before a rest, the rest
    /// itself, and the patterns after it.
    fn match_array_shape(
        &mut self,
        prefix: &[crate::ast::MatchPattern],
        rest: &Option<Option<String>>,
        suffix: &[crate::ast::MatchPattern],
        held: &[Object],
        bindings: &mut HashMap<String, Object>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let Some(rest) = rest else {
            if prefix.len() != held.len() {
                self.note_length_mismatch(held, prefix.len(), false, position);
                return Ok(false);
            }
            for (pattern, element) in prefix.iter().zip(held.iter()) {
                if !self.match_pattern(pattern, element, bindings, position)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        };
        if held.len() < prefix.len() + suffix.len() {
            self.note_length_mismatch(held, prefix.len() + suffix.len(), true, position);
            return Ok(false);
        }
        for (at, pattern) in prefix.iter().enumerate() {
            if !self.match_pattern(pattern, &held[at], bindings, position)? {
                return Ok(false);
            }
        }
        let after = held.len() - suffix.len();
        for (at, pattern) in suffix.iter().enumerate() {
            if !self.match_pattern(pattern, &held[after + at], bindings, position)? {
                return Ok(false);
            }
        }
        if let Some(name) = rest {
            bindings.insert(
                name.clone(),
                Object::array(held[prefix.len()..after].to_vec()),
            );
        }
        Ok(true)
    }

    /// Match the keys a hash pattern names, and bind the rest where it asked
    /// for them.
    fn match_hash_shape(
        &mut self,
        entries: &[(String, Option<crate::ast::MatchPattern>)],
        rest: &crate::ast::HashPatternRest,
        held: &indexmap::IndexMap<String, Object>,
        bindings: &mut HashMap<String, Object>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        // A hash pattern naming nothing at all matches only a hash holding
        // nothing, the way `**nil` does.
        if entries.is_empty()
            && matches!(rest, crate::ast::HashPatternRest::Silent)
            && !held.is_empty()
        {
            return Ok(false);
        }
        for (key, pattern) in entries {
            // A hash holds its symbol keys under the name with the colon in
            // front, and a pattern names symbol keys alone.
            let Some(value) = held.get(&format!(":{}", key)) else {
                self.note_pattern_failure(PatternFailure::MissingKey {
                    matchee: Object::Dict(Rc::new(RefCell::new(held.clone()))),
                    key: key.clone(),
                });
                return Ok(false);
            };
            let value = value.clone();
            match pattern {
                Some(pattern) => {
                    if !self.match_pattern(pattern, &value, bindings, position)? {
                        return Ok(false);
                    }
                }
                None => {
                    bindings.insert(key.clone(), value);
                }
            }
        }
        let named: std::collections::HashSet<String> =
            entries.iter().map(|(key, _)| format!(":{}", key)).collect();
        match rest {
            crate::ast::HashPatternRest::Refused => {
                let left: indexmap::IndexMap<String, Object> = held
                    .iter()
                    .filter(|(key, _)| !named.contains(*key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                if !left.is_empty() {
                    let left = Object::Dict(Rc::new(RefCell::new(left)));
                    let written = self.written_out(&left, position);
                    self.note_pattern_failure(PatternFailure::Detail(format!(
                        "rest of {written} is not empty"
                    )));
                    return Ok(false);
                }
            }
            crate::ast::HashPatternRest::Named(name) => {
                let left: indexmap::IndexMap<String, Object> = held
                    .iter()
                    .filter(|(key, _)| !named.contains(*key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                bindings.insert(name.clone(), Object::Dict(Rc::new(RefCell::new(left))));
            }
            _ => {}
        }
        Ok(true)
    }

    /// Match an array pattern against an array value.
    pub(crate) fn match_array_pattern(
        &mut self,
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
        &mut self,
        key_patterns: &[(String, crate::ast::MatchPattern)],
        dict: &indexmap::IndexMap<String, Object>,
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
    pub(crate) fn apply_pattern_bindings(&mut self, bindings: &HashMap<String, Object>) {
        for (name, value) in bindings {
            self.bind_pattern_name(name, value.clone());
        }
    }

    /// Bind a name a pattern gave. A name the scope already holds is written
    /// where it stands, so a pattern inside a block writes the local the
    /// block closed over rather than one of its own.
    fn bind_pattern_name(&mut self, name: &str, value: Object) {
        if self.environment().get(name).is_some() {
            self.environment_mut().set(name, value);
            return;
        }
        self.environment_mut().define(name.to_string(), value);
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
    fn evaluate_guard(&mut self, guard_expr: Option<&Expression>) -> Result<bool, MetorexError> {
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
fn literal_pattern_spelling(pattern: &crate::ast::MatchPattern) -> Option<String> {
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
fn names_the_same_value(left: &Object, right: &Object) -> bool {
    match (left, right) {
        (Object::Array(one), Object::Array(other)) => Rc::ptr_eq(one, other),
        (Object::Dict(one), Object::Dict(other)) => Rc::ptr_eq(one, other),
        (Object::Instance(one), Object::Instance(other)) => Rc::ptr_eq(one, other),
        (Object::String(one), Object::String(other)) => Rc::ptr_eq(one, other),
        _ => left == right,
    }
}
