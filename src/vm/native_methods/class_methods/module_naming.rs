// The name a module reports, and the two classes whose `new`
// answers something other than a fresh instance.

use super::*;

impl VirtualMachine {
    /// `Proc.new`, `Exception.exception`, and the name a module reports.
    pub(crate) fn call_module_naming_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        match method_name {
            // `Proc.new { ... }` yields the block itself — there is no
            // separate instance to build.
            "new" if Rc::ptr_eq(class_rc, &self.builtins().proc_class) => {
                if let Some(block) = self.pending_block.take() {
                    return Ok(Answered(block));
                }
                let msg = "tried to create Proc object without a block";
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", msg.to_string()),
                    location: position_to_location(position),
                    message: msg.to_string(),
                });
            }
            // `Exception.exception` is another name for `new`.
            "new" | "exception" if method_name == "new" || self.is_exception_class(class_rc) => {
                return self
                    .invoke_callable(
                        Object::Class(Rc::clone(class_rc)),
                        arguments.to_vec(),
                        position,
                    )
                    .map(Answered);
            }
            "name" => {
                // A singleton class has no name of its own, even though it
                // displays as `#<Class:Something>`.
                let name = class_rc.ruby_name();
                if name.is_empty() || class_rc.is_singleton_class() {
                    return Ok(Answered(Object::Nil));
                }
                // A class answers one name object, not a fresh string each
                // time it is asked.
                let slot = format!("__class_name_{:p}_{}", Rc::as_ptr(class_rc), name);
                return Ok(Answered(self.memoized_text(&slot, &name)));
            }
            _ => {}
        }
        Ok(Unclaimed)
    }
}
