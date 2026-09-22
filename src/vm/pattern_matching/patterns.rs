// Matching one pattern against one value.

use super::*;

impl VirtualMachine {
    /// Match a pattern against a value and collect variable bindings.
    /// Returns true if the pattern matches, false otherwise.
    /// The value a constant named in a pattern stands for. The lexical scope
    /// answers first, and then the class the running method belongs to, which
    /// is where a class's own constants live.
    pub(crate) fn constant_for_pattern(&self, name: &str) -> Option<Object> {
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
    pub(crate) fn bind_pattern_name(&mut self, name: &str, value: Object) {
        if self.environment().get(name).is_some() {
            self.environment_mut().set(name, value);
            return;
        }
        self.environment_mut().define(name.to_string(), value);
    }
}
