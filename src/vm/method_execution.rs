//! Method and function body execution for the virtual machine.
//!
//! This module handles executing method bodies (with self) and standalone
//! function bodies (without self), including scope management, parameter
//! binding, and last-expression value capture.

use super::errors::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, VirtualMachine};
use crate::ast::{Expression, Statement, collect_assigned_locals};
use crate::callable::Callable;
use crate::class::Class;
use crate::error::{MetorexError, StackFrame};
use crate::lexer::Position;
use crate::object::{Method, Object};
use indexmap::IndexMap;
use std::rc::Rc;

use super::param_binding::{bind_params, positional_arg_count_for, split_keyword_args};

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

        // A name that `undef_method` retired raises the same NoMethodError as
        // a name nothing ever defined. The stored name carries the
        // `__class__` prefix for a class method, which the message drops.
        if method.is_undefined {
            let reported = method_name
                .strip_prefix("__class__")
                .unwrap_or(&method_name)
                .to_string();
            return Err(crate::vm::errors::undefined_method_error(
                &reported, &receiver, &arguments, position,
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

        // A stub copied under a new name (`define_singleton_method(:other,
        // method(:constants))`) still means the native method it was cut from.
        if method.body.is_empty()
            && method.captured_vars.is_none()
            && let Some(original) = &method.original_name
            && original != &method_name
            && let Some(result) =
                self.call_native_method(class.as_ref(), &receiver, original, &arguments, position)?
        {
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
        let takes_keywords =
            !method.keyword_parameters.is_empty() || method.keyword_rest_parameter.is_some();
        let mut positional_count = positional_arg_count_for(&arguments, takes_keywords);
        // If a trailing &block argument is passed and the method accepts a block parameter,
        // extract it as pending_block and don't count it as positional.
        if method.block_parameter.is_some()
            && !arguments.is_empty()
            && matches!(arguments.last(), Some(Object::Block(_)))
        {
            self.pending_block = arguments.pop();
            self.pending_block_from_ampersand = false;
            positional_count = positional_arg_count_for(&arguments, takes_keywords);
        }
        let has_variadic = method.variadic_param.is_some();
        let required =
            expected - method.default_parameters.len() - if has_variadic { 1 } else { 0 };
        if !has_variadic && (positional_count < required || positional_count > expected) {
            let accepted = if required == expected {
                crate::vm::errors::Arity::Exact(expected)
            } else {
                crate::vm::errors::Arity::Range(required, expected)
            };
            return Err(crate::vm::errors::argument_count_error(
                accepted,
                positional_count,
                position,
            ));
        }
        if has_variadic && positional_count < required {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(required),
                positional_count,
                position,
            ));
        }

        // Prefer the method's original owner class (if recorded) over the
        // class we dispatched through. For aliased/mixed-in methods this
        // lets `super` walk from the method's true defining class.
        let owning_class_name = method
            .owner
            .clone()
            .unwrap_or_else(|| class.name().to_string());
        // `super` follows the definition, so the frame is named for the
        // method as defined even when it was reached through an alias.
        let defined_name = method
            .original_name
            .clone()
            .unwrap_or_else(|| method_name.clone());
        let frame_name = format!("{}#{}", owning_class_name, defined_name);
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
        // A method body sees the refinements that were active where it was
        // defined, and only those: the caller's activations are not lexically
        // in scope for it.
        // A method defined inside a `refine` block reads its module's
        // refinements as they stand when it runs, so a sibling refinement
        // declared after it is still in force. Every other method sees the
        // snapshot taken where it was defined, so a refinement added to an
        // already-activated module does not reach it.
        let defined_in_refinement = method.owner_class.as_ref().is_some_and(|owner| {
            owner
                .get_class_var(crate::vm::REFINEMENT_LABEL_KEY)
                .is_some()
        });
        let captured_scope: Vec<crate::vm::core::RefinementEntry> = if defined_in_refinement {
            method
                .captured_refinements
                .iter()
                .flat_map(|(module, _)| Self::refinement_entries_for(module))
                .collect()
        } else {
            method
                .captured_refinements
                .iter()
                .map(|(module, classes)| crate::vm::core::RefinementEntry {
                    module: Rc::clone(module),
                    classes: classes.iter().cloned().collect(),
                })
                .collect()
        };
        let caller_scopes = std::mem::replace(&mut self.refinement_scopes, vec![captured_scope]);
        // Snapshot the positional args so `super` (bare form, inside the
        // body) can forward them to the parent method.
        self.method_arg_stack.push(arguments_for_body.clone());
        // Where this method was defined, so a `super` in its body starts from
        // the right link even when the module has no name.
        self.method_owner_stack.push(
            method
                .owner_class
                .clone()
                .or_else(|| Some(Rc::clone(&class))),
        );
        // `Module.nesting` inside the body reports where the method was
        // defined, not the scopes open at the call site.
        self.method_nesting_stack
            .push(method.captured_nesting.clone());
        let execution_result = self.with_call_frame(
            CallFrame::method(
                frame_name.clone(),
                frame_location_string,
                method_name.clone(),
                defined_name.clone(),
            )
            .with_source_file(self.current_source_file.clone()),
            move |vm| {
                vm.execute_method_body(
                    method_for_body.as_ref(),
                    self_for_body.clone(),
                    arguments_for_body.clone(),
                )
            },
        );
        self.method_nesting_stack.pop();
        self.method_owner_stack.pop();
        self.method_arg_stack.pop();
        self.refinement_scopes = caller_scopes;
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

        // A method produced by `Method#to_proc` keeps running against the
        // object it was extracted from, whatever receiver it is invoked on.
        let self_value = match &method.bound_self {
            Some(bound) => (**bound).clone(),
            None => self_value,
        };

        // Restore the lexical nesting the Proc was written in, so a nested
        // `def` in the body lands where Ruby's default definee points.
        let saved_def_scope = if method.captured_def_scope.is_empty() {
            None
        } else {
            Some(std::mem::replace(
                &mut self.def_scope_stack,
                method.captured_def_scope.clone(),
            ))
        };

        // The body runs in the file the method was defined in, which is what
        // a backtrace entry for a call made from here has to name.
        let saved_source_file = std::mem::replace(
            &mut self.current_source_file,
            method
                .source_location
                .as_ref()
                .and_then(|location| location.filename.clone()),
        );

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

            let (positional, kwargs) = split_keyword_args(
                arguments,
                !method.keyword_parameters.is_empty() || method.keyword_rest_parameter.is_some(),
            );
            bind_params(
                self,
                &method.parameters,
                &positional,
                &method.default_parameters,
                &method.variadic_param,
            )?;
            self.bind_keyword_params(
                &method.keyword_parameters,
                method.keyword_rest_parameter.as_deref(),
                kwargs,
            )?;

            // Bind the block: define block_given? as a Bool, __block__ for internal use,
            // and the named &block parameter if the method declared one.
            // A body that came from a `define_method` block keeps the
            // `block_given?` of the frame the block was written in, which is
            // why Ruby answers false there however the method is called.
            let inherits_block_given = method
                .captured_vars
                .as_ref()
                .is_some_and(|captured| captured.contains_key("block_given?"));
            if !inherits_block_given {
                self.environment_mut()
                    .define("block_given?".to_string(), Object::Bool(block.is_some()));
            }
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

            self.execute_body_statements(method.body(), method.lambda_body)
        })();

        if let Some(previous) = saved_def_scope {
            self.def_scope_stack = previous;
        }
        self.current_source_file = saved_source_file;
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
            let (positional, kwargs) = split_keyword_args(
                arguments,
                !function.keyword_parameters.is_empty()
                    || function.keyword_rest_parameter.is_some(),
            );
            bind_params(
                self,
                &function.parameters,
                &positional,
                &function.default_parameters,
                &function.variadic_param,
            )?;
            self.bind_keyword_params(
                &function.keyword_parameters,
                function.keyword_rest_parameter.as_deref(),
                kwargs,
            )?;

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

            self.execute_body_statements(function.body(), false)
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
    fn execute_body_statements(
        &mut self,
        body: &[Statement],
        lambda_semantics: bool,
    ) -> Result<Object, MetorexError> {
        if !lambda_semantics {
            return self.run_body_statements(body, false);
        }
        // A lambda-style body follows Proc-from-lambda control flow: `break`
        // and `next` finish the body with a value, and `redo` restarts it.
        loop {
            match self.run_body_statements(body, true) {
                Err(MetorexError::BlockRedo { .. }) => continue,
                Err(MetorexError::BlockNext { value, .. })
                | Err(MetorexError::BlockBreak { value, .. }) => return Ok(value),
                other => return other,
            }
        }
    }

    /// The value a statement produces when it is the last one in a body.
    /// Ruby's `if`, `unless`, `begin`, and assignment are expressions, so a
    /// method or block ending in one evaluates to its value. Returns `None`
    /// for statement kinds that carry no value of their own.
    pub(crate) fn terminal_statement_value(
        &mut self,
        statement: &Statement,
    ) -> Result<Option<Object>, MetorexError> {
        let value = match statement {
            Statement::Expression { expression, .. } => self.evaluate_expression(expression)?,
            Statement::Assignment { value, target, .. } => {
                let evaluated = self.evaluate_expression(value)?;
                self.assign_value(target, evaluated.clone())?;
                evaluated
            }
            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                ..
            } => {
                self.evaluate_if_expression(condition, then_branch, elsif_branches, else_branch)?
            }
            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.evaluate_unless_expression(condition, then_branch, else_branch)?,
            Statement::Begin {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                ..
            } => self.evaluate_begin_value(
                body,
                rescue_clauses,
                else_clause.as_deref(),
                ensure_block.as_deref(),
            )?,
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    /// Run a method body once, without the lambda-style restart loop.
    fn run_body_statements(
        &mut self,
        body: &[Statement],
        lambda_semantics: bool,
    ) -> Result<Object, MetorexError> {
        // Pre-define every local syntactically assigned-to in this body as
        // `nil`, matching Ruby's parser-level local hoisting. Without this,
        // an `ensure`/`rescue` clause that reads a variable defined later in
        // the body raises NameError when the body short-circuited via raise
        // before the assignment actually ran.
        for name in collect_assigned_locals(body) {
            if self.environment().get(&name).is_none() {
                self.environment_mut().define(name, Object::Nil);
            }
        }

        let mut last_value = Object::Nil;

        for (i, statement) in body.iter().enumerate() {
            let is_last = i == body.len() - 1;

            // If this is the last statement, capture its value
            if is_last && let Some(value) = self.terminal_statement_value(statement)? {
                last_value = value;
                continue;
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
                ControlFlow::Break { value, position } => {
                    if lambda_semantics {
                        return Ok(value);
                    }
                    return Err(loop_control_error("break", position));
                }
                ControlFlow::Redo { position } => {
                    if lambda_semantics {
                        return Err(MetorexError::BlockRedo {
                            location: position_to_location(position),
                        });
                    }
                    return Err(loop_control_error("redo", position));
                }
                ControlFlow::Continue { value, position } => {
                    if lambda_semantics {
                        return Ok(value);
                    }
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
        keyword_rest_parameter: Option<&str>,
        kwargs: IndexMap<String, Object>,
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

        if let Some(rest_name) = keyword_rest_parameter {
            let declared: std::collections::HashSet<&str> = keyword_parameters
                .iter()
                .map(|(name, _)| name.as_str())
                .collect();
            let rest: IndexMap<String, Object> = kwargs
                .iter()
                .filter(|(name, _)| !declared.contains(name.as_str()))
                .map(|(name, value)| (format!(":{}", name), value.clone()))
                .collect();
            self.environment_mut().define(
                rest_name.to_string(),
                Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(rest))),
            );
        }

        Ok(())
    }
}
