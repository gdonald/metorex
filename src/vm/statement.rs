// Core statement execution logic for the Metorex VM.
// This module handles the dispatcher and basic statement execution.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::errors::*;
use super::utils::*;

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::object::Object;

impl VirtualMachine {
    /// Evaluate a statement and produce control-flow information for the caller.
    pub(crate) fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, MetorexError> {
        match statement {
            Statement::Expression { expression, .. } => {
                self.evaluate_expression(expression)?;
                Ok(ControlFlow::Next)
            }
            Statement::Assignment {
                target,
                value,
                position: _,
            } => {
                let evaluated = self.evaluate_expression(value)?;
                self.assign_value(target, evaluated)?;
                Ok(ControlFlow::Next)
            }
            Statement::Return { value, position } => {
                let result = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Object::Nil,
                };
                Ok(ControlFlow::Return {
                    value: result,
                    position: *position,
                })
            }
            Statement::Break { position } => Ok(ControlFlow::Break {
                position: *position,
            }),
            Statement::Continue { position } => Ok(ControlFlow::Continue {
                position: *position,
            }),
            Statement::Block {
                statements,
                position: _,
            } => self.execute_block(statements),
            Statement::If {
                condition,
                then_branch,
                else_branch,
                position: _,
            } => self.execute_if(condition, then_branch, else_branch),
            Statement::While {
                condition,
                body,
                position: _,
            } => self.execute_while(condition, body),
            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => self.execute_for(variable, iterable, body, *position),
            Statement::ClassDef {
                name,
                superclass,
                body,
                position,
            } => self.execute_class_def(name, superclass.as_deref(), body, *position),
            Statement::MethodDef { .. } => {
                // MethodDef should only appear inside ClassDef bodies, not at top level
                Err(unimplemented_statement_error(statement))
            }
            Statement::Begin {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                position,
            } => self.execute_begin(body, rescue_clauses, else_clause, ensure_block, *position),
            Statement::Raise {
                exception,
                position,
            } => self.execute_raise(exception, *position),
            Statement::Match {
                expression,
                cases,
                position,
            } => self.execute_match(expression, cases, *position),
            Statement::FunctionDef {
                name,
                parameters,
                body,
                position: _,
            } => self.execute_function_def(name, parameters, body),
        }
    }

    /// Execute statements within a new lexical scope.
    pub(crate) fn execute_block(
        &mut self,
        statements: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        self.environment_mut().push_scope();
        let result = self.execute_statements_internal(statements);
        self.environment_mut().pop_scope();
        result
    }

    /// Core statement execution loop used by program and block execution.
    pub(crate) fn execute_statements_internal(
        &mut self,
        statements: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Next => continue,
                flow => return Ok(flow),
            }
        }
        Ok(ControlFlow::Next)
    }

    /// Assign a value to the given target expression.
    pub(crate) fn assign_value(
        &mut self,
        target: &Expression,
        value: Object,
    ) -> Result<(), MetorexError> {
        match target {
            Expression::Identifier { name, .. } => {
                if !self.environment_mut().set(name, value.clone()) {
                    self.environment_mut().define(name.clone(), value);
                }
                Ok(())
            }
            Expression::InstanceVariable { name, position } => {
                // Instance variables can only be set within a method (where 'self' is defined)
                match self.environment().get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let mut instance = instance_rc.borrow_mut();
                        instance.set_var(name.clone(), value);
                        Ok(())
                    }
                    Some(_) => Err(MetorexError::runtime_error(
                        format!("Cannot set instance variable @{} on non-instance", name),
                        position_to_location(*position),
                    )),
                    None => Err(MetorexError::runtime_error(
                        format!(
                            "Instance variable @{} can only be used within a method",
                            name
                        ),
                        position_to_location(*position),
                    )),
                }
            }
            Expression::ClassVariable { name, position } => {
                // Class variables can only be set within a method or class context
                // For now, we'll look for 'self' to get the class
                match self.environment().get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let instance = instance_rc.borrow();
                        instance.class.set_class_var(name.clone(), value);
                        Ok(())
                    }
                    Some(Object::Class(class)) => {
                        class.set_class_var(name.clone(), value);
                        Ok(())
                    }
                    Some(_) => Err(MetorexError::runtime_error(
                        format!("Cannot set class variable @@{} in this context", name),
                        position_to_location(*position),
                    )),
                    None => Err(MetorexError::runtime_error(
                        format!(
                            "Class variable @@{} can only be used within a class or method",
                            name
                        ),
                        position_to_location(*position),
                    )),
                }
            }
            Expression::Index { .. } => Err(invalid_assignment_target_error(target)),
            _ => Err(invalid_assignment_target_error(target)),
        }
    }
}
