// What a pattern reads out of a value before matching it.

use super::*;

impl VirtualMachine {
    /// Whether the constant a pattern was written with covers the value, and
    /// true where the pattern named none.
    pub(crate) fn pattern_constant_matches(
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
    pub(crate) fn pattern_case_equal(
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
    pub(crate) fn deconstructed(
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
    pub(crate) fn answers_to_pattern_method(
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
    pub(crate) fn refined_pattern_method(
        &self,
        value: &Object,
        named: &str,
    ) -> Option<std::rc::Rc<crate::object::Method>> {
        let target = crate::vm::method_lookup::refinement_target_name(value, self)?;
        self.find_refined_method(&target, named)
    }

    /// Send a pattern's own method, reaching a refinement in force here
    /// before the value's own.
    pub(crate) fn send_pattern_method(
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
    pub(crate) fn deconstructed_keys(
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
}
