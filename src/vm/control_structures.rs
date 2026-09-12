// Control structure execution for the Metorex VM.
// This module handles if/else, while loops, and for loops.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::utils::*;

use crate::ast::{ElsifBranch, Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::Position;

impl VirtualMachine {
    /// Execute an if/elsif/else statement.
    pub(crate) fn execute_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        elsif_branches: &[ElsifBranch],
        else_branch: &Option<Vec<Statement>>,
    ) -> Result<ControlFlow, MetorexError> {
        let condition_value = self.evaluate_expression(condition)?;

        if is_truthy(&condition_value) {
            self.execute_statements_internal(then_branch)
        } else {
            // Try each elsif branch
            for elsif in elsif_branches {
                let elsif_condition_value = self.evaluate_expression(&elsif.condition)?;
                if is_truthy(&elsif_condition_value) {
                    return self.execute_statements_internal(&elsif.body);
                }
            }

            // If no elsif matched, try else branch
            if let Some(else_stmts) = else_branch {
                self.execute_statements_internal(else_stmts)
            } else {
                Ok(ControlFlow::Next)
            }
        }
    }

    /// Execute an unless statement (inverted if).
    pub(crate) fn execute_unless(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: &Option<Vec<Statement>>,
    ) -> Result<ControlFlow, MetorexError> {
        let condition_value = self.evaluate_expression(condition)?;

        if !is_truthy(&condition_value) {
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

            // `redo` runs the body again without reading the condition.
            let mut pass = self.loop_pass(body)?;
            while matches!(pass, ControlFlow::Redo { .. }) {
                pass = self.loop_pass(body)?;
            }
            match pass {
                ControlFlow::Next | ControlFlow::Value(_) => continue,
                // A loop answers nil, except when a `break` carried a value
                // out of it, which is what the loop then answers.
                ControlFlow::Break { value, .. } => return Ok(ControlFlow::Value(value)),
                ControlFlow::Redo { .. }
                | ControlFlow::Retry { .. }
                | ControlFlow::Continue { .. } => continue,
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

    /// `begin ... end while cond`, which runs its body once before it first
    /// reads the condition.
    pub(crate) fn execute_do_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        loop {
            // `redo` runs the body again without reading the condition.
            loop {
                match self.loop_pass(body)? {
                    ControlFlow::Redo { .. } | ControlFlow::Retry { .. } => continue,
                    ControlFlow::Next | ControlFlow::Value(_) | ControlFlow::Continue { .. } => {
                        break;
                    }
                    ControlFlow::Break { value, .. } => return Ok(ControlFlow::Value(value)),
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
            let tested = self.evaluate_expression(condition)?;
            if !is_truthy(&tested) {
                return Ok(ControlFlow::Value(crate::object::Object::Nil));
            }
        }
    }

    /// One turn through a loop body. A `next`, `redo`, or `break` written
    /// where a value goes arrives as an unwinding signal rather than as plain
    /// control flow, and the loop reads it as the jump it stands for.
    fn loop_pass(&mut self, body: &[Statement]) -> Result<ControlFlow, MetorexError> {
        match self.execute_statements_internal(body) {
            Ok(flow) => Ok(flow),
            Err(MetorexError::BlockNext { .. }) => Ok(ControlFlow::Next),
            Err(MetorexError::BlockRedo { .. }) => Ok(ControlFlow::Redo {
                position: Position::default(),
            }),
            // A `break` written where a value goes unwinds as a signal. One
            // that has passed a block boundary already carries the frame it
            // belongs to and is on its way somewhere else, so only a signal
            // raised inside this body is the loop's own.
            Err(MetorexError::BlockBreak {
                value,
                home_frame: None,
                ..
            }) => Ok(ControlFlow::Break {
                value,
                position: Position::default(),
            }),
            Err(other) => Err(other),
        }
    }

    /// Execute a for loop over an iterable.
    pub(crate) fn execute_for(
        &mut self,
        variable: &str,
        iterable_expr: &Expression,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        let looped = crate::ast::for_loop::for_over_each(
            vec![Expression::Identifier {
                name: variable.to_string(),
                position,
            }],
            false,
            iterable_expr.clone(),
            body.to_vec(),
            position,
        );
        self.execute_statement(&looped)
    }
}
