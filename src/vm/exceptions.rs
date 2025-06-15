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
                    // Instantiate the exception class with an empty message
                    Object::exception(class.name(), String::new())
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
            // Bare raise - re-raise current exception
            // For now, we'll check if there's a $! variable (current exception)
            // If not, it's an error to use bare raise outside a rescue block
            match self.environment().get("$!") {
                Some(Object::Exception(_)) => self.environment().get("$!").unwrap(),
                _ => {
                    return Err(MetorexError::runtime_error(
                        "No exception to re-raise (bare raise only allowed in rescue blocks)"
                            .to_string(),
                        position_to_location(position),
                    ));
                }
            }
        };

        Ok(ControlFlow::Exception {
            exception: exception_obj,
            position,
        })
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
        let body_result = self.execute_statements_internal(body);

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

        // If an exception occurred, try to match rescue clauses
        if let Ok(ControlFlow::Exception {
            exception,
            position: _ex_pos,
        }) = &final_result
        {
            // Store the current exception in $! for access in rescue blocks
            self.environment_mut()
                .define("$!".to_string(), exception.clone());

            // Try each rescue clause in order
            for rescue_clause in rescue_clauses {
                if self.exception_matches(exception, &rescue_clause.exception_types)? {
                    // Bind exception to variable if specified (=> e)
                    if let Some(var_name) = &rescue_clause.variable_name {
                        self.environment_mut()
                            .define(var_name.clone(), exception.clone());
                    }

                    // Execute the rescue block
                    final_result = self.execute_statements_internal(&rescue_clause.body);
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
                self.environment_mut().define("$!".to_string(), Object::Nil);
            }
        } else if final_result.is_ok() && matches!(final_result, Ok(ControlFlow::Next)) {
            // No exception occurred - execute else clause if present
            if let Some(else_stmts) = else_clause {
                final_result = self.execute_statements_internal(else_stmts);
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

    /// Check if an exception matches the given exception type list.
    pub(crate) fn exception_matches(
        &self,
        exception: &Object,
        exception_types: &[String],
    ) -> Result<bool, MetorexError> {
        // Empty exception_types list means catch all exceptions
        if exception_types.is_empty() {
            return Ok(true);
        }

        // Get the exception's type name
        let exception_type_name = match exception {
            Object::Exception(ex) => ex.borrow().exception_type.clone(),
            _ => return Ok(false),
        };

        // Check if the exception's type matches any of the specified types
        for type_name in exception_types {
            // Look up the exception type class in the environment
            if let Some(Object::Class(target_class)) = self.environment().get(type_name) {
                // Get the class for this exception type
                if let Some(Object::Class(exception_class)) =
                    self.environment().get(&exception_type_name)
                {
                    // Check if exception_class is the target_class or a subclass of it
                    if Self::is_class_or_subclass(&exception_class, &target_class) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
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
