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
        trailing_block: Option<&Expression>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let receiver = self.evaluate_expression(receiver_expr)?;
        let mut arguments = Vec::with_capacity(argument_exprs.len());
        for argument in argument_exprs {
            arguments.push(self.evaluate_expression(argument)?);
        }

        // If there's a trailing block, evaluate it and append to arguments
        if let Some(block_expr) = trailing_block {
            let block_obj = self.evaluate_expression(block_expr)?;
            arguments.push(block_obj);
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
                    // Try method_missing as a final fallback
                    if let Some((method_missing_class, method_missing_method)) =
                        self.lookup_method(&receiver, "method_missing")
                    {
                        // Call method_missing with the method name as a string argument
                        let method_name_obj = Object::String(Rc::new(method_name.to_string()));
                        self.invoke_method(
                            method_missing_class,
                            method_missing_method,
                            receiver,
                            vec![method_name_obj],
                            position,
                        )
                    } else {
                        Err(undefined_method_error(method_name, &receiver, position))
                    }
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
