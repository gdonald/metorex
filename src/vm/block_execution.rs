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
/// Bind one block parameter. A `|(a, b)|` group spreads the value it is given
/// across the names in the group, filling nil where the array is shorter.
fn define_block_param(vm: &mut VirtualMachine, param: &str, value: Object) {
    let Some(names) = param.strip_prefix(crate::object::DESTRUCTURED_GROUP_PREFIX) else {
        vm.environment_mut().define(param.to_string(), value);
        return;
    };
    bind_group_names(vm, names, value);
}

/// The names a destructuring group holds, split at the commas that stand
/// outside any nested group.
fn group_parts(names: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut held = String::new();
    let mut depth = 0usize;
    for letter in names.chars() {
        match letter {
            '(' => {
                depth += 1;
                held.push(letter);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                held.push(letter);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut held)),
            _ => held.push(letter),
        }
    }
    if !held.is_empty() {
        parts.push(held);
    }
    parts
}

/// Spread one value across the names of a destructuring group. A group may
/// hold groups of its own, and one name may take whatever the others leave.
fn bind_group_names(vm: &mut VirtualMachine, names: &str, value: Object) {
    let spread = match &value {
        Object::Array(elements) => elements.borrow().clone(),
        other => vec![other.clone()],
    };
    let parts = group_parts(names);
    let star_at = parts.iter().position(|part| part.starts_with('*'));
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        // Everything the named parts do not take goes to the starred one.
        let taken = match star_at {
            Some(star) if index == star => {
                let after = parts.len() - index - 1;
                let upto = spread.len().saturating_sub(after).max(index);
                Object::array(spread.get(index..upto).unwrap_or(&[]).to_vec())
            }
            Some(star) if index > star => {
                let from_end = parts.len() - index;
                spread
                    .len()
                    .checked_sub(from_end)
                    .and_then(|at| spread.get(at).cloned())
                    .unwrap_or(Object::Nil)
            }
            _ => spread.get(index).cloned().unwrap_or(Object::Nil),
        };
        match part
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix(')'))
        {
            Some(nested) => bind_group_names(vm, nested, taken),
            None => {
                let name = part.strip_prefix('*').unwrap_or(part);
                if name.is_empty() {
                    continue;
                }
                vm.environment_mut().define(name.to_string(), taken);
            }
        }
    }
}

