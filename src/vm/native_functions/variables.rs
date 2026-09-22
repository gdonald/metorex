// The variables in scope where the call was written.

use super::*;

impl VirtualMachine {
    /// `catch(tag) { |tag| ... }` runs the block, answering a matching
    /// `throw`'s value or, absent one, the block's own value. Called
    /// with no tag it makes a fresh object and yields that.
    /// `global_variables` names every global variable, sigil included.
    pub(crate) fn global_variable_names(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !arguments.is_empty() {
            return Err(MetorexError::runtime_error(
                format!(
                    "global_variables() expects 0 arguments, got {}",
                    arguments.len()
                ),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let names: Vec<Object> = self
            .globals()
            .variable_names()
            .map(|name| Object::symbol(format!("${}", name)))
            .collect();
        Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
            names,
        ))))
    }

    /// Kernel#local_variables — the names bound in the current scope
    /// chain, as Symbols. `self` is bound like a variable internally
    /// but is not a local, and a name rebound in an inner scope is
    /// reported once.
    pub(crate) fn local_variable_names(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if !arguments.is_empty() {
            return Err(MetorexError::runtime_error(
                format!(
                    "local_variables() expects 0 arguments, got {}",
                    arguments.len()
                ),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let mut names: Vec<String> = self
            .environment()
            .local_variable_names()
            .into_iter()
            .filter(|name| {
                name != "self"
                        // A builtin the root scope seeded is not a local,
                        // but a local of the same name shadows it, and
                        // that one is reported like any other.
                        && !(self.seeded_global_names.contains(name)
                            && self.globals().get(name) == self.environment().get(name))
                        && !self
                            .environment()
                            .get(name)
                            .is_some_and(|value| self.name_is_a_definition(name, &value))
            })
            .collect();
        let mut seen = std::collections::HashSet::new();
        names.retain(|name| seen.insert(name.clone()));
        let names: Vec<Object> = names.into_iter().map(Object::symbol).collect();
        Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
            names,
        ))))
    }
}
