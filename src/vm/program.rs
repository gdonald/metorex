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
            // `public_send` is what Ruby's symbol proc uses, so a name the
            // receiver keeps to itself is refused rather than reached. It is
            // also not `send`, which a class of the program's own may name
            // for something else, as a socket does.
            method: "public_send".to_string(),
            arguments: vec![Expression::Symbol {
                value: sym.to_string(),
                position: pos,
            }],
            trailing_block: None,
            position: pos,
        },
        position: pos,
    }];
    let mut made = BlockStatement::new(
        vec!["x".to_string()],
        body,
        std::collections::HashMap::new(),
    );
    // What the callable stands for, which is what it says of itself in place
    // of a file and a line.
    made.from_symbol = Some(sym.to_string());
    made
}

/// Build the block `&some_method` hands over: `{ |*args| target.call(*args) }`,
/// where `target` is the Method itself. The call keeps the method's own arity,
/// so a callable given the wrong number of arguments still says so.
fn method_to_proc_block(target: &Object, position: Position) -> BlockStatement {
    let mut parameters = Vec::new();
    let mut parameter_defaults = Vec::new();
    let mut forwarded = Vec::new();
    if let Object::Method(method) = target {
        let optional: std::collections::HashSet<usize> = method
            .default_parameters
            .iter()
            .map(|(index, _)| *index)
            .collect();
        // The splat's own name sits among the positional ones, and it is
        // forwarded as a splat rather than as one more positional.
        let splat_at = method.variadic_param.as_ref().map(|(index, _)| *index);
        for index in 0..method.parameters.len() {
            if splat_at == Some(index) {
                continue;
            }
            let name = format!("__method_proc_p{index}");
            if optional.contains(&index) {
                parameter_defaults.push((parameters.len(), Expression::NilLiteral { position }));
            }
            parameters.push(name.clone());
            forwarded.push(Expression::Identifier { name, position });
        }
        if method.variadic_param.is_some() {
            let name = "__method_proc_rest".to_string();
            parameters.push(format!("*{name}"));
            forwarded.push(Expression::Splat {
                expression: Box::new(Expression::Identifier { name, position }),
                position,
            });
        }
    }
    let call = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "__method_proc_target".to_string(),
            position,
        }),
        method: "call".to_string(),
        arguments: forwarded,
        trailing_block: None,
        position,
    };
    let mut made = BlockStatement::new(
        parameters,
        vec![Statement::Expression {
            expression: call,
            position,
        }],
        std::collections::HashMap::new(),
    );
    made.parameter_defaults = parameter_defaults;
    // A Method takes its arguments exactly, so the block standing for one is
    // a lambda rather than an ordinary block.
    made.is_lambda = true;
    made
}

impl VirtualMachine {
    /// Execute a sequence of statements and return an optional result (from return statements).
    pub fn execute_program(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        // A `return` written at the top level ends the program, carrying its
        // value out however deep in blocks it was written.
        match self.run_program_statements(statements) {
            Err(MetorexError::NonLocalReturn { value, .. }) => Ok(Some(value)),
            other => other,
        }
    }