fn bind_block_params(
    vm: &mut VirtualMachine,
    params: &[String],
    defaults: &[(usize, crate::ast::Expression)],
    arguments: Vec<Object>,
) {
    // A trailing keyword-argument hash feeds the `name:` parameters, and
    // what is left over is bound by position.
    let keyword_params: Vec<String> = params
        .iter()
        .filter(|param| param.starts_with(crate::object::KEYWORD_PARAM_PREFIX))
        .cloned()
        .collect();
    let mut arguments = arguments;
    if !keyword_params.is_empty() {
        let named = match arguments.last() {
            Some(Object::Dict(entries)) => {
                let taken = entries.borrow().clone();
                arguments.pop();
                taken
            }
            _ => indexmap::IndexMap::new(),
        };
        for param in &keyword_params {
            let name = param
                .strip_prefix(crate::object::KEYWORD_PARAM_PREFIX)
                .unwrap_or(param)
                .to_string();
            let given = named
                .get(&format!(":{}", name))
                .or_else(|| named.get(&name))
                .cloned();
            let value = match given {
                Some(value) => value,
                // A keyword the call left out takes its default, and nil when
                // it declares none.
                None => {
                    let index = params
                        .iter()
                        .position(|declared| declared == param)
                        .unwrap_or(usize::MAX);
                    match defaults.iter().find(|(at, _)| *at == index) {
                        Some((_, default)) => {
                            vm.evaluate_expression(default).unwrap_or(Object::Nil)
                        }
                        None => Object::Nil,
                    }
                }
            };
            vm.environment_mut().define(name, value);
        }
    }
    // A `**kwargs` parameter takes no positional value, so it is left out of
    // the positional binding entirely.
    let params: Vec<String> = params
        .iter()
        .filter(|param| {
            !param.starts_with("**") && !param.starts_with(crate::object::KEYWORD_PARAM_PREFIX)
        })
        .cloned()
        .collect();
    let params = params.as_slice();
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
            if name.is_empty() {
                continue;
            }
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
            define_block_param(vm, param, value);
        }
    }

    // A `&name` parameter takes the block the call was handed, which is nil
    // when it was handed none.
    if let Some(bi) = block_idx {
        let name = params[bi].trim_start_matches('&').to_string();
        if !name.is_empty() {
            let given = vm.pending_block.take().unwrap_or(Object::Nil);
            vm.environment_mut().define(name, given);
        }
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

        // Variadic params accept any number of args; skip strict arity check.
        // Only a lambda checks arity at all: a proc pads missing arguments
        // with nil and drops extras.
        if !destructured {
            let _ = (has_variadic, has_block_param, required, expected);
            strict_arity_check(block, found, position)?;
        }

        let frame_name = block.name().to_string();
        let frame_location = position_to_location(position);
        let frame_location_string = Some(format!("{}", frame_location));

        let frame = match block.defining_method.clone() {
            Some((callee, defined)) => {
                CallFrame::method(frame_name.clone(), frame_location_string, callee, defined)
            }
            None => CallFrame::boundary(frame_name.clone()),
        }
        .with_source_file(self.current_source_file.clone());
        // The body runs in the file the block was written in, which is what a
        // backtrace entry for a call made from here has to name.
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);
        let execution_result =
            self.with_call_frame(frame, move |vm| vm.execute_block_body(block, arguments));
        self.current_source_file = saved_source_file;

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(frame_name, frame_location))),
        }
    }

    /// Run Ruby source with `self` bound to `receiver`, which is what the
    /// String form of `instance_eval` does. The source sees the receiver's
    /// instance variables and defines methods on its singleton class.
    pub(crate) fn evaluate_source_with_receiver(
        &mut self,
        source: &str,
        receiver: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let tokens = crate::lexer::Lexer::new(source).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .map_err(|errors| {
                MetorexError::runtime_error(
                    format!(
                        "instance_eval: parse error: {}",
                        errors
                            .iter()
                            .map(|error| error.to_string())
                            .collect::<Vec<_>>()
                            .join("; ")
                    ),
                    position_to_location(position),
                )
            })?;
        let singleton = self.singleton_class_of(&receiver);
        self.environment_mut().push_isolated_scope();
        self.environment_mut()
            .define("self".to_string(), receiver.clone());
        self.def_scope_stack.push(singleton);
        let mut last = Object::Nil;
        let result = (|| -> Result<(), MetorexError> {
            for statement in &statements {
                if let Statement::Expression { expression, .. } = statement {
                    if !self.tracepoints.is_empty() {
                        self.fire_line_event(statement.position())?;
                    }
                    last = self.evaluate_expression(expression)?;
                    continue;
                }
                if let ControlFlow::Value(value) = self.execute_statement(statement)? {
                    last = value;
                }
            }
            Ok(())
        })();
        self.def_scope_stack.pop();
        self.environment_mut().pop_scope();
        result?;
        Ok(last)
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
        let frame = match block.defining_method.clone() {
            Some((callee, defined)) => {
                CallFrame::method(frame_name.clone(), frame_location_string, callee, defined)
            }
            None => CallFrame::boundary(frame_name.clone()),
        }
        .with_source_file(self.current_source_file.clone());
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);
        // A `def` in the body defines a method on the receiver alone, and a
        // class variable written there belongs to the class or module the
        // block was written in.
        // The class or module the block was written in. A block written in a
        // method body belongs to the class holding that method, whatever
        // scope was open where the method was called from.
        let lexical = self
            .method_owner_stack
            .last()
            .cloned()
            .flatten()
            .or_else(|| block.captured_def_scope.last().cloned())
            .or_else(|| {
                self.method_nesting_stack
                    .last()
                    .and_then(|nesting| nesting.last().cloned())
            })
            .or_else(|| self.def_scope_stack.last().cloned())
            .or_else(|| match self.environment().get("self") {
                Some(Object::Class(held) | Object::Module(held)) => Some(held),
                _ => None,
            });

        let carried = lexical.is_some();
        if let Some(home) = lexical {
            self.class_var_home.push(home);
        }
        // A `def` written at the top of the body belongs to the receiver
        // alone. Anything the body calls out to keeps its own definee, so
        // this reaches only the statements written here.
        let definee = block
            .body()
            .iter()
            .any(|statement| {
                matches!(
                    statement,
                    Statement::MethodDef { .. } | Statement::FunctionDef { .. }
                )
            })
            .then(|| self.singleton_class_of(&receiver));
        let execution_result = self.with_call_frame(frame, move |vm| {
            vm.environment_mut().push_isolated_scope();
            let result = (|| -> Result<Object, MetorexError> {
                for (name, value_ref) in block.captured_vars() {
                    vm.environment_mut()
                        .define_captured(name.clone(), value_ref.clone());
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

                let mut last_value;
                // `redo` runs the block's body again over the same arguments.
                'again: loop {
                    last_value = Object::Nil;
                    for statement in block.body() {
                        if let Some(definee) = &definee
                            && matches!(
                                statement,
                                Statement::MethodDef { .. } | Statement::FunctionDef { .. }
                            )
                        {
                            let definee = std::rc::Rc::clone(definee);
                            last_value = vm.apply_class_body_statements(
                                &definee,
                                std::slice::from_ref(statement),
                                statement.position(),
                            )?;
                            continue;
                        }
                        if let Statement::Expression { expression, .. } = statement {
                            if !vm.tracepoints.is_empty() {
                                vm.fire_line_event(statement.position())?;
                            }
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
                                break 'again;
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
                                    home_frame: None,
                                });
                            }
                            ControlFlow::Redo { .. } => continue 'again,
                            ControlFlow::Continue { position, .. } => {
                                return Err(loop_control_error("continue", position));
                            }
                        }
                    }
                    break;
                }
                Ok(last_value)
            })();
            vm.environment_mut().pop_scope();
            result
        });
        if carried {
            self.class_var_home.pop();
        }
        self.current_source_file = saved_source_file;

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
        // A block body runs in its own scope seeded from `captured_vars`, not
        // chained to whatever method happens to be invoking it. Ruby's block
        // sees the locals of the scope it was written in, and nothing of its
        // caller's.
        self.environment_mut().push_isolated_scope();
        // Restore the lexical class/module nesting from the block's
        // definition site so an uppercase `Foo = ...` inside the body
        // assigns to the same enclosing module the surrounding code would
        // have. Saved/restored in pure stack fashion in case the caller's
        // current def_scope_stack is non-empty (e.g. block invoked from
        // inside a class body).
        let saved_def_scope =
            std::mem::replace(&mut self.def_scope_stack, block.captured_def_scope.clone());

        // A trace sees a block body opening and closing, and reads the
        // parameters the block declared off either event.
        let block_position = block
            .body
            .first()
            .map(|held| held.position())
            .unwrap_or_else(|| Position::new(0, 0, 0));
        let declared = crate::vm::native_methods::block_parameter_list(block);
        self.fire_event(
            "b_call",
            block_position,
            vec![("parameters", declared.clone())],
        )?;

        let result = (|| -> Result<Object, MetorexError> {
            // Define captured variables using shared references
            for (name, value_ref) in block.captured_vars() {
                self.environment_mut()
                    .define_captured(name.clone(), value_ref.clone());
            }

            // A lambda takes its arguments the way a method does, so the
            // count has to match what it declared.
            if block.is_lambda {
                check_lambda_arity(block, &arguments, Position::new(0, 0, 0))?;
            }

            // Define parameters as regular variables (handles *args/&block prefixes)
            bind_block_params(
                self,
                &block.binding_parameters(),
                &block.parameter_defaults,
                arguments,
            );

            // A name written after the `;` in the parameter list is a local
            // of the block, which starts as nil however the outer scope reads.
            for name in block.block_locals() {
                self.environment_mut().define(name, Object::Nil);
            }

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

            let mut last_value;

            // `redo` runs the block's body again over the same arguments.
            'again: loop {
                last_value = Object::Nil;
                for statement in block.body() {
                    if let Statement::Expression { expression, .. } = statement {
                        if !self.tracepoints.is_empty() {
                            self.fire_line_event(statement.position())?;
                        }
                        last_value = self.evaluate_expression(expression)?;
                        continue;
                    }

                    match self.execute_statement(statement)? {
                        ControlFlow::Next => {}
                        ControlFlow::Value(value) => {
                            last_value = value;
                        }
                        // `return` in a lambda returns from the lambda. In a
                        // proc or ordinary block it is a long return: it unwinds
                        // to the method that lexically created the block.
                        ControlFlow::Return { value, position } => {
                            if block.is_lambda {
                                last_value = value;
                                break 'again;
                            }
                            // The invocation the block was written in may have
                            // returned already, and then the return has nowhere
                            // to go.
                            if let Some(home) = block.home_frame
                                && !self.live_frames.contains(&home)
                            {
                                return Err(orphaned_return_error(value, position));
                            }
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: position_to_location(position),
                                home_frame: block.home_frame,
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
                            // `break` in a lambda ends the lambda, the way a
                            // `return` written there does.
                            if block.is_lambda {
                                last_value = value;
                                break 'again;
                            }
                            // Ruby: `break <value>` inside a block unwinds to the
                            // method that received the block, returning `value`
                            // from that method call. Uses BlockBreak so the signal
                            // survives `execute_method_body` (which only swallows
                            // NonLocalReturn) and is caught at the invoke boundary.
                            return Err(MetorexError::BlockBreak {
                                value,
                                location: position_to_location(position),
                                home_frame: None,
                            });
                        }
                        ControlFlow::Redo { .. } => continue 'again,
                        // `next <value>` ends this run of the block with that
                        // value, which is what the method holding the block sees.
                        ControlFlow::Continue { value, .. } => {
                            last_value = value;
                            break 'again;
                        }
                    }
                }
                break;
            }

            Ok(last_value)
        })();

        self.environment_mut().pop_scope();
        self.def_scope_stack = saved_def_scope;
        // A `break` leaving this body belongs to the invocation that was
        // handed the block, which is the call made from the frame the block
        // was written in.
        match result {
            // `next` written inside an expression unwinds to here, and ends
            // this run of the block with the value it carried.
            Err(MetorexError::BlockNext { value, .. }) => Ok(value),
            Err(MetorexError::BlockBreak {
                value,
                location,
                home_frame: None,
            }) => Err(MetorexError::BlockBreak {
                value,
                location,
                home_frame: block.home_frame,
            }),
            other => other,
        }
    }

    /// Execute a block body and return ControlFlow (for use in iterators like .each)
    /// This version propagates Break/Continue instead of converting them to errors
    pub(crate) fn execute_block_with_control_flow(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
    ) -> Result<ControlFlow, MetorexError> {
        // `{ |x, y| }` handed a single array spreads it across the parameters,
        // which is how `[[1, 2]].each { |x, y| }` binds x and y.
        let arguments = match (block.destructures_single_array(), arguments.first()) {
            (true, Some(Object::Array(elements))) if arguments.len() == 1 => {
                elements.borrow().clone()
            }
            _ => arguments,
        };
        // A lambda takes its arguments the way a method does, so a yield that
        // does not match its parameters is refused rather than padded.
        strict_arity_check(block, arguments.len(), Position::new(0, 0, 0))?;
        self.environment_mut().push_isolated_scope();
        // The body belongs to the file the block was written in.
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);

        let result = (|| -> Result<ControlFlow, MetorexError> {
            // Define captured variables using shared references
            for (name, value_ref) in block.captured_vars() {
                self.environment_mut()
                    .define_captured(name.clone(), value_ref.clone());
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

            // `redo` runs the block's body again over the same arguments,
            // which are already bound in this scope.
            loop {
                let mut again = false;
                for statement in block.body() {
                    match self.execute_statement(statement)? {
                        ControlFlow::Next | ControlFlow::Value(_) => {}
                        ControlFlow::Redo { .. } => {
                            again = true;
                            break;
                        }
                        flow @ (ControlFlow::Return { .. }
                        | ControlFlow::Break { .. }
                        | ControlFlow::Continue { .. }
                        | ControlFlow::Exception { .. }) => {
                            return Ok(flow);
                        }
                    }
                }
                if !again {
                    return Ok(ControlFlow::Next);
                }
            }
        })();

        self.current_source_file = saved_source_file;
        self.environment_mut().pop_scope();
        result
    }
}

