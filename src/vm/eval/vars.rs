// Variable read expressions: SelfExpr, InstanceVariable, ClassVariable, ScopeResolution.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

use crate::vm::core::VirtualMachine;
use crate::vm::errors::undefined_self_error;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Evaluate a bare `self` expression.
    pub(super) fn eval_self(&self, position: Position) -> Result<Object, MetorexError> {
        self.environment()
            .get("self")
            .ok_or_else(|| undefined_self_error(position))
    }

    /// Evaluate an instance variable read (`@name`). Falls back to nil for
    /// undefined ivars on instances; reads class-level ivars when self is a
    /// Class or Module.
    pub(crate) fn eval_instance_var_read(
        &self,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match self.environment().get("self") {
            Some(Object::Instance(instance_rc)) => {
                let instance = instance_rc.borrow();
                Ok(instance.get_var(name).cloned().unwrap_or(Object::Nil))
            }
            Some(Object::Exception(details)) => Ok(details
                .borrow()
                .instance_vars
                .get(name)
                .cloned()
                .unwrap_or(Object::Nil)),
            Some(Object::Class(class_rc)) => Ok(class_rc
                .get_class_var(&format!("@{}", name))
                .unwrap_or(Object::Nil)),
            Some(Object::Module(module_rc)) => Ok(module_rc
                .get_class_var(&format!("@{}", name))
                .unwrap_or(Object::Nil)),
            // Immediates (Int/Float/Symbol/Bool/Nil/etc.) and other non-instance
            // selves: ivar reads return nil — matching Ruby, where ivars on
            // immediates default to nil even if no writer ever ran.
            Some(_) => Ok(Object::Nil),
            None => Err(MetorexError::runtime_error(
                format!(
                    "Instance variable @{} can only be used within a method",
                    name
                ),
                position_to_location(position),
            )),
        }
    }

    /// Evaluate a class variable read (`@@name`).
    pub(super) fn eval_class_var_read(
        &self,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match self.environment().get("self") {
            Some(Object::Instance(instance_rc)) => {
                let class = std::rc::Rc::clone(&instance_rc.borrow().class);
                Self::inherited_class_var(&class, name)
                    .ok_or_else(|| uninitialized_class_var_error(name, &class, position))
            }
            Some(Object::Class(class)) => Self::inherited_class_var(&class, name)
                .ok_or_else(|| uninitialized_class_var_error(name, &class, position)),
            Some(_) => Err(MetorexError::runtime_error(
                format!("Cannot read class variable @@{} in this context", name),
                position_to_location(position),
            )),
            None => Err(MetorexError::runtime_error(
                format!(
                    "Class variable @@{} can only be used within a class or method",
                    name
                ),
                position_to_location(position),
            )),
        }
    }

    /// A class variable as seen from `class`, which Ruby looks for up the
    /// superclass chain rather than on the one class alone.
    fn inherited_class_var(class: &std::rc::Rc<crate::class::Class>, name: &str) -> Option<Object> {
        let mut cursor = Some(std::rc::Rc::clone(class));
        while let Some(current) = cursor {
            if let Some(value) = current.get_class_var(name) {
                return Some(value);
            }
            cursor = current.superclass();
        }
        None
    }
}

/// The NameError Ruby raises for a class variable that was never assigned.
fn uninitialized_class_var_error(
    name: &str,
    class: &std::rc::Rc<crate::class::Class>,
    position: Position,
) -> MetorexError {
    let message = format!(
        "uninitialized class variable @@{} in {}",
        name,
        class.inspect_name()
    );
    let exception = Object::exception("NameError", message.clone());
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details.name = Some(format!("@@{}", name));
        details.receiver = Some(Box::new(Object::Class(std::rc::Rc::clone(class))));
    }
    MetorexError::UncaughtException {
        exception,
        location: position_to_location(position),
        message,
    }
}
