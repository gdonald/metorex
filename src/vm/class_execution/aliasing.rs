// Pointing one method name at another outside a class body.

use super::*;

impl VirtualMachine {
    /// Execute `alias new_name old_name` outside a class/module body. Ruby
    /// treats this as an alias on Object so it is visible in all instances.
    pub(crate) fn execute_alias(
        &mut self,
        new_name: &str,
        old_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // `alias $ERROR_INFO $!` gives one global a second name, which reads
        // and writes the same value the first one does.
        if let (Some(new_global), Some(old_global)) =
            (new_name.strip_prefix('$'), old_name.strip_prefix('$'))
        {
            self.global_aliases
                .insert(new_global.to_string(), old_global.to_string());
            return Ok(ControlFlow::Next);
        }
        if let Some(enclosing) = self.def_scope_stack.last().cloned() {
            self.install_alias(&enclosing, new_name, old_name, position)?;
            self.invoke_class_hook(&enclosing, "method_added", new_name, position)?;
            return Ok(ControlFlow::Next);
        }
        if let Some(Object::Class(object_class)) = self.globals().get("Object")
            && !object_class.alias_method(new_name, old_name)
        {
            return Err(MetorexError::runtime_error(
                format!("undefined method '{}' for alias", old_name),
                position_to_location(position),
            ));
        }
        Ok(ControlFlow::Next)
    }

    /// Evaluate attribute-name expressions for `attr_reader`/`attr_writer`/
    /// `attr_accessor`. Symbols and strings resolve directly; other values
    /// are coerced via `#to_str`. Mirrors Ruby's `Module#attr_*` semantics.
    pub(crate) fn resolve_attribute_names(
        &mut self,
        attributes: &[Expression],
        position: Position,
    ) -> Result<Vec<String>, MetorexError> {
        let mut names = Vec::with_capacity(attributes.len());
        for attr_expr in attributes {
            let value = self.evaluate_expression(attr_expr)?;
            names.push(self.coerce_method_name(&value, "attr_accessor", position)?);
        }
        Ok(names)
    }
}
