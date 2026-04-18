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
        forward_args: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
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
            (
                current_frame[..pos].to_string(),
                current_frame[pos + 1..].to_string(),
            )
        } else {
            return Err(MetorexError::runtime_error(
                "super called in invalid context (no class information)".to_string(),
                position_to_location(position),
            ));
        };

        // Class-method super: `def self.foo; super; end` — walk self's
        // ancestor chain looking for a matching *class* method (stored under
        // the `__class__` prefix or on the class's singleton class). When the
        // method was mixed in (e.g. `class << F; include M; end`), the defining
        // class may not be self's class itself; walking from self's immediate
        // superclass handles that case.
        if let Some(Object::Class(self_class)) = self.environment().get("self") {
            let evaluated_args = if forward_args {
                self.method_arg_stack.last().cloned().unwrap_or_default()
            } else {
                let mut evaluated_args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }
                evaluated_args
            };
            let class_method_key = format!("__class__{}", method_name);
            let mut current = self_class.superclass();
            while let Some(cls) = current {
                let candidate = cls
                    .singleton_class_slot()
                    .clone()
                    .and_then(|sc| sc.find_method(&method_name))
                    .or_else(|| cls.find_method(&class_method_key));
                if let Some(method) = candidate {
                    return self.invoke_method(
                        Rc::clone(&cls),
                        method,
                        Object::Class(self_class),
                        evaluated_args,
                        position,
                    );
                }
                current = cls.superclass();
            }
            // No superclass class method. Ruby-level `Class#inherited` is a
            // silent no-op on Object, so we mirror that for the `inherited`
            // hook only; any other unresolved class-method super is an error.
            if method_name == "inherited" {
                return Ok(Object::Nil);
            }
            return Err(MetorexError::runtime_error(
                format!(
                    "super: no superclass method '{}' for {}",
                    method_name,
                    self_class.ruby_name()
                ),
                position_to_location(position),
            ));
        }

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

        // Get the parent class of the defining class. If there's no superclass,
        // fall back to Object semantics for well-known methods.
        let parent_class = match defining_class.superclass() {
            Some(p) => p,
            None => {
                // Object#<=> returns 0 if self.equal?(other), else nil
                if method_name == "<=>" {
                    drop(instance_borrowed);
                    let mut evaluated_args = Vec::with_capacity(arguments.len());
                    for arg in arguments {
                        evaluated_args.push(self.evaluate_expression(arg)?);
                    }
                    let self_val = self.environment().get("self").unwrap_or(Object::Nil);
                    if let Some(other) = evaluated_args.first() {
                        let same = matches!(
                            (&self_val, other),
                            (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b)
                        );
                        return Ok(if same { Object::Int(0) } else { Object::Nil });
                    }
                    return Ok(Object::Nil);
                }
                return Err(MetorexError::runtime_error(
                    format!("Class {} has no superclass", class_name),
                    position_to_location(position),
                ));
            }
        };

        // Look up the method in the parent class
        let method = parent_class.find_method(&method_name).ok_or_else(|| {
            MetorexError::runtime_error(
                format!(
                    "Superclass {} does not define method '{}'",
                    parent_class.name(),
                    method_name
                ),
                position_to_location(position),
            )
        })?;

        // Evaluate the arguments (or forward the enclosing method's args for
        // bare `super`).
        let evaluated_args = if forward_args {
            self.method_arg_stack.last().cloned().unwrap_or_default()
        } else {
            let mut evaluated_args = Vec::with_capacity(arguments.len());
            for arg in arguments {
                evaluated_args.push(self.evaluate_expression(arg)?);
            }
            evaluated_args
        };

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
