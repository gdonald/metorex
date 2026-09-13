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
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

use super::core::VirtualMachine;
use super::utils::{format_exception, is_truthy, position_to_location};

impl VirtualMachine {
    /// The String an interpolated literal makes. A piece that holds bytes
    /// rather than characters carries its encoding out to the whole, which is
    /// what keeps a byte spliced into an ASCII literal a byte.
    pub(crate) fn evaluate_interpolated_object(
        &mut self,
        parts: &[InterpolationPart],
    ) -> Result<Object, MetorexError> {
        let (text, carried) = self.interpolate(parts)?;
        let ascii_literal = parts.iter().all(|part| match part {
            InterpolationPart::Text(written) => written.is_ascii(),
            InterpolationPart::Expression(_) => true,
        });
        let Some(encoding) = carried.filter(|_| ascii_literal) else {
            return Ok(Object::string(text));
        };
        let spelled = crate::object::StringValue::with_encoding(text, encoding);
        spelled.mark_bytes();
        Ok(Object::String(std::rc::Rc::new(spelled)))
    }

    /// The text the parts spell, along with the encoding of the first piece
    /// that held bytes rather than characters.
    fn interpolate(
        &mut self,
        parts: &[InterpolationPart],
    ) -> Result<(String, Option<String>), MetorexError> {
        let mut carried: Option<String> = None;
        let mut buffer = String::new();

        for part in parts {
            match part {
                InterpolationPart::Text(text) => buffer.push_str(text),
                InterpolationPart::Expression(expr) => {
                    let value = self.evaluate_expression(expr)?;
                    // Interpolation uses #to_s semantics: a Symbol renders
                    // as its bare name, not the `:name` inspect form.
                    match &value {
                        Object::Symbol(s) => buffer.push_str(&s.as_str()),
                        // `nil.to_s` is the empty string, so `"#{nil}"` adds
                        // nothing rather than the `nil` inspect form.
                        Object::Nil => {}
                        // An object of the program's own is asked for its own
                        // `to_s`, which is the text Ruby puts in the hole.
                        Object::Instance(_) => {
                            let written = self.send_to_object(
                                value.clone(),
                                "to_s",
                                Vec::new(),
                                expr.position(),
                            )?;
                            match written {
                                Object::String(text) => buffer.push_str(&text.as_str()),
                                other => buffer.push_str(&other.to_string()),
                            }
                        }
                        // A pattern writes itself as the group it stands
                        // for, which is what `Regexp#to_s` answers.
                        Object::Regex(_, _) => {
                            let written = self.send_to_object(
                                value.clone(),
                                "to_s",
                                Vec::new(),
                                expr.position(),
                            )?;
                            match written {
                                Object::String(text) => buffer.push_str(&text.as_str()),
                                other => buffer.push_str(&other.to_string()),
                            }
                        }
                        _ => buffer.push_str(&value.to_string()),
                    }
                    if let Object::String(spelled) = &value
                        && spelled.holds_bytes()
                        && carried.is_none()
                    {
                        carried = Some(spelled.encoding_name());
                    }
                }
            }
        }

        Ok((buffer, carried))
    }

    /// Evaluate array literal expressions.
    pub(crate) fn evaluate_array_literal(
        &mut self,
        elements: &[Expression],
    ) -> Result<Object, MetorexError> {
        let mut evaluated = Vec::with_capacity(elements.len());
        for element in elements {
            // `[first, *rest]` splices the splatted array in place rather
            // than nesting it as one element.
            if let Expression::Splat {
                expression,
                position: splat_at,
            } = element
            {
                match self.evaluate_expression(expression)? {
                    Object::Array(items) => evaluated.extend(items.borrow().iter().cloned()),
                    Object::Nil => {}
                    other => {
                        // Anything else that answers `to_a` spreads into what
                        // that gives, which is what `[*"a".."z"]` names.
                        match self.splat_through_to_a(other, *splat_at)? {
                            Ok(items) => evaluated.extend(items),
                            Err(held) => evaluated.push(held),
                        }
                    }
                }
                continue;
            }
            evaluated.push(self.evaluate_expression(element)?);
        }
        Ok(Object::Array(Rc::new(RefCell::new(evaluated))))
    }

