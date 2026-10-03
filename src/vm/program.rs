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

/// The Proc `rb_proc_new` makes around a C block function: not a lambda,
/// written nowhere in the program, taking any number of values and the block
/// it is called with, and handing them all to `target`.
pub(crate) fn proc_for_c_function(target: Object) -> Object {
    let position = Position::default();
    let rest = "__c_function_values".to_string();
    let handed = "__c_function_block".to_string();
    let call = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "__method_proc_target".to_string(),
            position,
        }),
        method: "call".to_string(),
        arguments: vec![
            Expression::Splat {
                expression: Box::new(Expression::Identifier {
                    name: rest.clone(),
                    position,
                }),
                position,
            },
            Expression::BlockArg {
                expression: Box::new(Expression::Identifier {
                    name: handed.clone(),
                    position,
                }),
                position,
            },
        ],
        trailing_block: None,
        position,
    };
    let mut made = BlockStatement::new(
        vec![format!("*{rest}"), format!("&{handed}")],
        vec![Statement::Expression {
            expression: call,
            position,
        }],
        std::collections::HashMap::new(),
    );
    made.captured_vars.insert(
        "__method_proc_target".to_string(),
        std::rc::Rc::new(std::cell::RefCell::new(target)),
    );
    Object::Block(std::rc::Rc::new(made))
}

/// The block `&target` hands over for a Method, which calls the Method with
/// what the block is given.
pub(crate) fn block_for_method(target: Object, position: Position) -> Object {
    let mut block = method_to_proc_block(&target, position);
    block.captured_vars.insert(
        "__method_proc_target".to_string(),
        std::rc::Rc::new(std::cell::RefCell::new(target)),
    );
    Object::Block(std::rc::Rc::new(block))
}

impl VirtualMachine {
    /// Execute a sequence of statements and return an optional result (from return statements).
    pub fn execute_program(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        // Each file's top level starts with its methods private.
        let caller_toplevel_public = std::mem::replace(&mut self.toplevel_public, false);
        let caller_toplevel_frame =
            std::mem::replace(&mut self.toplevel_frame, self.current_method_frame);
        self.borrowed_frames.push(self.current_method_frame);
        // A file's top level sits in no block, whatever block loaded it.
        let caller_lexical_home = self.lexical_home_frame.take();
        // A `return` written at the top level ends the program, carrying its
        // value out however deep in blocks it was written.
        let result = match self.run_program_statements(statements) {
            Err(MetorexError::NonLocalReturn { value, .. }) => Ok(Some(value)),
            other => other,
        };
        self.toplevel_public = caller_toplevel_public;
        self.toplevel_frame = caller_toplevel_frame;
        self.borrowed_frames.pop();
        self.lexical_home_frame = caller_lexical_home;
        result
    }

    /// Run a unit whose `return` belongs to the scope around it rather than
    /// ending the unit, which is what code handed to `eval` does.
    pub(crate) fn run_eval_statements(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        self.run_statements_of_a_unit(statements, true)
    }

    pub(crate) fn run_program_statements(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        self.run_statements_of_a_unit(statements, false)
    }

