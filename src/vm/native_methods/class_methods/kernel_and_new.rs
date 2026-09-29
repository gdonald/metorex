// Kernel as a module, and the classes that build classes.

use super::*;

impl VirtualMachine {
    /// The Kernel module methods, and `Class.new` / `Module.new`.
    pub(crate) fn call_kernel_and_new_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        if class_rc.name() == "Kernel"
            && let Some(result) = self.call_kernel_conversion(method_name, arguments, position)?
        {
            return Ok(Answered(result));
        }
        if class_rc.name() == "Kernel" && method_name == "abort" {
            return self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Answered);
        }
        // `Kernel.block_given?` reports on the frame that called it, the same
        // as the bare form.
        if class_rc.name() == "Kernel" && method_name == "block_given?" {
            return Ok(Answered(Object::Bool(matches!(
                self.environment().get("block_given?"),
                Some(Object::Bool(true))
            ))));
        }
        if class_rc.name() == "Kernel" && method_name == "binding" {
            return self
                .call_native_function("binding_kernel", arguments.to_vec(), position)
                .map(Answered);
        }
        if method_name == "new" && class_rc.name() == "Class" {
            let superclass = match arguments.first() {
                Some(Object::Class(c)) => {
                    // Singleton (meta) classes can't be used as a superclass —
                    // MRI raises TypeError("can't make subclass of singleton
                    // class") in this case.
                    if c.get_class_var("__singleton__").is_some() {
                        return Err(MetorexError::type_error(
                            "can't make subclass of singleton class",
                            position_to_location(position),
                        ));
                    }
                    Some(Rc::clone(c))
                }
                Some(other) => {
                    return Err(MetorexError::type_error(
                        format!("superclass must be a Class (given {})", other.type_name()),
                        position_to_location(position),
                    ));
                }
                None => self.globals().get("Object").and_then(|o| {
                    if let Object::Class(c) = o {
                        Some(c)
                    } else {
                        None
                    }
                }),
            };
            let anon = Class::new("", superclass.clone());
            if let Some(sc) = &superclass {
                sc.add_subclass(&anon);
            }
            // Extract the pending block before triggering the `inherited`
            // hook — the hook's own invoke_method would otherwise consume it.
            let pending = self.pending_block.take();
            // Ruby: `inherited` hook fires before the block runs, so a hook
            // that records `self` sees the parent first, then the block's
            // `self` push appends the subclass.
            if let Some(sc) = &superclass {
                self.trigger_inherited_hook(sc, Rc::clone(&anon), position)?;
            }
            if let Some(Object::Block(block)) = pending {
                self.apply_block_as_class_body(&anon, &block, position)?;
            }
            return Ok(Answered(Object::Class(anon)));
        }
        if method_name == "new" && class_rc.name() == "Module" {
            let anon = Class::new_module("");
            if let Some(Object::Block(block)) = self.pending_block.take() {
                self.apply_block_as_class_body_with_self(
                    &anon,
                    &block,
                    position,
                    Object::Module(Rc::clone(&anon)),
                )?;
            }
            return Ok(Answered(Object::Module(anon)));
        }
        Ok(Unclaimed)
    }
}
