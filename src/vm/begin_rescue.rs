//! Begin/rescue/else/ensure evaluation for the virtual machine.
//!
//! This module handles evaluating begin/rescue blocks as expressions,
//! returning the value of the last successfully executed statement.

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
        // `retry` in a rescue body runs the whole begin body again, which is
        // what this loop is for.
        loop {
            match self.begin_once(body, rescue_clauses, else_clause, ensure_block) {
                Err(MetorexError::BlockRetry { .. }) => continue,
                other => return other,
            }
        }
    }

    /// One pass over a begin body and whichever clause answers for it.
    fn begin_once(
        &mut self,
        body: &[Statement],
        rescue_clauses: &[crate::ast::RescueClause],
        else_clause: Option<&[Statement]>,
        ensure_block: Option<&[Statement]>,
    ) -> Result<Object, MetorexError> {
        // What `$!` named before this form ran, which a clause handling an
        // exception of its own puts back when it is done.
        let standing = self.globals().get("!").unwrap_or(Object::Nil);
        let body_result = self.execute_statements_for_value(body);

        // An internal RuntimeError or TypeError is rescuable the way Ruby's
        // own are, so `rescue Object => e` and the other clauses catch it.
        let body_result = body_result.map_err(as_rescuable);

        let mut final_value = body_result.clone();
        let mut handled = false;

        if let Err(MetorexError::UncaughtException {
            exception,
            location,
            ..
        }) = &body_result
        {
            self.note_exception_location(exception, location);
            self.set_current_exception(exception.clone());
            for rescue_clause in rescue_clauses {
                if self.rescue_clause_matches(rescue_clause, exception, rescue_clause.position)? {
                    // A trace sees the clause take the exception.
                    self.fire_event(
                        "rescue",
                        rescue_clause.position,
                        vec![("raised_exception", exception.clone())],
                    )?;
                    // The name is assigned the way `=` assigns it, so a
                    // variable of an enclosing scope is the one set.
                    if let Some(var_name) = &rescue_clause.variable_name
                        && !self.environment_mut().set(var_name, exception.clone())
                    {
                        self.environment_mut()
                            .define(var_name.clone(), exception.clone());
                    }
                    // `rescue E => held.error` stores the exception wherever
                    // the target names, the way an assignment would.
                    if let Some(target) = &rescue_clause.variable_target {
                        self.assign_value(target, exception.clone())?;
                    }
                    final_value = self.execute_statements_for_value(&rescue_clause.body);
                    // An exception raised by the rescue body takes the one
                    // being handled as its cause.
                    if let Err(MetorexError::UncaughtException {
                        exception: raised, ..
                    }) = &final_value
                    {
                        Self::record_cause(raised, exception);
                    }
                    handled = true;
                    break;
                }
            }
            // A clause that raised leaves `$!` naming what it raised, which
            // an ensure clause and an enclosing rescue see.
            match &final_value {
                Err(MetorexError::UncaughtException {
                    exception: raised, ..
                }) if handled => self.set_current_exception(raised.clone()),
                _ if handled => self.restore_current_exception(standing.clone()),
                _ => {}
            }
        } else if body_result.is_ok()
            && let Some(else_stmts) = else_clause
        {
            final_value = self.execute_statements_for_value(else_stmts);
        }

        if let Some(ensure_stmts) = ensure_block {
            // If ensure raises (NonLocalReturn or exception), it overrides the prior result.
            if let Err(leaving) = self.execute_statements_for_value(ensure_stmts) {
                // An ensure clause that leaves by `return`, `break` or `next`
                // drops the exception in flight, and `$!` goes back to what
                // it named before.
                if final_value.is_err()
                    && !matches!(leaving, MetorexError::UncaughtException { .. })
                {
                    self.restore_current_exception(standing);
                }
                return Err(leaving);
            }
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
            if is_last && let Some(value) = self.terminal_statement_value(statement)? {
                // The last statement answered here rather than through
                // `execute_statement`, so it is counted here too.
                if self.coverage.is_some() {
                    self.coverage_count(statement.position().line);
                }
                last_value = value;
                continue;
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
                        home_frame: self.current_method_frame,
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
                ControlFlow::Break { value, position } => {
                    // Bubble `break` out as a BlockBreak signal so the
                    // enclosing iterator (the block-form `each`, etc.) sees
                    // it and unwinds. Without this, `break` inside a
                    // rescue/ensure body would either become a hard error
                    // or be silently swallowed by the begin-as-expression
                    // wrapper.
                    return Err(self.break_signal(value, position_to_location(position)));
                }
                // `retry` unwinds to the `begin` whose rescue body it sits
                // in, which runs that body again.
                ControlFlow::Retry { position } => {
                    return Err(MetorexError::BlockRetry {
                        location: position_to_location(position),
                    });
                }
                ControlFlow::Redo { position } => {
                    // Like `break` above, `redo` and `next` written where a
                    // value goes unwind to the enclosing body rather than
                    // being swallowed by the begin-as-expression wrapper.
                    return Err(MetorexError::BlockRedo {
                        location: position_to_location(position),
                    });
                }
                ControlFlow::Continue { value, position } => {
                    return Err(MetorexError::BlockNext {
                        value,
                        location: position_to_location(position),
                    });
                }
            }
        }
        Ok(last_value)
    }
}

/// `error` as an exception Ruby code can rescue: the interpreter's own
/// RuntimeError and TypeError become the exceptions of those classes, and
/// every other error stays as it is.
pub(crate) fn as_rescuable(error: MetorexError) -> MetorexError {
    let (class_name, message, location) = match error {
        MetorexError::RuntimeError {
            message, location, ..
        } => ("RuntimeError", message, location),
        MetorexError::TypeError {
            message, location, ..
        } => ("TypeError", message, location),
        other => return other,
    };
    MetorexError::UncaughtException {
        exception: Object::exception(class_name, message.clone()),
        location,
        message,
    }
}
