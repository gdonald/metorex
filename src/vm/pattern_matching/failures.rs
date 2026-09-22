// Recording why a pattern did not cover a value.

use super::*;

impl VirtualMachine {
    /// Record why the pattern being tried failed. The innermost pattern fails
    /// first, and its reason is the one Ruby reports, so a reason already
    /// recorded stands.
    pub(crate) fn note_pattern_failure(&mut self, failure: PatternFailure) {
        if self.pattern_failure.is_none() {
            self.pattern_failure = Some(failure);
        }
    }

    /// Record that `named` did not cover `value`, which is how Ruby words
    /// every failure of `===`.
    pub(crate) fn note_case_equal_failure(
        &mut self,
        named: &Object,
        value: &Object,
        position: Position,
    ) {
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
    pub(crate) fn written_out(&mut self, value: &Object, position: Position) -> String {
        self.get_inspect_representation(value, position)
            .unwrap_or_else(|_| format!("{value}"))
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

    /// Record that a run of elements a find pattern looks for sits nowhere
    /// in the value.
    pub(crate) fn note_find_pattern_failure(&mut self, held: &[Object], position: Position) {
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
    pub(crate) fn note_length_mismatch(
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
}

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
