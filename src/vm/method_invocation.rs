//! Method invocation and execution for the virtual machine.
//!
//! This module handles the actual execution of methods and callable objects,
//! including scope management and control flow handling.

use super::errors::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, VirtualMachine};
use crate::ast::{Expression, Statement};
use crate::callable::Callable;
use crate::class::Class;
use crate::error::{MetorexError, StackFrame};
use crate::lexer::Position;
use crate::object::{BlockStatement, Method, Object};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

impl VirtualMachine {
    /// Invoke a resolved method with evaluated arguments.
    pub(crate) fn invoke_callable(
        &mut self,
        callable: Object,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match callable {
            Object::Block(block) => block.call(self, arguments, position),
            Object::Method(method) => {
                // Call standalone function (represented as Method object)
                // Validate positional argument count (kwargs dict doesn't count)
                let expected = method.parameters.len();
                let positional_count = positional_arg_count(&arguments);
                if expected != positional_count {
                    return Err(method_argument_error(
                        &method.name,
                        expected,
                        positional_count,
                        position,
                    ));
                }
                // Execute function body without self
                self.execute_function_body(&method, arguments)
            }
            Object::Class(class) => {
                // Check if this is an exception class
                let is_exception_class = self.is_exception_class(&class);

                if is_exception_class {
                    // Create an Exception object instead of a regular Instance
                    let message = if arguments.is_empty() {
                        String::new()
                    } else if arguments.len() == 1 {
                        // Extract message from first argument
                        match &arguments[0] {
                            Object::String(s) => (**s).clone(),
                            other => format!("{:?}", other),
                        }
                    } else {
                        return Err(MetorexError::runtime_error(
                            format!(
                                "Exception.new takes 0 or 1 argument, got {}",
                                arguments.len()
                            ),
                            position_to_location(position),
                        ));
                    };

                    let exception_obj = Object::exception(class.name(), message);
                    Ok(exception_obj)
                } else {
                    // Create a new instance of the class
                    let instance = Rc::new(RefCell::new(crate::object::Instance::new(Rc::clone(
                        &class,
                    ))));
                    let instance_obj = Object::Instance(Rc::clone(&instance));

                    // Look for an 'initialize' method and call it if present
                    if let Some(init_method) = class.find_method("initialize") {
                        self.invoke_method(
                            class,
                            init_method,
                            instance_obj.clone(),
                            arguments,
                            position,
                        )?;
                    } else if !arguments.is_empty() {
                        // If no initialize method exists, reject non-empty arguments
                        return Err(MetorexError::runtime_error(
                            format!(
                                "No initialize method defined, but {} argument(s) provided",
                                arguments.len()
                            ),
                            position_to_location(position),
                        ));
                    }

                    Ok(instance_obj)
                }
            }
            Object::NativeFunction(name) => self.call_native_function(&name, arguments, position),
            other => Err(not_callable_error(&other, position)),
        }
    }