/// A `return` from a block whose defining method has already returned. Ruby
/// reports it as a LocalJumpError carrying the value and the reason.
fn orphaned_return_error(value: Object, position: Position) -> MetorexError {
    let message = "unexpected return".to_string();
    let exception = Object::exception("LocalJumpError", message.clone());
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details
            .instance_vars
            .insert("@exit_value".to_string(), value);
        details
            .instance_vars
            .insert("@reason".to_string(), Object::symbol("return".to_string()));
    }
    MetorexError::UncaughtException {
        exception,
        location: position_to_location(position),
        message,
    }
}

/// A lambda refuses a call that gives it the wrong number of arguments, the
/// way a method does. A splat parameter makes the upper bound open, and a
/// parameter with a default makes the lower bound smaller.
fn check_lambda_arity(
    block: &BlockStatement,
    arguments: &[Object],
    position: Position,
) -> Result<(), MetorexError> {
    let names = block.binding_parameters();
    let positional: Vec<&String> = names
        .iter()
        .filter(|name| {
            !name.starts_with('&')
                && !name.starts_with(crate::object::KEYWORD_PARAM_PREFIX)
                && !name.starts_with("**")
        })
        .collect();
    if positional.iter().any(|name| name.starts_with('*')) {
        return Ok(());
    }
    let takes_keywords = names
        .iter()
        .any(|name| name.starts_with(crate::object::KEYWORD_PARAM_PREFIX));
    let given = if takes_keywords && matches!(arguments.last(), Some(Object::Dict(_))) {
        arguments.len().saturating_sub(1)
    } else {
        arguments.len()
    };
    let expected = positional.len();
    let optional = block
        .parameter_defaults
        .iter()
        .filter(|(index, _)| {
            names
                .get(*index)
                .is_some_and(|name| !name.starts_with(crate::object::KEYWORD_PARAM_PREFIX))
        })
        .count();
    let required = expected.saturating_sub(optional);
    if given >= required && given <= expected {
        return Ok(());
    }
    let accepted = if required == expected {
        crate::vm::errors::Arity::Exact(expected)
    } else {
        crate::vm::errors::Arity::Range(required, expected)
    };
    Err(crate::vm::errors::argument_count_error(
        accepted, given, position,
    ))
}

/// A lambda counts its arguments the way a method does. A proc pads what is
/// missing with nil and drops what is extra, so nothing is checked for one.
fn strict_arity_check(
    block: &BlockStatement,
    found: usize,
    position: Position,
) -> Result<(), MetorexError> {
    if !block.is_lambda {
        return Ok(());
    }
    let parameters = block.binding_parameters();
    if parameters
        .iter()
        .any(|name| name.starts_with('*') || name.starts_with('&'))
    {
        return Ok(());
    }
    let expected = parameters.len();
    let required = expected.saturating_sub(block.parameter_defaults.len());
    if found >= required && found <= expected {
        return Ok(());
    }
    let accepted = if required == expected {
        crate::vm::errors::Arity::Exact(expected)
    } else {
        crate::vm::errors::Arity::Range(required, expected)
    };
    Err(crate::vm::errors::argument_count_error(
        accepted, found, position,
    ))
}
