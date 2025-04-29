// Control structure execution for the Metorex VM.
// This module handles if/else, while loops, and for loops.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::utils::*;

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

impl VirtualMachine {
    /// Execute an if/else statement.
    pub(crate) fn execute_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: &Option<Vec<Statement>>,
    ) -> Result<ControlFlow, MetorexError> {
        let condition_value = self.evaluate_expression(condition)?;

        if is_truthy(&condition_value) {
            self.execute_statements_internal(then_branch)
        } else if let Some(else_stmts) = else_branch {
            self.execute_statements_internal(else_stmts)
        } else {
            Ok(ControlFlow::Next)
        }
    }

    /// Execute a while loop.
    pub(crate) fn execute_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        loop {
            let condition_value = self.evaluate_expression(condition)?;

            if !is_truthy(&condition_value) {
                break;
            }

            match self.execute_statements_internal(body)? {
                ControlFlow::Next => continue,
                ControlFlow::Break { .. } => break,
                ControlFlow::Continue { .. } => continue,
                ControlFlow::Return { value, position } => {
                    return Ok(ControlFlow::Return { value, position });
                }
                ControlFlow::Exception {
                    exception,
                    position,
                } => {
                    return Ok(ControlFlow::Exception {
                        exception,
                        position,
                    });
                }
            }
        }

        Ok(ControlFlow::Next)
    }

    /// Execute a for loop over an iterable.
    pub(crate) fn execute_for(
        &mut self,
        variable: &str,
        iterable_expr: &Expression,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        let iterable = self.evaluate_expression(iterable_expr)?;

        let elements = match iterable {
            Object::Array(array_rc) => {
                let arr = array_rc.borrow();
                arr.clone()
            }
            Object::Range {
                start,
                end,
                exclusive,
            } => {
                // Convert range to array of integers
                match (*start, *end) {
                    (Object::Int(start_val), Object::Int(end_val)) => {
                        let mut elements = Vec::new();
                        let end_inclusive = if exclusive { end_val - 1 } else { end_val };

                        if start_val <= end_inclusive {
                            for i in start_val..=end_inclusive {
                                elements.push(Object::Int(i));
                            }
                        } else {
                            // Reverse range
                            for i in (end_inclusive..=start_val).rev() {
                                elements.push(Object::Int(i));
                            }
                        }
                        elements
                    }
                    _ => {
                        return Err(MetorexError::type_error(
                            "Range bounds must be integers for iteration",
                            position_to_location(position),
                        ));
                    }
                }
            }
            other => {
                return Err(MetorexError::type_error(
                    format!(
                        "Cannot iterate over type '{}', expected Array or Range",
                        other.type_name()
                    ),
                    position_to_location(position),
                ));
            }
        };

        for element in elements {
            self.environment_mut().push_scope();
            self.environment_mut().define(variable.to_string(), element);

            let result = self.execute_statements_internal(body);

            self.environment_mut().pop_scope();

            match result? {
                ControlFlow::Next => continue,
                ControlFlow::Break { .. } => break,
                ControlFlow::Continue { .. } => continue,
                ControlFlow::Return { value, position } => {
                    return Ok(ControlFlow::Return { value, position });
                }
                ControlFlow::Exception {
                    exception,
                    position,
                } => {
                    return Ok(ControlFlow::Exception {
                        exception,
                        position,
                    });
                }
            }
        }

        Ok(ControlFlow::Next)
    }
}