    /// Execute a block callable within the VM, handling scope capture and return semantics.
    pub(crate) fn execute_block_callable(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let expected = block.arity();
        let found = arguments.len();

        if expected != found {
            return Err(callable_argument_error(
                block.name(),
                expected,
                found,
                position,
            ));
        }

        let frame_name = block.name().to_string();
        let frame_location = position_to_location(position);
        let frame_location_string = Some(format!("{}", frame_location));

        let execution_result = self.with_call_frame(
            CallFrame::new(frame_name.clone(), frame_location_string),
            move |vm| vm.execute_block_body(block, arguments),
        );

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(frame_name, frame_location))),
        }
    }

    /// Execute the statements inside a block object with its captured scope.
    pub(crate) fn execute_block_body(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        self.environment_mut().push_scope();

        let result = (|| -> Result<Object, MetorexError> {
            // Define captured variables using shared references
            for (name, value_ref) in block.captured_vars() {
                self.environment_mut()
                    .define_shared(name.clone(), value_ref.clone());
            }

            // Define parameters as regular variables
            for (param, argument) in block.parameters().iter().zip(arguments.into_iter()) {
                self.environment_mut().define(param.clone(), argument);
            }

            let mut last_value = Object::Nil;

            for statement in block.body() {
                if let Statement::Expression { expression, .. } = statement {
                    last_value = self.evaluate_expression(expression)?;
                    continue;
                }

                match self.execute_statement(statement)? {
                    ControlFlow::Next => {}
                    ControlFlow::Return { value, .. } => {
                        last_value = value;
                        break;
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
        })();

        self.environment_mut().pop_scope();
        result
    }

    /// Execute a block body and return ControlFlow (for use in iterators like .each)
    /// This version propagates Break/Continue instead of converting them to errors
    pub(crate) fn execute_block_with_control_flow(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
    ) -> Result<ControlFlow, MetorexError> {
        self.environment_mut().push_scope();

        let result = (|| -> Result<ControlFlow, MetorexError> {
            // Define captured variables using shared references
            for (name, value_ref) in block.captured_vars() {
                self.environment_mut()
                    .define_shared(name.clone(), value_ref.clone());
            }

            // Define parameters as regular variables
            for (param, argument) in block.parameters().iter().zip(arguments.into_iter()) {
                self.environment_mut().define(param.clone(), argument);
            }

            for statement in block.body() {
                match self.execute_statement(statement)? {
                    ControlFlow::Next => {}
                    flow @ (ControlFlow::Return { .. }
                    | ControlFlow::Break { .. }
                    | ControlFlow::Continue { .. }
                    | ControlFlow::Exception { .. }) => {
                        return Ok(flow);
                    }
                }
            }

            Ok(ControlFlow::Next)
        })();

        self.environment_mut().pop_scope();
        result
    }

    /// Invoke a resolved method with evaluated arguments.
    pub(crate) fn invoke_method(
        &mut self,
        class: Rc<Class>,
        method: Rc<Method>,
        receiver: Object,
        arguments: Vec<Object>,
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
        let positional_count = positional_arg_count(&arguments);
        if expected != positional_count {
            return Err(method_argument_error(
                &method_name,
                expected,
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
        self.environment_mut().push_scope();

        // Take the pending block now so nested calls don't see it.
        let block = self.pending_block.take();

        let result = (|| -> Result<Object, MetorexError> {
            self.environment_mut()
                .define("self".to_string(), self_value.clone());

            // Inject captured closure variables (from define_method blocks)
            if let Some(captured) = &method.captured_vars {
                for (name, value_ref) in captured {
                    self.environment_mut()
                        .define_shared(name.clone(), value_ref.clone());
                }
            }

            let (positional, kwargs) = split_keyword_args(arguments);
            for (param, value) in method.parameters.iter().zip(positional.into_iter()) {
                self.environment_mut().define(param.clone(), value);
            }
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
            }

            // Execute all statements, tracking the last expression value
            let body = method.body();
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
                            last_value = self.evaluate_unless_expression(
                                condition,
                                then_branch,
                                else_branch,
                            )?;
                            continue;
                        }
                        _ => {}
                    }
                }

                match self.execute_statement(statement)? {
                    ControlFlow::Next => continue,
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
        })();

        self.environment_mut().pop_scope();
        result
    }

    /// Execute the body of a standalone function within a fresh scope (no self).
    pub(crate) fn execute_function_body(
        &mut self,
        function: &Method,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        self.environment_mut().push_scope();

        // Take the pending block now so nested calls don't see it.
        let block = self.pending_block.take();

        let result = (|| -> Result<Object, MetorexError> {
            // Bind parameters to arguments (no self for standalone functions)
            let (positional, kwargs) = split_keyword_args(arguments);
            for (param, value) in function.parameters.iter().zip(positional.into_iter()) {
                self.environment_mut().define(param.clone(), value);
            }
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
            }

            // Execute all statements, tracking the last expression value
            let body = function.body();
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
                            last_value = self.evaluate_unless_expression(
                                condition,
                                then_branch,
                                else_branch,
                            )?;
                            continue;
                        }
                        _ => {}
                    }
                }

                match self.execute_statement(statement)? {
                    ControlFlow::Next => continue,
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
        })();

        self.environment_mut().pop_scope();
        result
    }

    /// Check if a class is an exception class (Exception or its subclasses)
    pub(crate) fn is_exception_class(&self, class: &Class) -> bool {
        Self::is_exception_class_static(class)
    }

    /// Static helper to check if a class is an exception class
    fn is_exception_class_static(class: &Class) -> bool {
        let exception_classes = [
            "Exception",
            "StandardError",
            "RuntimeError",
            "TypeError",
            "ValueError",
        ];

        // Check if the class name matches any exception class
        if exception_classes.contains(&class.name()) {
            return true;
        }

        // Check if any parent class is an exception class
        if let Some(superclass) = class.superclass() {
            return Self::is_exception_class_static(&superclass);
        }

        false
    }

    /// Bind named keyword parameters to the current scope.
    /// `kwargs` is a HashMap of string key → Object extracted from the keyword-args dict.
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

/// Count the number of positional arguments, excluding a trailing kwargs dict.
fn positional_arg_count(arguments: &[Object]) -> usize {
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if !dict.is_empty() && dict.keys().all(|k| k.starts_with(':')) {
            return arguments.len() - 1;
        }
    }
    arguments.len()
}

/// Split a list of evaluated arguments into positional args and keyword args.
/// If the last argument is a Dict with symbol-style keys, it's treated as keyword args.
fn split_keyword_args(mut arguments: Vec<Object>) -> (Vec<Object>, HashMap<String, Object>) {
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        // Keyword arg dicts use symbol-style keys (starting with ':')
        let all_symbol_keys = !dict.is_empty() && dict.keys().all(|k| k.starts_with(':'));
        if all_symbol_keys {
            let kwargs: HashMap<String, Object> = dict
                .iter()
                .map(|(k, v)| (k[1..].to_string(), v.clone()))
                .collect();
            drop(dict);
            arguments.pop();
            return (arguments, kwargs);
        }
    }
    (arguments, HashMap::new())
}