    fn run_program_statements(
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
                        // Carrying the exception keeps it reachable as `$!`
                        // for the `at_exit` handlers that run next.
                        return Err(MetorexError::UncaughtException {
                            message: format_exception(&exception),
                            exception,
                            location: position_to_location(position),
                        });
                    }
                    ControlFlow::Break { position, .. } => {
                        return Err(loop_control_error("break", position));
                    }
                    ControlFlow::Retry { position } => {
                        return Err(MetorexError::BlockRetry {
                            location: position_to_location(position),
                        });
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
                    return Err(MetorexError::UncaughtException {
                        message: format_exception(&exception),
                        exception,
                        location: position_to_location(position),
                    });
                }
                ControlFlow::Break { position, .. } => {
                    return Err(loop_control_error("break", position));
                }
                ControlFlow::Retry { position } => {
                    return Err(MetorexError::BlockRetry {
                        location: position_to_location(position),
                    });
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
                Expression::Splat {
                    expression,
                    position: splat_at,
                } => {
                    let value = self.evaluate_expression(expression)?;
                    match value {
                        Object::Array(arr) => {
                            args.extend(arr.borrow().iter().cloned());
                        }
                        // `*nil` spreads into nothing, the way `[*nil]` does,
                        // so a call written with one passes no argument there.
                        Object::Nil => {}
                        other => match self.splat_through_to_a(other, *splat_at)? {
                            Ok(items) => args.extend(items),
                            Err(held) => args.push(held),
                        },
                    }
                }
                Expression::KeywordSplat {
                    expression,
                    position: splat_position,
                } => {
                    // `**hash`: an empty Hash contributes no argument, so
                    // `f(**{})` calls `f` with nothing at all.
                    let splat_position = *splat_position;
                    let mut value = self.evaluate_expression(expression)?;
                    // Anything else is asked for a Hash of its own, which is
                    // how an object stands in for keyword arguments.
                    if !matches!(value, Object::Dict(_))
                        && self.lookup_method(&value, "to_hash").is_some()
                    {
                        value = self.send_to_object(value, "to_hash", vec![], splat_position)?;
                    }
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
                Expression::BlockArg {
                    expression,
                    position: other_position,
                } => {
                    let other_position = *other_position;
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
                            self.pending_block = Some(Object::Block(std::rc::Rc::new(
                                symbol_to_proc_block(&sym.as_str()),
                            )));
                            self.pending_block_from_ampersand = true;
                        }
                        // `&some_method` hands the method over as the block,
                        // which is what `to_proc` on a Method answers.
                        target @ Object::Method(_) => {
                            let mut block = method_to_proc_block(&target, other_position);
                            block.captured_vars.insert(
                                "__method_proc_target".to_string(),
                                std::rc::Rc::new(std::cell::RefCell::new(target)),
                            );
                            self.pending_block = Some(Object::Block(std::rc::Rc::new(block)));
                            self.pending_block_from_ampersand = true;
                        }
                        // An object of the program's own becomes a block
                        // through `to_proc`, which is how a Yielder reaches
                        // a method that takes one.
                        other @ Object::Instance(_)
                            if self
                                .send_to_object(
                                    other.clone(),
                                    "respond_to?",
                                    vec![Object::symbol("to_proc".to_string())],
                                    other_position,
                                )?
                                .is_truthy() =>
                        {
                            let made = self.send_to_object(
                                other.clone(),
                                "to_proc",
                                Vec::new(),
                                other_position,
                            )?;
                            match made {
                                Object::Block(_) => {
                                    self.pending_block = Some(made);
                                    self.pending_block_from_ampersand = true;
                                }
                                // A `to_proc` that answers a Method stands
                                // for a block the same way `&method` does.
                                target @ Object::Method(_) => {
                                    let mut block = method_to_proc_block(&target, other_position);
                                    block.captured_vars.insert(
                                        "__method_proc_target".to_string(),
                                        std::rc::Rc::new(std::cell::RefCell::new(target)),
                                    );
                                    self.pending_block =
                                        Some(Object::Block(std::rc::Rc::new(block)));
                                    self.pending_block_from_ampersand = true;
                                }
                                _ => args.push(other),
                            }
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
        // Guard against infinite recursion. The count belongs to the thread
        // running the program, so two virtual machines running side by side
        // do not add their nesting together.
        thread_local! {
            static DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
        }
        let reached = DEPTH.with(|depth| {
            let reached = depth.get();
            depth.set(reached + 1);
            reached
        });
        if reached > 1000 {
            DEPTH.with(|depth| depth.set(depth.get() - 1));
            return Err(MetorexError::runtime_error(
                "SystemStackError: stack level too deep".to_string(),
                crate::vm::utils::position_to_location(expression.position()),
            ));
        }
        let result = self.evaluate_expression_inner(expression);
        DEPTH.with(|depth| depth.set(depth.get() - 1));
        result
    }
}

impl VirtualMachine {
    /// Run the `at_exit` handlers, last registered first, once the program is
    /// over. Each one runs even if an earlier raised, and a handler that calls
    /// `exit` decides the status over the one the script was ending with.
    ///
    /// Answers the exit status to end with, and whether anything reported an
    /// error, given the status the program had reached and the exception that
    /// ended it, if any.
    /// Say how many frames under the top one a report writes out. The value
    /// is also what `Thread::Backtrace.limit` answers.
    pub fn set_backtrace_limit(&mut self, limit: i64) {
        self.backtrace_limit = limit;
        self.globals_mut()
            .set("__backtrace_limit__", Object::Int(limit));
    }

    pub fn run_at_exit_handlers(&mut self, status: i32, ending: Option<Object>) -> i32 {
        if let Some(exception) = ending {
            self.set_current_exception(exception);
        }
        let mut status = status;
        while let Some(handler) = self.at_exit_handlers.pop() {
            let Object::Block(block) = handler else {
                continue;
            };
            let position = crate::lexer::Position {
                line: 0,
                column: 0,
                offset: 0,
                prelude: false,
            };
            let outcome = self.execute_block_callable(&block, Vec::new(), position);
            // What a handler printed belongs before whatever the next one
            // reports, and stdout would otherwise hold it until exit.
            use std::io::Write as _;
            let _ = std::io::stdout().flush();
            match outcome {
                Ok(_) => {}
                Err(crate::error::MetorexError::UncaughtException {
                    exception: Object::Exception(details),
                    ..
                }) if details.borrow().is_system_exit() => {
                    // `exit` inside a handler ends that handler and settles
                    // the status the program leaves with.
                    if let Some(carried) = details.borrow().status {
                        status = carried as i32;
                    }
                    self.set_current_exception(Object::Exception(std::rc::Rc::clone(&details)));
                }
                Err(error) => {
                    // A handler that raises reports the way the program does,
                    // naming where it was raised and the class it is.
                    if let crate::error::MetorexError::UncaughtException { exception, .. } = &error
                    {
                        let held = exception.clone();
                        match self.uncaught_report(&held) {
                            Some(report) => eprint!("{}", report),
                            None => eprintln!("{}", error),
                        }
                        self.set_current_exception(held);
                    } else {
                        eprintln!("{}", error);
                    }
                    status = 1;
                }
            }
        }
        status
    }
}