    fn run_statements_of_a_unit(
        &mut self,
        statements: &[Statement],
        a_return_unwinds: bool,
    ) -> Result<Option<Object>, MetorexError> {
        // Ruby's parser reserves every local a file assigns to before any of
        // it runs, so a name read ahead of its assignment answers nil and
        // TOPLEVEL_BINDING names it from the start.
        for name in crate::ast::scope_locals::collect_assigned_locals(statements) {
            if self.environment().assignment_introduces_a_local(&name) {
                self.environment_mut().hoist(name);
            }
        }
        // Every `BEGIN` body runs before the rest of the unit, in the order
        // they were written, and shares the unit's own scope.
        for statement in statements {
            if let Statement::BeginBlock { body, position } = statement {
                // A program read one line at a time runs the same unit once
                // per line, and the body belongs to the unit rather than to
                // each pass over it.
                let site = (
                    self.current_source_file.clone().unwrap_or_default(),
                    position.line,
                    position.column,
                );
                if self.opened_blocks.insert(site)
                    && let ControlFlow::Return {
                        value,
                        position: at,
                    } = self.execute_statements_internal(body)?
                    && a_return_unwinds
                {
                    return Err(self.unwinding_return(value, at));
                }
            }
        }
        let mut last_value = None;

        for statement in statements {
            // If it's an expression statement, track its value
            if let Statement::Expression {
                expression,
                position,
            } = statement
            {
                if self.coverage.is_some() {
                    self.coverage_count(position.line);
                }
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
                ControlFlow::Return {
                    value,
                    position: at,
                } => {
                    if a_return_unwinds {
                        return Err(self.unwinding_return(value, at));
                    }
                    return Ok(Some(value));
                }
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

    /// A `return` leaving an eval, which unwinds to the invocation the code
    /// around the eval belongs to.
    fn unwinding_return(&self, value: Object, at: crate::lexer::Position) -> MetorexError {
        // The return belongs to the innermost lambda or method around the
        // eval, whichever the unwinding reaches first, so it names no frame
        // of its own.
        MetorexError::NonLocalReturn {
            value,
            location: position_to_location(at),
            home_frame: None,
        }
    }

    /// Evaluate a list of argument expressions, expanding any splat (`*expr`)
    /// arguments and routing block-arg (`&expr`) arguments to `pending_block`.
    pub(crate) fn evaluate_arguments(
        &mut self,
        argument_exprs: &[Expression],
    ) -> Result<Vec<Object>, MetorexError> {
        let mut args = Vec::with_capacity(argument_exprs.len());
        // Whether the last argument was spread out of a splat, which is what
        // lets a hash marked by `ruby2_keywords` be passed on as keywords.
        let mut tail_from_splat = false;
        for arg in argument_exprs {
            tail_from_splat = matches!(arg, Expression::Splat { .. });
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
                        // `**nil` passes no keywords, the same as `**{}`.
                        Object::Nil => {}
                        Object::Dict(entries) if entries.borrow().is_empty() => {}
                        Object::Dict(entries) => {
                            // Keywords spread after other keywords join them,
                            // a later key taking the place of an earlier one.
                            if let Some(Object::Dict(gathered)) = args.last()
                                && gathered.borrow().contains_key("__MX_KWARGS__")
                            {
                                let mut joined = gathered.borrow().clone();
                                for (key, value) in entries.borrow().iter() {
                                    joined.insert(key.clone(), value.clone());
                                }
                                args.pop();
                                args.push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                                    joined,
                                ))));
                                continue;
                            }
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
                    // A refinement in force may say how an object becomes a
                    // block, which stands ahead of whatever its own kind does.
                    let refined_to_proc =
                        crate::vm::method_lookup::refinement_target_name(&value, self)
                            .and_then(|target| self.find_refined_method(&target, "to_proc"))
                            .is_some();
                    match value {
                        Object::Nil => {}
                        Object::Block(_) => {
                            self.pending_block = Some(value);
                            self.pending_block_from_ampersand = true;
                        }
                        other if refined_to_proc => {
                            let made = self.send_to_object(
                                other.clone(),
                                "to_proc",
                                Vec::new(),
                                other_position,
                            )?;
                            self.pending_block = Some(made);
                            self.pending_block_from_ampersand = true;
                            self.pending_block_source = Some(other);
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
                            self.pending_block = Some(block_for_method(target, other_position));
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
                                    self.pending_block_source = Some(other.clone());
                                }
                                // A `to_proc` that answers a Method stands
                                // for a block the same way `&method` does.
                                target @ Object::Method(_) => {
                                    self.pending_block =
                                        Some(block_for_method(target, other_position));
                                    self.pending_block_from_ampersand = true;
                                }
                                // Anything else is no block at all, which
                                // Ruby names both sides of.
                                answered => {
                                    let named = self.builtins().class_of(&other).ruby_name();
                                    let gives = self.builtins().class_of(&answered).ruby_name();
                                    let message = format!(
                                        "can't convert {} into Proc ({}#to_proc gives {})",
                                        named, named, gives
                                    );
                                    return Err(crate::vm::errors::simple_exception(
                                        "TypeError",
                                        &message,
                                        other_position,
                                    ));
                                }
                            }
                        }
                        // An object with no `to_proc` of its own is no block,
                        // and Ruby refuses it rather than counting it among
                        // the arguments.
                        other @ Object::Instance(_) => {
                            let named = self.builtins().class_of(&other).ruby_name();
                            let message = format!("no implicit conversion of {} into Proc", named);
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                &message,
                                other_position,
                            ));
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
        // A hash marked by `ruby2_keywords` and spread out of a splat goes on
        // as the keyword arguments it was gathered from.
        if tail_from_splat
            && let Some(Object::Dict(entries)) = args.last()
            && entries
                .borrow()
                .contains_key(crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY)
        {
            let mut keywords: indexmap::IndexMap<String, Object> = entries
                .borrow()
                .iter()
                .filter(|(key, _)| {
                    key.as_str() != crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                })
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
            args.pop();
            args.push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                keywords,
            ))));
        }
        Ok(args)
    }

    /// Evaluate an expression to a runtime value.
    pub(crate) fn evaluate_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<Object, MetorexError> {
        enter_nesting(expression.position())?;
        let result = self.evaluate_expression_inner(expression);
        leave_nesting();
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
        // A program that is ending takes its threads with it, and each one
        // unwinds where it stands.
        self.end_live_threads();
        // `Signal.trap(:EXIT, ...)` names what to run as the program ends,
        // ahead of everything `at_exit` left.
        if let Some(held) = self.signal_handlers.get("EXIT").cloned()
            && !matches!(&held, Object::String(text) if matches!(
                &*text.as_str(),
                "DEFAULT" | "SYSTEM_DEFAULT" | "IGNORE"
            ))
            && !matches!(held, Object::Nil)
        {
            let position = crate::lexer::Position::new(0, 0, 0);
            let _ = self.send_to_object(held, "call", Vec::new(), position);
            use std::io::Write as _;
            let _ = std::io::stdout().flush();
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

/// How deeply expressions and method calls may nest, together, before one
/// raises SystemStackError rather than running out of stack.
const MOST_NESTED: usize = 1200;

thread_local! {
    /// How deeply the program running on this thread is nested. It belongs
    /// to the thread, so two virtual machines running side by side do not
    /// add their nesting together.
    static NESTING: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Go one level deeper, or raise SystemStackError when the program is as deep
/// as it may go. A level entered is left with `leave_nesting`.
pub(crate) fn enter_nesting(position: crate::lexer::Position) -> Result<(), MetorexError> {
    let reached = NESTING.with(|depth| depth.get());
    if reached >= MOST_NESTED {
        return Err(crate::vm::errors::simple_exception(
            "SystemStackError",
            "stack level too deep",
            position,
        ));
    }
    NESTING.with(|depth| depth.set(reached + 1));
    Ok(())
}

/// Come back up the level `enter_nesting` went down.
pub(crate) fn leave_nesting() {
    NESTING.with(|depth| depth.set(depth.get() - 1));
}
