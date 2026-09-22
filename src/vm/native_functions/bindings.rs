// The scope a call sits in, and the methods defined at the top level.

use super::*;

impl VirtualMachine {
    /// `binding` captures the frame that called it, not the receiver
    /// it was sent to: the local variables in scope (as shared cells,
    /// so an assignment through the binding is visible to both) and the
    /// `self` in force there.
    pub(crate) fn kernel_binding(&mut self, position: Position) -> Result<Object, MetorexError> {
        // Only the locals in force where the call sits belong to a
        // binding. The builtins the root scope holds are constants
        // and methods, which `local_variables` does not name.
        let named = self.environment().binding_variable_names();
        let mut variables = std::collections::HashMap::new();
        let mut order: Vec<String> = Vec::new();
        for name in named {
            // The root scope also holds the builtins, which are not
            // locals of the program. A name the globals hold is one
            // of those only while it still answers with the same
            // object: a local of the same name shadows it, and that
            // one is a local like any other.
            if let Some(builtin) = self.globals().get(&name)
                && self.environment().get(&name) == Some(builtin)
            {
                continue;
            }
            // A local is named the way Ruby lets one be named, which
            // rules out the globals and constants sharing the scope.
            if !name.starts_with(|held: char| held == '_' || held.is_lowercase()) {
                continue;
            }
            if variables.contains_key(&name) {
                continue;
            }
            if let Some(cell) = self.environment().get_ref(&name) {
                order.push(name.clone());
                variables.insert(name, cell);
            }
        }
        // At file scope there is no `self` binding; Ruby's top-level
        // self is `main`, which is what TOPLEVEL_BINDING holds.
        let receiver = self
            .environment()
            .get("self")
            .or_else(|| match self.globals().get("TOPLEVEL_BINDING") {
                Some(Object::Binding(b)) => b.receiver.clone(),
                _ => None,
            })
            .unwrap_or(Object::Nil);
        let held = crate::object::Binding::with_receiver(variables, receiver);
        held.set_order(order);
        // The method the call sits in, which code run through the
        // binding names as its own.
        *held.method.borrow_mut() = self.enclosing_method_names();
        // The refinements in force here, which code run through the
        // binding sees.
        *held.refinements.borrow_mut() = self.snapshot_active_refinements();
        // The classes and modules open here, which a class opened by
        // code run through the binding is nested in.
        *held.nesting.borrow_mut() = match self.method_nesting_stack.last() {
            Some(captured) => captured.clone(),
            None => self.snapshot_lexical_nesting(),
        };
        // Where the call sits, which `source_location` reports.
        *held.source.borrow_mut() = Some((
            self.current_file
                .as_ref()
                .map(|file| file.display().to_string())
                .unwrap_or_default(),
            position.line,
        ));
        Ok(Object::Binding(std::rc::Rc::new(held)))
    }

    pub(crate) fn define_top_level_method(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use crate::object::Method;
        use std::rc::Rc;
        if arguments.len() != 1 {
            return Err(MetorexError::runtime_error(
                format!("define_method expects 1 argument, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let method_name = match &arguments[0] {
            Object::Symbol(s) => s.as_str().to_string(),
            Object::String(s) => s.as_str().to_string(),
            other => {
                return Err(MetorexError::runtime_error(
                    format!(
                        "define_method expects a Symbol or String, got {}",
                        other.type_name()
                    ),
                    crate::vm::utils::position_to_location(position),
                ));
            }
        };
        let block = match self.pending_block.take() {
            Some(Object::Block(b)) => b,
            _ => {
                return Err(MetorexError::runtime_error(
                    "define_method requires a block".to_string(),
                    crate::vm::utils::position_to_location(position),
                ));
            }
        };
        let params: Vec<String> = block.parameters.clone();
        let body: Vec<crate::ast::Statement> = block.body.clone();
        let mut method = Method::new(method_name.clone(), params, body);
        method.captured_vars = Some(block.captured_vars().clone());
        // Optional block params (`|a, b = 1|`) become the method's
        // default parameters.
        for (orig_idx, expr) in block.parameter_defaults.iter() {
            let reg_idx = block.parameters[..*orig_idx]
                .iter()
                .filter(|p| !p.starts_with('&'))
                .count();
            method.default_parameters.push((reg_idx, expr.clone()));
        }
        let method_rc = Rc::new(method);
        // Install on current self if it's a Class/Module (e.g. inside class_eval),
        // otherwise on global Object (top-level `define_method` semantics).
        let target = match self.environment().get("self") {
            Some(Object::Class(c)) | Some(Object::Module(c)) => Some(c),
            _ => match self.globals().get("Object") {
                Some(Object::Class(c)) => Some(c),
                _ => None,
            },
        };
        if let Some(class) = target {
            class.define_method(method_name.clone(), method_rc);
        }
        Ok(Object::symbol(method_name))
    }
}
