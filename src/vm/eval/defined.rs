// `defined?` introspection: returns a description string or nil.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::object::Object;
use std::rc::Rc;

use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// Evaluate `defined?(expr)`. Returns a description string for defined
    /// expressions, or `nil` for undefined ones.
    pub(super) fn eval_defined(&mut self, expression: &Expression) -> Result<Object, MetorexError> {
        let result = match expression {
            Expression::Identifier { name, .. } => match self.environment().get(name) {
                Some(Object::Method(_)) => Some("method"),
                Some(Object::Class(_)) | Some(Object::Module(_)) => Some("constant"),
                Some(_) => Some("local-variable"),
                None => {
                    if self.globals().contains(name) {
                        Some("method")
                    } else {
                        None
                    }
                }
            },
            Expression::GlobalVariable { name, .. } => {
                if self.globals().get(name).is_some_and(|v| v != Object::Nil) {
                    Some("global-variable")
                } else {
                    None
                }
            }
            Expression::InstanceVariable { name, .. } => {
                // Instance vars are stored without @ prefix
                match self.environment().get("self") {
                    Some(Object::Instance(inst)) => {
                        if inst.borrow().get_var(name).is_some() {
                            Some("instance-variable")
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Expression::ClassVariable { .. } => {
                if self.evaluate_expression(expression).is_ok() {
                    Some("class variable")
                } else {
                    None
                }
            }
            Expression::ScopeResolution { .. } => {
                if self.evaluate_expression(expression).is_ok() {
                    Some("constant")
                } else {
                    None
                }
            }
            Expression::MethodCall { receiver, .. } => {
                // Only "defined" if the receiver can be evaluated without error.
                if self.evaluate_expression(receiver).is_ok() {
                    Some("method")
                } else {
                    None
                }
            }
            Expression::Call { .. } => {
                if self.evaluate_expression(expression).is_ok() {
                    Some("method")
                } else {
                    None
                }
            }
            Expression::Index { array, .. } => {
                if self.evaluate_expression(array).is_ok() {
                    Some("method")
                } else {
                    None
                }
            }
            Expression::Yield { .. } => {
                if self.environment().get("__block__").is_some() {
                    Some("yield")
                } else {
                    None
                }
            }
            // Literals are always defined
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::NilLiteral { .. }
            | Expression::Symbol { .. }
            | Expression::Array { .. }
            | Expression::Dictionary { .. }
            | Expression::RegexLiteral { .. } => Some("expression"),
            Expression::Super { .. } => Some("super"),
            Expression::SelfExpr { .. } => Some("self"),
            // For anything else, try evaluating and check
            _ => {
                if self.evaluate_expression(expression).is_ok() {
                    Some("expression")
                } else {
                    None
                }
            }
        };
        match result {
            Some(desc) => Ok(Object::String(Rc::new(desc.to_string()))),
            None => Ok(Object::Nil),
        }
    }
}
