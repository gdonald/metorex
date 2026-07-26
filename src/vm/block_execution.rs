//! Block execution for the virtual machine.
//!
//! This module handles the execution of block/lambda/proc objects,
//! including scope capture, control flow, and instance_exec semantics.

use super::errors::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, VirtualMachine};
use crate::ast::{Statement, collect_assigned_locals};
use crate::callable::Callable;
use crate::error::{MetorexError, StackFrame};
use crate::lexer::Position;
use crate::object::{BlockStatement, Object};

/// Bind block parameters to arguments, handling `*args` (variadic) and
/// `&block` (block) prefixes in parameter names. `defaults` carries
/// default-value expressions keyed by index into `params`; they evaluate
/// in the block's fresh scope when the corresponding argument is missing.
fn bind_block_params(
    vm: &mut VirtualMachine,
    params: &[String],
    defaults: &[(usize, crate::ast::Expression)],
    arguments: Vec<Object>,
) {
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
        let positional: Vec<(usize, &String)> = params
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.starts_with('&'))
            .collect();
        for (pos, (orig_idx, param)) in positional.iter().enumerate() {
            let value = match arguments.get(pos) {
                Some(v) => v.clone(),
                None => match defaults.iter().find(|(di, _)| di == orig_idx) {
                    Some((_, default_expr)) => {
                        vm.evaluate_expression(default_expr).unwrap_or(Object::Nil)
                    }
                    None => Object::Nil,
                },
            };
            vm.environment_mut().define((*param).clone(), value);
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
        // `{ |a,| }` destructures a lone array argument across its parameters,
        // discarding any elements it has no parameter for.
        let mut destructured = false;
        let arguments = match (block.destructures_single_array(), arguments.first()) {
            (true, Some(Object::Array(elements))) if arguments.len() == 1 => {
                destructured = true;
                elements.borrow().clone()
            }
            _ => arguments,
        };

        let parameters = block.binding_parameters();
        let expected = parameters.len();
        let found = arguments.len();
        let has_variadic = parameters.iter().any(|p| p.starts_with('*'));
        let has_block_param = parameters.iter().any(|p| p.starts_with('&'));

        // Optional params (`|a, b = 1|`) widen the accepted count: `found`
        // may run from `expected - defaults` up to `expected`.
        let required = expected.saturating_sub(block.parameter_defaults.len());

        // Variadic params accept any number of args; skip strict arity check
        if !has_variadic
            && !has_block_param
            && !destructured
            && (found < required || found > expected)
        {
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

                    bind_block_params(vm, block.parameters(), &block.parameter_defaults, arguments);

                    // Pre-bind syntactically assigned locals to nil (Ruby's
                    // parser-level local hoisting) so an `ensure`/`rescue`
                    // clause that reads a variable defined later in the body
                    // returns nil instead of NameError when execution
                    // short-circuits via raise.
                    for name in collect_assigned_locals(block.body()) {
                        if vm.environment().get(&name).is_none() {
                            vm.environment_mut().define(name, Object::Nil);
                        }
                    }

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
                            ControlFlow::Break { value, position } => {
                                return Err(MetorexError::BlockBreak {
                                    value,
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
        // Restore the lexical class/module nesting from the block's
        // definition site so an uppercase `Foo = ...` inside the body
        // assigns to the same enclosing module the surrounding code would
        // have. Saved/restored in pure stack fashion in case the caller's
        // current def_scope_stack is non-empty (e.g. block invoked from
        // inside a class body).
        let saved_def_scope =
            std::mem::replace(&mut self.def_scope_stack, block.captured_def_scope.clone());

        let result = (|| -> Result<Object, MetorexError> {
            // Define captured variables using shared references
            for (name, value_ref) in block.captured_vars() {
                self.environment_mut()
                    .define_shared(name.clone(), value_ref.clone());
            }

            // Define parameters as regular variables (handles *args/&block prefixes)
            bind_block_params(
                self,
                &block.binding_parameters(),
                &block.parameter_defaults,
                arguments,
            );

            // Pre-define every local syntactically assigned-to in this block
            // body as `nil`, so a read that runs before its assignment line
            // (e.g. inside an `ensure` clause that fires after an early raise)
            // returns nil rather than raising NameError. Mirrors Ruby's
            // parser-level local-variable hoisting.
            for name in collect_assigned_locals(block.body()) {
                if self.environment().get(&name).is_none() {
                    self.environment_mut().define(name, Object::Nil);
                }
            }

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
                    ControlFlow::Break { value, position } => {
                        // Ruby: `break <value>` inside a block unwinds to the
                        // method that received the block, returning `value`
                        // from that method call. Uses BlockBreak so the signal
                        // survives `execute_method_body` (which only swallows
                        // NonLocalReturn) and is caught at the invoke boundary.
                        return Err(MetorexError::BlockBreak {
                            value,
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
        })();

        self.environment_mut().pop_scope();
        self.def_scope_stack = saved_def_scope;
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
            bind_block_params(
                self,
                &block.binding_parameters(),
                &block.parameter_defaults,
                arguments,
            );

            // Pre-bind syntactically assigned locals to nil — see
            // execute_block_body for the rationale.
            for name in collect_assigned_locals(block.body()) {
                if self.environment().get(&name).is_none() {
                    self.environment_mut().define(name, Object::Nil);
                }
            }

            for statement in block.body() {
                match self.execute_statement(statement)? {
                    ControlFlow::Next | ControlFlow::Value(_) => {}
                    flow @ (ControlFlow::Return { .. }
                    | ControlFlow::Break { .. }
                    | ControlFlow::Redo { .. }
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
