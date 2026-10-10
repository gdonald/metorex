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
        // A global given a second name, such as `$>` for `$stdout`, is listed
        // under both.
        let mut aliases: Vec<String> = self.global_aliases.keys().cloned().collect();
        aliases.sort();
        let mut listed: Vec<String> = self
            .globals()
            .variable_names()
            .map(|name| name.to_string())
            .collect();
        for alias in aliases {
            if !listed.contains(&alias) {
                listed.push(alias);
            }
        }
        let names: Vec<Object> = listed
            .into_iter()
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
        let names: Vec<Object> = self
            .visible_local_names()
            .into_iter()
            .map(Object::symbol)
            .collect();
        Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
            names,
        ))))
    }

    /// The locals the code running now sees, each once, in the order they
    /// were bound.
    pub(crate) fn visible_local_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .environment()
            .binding_variable_names()
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
        names
    }
}
