//! Method and function body execution for the virtual machine.
//!
//! This module handles executing method bodies (with self) and standalone
//! function bodies (without self), including scope management, parameter
//! binding, and last-expression value capture.

use super::errors::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, VirtualMachine};
use crate::ast::{Expression, Statement};
use crate::callable::Callable;
use crate::class::Class;
use crate::error::{MetorexError, StackFrame};
use crate::lexer::Position;
use crate::object::{Method, Object};
use std::collections::HashMap;
use std::rc::Rc;

use super::param_binding::{bind_params, positional_arg_count, split_keyword_args};

impl VirtualMachine {
    /// Invoke a resolved method with evaluated arguments.
    pub(crate) fn invoke_method(
        &mut self,
        class: Rc<Class>,
        method: Rc<Method>,
        receiver: Object,
        mut arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let method_name = method.name.clone();

        // Check for undefined methods (created by undef_method)
        if method.is_undefined {
            return Err(MetorexError::runtime_error(
                format!(
                    "Undefined method '{}' for type '{}'",
                    method_name,
                    class.name()
                ),
                position_to_location(position),
            ));
        }

        if let Some(result) = self.call_native_method(
            class.as_ref(),
            &receiver,
            &method_name,
            &arguments,
            position,
        )? {
            return Ok(result);
        }

        // For stub methods (empty body, registered on Object for introspection),
        // fall through to base Object native methods (class, to_s, respond_to?, etc.)
        if method.body.is_empty()
            && method.captured_vars.is_none()
            && let Some(result) =
                self.call_object_method(&receiver, &method_name, &arguments, position)?
        {
            return Ok(result);
        }

        let expected = method.parameters.len();
        let mut positional_count = positional_arg_count(&arguments);
        // If a trailing &block argument is passed and the method accepts a block parameter,
        // extract it as pending_block and don't count it as positional.
        if method.block_parameter.is_some()
            && !arguments.is_empty()
            && matches!(arguments.last(), Some(Object::Block(_)))
        {
            self.pending_block = arguments.pop();
            positional_count = positional_arg_count(&arguments);
        }
        let has_variadic = method.variadic_param.is_some();
        let required =
            expected - method.default_parameters.len() - if has_variadic { 1 } else { 0 };
        if !has_variadic && (positional_count < required || positional_count > expected) {
            return Err(method_argument_error(
                &method_name,
                expected,
                positional_count,
                position,
            ));
        }
        if has_variadic && positional_count < required {
            return Err(method_argument_error(
                &method_name,
                required,
                positional_count,
                position,
            ));
        }

        let frame_name = format!("{}#{}", class.name(), method_name);
        let frame_location = position_to_location(position);
        let frame_location_string = Some(format!("{}", frame_location));

        let method_for_body = Rc::clone(&method);
        let self_for_body = method
            .receiver()
            .cloned()
            .unwrap_or_else(|| receiver.clone());
        let arguments_for_body = arguments.clone();
        self.user_def_nesting += 1;
        // Activate the lexically-captured refinements from when this method
        // was defined, as a fresh scope. Reset the live user_def_nesting to 0
        // while these refinements are active (method body re-enters top-level
        // lexical scope semantically for nested defs).
        let has_captured = !method.captured_refinements.is_empty();
        if has_captured {
            self.refinement_scopes.push(
                method
                    .captured_refinements
                    .iter()
                    .map(|(m, cs)| crate::vm::core::RefinementEntry {
                        module: Rc::clone(m),
                        classes: cs.iter().cloned().collect(),
                    })
                    .collect(),
            );
        }
        let execution_result = self.with_call_frame(
            CallFrame::new(frame_name.clone(), frame_location_string),
            move |vm| {
                vm.execute_method_body(
                    method_for_body.as_ref(),
                    self_for_body.clone(),
                    arguments_for_body.clone(),
                )
            },
        );
        if has_captured {
            self.refinement_scopes.pop();
        }
        self.user_def_nesting = self.user_def_nesting.saturating_sub(1);

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(frame_name, frame_location))),
        }
    }

    /// Execute the body of a method within a fresh scope.
    pub(crate) fn execute_method_body(
        &mut self,
        method: &Method,
        self_value: Object,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        self.environment_mut().push_isolated_scope();

        // Take the pending block now so nested calls don't see it.
        let block = self.pending_block.take();

        let result = (|| -> Result<Object, MetorexError> {
            self.environment_mut()
                .define("self".to_string(), self_value.clone());

            // Inject captured closure variables (from define_method blocks).
            // Skip `self` — the method receiver should always be the method's
            // `self_value`, not the captured `self` from where the block was created.
            if let Some(captured) = &method.captured_vars {
                for (name, value_ref) in captured {
                    if name == "self" {
                        continue;
                    }
                    self.environment_mut()
                        .define_shared(name.clone(), value_ref.clone());
                }
            }

            let (positional, kwargs) = split_keyword_args(arguments);
            bind_params(
                self,
                &method.parameters,
                &positional,
                &method.default_parameters,
                &method.variadic_param,
            )?;
            self.bind_keyword_params(&method.keyword_parameters, kwargs)?;

            // Bind the block: define block_given? as a Bool, __block__ for internal use,
            // and the named &block parameter if the method declared one.
            self.environment_mut()
                .define("block_given?".to_string(), Object::Bool(block.is_some()));
            if let Some(block_value) = block {
                self.environment_mut()
                    .define("__block__".to_string(), block_value.clone());
                if let Some(block_param) = &method.block_parameter {
                    self.environment_mut()
                        .define(block_param.clone(), block_value);
                }
            } else if let Some(block_param) = &method.block_parameter {
                self.environment_mut()
                    .define(block_param.clone(), Object::Nil);
            }

            self.execute_body_statements(method.body())
        })();

        self.environment_mut().pop_scope();
        match result {
            Err(MetorexError::NonLocalReturn { value, .. }) => Ok(value),
            other => other,
        }
    }

    /// Execute the body of a standalone function within a fresh scope (no self).
    pub(crate) fn execute_function_body(
        &mut self,
        function: &Method,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        self.environment_mut().push_isolated_scope();

        // Take the pending block now so nested calls don't see it.
        let block = self.pending_block.take();

        let result = (|| -> Result<Object, MetorexError> {
            // Bind parameters to arguments (no self for standalone functions)
            let (positional, kwargs) = split_keyword_args(arguments);
            bind_params(
                self,
                &function.parameters,
                &positional,
                &function.default_parameters,
                &function.variadic_param,
            )?;
            self.bind_keyword_params(&function.keyword_parameters, kwargs)?;

            // Bind the block: define block_given? as a Bool, __block__ for internal use,
            // and the named &block parameter if the function declared one.
            self.environment_mut()
                .define("block_given?".to_string(), Object::Bool(block.is_some()));
            if let Some(block_value) = block {
                self.environment_mut()
                    .define("__block__".to_string(), block_value.clone());
                if let Some(block_param) = &function.block_parameter {
                    self.environment_mut()
                        .define(block_param.clone(), block_value);
                }
            } else if let Some(block_param) = &function.block_parameter {
                self.environment_mut()
                    .define(block_param.clone(), Object::Nil);
            }

            self.execute_body_statements(function.body())
        })();

        self.environment_mut().pop_scope();
        match result {
            Err(MetorexError::NonLocalReturn { value, .. }) => Ok(value),
            other => other,
        }
    }

    /// Execute a list of statements as a method/function body, capturing the
    /// value of the last expression. Shared between execute_method_body and
    /// execute_function_body to eliminate duplication.
    fn execute_body_statements(&mut self, body: &[Statement]) -> Result<Object, MetorexError> {
        let mut last_value = Object::Nil;

        for (i, statement) in body.iter().enumerate() {
            let is_last = i == body.len() - 1;

            // If this is the last statement, capture its value
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
                    Statement::If {
                        condition,
                        then_branch,
                        elsif_branches,
                        else_branch,
                        ..
                    } => {
                        last_value = self.evaluate_if_expression(
                            condition,
                            then_branch,
                            elsif_branches,
                            else_branch,
                        )?;
                        continue;
                    }
                    Statement::Unless {
                        condition,
                        then_branch,
                        else_branch,
                        ..
                    } => {
                        last_value =
                            self.evaluate_unless_expression(condition, then_branch, else_branch)?;
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

            let is_last = i == body.len() - 1;
            match self.execute_statement(statement)? {
                ControlFlow::Next => continue,
                ControlFlow::Value(v) => {
                    if is_last {
                        last_value = v;
                    }
                    continue;
                }
                ControlFlow::Return { value, .. } => return Ok(value),
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

    /// Bind named keyword parameters to the current scope.
    pub(crate) fn bind_keyword_params(
        &mut self,
        keyword_parameters: &[(String, Option<Expression>)],
        kwargs: HashMap<String, Object>,
    ) -> Result<(), MetorexError> {
        for (name, default_expr) in keyword_parameters {
            let value = if let Some(v) = kwargs.get(name) {
                v.clone()
            } else if let Some(expr) = default_expr {
                self.evaluate_expression(expr)?
            } else {
                return Err(MetorexError::runtime_error(
                    format!("Missing required keyword argument: {}", name),
                    crate::error::SourceLocation::new(0, 0, 0),
                ));
            };
            self.environment_mut().define(name.clone(), value);
        }
        Ok(())
    }
}
