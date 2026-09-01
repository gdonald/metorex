// `defined?` introspection: returns a description string or nil.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::object::Object;
use std::rc::Rc;

use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// Evaluate `defined?(expr)`. Returns a description string for defined
    /// expressions, or `nil` for undefined ones.
    pub(crate) fn eval_defined(&mut self, expression: &Expression) -> Result<Object, MetorexError> {
        let result = match expression {
            // A constant named inside a class that does not descend from
            // Object is undefined there, however the top level binds it.
            Expression::Identifier { name, .. }
                if name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    && !self.lexical_scope_reaches_top_level() =>
            {
                let found = self
                    .def_scope_stack
                    .iter()
                    .rev()
                    .any(|enclosing| enclosing.get_class_var(name).is_some());
                if found { Some("constant") } else { None }
            }
            Expression::Identifier { name, .. } => match self.environment().get(name) {
                Some(Object::Method(_)) => Some("method"),
                Some(Object::Class(_)) | Some(Object::Module(_)) => Some("constant"),
                Some(_) => Some("local-variable"),
                None => {
                    // Walk lexically enclosing class/module bodies for a
                    // matching constant or autoload registration. Mirrors
                    // identifier evaluation but reports without triggering
                    // the autoload — `defined?` does not run it.
                    let mut found_constant = false;
                    let scopes: Vec<_> = self.def_scope_stack.iter().rev().cloned().collect();
                    for enclosing in scopes {
                        if enclosing.get_class_var(name).is_some()
                            || self.effective_autoload(&enclosing, name).is_some()
                        {
                            found_constant = true;
                            break;
                        }
                    }
                    if found_constant {
                        Some("constant")
                    } else if self.globals().contains(name) {
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
            Expression::ScopeResolution {
                namespace, name, ..
            } => {
                // Check the namespace exists, then look up the constant
                // *without* triggering autoload (Ruby's `defined?` does NOT
                // run the autoload — it just reports whether the entry is
                // registered, autoload included).
                match self.evaluate_expression(namespace) {
                    Ok(Object::Class(c)) | Ok(Object::Module(c)) => {
                        if c.get_class_var(name).is_some()
                            || self.effective_autoload(&c, name).is_some()
                        {
                            Some("constant")
                        } else {
                            None
                        }
                    }
                    _ => None,
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
