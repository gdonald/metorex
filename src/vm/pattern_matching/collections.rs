// Matching the elements of an array and the keys of a hash.

use super::*;

impl VirtualMachine {
    /// Match the elements against the patterns before a rest, the rest
    /// itself, and the patterns after it.
    pub(crate) fn match_array_shape(
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
    pub(crate) fn match_hash_shape(
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
}
