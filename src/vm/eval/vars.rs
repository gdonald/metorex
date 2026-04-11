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
    pub(super) fn eval_instance_var_read(
        &self,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match self.environment().get("self") {
            Some(Object::Instance(instance_rc)) => {
                let instance = instance_rc.borrow();
                Ok(instance.get_var(name).cloned().unwrap_or(Object::Nil))
            }
            Some(Object::Class(class_rc)) => Ok(class_rc
                .get_class_var(&format!("@{}", name))
                .unwrap_or(Object::Nil)),
            Some(Object::Module(module_rc)) => Ok(module_rc
                .get_class_var(&format!("@{}", name))
                .unwrap_or(Object::Nil)),
            Some(_) => Err(MetorexError::runtime_error(
                format!("Cannot read instance variable @{} on non-instance", name),
                position_to_location(position),
            )),
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
                let instance = instance_rc.borrow();
                Ok(instance.class.get_class_var(name).unwrap_or(Object::Nil))
            }
            Some(Object::Class(class)) => Ok(class.get_class_var(name).unwrap_or(Object::Nil)),
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
}
