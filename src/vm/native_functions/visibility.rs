// Applying a visibility modifier written without a receiver.

use super::*;

impl VirtualMachine {
    /// enclosing class or module; at the top level it applies to Object.
    pub(crate) fn apply_visibility(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.pending_block.take();
        if let Some(class) = self.current_definee() {
            return self.apply_class_visibility_modifier(&class, name, &arguments, position);
        }
        self.apply_visibility_modifier(name, arguments, position)
    }

    pub(crate) fn apply_constant_visibility(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        // Apply visibility marks to constants on the current `self`
        // module/class. Inside a `module M; ...; end` body, `self`
        // is the module being defined, so qualified accesses like
        // `M::PrivConst` from outside raise NameError /private
        // constant/. `public_constant` is the inverse, and
        // `deprecate_constant` says a read of the name warns.
        self.pending_block.take();
        let target = match self.environment().get("self") {
            Some(Object::Class(c)) | Some(Object::Module(c)) => c,
            _ => return Ok(Object::Nil),
        };
        for arg in &arguments {
            let const_name = match arg {
                Object::Symbol(s) => s.as_str().to_string(),
                Object::String(s) => s.as_str().to_string(),
                _ => continue,
            };
            match name {
                "private_constant" => target.mark_private_constant(const_name),
                "deprecate_constant" => target.mark_deprecated_constant(const_name),
                _ => target.unmark_private_constant(&const_name),
            }
        }
        Ok(Object::Nil)
    }

    /// A receiverless `module_function` toggles the module-function
    /// state on the enclosing module.
    pub(crate) fn apply_module_function(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.pending_block.take();
        let current_self = self
            .current_definee()
            .map(Object::Module)
            .unwrap_or(Object::Nil);
        if let Object::Class(class) | Object::Module(class) = &current_self {
            if arguments.is_empty() {
                class.set_current_visibility(crate::vm::native_methods::MODULE_FUNCTION_VISIBILITY);
                return Ok(Object::Nil);
            }
            let class = Rc::clone(class);
            let mut names = Vec::with_capacity(arguments.len());
            for argument in &arguments {
                let name = self.coerce_method_name(argument, "module_function", position)?;
                self.copy_to_module_function(&class, &name, position)?;
                names.push(Object::symbol(name));
            }
            return Ok(match names.len() {
                1 => names.remove(0),
                _ => Object::Array(Rc::new(std::cell::RefCell::new(names))),
            });
        }
        Ok(Object::Nil)
    }

    /// A receiverless `private_class_method` / `public_class_method`
    /// inside a class or module body applies to that class.
    pub(crate) fn apply_class_method_visibility(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.pending_block.take();
        let current_self = self
            .current_definee()
            .map(Object::Module)
            .unwrap_or(Object::Nil);
        if let Object::Class(class) | Object::Module(class) = &current_self {
            let class = Rc::clone(class);
            if let Some(result) = self.call_class_methods(&class, name, &arguments, position)? {
                return Ok(result);
            }
        }
        Ok(Object::Nil)
    }
}
