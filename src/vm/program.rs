// Top-level program execution and the expression-evaluation entrypoint.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::errors::*;
use super::utils::*;

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{BlockStatement, Object};

/// Build a `BlockStatement` for `&:symbol` (symbol-to-proc): a one-arg block
/// `|x| x.send(:symbol)`.
fn symbol_to_proc_block(sym: &str) -> BlockStatement {
    let pos = Position::new(0, 0, 0);
    let body = vec![Statement::Expression {
        expression: Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "x".to_string(),
                position: pos,
            }),
            method: "send".to_string(),
            arguments: vec![Expression::Symbol {
                value: sym.to_string(),
                position: pos,
            }],
            trailing_block: None,
            position: pos,
        },
        position: pos,
    }];
    BlockStatement::new(
        vec!["x".to_string()],
        body,
        std::collections::HashMap::new(),
    )
}

impl VirtualMachine {
    /// Execute a sequence of statements and return an optional result (from return statements).
    pub fn execute_program(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        let mut last_value = None;

        for statement in statements {
            // If it's an expression statement, track its value
            if let Statement::Expression {
                expression,
                position,
            } = statement
            {
                let result = self.evaluate_expression(expression)?;

                // Ruby-style auto-call: if expression statement evaluates to a Method
                // and the expression is a bare identifier, auto-call it with zero args
                if matches!(expression, Expression::Identifier { .. })
                    && matches!(result, Object::Method(_))
                {
                    last_value = Some(self.invoke_callable(result, vec![], *position)?);
                    continue;
                }

                last_value = Some(result);
                continue;
            }

            // Match/CaseIn statements also produce values
            if matches!(
                statement,
                Statement::Match { .. } | Statement::CaseIn { .. }
            ) {
                match self.execute_statement(statement)? {
                    ControlFlow::Return { value, .. } | ControlFlow::Value(value) => {
                        last_value = Some(value);
                        continue;
                    }
                    ControlFlow::Next => {}
                    ControlFlow::Exception {
                        exception,
                        position,
                    } => {
                        return Err(MetorexError::runtime_error(
                            format!("Uncaught exception: {}", format_exception(&exception)),
                            position_to_location(position),
                        ));
                    }
                    ControlFlow::Break { position, .. } => {
                        return Err(loop_control_error("break", position));
                    }
                    ControlFlow::Redo { position } => {
                        return Err(loop_control_error("redo", position));
                    }
                    ControlFlow::Continue { position, .. } => {
                        return Err(loop_control_error("continue", position));
                    }
                }
                continue;
            }

            // Execute other statements
            match self.execute_statement(statement)? {
                ControlFlow::Next => {}
                ControlFlow::Value(value) => {
                    last_value = Some(value);
                }
                ControlFlow::Return { value, .. } => return Ok(Some(value)),
                ControlFlow::Exception {
                    exception,
                    position,
                } => {
                    return Err(MetorexError::runtime_error(
                        format!("Uncaught exception: {}", format_exception(&exception)),
                        position_to_location(position),
                    ));
                }
                ControlFlow::Break { position, .. } => {
                    return Err(loop_control_error("break", position));
                }
                ControlFlow::Redo { position } => {
                    return Err(loop_control_error("redo", position));
                }
                ControlFlow::Continue { position, .. } => {
                    return Err(loop_control_error("continue", position));
                }
            }
        }

        Ok(last_value)
    }

    /// Evaluate a list of argument expressions, expanding any splat (`*expr`)
    /// arguments and routing block-arg (`&expr`) arguments to `pending_block`.
    pub(crate) fn evaluate_arguments(
        &mut self,
        argument_exprs: &[Expression],
    ) -> Result<Vec<Object>, MetorexError> {
        let mut args = Vec::with_capacity(argument_exprs.len());
        for arg in argument_exprs {
            match arg {
                Expression::Splat { expression, .. } => {
                    let value = self.evaluate_expression(expression)?;
                    match value {
                        Object::Array(arr) => {
                            args.extend(arr.borrow().iter().cloned());
                        }
                        other => {
                            // Non-array splat: treat as single argument
                            args.push(other);
                        }
                    }
                }
                Expression::KeywordSplat { expression, .. } => {
                    // `**hash`: an empty Hash contributes no argument, so
                    // `f(**{})` calls `f` with nothing at all.
                    let value = self.evaluate_expression(expression)?;
                    match &value {
                        Object::Dict(entries) if entries.borrow().is_empty() => {}
                        Object::Dict(entries) => {
                            let mut keywords = entries.borrow().clone();
                            keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
                            args.push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                                keywords,
                            ))));
                        }
                        _ => args.push(value),
                    }
                }
                Expression::BlockArg { expression, .. } => {
                    // `&expr`: bind the value as the pending block. If the
                    // value is nil, the call is treated as if no block were
                    // given (the arg is dropped, not pushed).
                    let value = self.evaluate_expression(expression)?;
                    match value {
                        Object::Nil => {}
                        Object::Block(_) => {
                            self.pending_block = Some(value);
                            self.pending_block_from_ampersand = true;
                        }
                        Object::Symbol(sym) => {
                            // `&:method` is symbol-to-proc: synthesize a block
                            // `|x| x.send(:method)`. The block has no captured
                            // vars and a one-statement body that calls .send
                            // on the parameter.
                            self.pending_block =
                                Some(Object::Block(std::rc::Rc::new(symbol_to_proc_block(&sym))));
                            self.pending_block_from_ampersand = true;
                        }
                        other => {
                            // Non-block, non-nil &arg: push as positional so
                            // the existing trailing-block-extraction in
                            // `invoke_method` can pick it up if it happens to
                            // be a Method/Proc that the user wants to coerce.
                            args.push(other);
                        }
                    }
                }
                _ => {
                    args.push(self.evaluate_expression(arg)?);
                }
            }
        }
        Ok(args)
    }

    /// Evaluate an expression to a runtime value.
    pub(crate) fn evaluate_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<Object, MetorexError> {
        // Guard against infinite recursion
        use std::sync::atomic::{AtomicUsize, Ordering};
        static DEPTH: AtomicUsize = AtomicUsize::new(0);
        let d = DEPTH.fetch_add(1, Ordering::Relaxed);
        if d > 1000 {
            DEPTH.store(0, Ordering::Relaxed);
            return Err(MetorexError::runtime_error(
                "SystemStackError: stack level too deep".to_string(),
                crate::vm::utils::position_to_location(expression.position()),
            ));
        }
        let result = self.evaluate_expression_inner(expression);
        DEPTH.fetch_sub(1, Ordering::Relaxed);
        result
    }
}
