// Mixing a module into one object's singleton class.

use super::*;

impl VirtualMachine {
    /// Extend `receiver`'s singleton class with `module_rc`, dispatching to a
    /// user-defined `extend_object` on the module's singleton class when one
    /// exists; otherwise apply the default behavior.
    pub(crate) fn apply_module_extend(
        &mut self,
        receiver: &Object,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if let Some(method) = module_rc.find_method("__class__extend_object") {
            self.invoke_method(
                Rc::clone(module_rc),
                method,
                Object::Module(Rc::clone(module_rc)),
                vec![receiver.clone()],
                position,
            )?;
        } else if let Some(sc) = module_rc.singleton_class_slot().clone()
            && let Some(method) = sc.find_method("extend_object")
        {
            self.invoke_method(
                sc,
                method,
                Object::Module(Rc::clone(module_rc)),
                vec![receiver.clone()],
                position,
            )?;
        } else {
            self.default_extend_object(receiver, module_rc, position)?;
        }
        self.invoke_extended_hook(receiver, module_rc, position)
    }

    /// Fire `mod.extended(obj)` after the object has been extended, matching
    /// Ruby's ordering of `extend_object` first, then the `extended` hook.
    fn invoke_extended_hook(
        &mut self,
        receiver: &Object,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if let Some(method) = module_rc.find_method("__class__extended") {
            self.invoke_method(
                Rc::clone(module_rc),
                method,
                Object::Module(Rc::clone(module_rc)),
                vec![receiver.clone()],
                position,
            )?;
            return Ok(());
        }
        if let Some(sc) = module_rc.singleton_class_slot().clone()
            && let Some(method) = sc.find_method("extended")
        {
            self.invoke_method(
                sc,
                method,
                Object::Module(Rc::clone(module_rc)),
                vec![receiver.clone()],
                position,
            )?;
        }
        Ok(())
    }

    /// Default `Module#extend_object(obj)` behavior: raise FrozenError when
    /// the object is frozen, otherwise add the module to the object's
    /// singleton class mixin chain.
    pub(crate) fn default_extend_object(
        &mut self,
        receiver: &Object,
        module_rc: &Rc<Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if self.object_is_frozen(receiver) {
            let class_name = self.builtins().class_of(receiver).name().to_string();
            let msg = format!("can't modify frozen {}", class_name);
            let exc = Object::exception("FrozenError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        let singleton = self.singleton_class_of(receiver);
        singleton.add_mixin(Rc::clone(module_rc));
        Ok(())
    }
}
