//! Begin/rescue/else/ensure evaluation for the virtual machine.
//!
//! This module handles evaluating begin/rescue blocks as expressions,
//! returning the value of the last successfully executed statement.

use super::errors::*;
use super::utils::*;
use super::{ControlFlow, VirtualMachine};
use crate::ast::Statement;
use crate::error::MetorexError;
use crate::object::Object;

impl VirtualMachine {
    /// Evaluate a Begin block as an expression, returning the value of the
    /// last successfully executed statement (in body, rescue, or else).
    pub(crate) fn evaluate_begin_value(
        &mut self,
        body: &[Statement],
        rescue_clauses: &[crate::ast::RescueClause],
        else_clause: Option<&[Statement]>,
        ensure_block: Option<&[Statement]>,
    ) -> Result<Object, MetorexError> {
        let body_result = self.execute_statements_for_value(body);

        // Convert internal RuntimeError/TypeError to a rescuable UncaughtException so that
        // `rescue Object => e` (and other rescue clauses) can catch them — mirroring Ruby's
        // behaviour where all errors are rescuable.
        let body_result = match body_result {
            Err(MetorexError::RuntimeError {
                ref message,
                ref location,
                ..
            }) => {
                let exc = Object::exception("RuntimeError", message.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: location.clone(),
                    message: message.clone(),
                })
            }
            Err(MetorexError::TypeError {
                ref message,
                ref location,
                ..
            }) => {
                let exc = Object::exception("TypeError", message.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: location.clone(),
                    message: message.clone(),
                })
            }
            other => other,
        };

        let mut final_value = body_result.clone();
        let mut handled = false;

        if let Err(MetorexError::UncaughtException { exception, .. }) = &body_result {
            self.environment_mut()
                .define("$!".to_string(), exception.clone());
            for rescue_clause in rescue_clauses {
                if self.exception_matches(exception, &rescue_clause.exception_types)? {
                    if let Some(var_name) = &rescue_clause.variable_name {
                        self.environment_mut()
                            .define(var_name.clone(), exception.clone());
                    }
                    final_value = self.execute_statements_for_value(&rescue_clause.body);
                    handled = true;
                    break;
                }
            }
            if handled {
                self.environment_mut().define("$!".to_string(), Object::Nil);
            }
        } else if body_result.is_ok()
            && let Some(else_stmts) = else_clause
        {
            final_value = self.execute_statements_for_value(else_stmts);
        }

        if let Some(ensure_stmts) = ensure_block {
            // If ensure raises (NonLocalReturn or exception), it overrides the prior result.
            self.execute_statements_for_value(ensure_stmts)?;
        }

        final_value
    }

    /// Execute statements and return the value of the last expression
    /// (similar to a method body, but without binding parameters or self).
    pub(crate) fn execute_statements_for_value(
        &mut self,
        statements: &[Statement],
    ) -> Result<Object, MetorexError> {
        let mut last_value = Object::Nil;
        for (i, statement) in statements.iter().enumerate() {
            let is_last = i == statements.len() - 1;
            if is_last {
                match statement {
                    Statement::Expression { expression, .. } => {
                        last_value = self.evaluate_expression(expression)?;
                        continue;
                    }
                    Statement::Assignment { value, target, .. } => {
                        let evaluated = self.evaluate_expression(value)?;
                        self.assign_value(target, evaluated.clone())?;
                        last_value = evaluated;
                        continue;
                    }
                    Statement::Begin {
                        body: begin_body,
                        rescue_clauses,
                        else_clause,
                        ensure_block,
                        ..
                    } => {
                        last_value = self.evaluate_begin_value(
                            begin_body,
                            rescue_clauses,
                            else_clause.as_deref(),
                            ensure_block.as_deref(),
                        )?;
                        continue;
                    }
                    _ => {}
                }
            }
            match self.execute_statement(statement)? {
                ControlFlow::Next => continue,
                ControlFlow::Value(v) => {
                    if is_last {
                        last_value = v;
                    }
                    continue;
                }
                ControlFlow::Return { value, position } => {
                    return Err(MetorexError::NonLocalReturn {
                        value,
                        location: position_to_location(position),
                    });
                }
                ControlFlow::Exception {
                    exception,
                    position,
                } => {
                    return Err(MetorexError::UncaughtException {
                        exception: exception.clone(),
                        location: position_to_location(position),
                        message: format_exception(&exception),
                    });
                }
                ControlFlow::Break { position } => {
                    return Err(loop_control_error("break", position));
                }
                ControlFlow::Continue { position } => {
                    return Err(loop_control_error("continue", position));
                }
            }
        }
        Ok(last_value)
    }
}