    /// What a splat spreads a value into. A value answering `to_a` spreads
    /// into what that gives; anything else stands for itself, which the Err
    /// side carries back.
    pub(crate) fn splat_through_to_a(
        &mut self,
        value: Object,
        position: crate::lexer::Position,
    ) -> Result<Result<Vec<Object>, Object>, MetorexError> {
        if !self.responds_to(&value, "to_a") {
            return Ok(Err(value));
        }
        match self.send_to_object(value.clone(), "to_a", vec![], position)? {
            Object::Array(items) => Ok(Ok(items.borrow().clone())),
            _ => Ok(Err(value)),
        }
    }

    /// Evaluate dictionary literal expressions.
    pub(crate) fn evaluate_dictionary_literal(
        &mut self,
        entries: &[(Expression, Expression)],
    ) -> Result<Object, MetorexError> {
        let mut map = IndexMap::with_capacity(entries.len());
        let mut key_objs: IndexMap<String, Object> = IndexMap::new();

        for (key_expr, value_expr) in entries {
            // `{**held}` copies every pair `held` carries into the hash.
            if let Expression::KeywordSplat { expression, .. } = key_expr {
                let spread = self.evaluate_expression(expression)?;
                if let Object::Dict(pairs) = spread {
                    for (name, held) in pairs.borrow().iter() {
                        map.insert(name.clone(), held.clone());
                    }
                }
                continue;
            }
            let key_value = self.evaluate_expression(key_expr)?;
            let key_position = key_expr.position();
            let mut probe = map.clone();
            for (slot, held) in &key_objs {
                probe.entry(slot.clone()).or_insert_with(|| held.clone());
            }
            probe.insert(
                "__MX_KEY_OBJECTS__".to_string(),
                Object::Dict(Rc::new(RefCell::new(key_objs.clone()))),
            );
            let key_string = self.dict_slot_in(&probe, &key_value, false, key_position)?;
            if !crate::vm::utils::is_primitive_key(&key_value) {
                key_objs
                    .entry(key_string.clone())
                    .or_insert_with(|| key_value.clone());
            }

            let value = self.evaluate_expression(value_expr)?;
            if map.contains_key(&key_string) && is_literal_key(key_expr) {
                self.warn_duplicated_key(&key_value, key_expr, value_expr)?;
            }
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

    /// Ruby names a key written twice in the same literal, reporting the line
    /// whose value wins. Only a literal key is reported, since two expressions
    /// that happen to answer the same object are not a duplicate as written.
    fn warn_duplicated_key(
        &mut self,
        key: &Object,
        key_expr: &Expression,
        value_expr: &Expression,
    ) -> Result<(), MetorexError> {
        let position = key_expr.position();
        let file = self
            .current_source_file
            .clone()
            .or_else(|| {
                self.current_file
                    .as_ref()
                    .map(|path| path.display().to_string())
            })
            .unwrap_or_default();
        if !self
            .reported_duplicate_keys
            .insert((file.clone(), position.line, position.column))
        {
            return Ok(());
        }
        let rendered = crate::vm::native_methods::array_methods::inspect_element(key);
        let message = format!(
            "{}:{}: warning: key {} is duplicated and overwritten on line {}\n",
            file,
            position.line,
            rendered,
            value_expr.position().line
        );
        self.warn_through_warning_module(message, position)
    }

    /// Evaluate indexing operations on arrays and dictionaries.
    /// The number an end of a span names, read through `to_int` when it is
    /// not already an Integer. A nil end names no number at all.
    pub(crate) fn span_end_index(
        &mut self,
        edge: &Object,
        position: Position,
    ) -> Result<Option<i64>, MetorexError> {
        match edge {
            Object::Nil => Ok(None),
            Object::Int(number) => Ok(Some(*number)),
            Object::Float(number) => Ok(Some(*number as i64)),
            other if self.responds_to(other, "to_int") => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    Object::Int(number) => Ok(Some(number)),
                    Object::Float(number) => Ok(Some(number as i64)),
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn evaluate_index_operation(
        &mut self,
        collection: Object,
        key: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // The environment names its variables in text, so a lookup reads what
        // it was handed as text first.
        if let Object::Dict(dict_rc) = &collection
            && self.dict_is_environment(dict_rc)
        {
            let held = Object::Dict(Rc::clone(dict_rc));
            return self
                .call_hash_method(&held, "[]", &[key], position)?
                .ok_or_else(|| {
                    MetorexError::runtime_error(
                        "ENV lookup answered nothing",
                        crate::vm::utils::position_to_location(position),
                    )
                });
        }
        match collection {
            Object::Array(elements_rc) => match key {
                Object::Int(index) => {
                    let elements = elements_rc.borrow();
                    // A negative index counts back from the end, and one past
                    // either end answers nil rather than raising.
                    let offset = if index < 0 {
                        match elements.len().checked_sub(index.unsigned_abs() as usize) {
                            Some(offset) => offset,
                            None => return Ok(Object::Nil),
                        }
                    } else {
                        index as usize
                    };
                    Ok(elements.get(offset).cloned().unwrap_or(Object::Nil))
                }
                // `ary[range]` answers the slice the range covers. A negative
                // bound counts from the end, and a start past the end answers
                // nil the way Ruby's does.
                Object::Range {
                    ref start,
                    ref end,
                    exclusive,
                } => {
                    let elements = elements_rc.borrow();
                    let length = elements.len() as i64;
                    let resolve = |bound: &Object| match bound {
                        Object::Int(value) if *value < 0 => value + length,
                        Object::Int(value) => *value,
                        _ => 0,
                    };
                    let from = match start.as_ref() {
                        Object::Nil => 0,
                        bound => resolve(bound),
                    };
                    let mut to = match end.as_ref() {
                        Object::Nil => length,
                        bound => {
                            let resolved = resolve(bound);
                            if exclusive { resolved } else { resolved + 1 }
                        }
                    };
                    if from < 0 || from > length {
                        return Ok(Object::Nil);
                    }
                    to = to.clamp(from, length);
                    let slice = elements[from as usize..to as usize].to_vec();
                    drop(elements);
                    Ok(Object::Array(Rc::new(RefCell::new(slice))))
                }
                _ => Err(MetorexError::type_error(
                    format!("Array index must be an Integer, found {}", key.type_name()),
                    position_to_location(position),
                )),
            },
            Object::Dict(dict_rc) => {
                let (fresh, held) = self.hash_locate_key(&dict_rc, &key, position)?;
                let key_string = held.unwrap_or(fresh);

                let dict = dict_rc.borrow();
                if let Some(value) = dict.get(&key_string) {
                    Ok(value.clone())
                } else if let Some(default) = dict.get("__MX_DEFAULT__") {
                    // `Hash.new(default)` answers that value for a key it has
                    // no entry for, without storing anything.
                    Ok(default.clone())
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
                        // Ruby keeps nothing for a key the block did not
                        // store, so the next read calls the block again.
                        Ok(block_result)
                    }
                } else {
                    Ok(Object::Nil)
                }
            }

            // Symbol#[] mirrors String#[] on the symbol's name.
            Object::Symbol(s) => {
                let as_string = Object::string(s.as_str().to_string());
                self.evaluate_index_operation(as_string, key, position)
            }

            // A String reads its own `[]`, which keeps one reading of the
            // characters for both the index form and the method call.
            Object::String(s) => {
                // A Float or an object that names an index arrives as one,
                // and a Range subclass stands for the span it holds.
                let key = match &key {
                    Object::Int(_)
                    | Object::Range { .. }
                    | Object::String(_)
                    | Object::Regex(_, _) => key,
                    Object::Float(number) => Object::Int(*number as i64),
                    other => match crate::vm::native_methods::as_range(other) {
                        Some(span) => span,
                        None if self.responds_to(other, "to_int") => {
                            self.send_to_object(other.clone(), "to_int", vec![], position)?
                        }
                        None => key,
                    },
                };
                let made = match key {
                    Object::Int(i) => {
                        let chars: Vec<char> = s.as_str().chars().collect();
                        let len = chars.len() as i64;
                        let idx = if i < 0 { len + i } else { i };
                        if idx < 0 || idx >= len {
                            Ok(Object::Nil)
                        } else {
                            Ok(Object::string(chars[idx as usize].to_string()))
                        }
                    }
                    Object::Range {
                        start,
                        end,
                        exclusive,
                    } => {
                        let chars: Vec<char> = s.as_str().chars().collect();
                        let len = chars.len() as i64;
                        let from = match self.span_end_index(start.as_ref(), position)? {
                            Some(number) => {
                                let placed = if number < 0 { len + number } else { number };
                                if placed < 0 || placed > len {
                                    return Ok(Object::Nil);
                                }
                                placed
                            }
                            None => 0,
                        };
                        let last = match self.span_end_index(end.as_ref(), position)? {
                            Some(number) => {
                                let placed = if number < 0 { len + number } else { number };
                                if exclusive { placed - 1 } else { placed }
                            }
                            None => len - 1,
                        };
                        let count = (last - from + 1).max(0);
                        let stop = (from + count).min(len);
                        let sliced: String = chars[from as usize..stop as usize].iter().collect();
                        Ok(Object::string(sliced))
                    }
                    // `text[other]` answers the other string when it appears,
                    // which is how Ruby looks a substring up.
                    Object::String(ref wanted) => {
                        if s.as_str().contains(&*wanted.as_str()) {
                            Ok(Object::string(wanted.as_str().to_string()))
                        } else {
                            Ok(Object::Nil)
                        }
                    }
                    // `text[pattern]` answers what the pattern matched, and
                    // the match is recorded so `$~` names it afterwards.
                    Object::Regex(ref pattern, ref flags) => {
                        let (pattern, flags) =
                            (pattern.as_str().to_string(), flags.as_str().to_string());
                        let subject = s.as_str().to_string();
                        match self.regexp_match_data(&pattern, &flags, &subject, 0, position)? {
                            Some(found) => self
                                .send_to_object(found, "[]", vec![Object::Int(0)], position)
                                .map(Ok)?,
                            None => Ok(Object::Nil),
                        }
                    }
                    _ => Err(MetorexError::type_error(
                        format!(
                            "String index must be Integer or Range, found {}",
                            key.type_name()
                        ),
                        position_to_location(position),
                    )),
                }?;
                // A piece of a string is written in the same encoding, and
                // its characters stand for what the whole one's stood for.
                if let Object::String(piece) = &made
                    && piece.encoding_name() == crate::object::string_value::DEFAULT_ENCODING
                {
                    piece.set_encoding(s.encoding_name());
                    if s.holds_bytes() {
                        piece.mark_bytes();
                    }
                }
                Ok(made)
            }

            Object::Class(_) | Object::Module(_) | Object::Instance(_) => {
                // A user-defined `[]` wins: on a class or module that is
                // `def self.[]`, stored under the `__class__` prefix.
                if let Some((owner, method)) = self.lookup_method(&collection, "[]")
                    && !method.is_undefined
                {
                    return self.invoke_method(
                        owner,
                        method,
                        collection.clone(),
                        vec![key],
                        position,
                    );
                }
                // Otherwise the native `[]` (Dir[], Hash[], and friends).
                let class = self.builtins().class_of(&collection);
                match self.call_native_method(
                    &class,
                    &collection,
                    "[]",
                    std::slice::from_ref(&key),
                    position,
                )? {
                    Some(val) => Ok(val),
                    // A class that answers what it was not asked for decides
                    // what indexing means, the same as it does for any other
                    // name it carries no method for.
                    None if self
                        .lookup_method(&collection, "method_missing")
                        .is_some_and(|(_, method)| !method.is_undefined) =>
                    {
                        self.send_to_object(
                            collection,
                            "method_missing",
                            vec![Object::symbol("[]"), key],
                            position,
                        )
                    }
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
                            home_frame: self.current_method_frame,
                        });
                    }
                    ControlFlow::Break { value, position } => {
                        // `break` inside an if/elsif/else branch must
                        // unwind to the enclosing iterator/block, not be
                        // silently swallowed by the if-as-expression
                        // wrapper. BlockBreak is the signal that
                        // execute_block_callable / each-style natives
                        // recognize.
                        return Err(MetorexError::BlockBreak {
                            value,
                            location: position_to_location(position),
                            home_frame: None,
                        });
                    }
                    ControlFlow::Retry { position } => {
                        // A `retry` written in a branch belongs to the begin
                        // whose rescue body holds it, so it unwinds there.
                        return Err(MetorexError::BlockRetry {
                            location: position_to_location(position),
                        });
                    }
                    ControlFlow::Redo { position } => {
                        // Like `break` above, `redo` and `next` inside an
                        // if-as-expression must unwind to the enclosing body
                        // rather than be swallowed here.
                        return Err(MetorexError::BlockRedo {
                            location: position_to_location(position),
                        });
                    }
                    ControlFlow::Continue { value, position } => {
                        return Err(MetorexError::BlockNext {
                            value,
                            location: position_to_location(position),
                        });
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

/// Whether a key is written out in the literal itself, which is what Ruby
/// checks for a duplicate rather than what the key evaluates to.
fn is_literal_key(key: &Expression) -> bool {
    matches!(
        key,
        Expression::Symbol { .. }
            | Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::NilLiteral { .. }
    )
}
