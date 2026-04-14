//! Block execution for the virtual machine.
//!
//! This module handles the execution of block/lambda/proc objects,
//! including scope capture, control flow, and instance_exec semantics.

use super::errors::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, VirtualMachine};
use crate::ast::Statement;
use crate::callable::Callable;
use crate::error::{MetorexError, StackFrame};
use crate::lexer::Position;
use crate::object::{BlockStatement, Object};

/// Bind block parameters to arguments, handling `*args` (variadic) and
/// `&block` (block) prefixes in parameter names.
fn bind_block_params(vm: &mut VirtualMachine, params: &[String], arguments: Vec<Object>) {
    // Find variadic param index (if any)
    let variadic_idx = params.iter().position(|p| p.starts_with('*'));
    let block_idx = params.iter().position(|p| p.starts_with('&'));
    let has_variadic = variadic_idx.is_some();

    if has_variadic {
        let vi = variadic_idx.unwrap();
        // Count non-block positional params
        let positional_params: Vec<&String> =
            params.iter().filter(|p| !p.starts_with('&')).collect();
        let params_after_splat = positional_params.len() - vi - 1;
        let min_positional = vi + params_after_splat;
        let splat_count = arguments.len().saturating_sub(min_positional);

        for (i, param) in positional_params.iter().enumerate() {
            let name = param.trim_start_matches('*').to_string();
            let value = if i < vi {
                arguments.get(i).cloned().unwrap_or(Object::Nil)
            } else if i == vi {
                let rest: Vec<Object> = arguments.get(vi..vi + splat_count).unwrap_or(&[]).to_vec();
                Object::Array(std::rc::Rc::new(std::cell::RefCell::new(rest)))
            } else {
                let offset_from_end = positional_params.len() - i;
                let idx = arguments.len().saturating_sub(offset_from_end);
                arguments.get(idx).cloned().unwrap_or(Object::Nil)
            };
            vm.environment_mut().define(name, value);
        }
    } else {
        for (param, argument) in params.iter().zip(arguments.into_iter()) {
            if param.starts_with('&') {
                continue; // block param, handled separately
            }
            vm.environment_mut().define(param.clone(), argument);
        }
    }

    // Bind block param to nil for now (no block passing through blocks)
    if let Some(bi) = block_idx {
        let name = params[bi].trim_start_matches('&').to_string();
        vm.environment_mut().define(name, Object::Nil);
    }
}

impl VirtualMachine {
    /// Execute a block callable within the VM, handling scope capture and return semantics.
    pub(crate) fn execute_block_callable(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let expected = block.arity();
        let found = arguments.len();
        let has_variadic = block.parameters().iter().any(|p| p.starts_with('*'));
        let has_block_param = block.parameters().iter().any(|p| p.starts_with('&'));

        // Variadic params accept any number of args; skip strict arity check
        if !has_variadic && !has_block_param && expected != found {
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

    /// Execute a block with a specific `self` receiver (for instance_exec/instance_eval).
    /// The receiver overrides any captured `self` from the block's closure.
    pub(crate) fn execute_block_with_receiver(
        &mut self,
        block: &BlockStatement,
        receiver: Object,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let frame_name = block.name().to_string();
        let frame_location = position_to_location(position);
        let frame_location_string = Some(format!("{}", frame_location));

        let execution_result = self.with_call_frame(
            CallFrame::new(frame_name.clone(), frame_location_string),
            move |vm| {
                vm.environment_mut().push_scope();
                let result = (|| -> Result<Object, MetorexError> {
                    for (name, value_ref) in block.captured_vars() {
                        vm.environment_mut()
                            .define_shared(name.clone(), value_ref.clone());
                    }
                    // Override `self` with the instance_exec receiver
                    vm.environment_mut().define("self".to_string(), receiver);

                    bind_block_params(vm, block.parameters(), arguments);

                    let mut last_value = Object::Nil;
                    for statement in block.body() {
                        if let Statement::Expression { expression, .. } = statement {
                            last_value = vm.evaluate_expression(expression)?;
                            continue;
                        }
                        match vm.execute_statement(statement)? {
                            ControlFlow::Next => {}
                            ControlFlow::Value(value) => {
                                last_value = value;
                            }
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
                vm.environment_mut().pop_scope();
                result
            },
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

            // Define parameters as regular variables (handles *args/&block prefixes)
            bind_block_params(self, block.parameters(), arguments);

            let mut last_value = Object::Nil;

            for statement in block.body() {
                if let Statement::Expression { expression, .. } = statement {
                    last_value = self.evaluate_expression(expression)?;
                    continue;
                }

                match self.execute_statement(statement)? {
                    ControlFlow::Next => {}
                    ControlFlow::Value(value) => {
                        last_value = value;
                    }
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

            // Define parameters as regular variables (handles *args/&block prefixes)
            bind_block_params(self, block.parameters(), arguments);

            for statement in block.body() {
                match self.execute_statement(statement)? {
                    ControlFlow::Next | ControlFlow::Value(_) => {}
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
}
