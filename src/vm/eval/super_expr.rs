// `super` expression: dispatch to the same method in the parent class.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

use crate::vm::core::VirtualMachine;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Evaluate a `super` call. Walks the inheritance chain from the receiver's
    /// class to find the class defining the current method, then invokes the
    /// same-named method on that class's superclass.
    pub(super) fn eval_super(
        &mut self,
        arguments: &[Expression],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Get the current self (must be an instance)
        let instance = match self.environment().get("self") {
            Some(Object::Instance(instance_rc)) => instance_rc,
            Some(_) => {
                return Err(MetorexError::runtime_error(
                    "super can only be called from within an instance method".to_string(),
                    position_to_location(position),
                ));
            }
            None => {
                return Err(MetorexError::runtime_error(
                    "super can only be called from within a method".to_string(),
                    position_to_location(position),
                ));
            }
        };

        // Get the current method name from the call stack.
        // The call stack stores method names as "Class#method".
        let current_frame = self.get_current_method_name().ok_or_else(|| {
            MetorexError::runtime_error(
                "super called outside of a method context".to_string(),
                position_to_location(position),
            )
        })?;

        // Extract the class name and method name (format: "Class#method")
        let (class_name, method_name) = if let Some(pos) = current_frame.rfind('#') {
            (&current_frame[..pos], &current_frame[pos + 1..])
        } else {
            return Err(MetorexError::runtime_error(
                "super called in invalid context (no class information)".to_string(),
                position_to_location(position),
            ));
        };

        // Get the instance's class to walk the inheritance chain
        let instance_borrowed = instance.borrow();
        let instance_class = &instance_borrowed.class;

        // Find the class that matches the current frame's class name
        let mut current_class = Some(Rc::clone(instance_class));
        let defining_class = loop {
            match current_class {
                Some(ref class) if class.name() == class_name => {
                    break Some(Rc::clone(class));
                }
                Some(ref class) => {
                    current_class = class.superclass();
                }
                None => break None,
            }
        };

        let defining_class = defining_class.ok_or_else(|| {
            MetorexError::runtime_error(
                format!(
                    "Could not find defining class '{}' in inheritance chain",
                    class_name
                ),
                position_to_location(position),
            )
        })?;

        // Get the parent class of the defining class
        let parent_class = defining_class.superclass().ok_or_else(|| {
            MetorexError::runtime_error(
                format!("Class {} has no superclass", class_name),
                position_to_location(position),
            )
        })?;

        // Look up the method in the parent class
        let method = parent_class.find_method(method_name).ok_or_else(|| {
            MetorexError::runtime_error(
                format!(
                    "Superclass {} does not define method '{}'",
                    parent_class.name(),
                    method_name
                ),
                position_to_location(position),
            )
        })?;

        // Evaluate the arguments
        let mut evaluated_args = Vec::with_capacity(arguments.len());
        for arg in arguments {
            evaluated_args.push(self.evaluate_expression(arg)?);
        }

        // Drop the borrow before invoking the method
        drop(instance_borrowed);

        // Invoke the parent method with self as the receiver
        self.invoke_method(
            parent_class,
            method,
            Object::Instance(Rc::clone(&instance)),
            evaluated_args,
            position,
        )
    }
}
