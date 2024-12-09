//! Method lookup and dispatch for the virtual machine.
//!
//! This module handles resolving method calls on receiver objects and dispatching
//! to the appropriate method implementation.

use super::VirtualMachine;
use super::errors::*;
use crate::ast::Expression;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use std::rc::Rc;

impl VirtualMachine {
    /// Evaluate a method call expression on a receiver object.
    pub(crate) fn evaluate_method_call(
        &mut self,
        receiver_expr: &Expression,
        method_name: &str,
        argument_exprs: &[Expression],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let receiver = self.evaluate_expression(receiver_expr)?;
        let mut arguments = Vec::with_capacity(argument_exprs.len());
        for argument in argument_exprs {
            arguments.push(self.evaluate_expression(argument)?);
        }

        match self.lookup_method(&receiver, method_name) {
            Some((class, method)) => {
                self.invoke_method(class, method, receiver, arguments, position)
            }
            None => {
                // Try native method as fallback
                let class = self.builtins().class_of(&receiver);
                if let Some(result) =
                    self.call_native_method(&class, &receiver, method_name, &arguments, position)?
                {
                    Ok(result)
                } else {
                    Err(undefined_method_error(method_name, &receiver, position))
                }
            }
        }
    }

    /// Look up a method on the receiver and return its class and method definition.
    pub(crate) fn lookup_method(
        &self,
        receiver: &Object,
        method_name: &str,
    ) -> Option<(Rc<Class>, Rc<Method>)> {
        match receiver {
            Object::Instance(instance_rc) => {
                let instance_ref = instance_rc.borrow();
                let class = Rc::clone(&instance_ref.class);
                drop(instance_ref);
                class.find_method(method_name).map(|method| (class, method))
            }
            Object::Class(class_rc) => class_rc
                .find_method(method_name)
                .map(|method| (Rc::clone(class_rc), method)),
            _ => {
                let class = self.builtins().class_of(receiver);
                class.find_method(method_name).map(|method| (class, method))
            }
        }
    }
}
