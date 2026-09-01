// Exception handling for the Metorex VM.
// This module handles raise and begin/rescue/else/ensure blocks.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::utils::*;

use crate::ast::Statement;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute a raise statement to throw an exception.
    pub(crate) fn execute_raise(
        &mut self,
        exception: &Option<crate::ast::Expression>,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        let exception_obj = if let Some(expr) = exception {
            // Evaluate the exception expression
            let value = self.evaluate_expression(expr)?;

            // If it's already an Exception object, use it directly
            // If it's a String, create a RuntimeError exception
            // If it's a Class (exception class), instantiate it
            match value {
                Object::Exception(_) => value,
                Object::String(message) => {
                    // Create a RuntimeError exception with the string message
                    Object::exception("RuntimeError", (*message).clone())
                }
                Object::Class(class) => {
                    // Instantiated the way `raise` does it, so the class
                    // travels with the exception and a subclass that writes
                    // its own `initialize` or `to_s` is honored.
                    let built =
                        self.invoke_callable(Object::Class(Rc::clone(&class)), vec![], position)?;
                    // An anonymous subclass records the nearest named
                    // ancestor, so `rescue StopIteration` still matches a
                    // `Class.new StopIteration`.
                    if class.ruby_name().is_empty()
                        && let Object::Exception(details) = &built
                    {
                        let mut named = class;
                        while named.ruby_name().is_empty() {
                            match named.superclass() {
                                Some(parent) => named = parent,
                                None => break,
                            }
                        }
                        details.borrow_mut().exception_type = named.name().to_string();
                    }
                    built
                }
                _ => {
                    return Err(MetorexError::runtime_error(
                        "Exception must be an Exception object, String, or exception class"
                            .to_string(),
                        position_to_location(position),
                    ));
                }
            }
        } else {
            // Bare `raise` re-raises `$!`. With nothing to re-raise Ruby
            // raises `RuntimeError: unhandled exception`, which is what
            // Kernel#raise called with no arguments does too.
            match self.environment().get("$!") {
                Some(Object::Exception(_)) => self.environment().get("$!").unwrap(),
                _ => Object::exception("RuntimeError", ""),
            }
        };
        // Capture stack trace and add source location to exception
        let exception_obj = self.add_stack_trace_to_exception(exception_obj, position);

        Ok(ControlFlow::Exception {
            exception: exception_obj,
            position,
        })
    }

    /// Build the exception `raise`/`fail` was handed. A String becomes a
    /// RuntimeError, a class is instantiated, and any other receiver is asked
    /// for one through `#exception`. With no arguments the current `$!` is
    /// re-raised, or a bare RuntimeError when there is none.
    pub(crate) fn build_raise_exception(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let message = arguments.get(1).cloned();
        let exception = match arguments.first() {
            None => match self.environment().get("$!") {
                Some(exception @ Object::Exception(_)) => exception,
                _ => Object::exception("RuntimeError", ""),
            },
            Some(Object::Exception(cell)) => {
                let existing = Object::Exception(Rc::clone(cell));
                if let Some(Object::String(text)) = &message {
                    cell.borrow_mut().message = (**text).clone();
                }
                existing
            }
            Some(Object::String(text)) => Object::exception("RuntimeError", (**text).clone()),
            // An exception class is instantiated with the message.
            Some(value @ Object::Class(_)) => {
                let call_arguments = message.into_iter().collect();
                self.invoke_callable(value.clone(), call_arguments, position)?
            }
            Some(value) => {
                // Any other receiver is asked for an exception of its own.
                let Some((class, method)) = self.lookup_method(value, "exception") else {
                    let msg = "exception class/object expected".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                };
                let call_arguments = message.into_iter().collect();
                self.invoke_method(class, method, value.clone(), call_arguments, position)?
            }
        };
        if !matches!(exception, Object::Exception(_)) {
            let msg = "exception object expected".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", msg.clone()),
                location: position_to_location(position),
                message: msg,
            });
        }
        Ok(self.add_stack_trace_to_exception(exception, position))
    }

    /// Add stack trace and source location to an exception object
    fn add_stack_trace_to_exception(&self, exception: Object, position: Position) -> Object {
        if let Object::Exception(exc_ref) = exception {
            let mut exc = exc_ref.borrow_mut();

            // Ruby sets `#cause` from the exception a rescue clause is
            // handling, once, and never to the exception itself.
            if exc.cause.is_none()
                && let Some(active) = self.globals().get("!")
                && let Object::Exception(active_ref) = &active
                && !Rc::ptr_eq(&exc_ref, active_ref)
            {
                exc.cause = Some(Box::new(active));
            }

            // Add source location if not already set
            if exc.location.is_none() {
                exc.location = Some(crate::object::SourceLocation::new(
                    "script".to_string(),
                    position.line,
                    position.column,
                ));
            }

            // Generate stack trace from call stack
            // Ruby's entries read `file:line:in 'label'`, so a backtrace is
            // usable for locating the code that raised.
            let entry = |path: &str, line: usize, label: &str| -> String {
                if label.is_empty() {
                    format!("{}:{}", path, line)
                } else {
                    format!("{}:{}:in '{}'", path, line, label)
                }
            };
            let raise_file = self
                .current_source_file
                .clone()
                .or_else(|| self.current_file.as_ref().map(|f| f.display().to_string()))
                .unwrap_or_default();
            let raising_method = self
                .call_stack()
                .last()
                .map(|frame| frame.name().to_string())
                .unwrap_or_default();
            // The raise site comes first, then each frame's own call site. A
            // frame records where it was called from, so its location pairs
            // with the name of the frame below it: the method that made the
            // call. The outermost one is the file body itself.
            let mut sites = vec![(raise_file.clone(), position.line, raising_method)];
            let frames: Vec<_> = self.call_stack().iter().rev().collect();
            for (index, frame) in frames.iter().enumerate() {
                let line = frame
                    .location()
                    .and_then(|location| {
                        location
                            .rsplit(':')
                            .nth(1)
                            .and_then(|line| line.parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                let path = frame
                    .source_file()
                    .map(|file| file.to_string())
                    .unwrap_or_else(|| raise_file.clone());
                let label = frames
                    .get(index + 1)
                    .map(|caller| caller.name().to_string())
                    .unwrap_or_else(|| "<main>".to_string());
                sites.push((path, line, label));
            }

            exc.backtrace = Some(
                sites
                    .iter()
                    .map(|(path, line, label)| entry(path, *line, label))
                    .collect(),
            );
            exc.backtrace_sites = Some(sites);

            drop(exc); // Release the borrow before returning
            Object::Exception(exc_ref)
        } else {
            exception
        }
    }

    /// Execute a begin/rescue/else/ensure block.
    pub(crate) fn execute_begin(
        &mut self,
        body: &[Statement],
        rescue_clauses: &[crate::ast::RescueClause],
        else_clause: &Option<Vec<Statement>>,
        ensure_block: &Option<Vec<Statement>>,
        _position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // Execute the try block
        let body_result = self.execute_begin_branch(body);

        // Track whether an exception was handled
        let mut handled_exception = false;
        let mut final_result = body_result;

        // Convert UncaughtException errors to ControlFlow::Exception
        if let Err(MetorexError::UncaughtException {
            exception,
            location,
            ..
        }) = &final_result
        {
            final_result = Ok(ControlFlow::Exception {
                exception: exception.clone(),
                position: Position {
                    line: location.line,
                    column: location.column,
                    offset: 0,
                },
            });
        }

        // Also convert internal RuntimeError / TypeError / InternalError to rescuable exceptions.
        // In Ruby, all internal errors (undefined method, type errors, etc.) are rescuable.
        let runtime_exception = match &final_result {
            Err(MetorexError::RuntimeError {
                message, location, ..
            }) => {
                let msg = message.clone();
                let pos = Position {
                    line: location.line,
                    column: location.column,
                    offset: 0,
                };
                Some((Object::exception("RuntimeError", msg), pos))
            }
            Err(MetorexError::TypeError {
                message, location, ..
            }) => {
                let msg = message.clone();
                let pos = Position {
                    line: location.line,
                    column: location.column,
                    offset: 0,
                };
                Some((Object::exception("TypeError", msg), pos))
            }
            _ => None,
        };
        if let Some((exception_obj, pos)) = runtime_exception {
            final_result = Ok(ControlFlow::Exception {
                exception: exception_obj,
                position: pos,
            });
        }

        // If an exception occurred, try to match rescue clauses
        if let Ok(ControlFlow::Exception {
            exception,
            position: _ex_pos,
        }) = &final_result
        {
            // Store the current exception in $! for access in rescue blocks
            self.set_current_exception(exception.clone());

            // Try each rescue clause in order
            for rescue_clause in rescue_clauses {
                if self.exception_matches(exception, &rescue_clause.exception_types)? {
                    // Bind exception to variable if specified (=> e)
                    if let Some(var_name) = &rescue_clause.variable_name {
                        self.environment_mut()
                            .define(var_name.clone(), exception.clone());
                    }

                    // Execute the rescue block
                    let handled_cause = exception.clone();
                    final_result = self.execute_begin_branch(&rescue_clause.body);
                    // An exception raised by the rescue body takes the one
                    // being handled as its cause.
                    if let Err(MetorexError::UncaughtException {
                        exception: raised, ..
                    }) = &final_result
                    {
                        Self::record_cause(raised, &handled_cause);
                    }
                    handled_exception = true;
                    break;
                }
            }

            // If exception wasn't handled, it will propagate
            if !handled_exception {
                // Keep the exception result to propagate it
                // Don't execute else clause
            } else {
                // Clear the $! variable since exception was handled
                self.set_current_exception(Object::Nil);
            }
        } else if final_result.is_ok()
            && matches!(
                final_result,
                Ok(ControlFlow::Next) | Ok(ControlFlow::Value(_))
            )
        {
            // No exception occurred - execute else clause if present
            if let Some(else_stmts) = else_clause {
                final_result = self.execute_begin_branch(else_stmts);
            }
        }

        // Always execute ensure block, regardless of what happened
        if let Some(ensure_stmts) = ensure_block {
            let ensure_result = self.execute_statements_internal(ensure_stmts);

            // If ensure block raises an exception or changes control flow,
            // it overrides the previous result
            match ensure_result {
                Ok(ControlFlow::Exception { .. }) => {
                    final_result = ensure_result;
                }
                Ok(ControlFlow::Next) => {
                    // Ensure completed normally, don't override final_result
                }
                Ok(_) => {
                    // Other control flow (return, break, continue)
                    final_result = ensure_result;
                }
                Err(_) => {
                    // Error in ensure block overrides previous result
                    final_result = ensure_result;
                }
            }
        }

        final_result
    }

    /// Run a begin/rescue/else clause body, returning `ControlFlow::Value`
    /// holding the value of the last expression statement when the body
    /// completes normally — so e.g. `begin; raise; rescue => e; e; end`
    /// evaluates to the rescued exception. Non-expression terminal
    /// statements still return their natural ControlFlow.
    fn execute_begin_branch(&mut self, body: &[Statement]) -> Result<ControlFlow, MetorexError> {
        let mut last_value: Option<Object> = None;
        for (idx, statement) in body.iter().enumerate() {
            let is_last = idx == body.len() - 1;
            if is_last && let Some(value) = self.terminal_statement_value(statement)? {
                return Ok(ControlFlow::Value(value));
            }
            match self.execute_statement(statement)? {
                ControlFlow::Next => {}
                ControlFlow::Value(v) => last_value = Some(v),
                flow => return Ok(flow),
            }
        }
        match last_value {
            Some(v) => Ok(ControlFlow::Value(v)),
            None => Ok(ControlFlow::Next),
        }
    }

    /// Check if an exception matches the given exception type list.
    pub(crate) fn exception_matches(
        &self,
        exception: &Object,
        exception_types: &[String],
    ) -> Result<bool, MetorexError> {
        // A bare `rescue` catches StandardError, not everything: an Exception
        // that is not one of its descendants goes past it.
        if exception_types.is_empty() {
            return self.exception_matches(exception, &["StandardError".to_string()]);
        }

        // Get the exception's type name, and the class it carries when it has
        // one. An anonymous or namespaced class has no name to look up later,
        // so the carried class is the only way to place it.
        let (exception_type_name, exception_class) = match exception {
            Object::Exception(ex) => {
                let details = ex.borrow();
                (details.exception_type.clone(), details.class.clone())
            }
            _ => return Ok(false),
        };

        // Check if the exception's type matches any of the specified types
        for type_name in exception_types {
            // Exact name match (covers stdlib exceptions like LoadError, Errno::*, etc.)
            if type_name == &exception_type_name {
                return Ok(true);
            }
            // `rescue Object` / `rescue BasicObject` catch everything (Ruby semantics).
            if type_name == "Object" || type_name == "BasicObject" {
                return Ok(true);
            }
            // Well-known ancestor catches: StandardError/Exception catches most errors.
            if (type_name == "StandardError" || type_name == "Exception")
                && Self::is_standard_exception_name(&exception_type_name)
            {
                return Ok(true);
            }
            // Otherwise place the exception by its class chain.
            let target_class = match self.environment().get(type_name) {
                Some(Object::Class(class)) => Some(class),
                _ => match self.resolve_qualified_constant(type_name) {
                    Some(Object::Class(class)) => Some(class),
                    _ => None,
                },
            };
            let raised_class = exception_class.clone().or_else(|| {
                match self.environment().get(&exception_type_name) {
                    Some(Object::Class(class)) => Some(class),
                    _ => None,
                }
            });
            if let (Some(target_class), Some(raised_class)) = (target_class, raised_class)
                && Self::is_class_or_subclass(&raised_class, &target_class)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn is_standard_exception_name(name: &str) -> bool {
        matches!(
            name,
            "StandardError"
                | "RuntimeError"
                | "TypeError"
                | "ValueError"
                | "ArgumentError"
                | "NameError"
                | "NoMethodError"
                | "LoadError"
                | "NotImplementedError"
                | "ZeroDivisionError"
                | "FloatDomainError"
                | "IndexError"
                | "KeyError"
                | "RangeError"
                | "StopIteration"
                | "IOError"
                | "EOFError"
                | "FrozenError"
        ) || name.starts_with("Errno::")
    }

    /// Check if a class is the same as or a subclass of another class.
    pub(crate) fn is_class_or_subclass(class: &Rc<Class>, target: &Rc<Class>) -> bool {
        if Rc::ptr_eq(class, target) {
            return true;
        }

        // Check superclass chain
        if let Some(superclass) = class.superclass() {
            return Self::is_class_or_subclass(&superclass, target);
        }

        false
    }
}
