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
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Methods answered natively can call one another without evaluating
        // an expression in between, so calls count toward how deep the
        // program is nested as well.
        crate::vm::program::enter_nesting(position)?;
        // A stub with no body and no source stands for a native method,
        // which a trace sees as a method written in C.
        let traced_as_c = !self.tracepoints.is_empty()
            && method.body.is_empty()
            && method
                .source_location
                .as_ref()
                .is_none_or(|written| written.filename.is_none());
        let answered = if traced_as_c {
            self.invoke_native_stub(class, method, receiver, arguments, position)
        } else {
            self.invoke_method_unguarded(class, method, receiver, arguments, position)
        };
        crate::vm::program::leave_nesting();
        answered
    }

    /// Run a method that stands for a native one between its `c_call` and
    /// `c_return` events. The calls it makes itself are Ruby's C code calling
    /// on, which fire no events of their own.
    fn invoke_native_stub(
        &mut self,
        class: Rc<Class>,
        method: Rc<Method>,
        receiver: Object,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let name = method.name.clone();
        self.fire_native_event("c_call", &name, &receiver, None, position)?;
        self.native_calls_running += 1;
        let answered =
            self.invoke_method_unguarded(class, method, receiver.clone(), arguments, position);
        self.native_calls_running -= 1;
        let value = answered?;
        self.fire_native_event("c_return", &name, &receiver, Some(&value), position)?;
        Ok(value)
    }

    fn invoke_method_unguarded(
        &mut self,
        class: Rc<Class>,
        method: Rc<Method>,
        receiver: Object,
        mut arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let method_name = method.name.clone();
        // A Proc cut from a bound Method carries the object it was bound to,
        // so the native tables answer for that one rather than for whatever
        // the new name was called on.
        let receiver = match &method.bound_self {
            Some(bound) => (**bound).clone(),
            None => receiver,
        };

        // A name that `undef_method` retired raises the same NoMethodError as
        // a name nothing ever defined. The stored name carries the
        // `__class__` prefix for a class method, which the message drops.
        if method.is_undefined {
            let reported = method_name
                .strip_prefix("__class__")
                .unwrap_or(&method_name)
                .to_string();
            let wording = self.receiver_wording_for(&receiver, position);
            return Err(crate::vm::errors::undefined_method_error_worded(
                &reported, &receiver, &arguments, wording, position,
            ));
        }

        // An instance of a String, Array, Set, or Hash subclass answers the
        // native table through the collection it is backed by, so a method the
        // subclass writes itself has to run its own body rather than be
        // shadowed by that table. Everything else reaches the native table
        // first, which is what puts `Integer#div` ahead of the `Numeric#div`
        // written for the subclasses a program defines.
        // A method written on a class object's own singleton runs its own
        // body rather than the native table's, which is what lets a program
        // stand one in for `Dir.tmpdir`. The library's own `def self.name`
        // methods live in the class's method table instead, and stay behind
        // the native table the way they were written to.
        let stands_for_a_class_method = match &receiver {
            Object::Class(held) | Object::Module(held) => {
                held.singleton_class_slot()
                    .as_ref()
                    .is_some_and(|singleton| singleton.find_own_method(&method_name).is_some())
                    || held
                        .find_own_method(&format!("__class__{}", method_name))
                        .is_some_and(|written| written_by_the_program(&written))
            }
            _ => false,
        };
        // A method written on the object's own singleton class runs its own
        // body rather than the native table's, which is what lets a program
        // stand one in for `Regexp#match` on a single pattern.
        let stands_on_its_own_singleton = class.is_singleton_class()
            && !matches!(receiver, Object::Class(_) | Object::Module(_))
            && class.find_own_method(&method_name).is_some();
        // A method a refinement puts in place of one the interpreter answers
        // natively runs its own body, which is the whole point of writing it.
        let stands_in_a_refinement = method.owner_class.as_ref().is_some_and(|owner| {
            owner
                .get_class_var(crate::vm::REFINEMENT_LABEL_KEY)
                .is_some()
        });
        let overrides_builtin = !method.body.is_empty()
            && (backs_a_collection(&receiver)
                || stands_for_a_class_method
                || stands_on_its_own_singleton
                || stands_in_a_refinement);
        if !overrides_builtin
            && let Some(result) = self.call_native_method(
                class.as_ref(),
                &receiver,
                &method_name,
                &arguments,
                position,
            )?
        {
            return Ok(result);
        }

        // A stub standing in for a native method dispatches under the name it
        // was cut from, even when the alias has been put back under that same
        // name. There is no body to run either way.
        if let Some(target) = method.native_alias.clone() {
            for owner in [Rc::clone(&class), self.builtins().class_of(&receiver)] {
                if let Some(result) = self.call_native_method(
                    owner.as_ref(),
                    &receiver,
                    &target,
                    &arguments,
                    position,
                )? {
                    return Ok(result);
                }
            }
            // Arithmetic on a number is not in any native table, and going
            // back through the name would reach a redefinition rather than
            // the operator this stub was cut from.
            if arguments.len() == 1
                && let Some(result) = self.builtin_number_operator(
                    &target,
                    receiver.clone(),
                    arguments[0].clone(),
                    position,
                )
            {
                return result;
            }
            // `new` on a class is the constructor rather than an entry in any
            // native table, so it is reached through the class itself.
            if target == "new"
                && let Object::Class(constructed) = &receiver
            {
                let constructed = Rc::clone(constructed);
                return self.invoke_class(constructed, arguments, position);
            }
            if let Some(result) =
                self.call_object_method(&receiver, &target, &arguments, position)?
            {
                return Ok(result);
            }
            // A Kernel function is reached without a receiver rather than
            // through any class's table, so the stub goes there for it.
            if crate::vm::native_methods::is_kernel_private_function(&target) {
                return self.call_native_function(&target, arguments, position);
            }
        }

        // A stub copied under a new name (`define_singleton_method(:other,
        // method(:constants))`) still means the native method it was cut from.
        // The receiver's own class is what dispatches it, since the stub may
        // live on a singleton class that names no native table.
        if method.body.is_empty()
            && method.captured_vars.is_none()
            && let Some(original) = &method.original_name
            && original != &method_name
        {
            let original = original.clone();
            for owner in [Rc::clone(&class), self.builtins().class_of(&receiver)] {
                if let Some(result) = self.call_native_method(
                    owner.as_ref(),
                    &receiver,
                    &original,
                    &arguments,
                    position,
                )? {
                    return Ok(result);
                }
            }
        }

        // A Kernel function is reached without a receiver rather than through
        // any class's table, so a stub standing in for one goes there for it,
        // naming the receiver it was bound to.
        if method.body.is_empty()
            && method.captured_vars.is_none()
            && method.owner.as_deref() == Some("Kernel")
            && crate::vm::native_methods::is_kernel_private_function(&method_name)
        {
            let held = self.kernel_function_receiver.replace(receiver.clone());
            let answered = self.call_native_function(&method_name, arguments, position);
            self.kernel_function_receiver = held;
            return answered;
        }

        // For stub methods (empty body, registered on Object for introspection),
        // fall through to base Object native methods (class, to_s, respond_to?, etc.)
        if method.body.is_empty() && method.captured_vars.is_none() {
            self.bound_stub_depth += 1;
            let answered = self.call_object_method(&receiver, &method_name, &arguments, position);
            self.bound_stub_depth -= 1;
            if let Some(result) = answered? {
                return Ok(result);
            }
        }

        let expected = method.parameters.len();
        let takes_keywords =
            !method.keyword_parameters.is_empty() || method.keyword_rest_parameter.is_some();
        let mut positional_count = positional_arg_count_for(&arguments, takes_keywords);
        // A trailing Proc argument stands in for the block only when the
        // method has no room for it as a positional, since `&` is what makes
        // a Proc the block: `detect(ifnone)` reads the Proc as the argument
        // it was written as.
        if method.block_parameter.is_some()
            && self.pending_block.is_none()
            && !arguments.is_empty()
            && matches!(arguments.last(), Some(Object::Block(_)))
            && method.variadic_param.is_none()
            && positional_count > method.parameters.len()
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
        // A method on a class or module is named with a dot, the way Ruby
        // writes `Foo.bar` in a backtrace, and an instance method with a hash.
        let separator = if matches!(receiver, Object::Class(_) | Object::Module(_)) {
            "."
        } else {
            "#"
        };
        let frame_name = format!("{}{}{}", owning_class_name, separator, defined_name);
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
        // A class variable written in the body belongs to the class or
        // module the method was written in, whatever the receiver is.
        self.class_var_cref_stack.push(
            method
                .captured_nesting
                .first()
                .cloned()
                .or_else(|| method.captured_def_scope.last().cloned())
                .or_else(|| Some(Rc::clone(&class))),
        );
        let written = method
            .source_location
            .as_ref()
            .and_then(|written| Some((written.filename.clone()?, written.line)));
        let opened_on = written.as_ref().map_or(position.line, |(_, line)| *line);
        self.enter_running_code(Rc::new(written.into_iter().collect()), opened_on);
        let execution_result =
            match self.fire_method_event("call", &method_name, &method, &class, None, position) {
                Ok(()) => self.with_call_frame(
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
                ),
                Err(error) => Err(error),
            };
        self.method_nesting_stack.pop();
        self.class_var_cref_stack.pop();
        self.method_owner_stack.pop();
        self.method_arg_stack.pop();
        self.refinement_scopes = caller_scopes;
        self.user_def_nesting = self.user_def_nesting.saturating_sub(1);

        let finished = match execution_result {
            Ok(value) => self
                .fire_method_event(
                    "return",
                    &method_name,
                    &method,
                    &class,
                    Some(&value),
                    position,
                )
                .map(|()| value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(frame_name, frame_location))),
        };
        self.leave_running_code();
        finished
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

        // A class variable written in this body belongs to this method's own
        // class, not to a block somewhere up the call stack.
        let saved_class_var_home = std::mem::take(&mut self.class_var_home);
        // The body runs in the file the method was defined in, which is what
        // a backtrace entry for a call made from here has to name.
        let saved_source_file = std::mem::replace(
            &mut self.current_source_file,
            method
                .source_location
                .as_ref()
                .and_then(|location| location.filename.clone()),
        );
        // A literal in the body is written in the encoding that file names,
        // not the one the caller's file names.
        let saved_source_encoding = std::mem::replace(
            &mut self.current_source_encoding,
            self.current_source_file
                .as_ref()
                .and_then(|named| self.file_encodings.get(named).cloned()),
        );

        // A call of a method written in a measured file is what the methods
        // mode counts.
        if self.coverage.is_some()
            && let Some(at) = method.source_location.as_ref()
            && let Some(named) = at.filename.as_deref()
        {
            let named = named.to_string();
            self.coverage_count_method(&named, &method.name, at.line);
        }

        // A `return` written in a block created inside this body unwinds to
        // this invocation and no other, so the body runs under an id the
        // blocks it makes record.
        let frame = self.next_method_frame;
        self.next_method_frame += 1;
        let saved_frame = self.current_method_frame.replace(frame);
        // A block opened in this body belongs to this method, whatever block
        // the call was made from.
        let saved_lexical_home = self.lexical_home_frame.take();
        self.live_frames.push(frame);

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

            let keyword_hash = arguments.last().cloned();
            let (positional, kwargs) = crate::vm::param_binding::split_keyword_args_for(
                arguments,
                !method.keyword_parameters.is_empty() || method.keyword_rest_parameter.is_some(),
                method.ruby2_keywords.get(),
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
            self.refuse_unknown_keywords(
                keyword_hash.as_ref(),
                &method.keyword_parameters,
                method.keyword_rest_parameter.as_deref(),
            )?;

            // Bind the block: define block_given? as a Bool, __block__ for internal use,
            // and the named &block parameter if the method declared one.
            // A body that came from a `define_method` block keeps the
            // `block_given?` of the frame the block was written in, which is
            // why Ruby answers false there however the method is called.
            let from_a_block = method.captured_vars.is_some();
            let inherits_block_given = method
                .captured_vars
                .as_ref()
                .is_some_and(|captured| captured.contains_key("block_given?"));
            if !inherits_block_given {
                self.environment_mut().define(
                    "block_given?".to_string(),
                    Object::Bool(block.is_some() && !from_a_block),
                );
            }
            if let Some(block_value) = block {
                if !from_a_block {
                    self.environment_mut()
                        .define("__block__".to_string(), block_value.clone());
                }
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
        self.current_source_encoding = saved_source_encoding;
        self.class_var_home = saved_class_var_home;
        self.current_method_frame = saved_frame;
        self.lexical_home_frame = saved_lexical_home;
        self.live_frames.pop();
        // A trace reading `binding` off a `return` event sees the method's
        // own locals, which are gone once the scope is popped.
        if !self.tracepoints.is_empty() {
            self.traced_binding = self
                .call_native_function("binding_kernel", Vec::new(), Position::new(0, 0, 0))
                .ok();
        }
        self.environment_mut().pop_scope();
        match result {
            Err(MetorexError::NonLocalReturn {
                value,
                location,
                home_frame,
            }) => {
                if home_frame.is_some_and(|home| home != frame) {
                    Err(MetorexError::NonLocalReturn {
                        value,
                        location,
                        home_frame,
                    })
                } else {
                    Ok(value)
                }
            }
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

        let frame = self.next_method_frame;
        self.next_method_frame += 1;
        let saved_frame = self.current_method_frame.replace(frame);
        // A block opened in this body belongs to this method, whatever block
        // the call was made from.
        let saved_lexical_home = self.lexical_home_frame.take();
        self.live_frames.push(frame);

        let result = (|| -> Result<Object, MetorexError> {
            // Bind parameters to arguments (no self for standalone functions)
            let keyword_hash = arguments.last().cloned();
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
            self.refuse_unknown_keywords(
                keyword_hash.as_ref(),
                &function.keyword_parameters,
                function.keyword_rest_parameter.as_deref(),
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

        self.current_method_frame = saved_frame;
        self.lexical_home_frame = saved_lexical_home;
        self.live_frames.pop();
        self.environment_mut().pop_scope();
        match result {
            Err(MetorexError::NonLocalReturn {
                value,
                location,
                home_frame,
            }) => {
                if home_frame.is_some_and(|home| home != frame) {
                    Err(MetorexError::NonLocalReturn {
                        value,
                        location,
                        home_frame,
                    })
                } else {
                    Ok(value)
                }
            }
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
            if self.environment().assignment_introduces_a_local(&name) {
                self.environment_mut().hoist(name);
            }
        }

        let mut last_value = Object::Nil;

        for (i, statement) in body.iter().enumerate() {
            let is_last = i == body.len() - 1;

            // The last statement is answered below rather than through
            // `execute_statement`, and it is a `:line` event all the same.
            if is_last && !self.tracepoints.is_empty() && answers_its_own_value(statement) {
                self.fire_line_event(statement.position())?;
            }
            // If this is the last statement, capture its value
            if is_last && let Some(value) = self.terminal_statement_value(statement)? {
                // The last statement answered here rather than through
                // `execute_statement`, so it is counted here too.
                if self.coverage.is_some() {
                    self.coverage_count(statement.position().line);
                }
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
                ControlFlow::Retry { position } => {
                    return Err(MetorexError::BlockRetry {
                        location: position_to_location(position),
                    });
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

    /// Refuse the keywords a call passed that the method declared no
    /// parameter for, unless it takes the rest in a `**` parameter. A method
    /// that declares no keywords at all takes them as a positional Hash.
    fn refuse_unknown_keywords(
        &mut self,
        keyword_hash: Option<&Object>,
        keyword_parameters: &[(String, Option<Expression>)],
        keyword_rest_parameter: Option<&str>,
    ) -> Result<(), MetorexError> {
        if keyword_parameters.is_empty() || keyword_rest_parameter.is_some() {
            return Ok(());
        }
        let Some(Object::Dict(entries)) = keyword_hash else {
            return Ok(());
        };
        let held = entries.borrow().clone();
        if !held.contains_key(crate::vm::param_binding::KWARGS_MARKER) {
            return Ok(());
        }
        let unknown: Vec<Object> = held
            .keys()
            .filter(|key| !crate::vm::native_methods::hash_methods::is_internal_key(key))
            .filter(|key| {
                !key.strip_prefix(':').is_some_and(|name| {
                    keyword_parameters
                        .iter()
                        .any(|(declared, _)| declared == name)
                })
            })
            .map(|key| crate::vm::native_methods::hash_methods::reconstruct_key(&held, key))
            .collect();
        if unknown.is_empty() {
            return Ok(());
        }
        let position = crate::lexer::Position::new(0, 0, 0);
        let mut named = Vec::with_capacity(unknown.len());
        for key in &unknown {
            named.push(self.get_inspect_representation(key, position)?);
        }
        let message = if named.len() == 1 {
            format!("unknown keyword: {}", named[0])
        } else {
            format!("unknown keywords: {}", named.join(", "))
        };
        Err(crate::vm::errors::simple_exception(
            "ArgumentError",
            &message,
            position,
        ))
    }

    /// Bind named keyword parameters to the current scope.
    pub(crate) fn bind_keyword_params(
        &mut self,
        keyword_parameters: &[(String, Option<Expression>)],
        keyword_rest_parameter: Option<&str>,
        kwargs: IndexMap<String, Object>,
    ) -> Result<(), MetorexError> {
        // Ruby names every keyword the call left out, all of them in one
        // message rather than one message for the first.
        let missing: Vec<&String> = keyword_parameters
            .iter()
            .filter(|(name, default_expr)| default_expr.is_none() && !kwargs.contains_key(name))
            .map(|(name, _)| name)
            .collect();
        if !missing.is_empty() {
            let named = missing
                .iter()
                .map(|name| format!(":{}", name))
                .collect::<Vec<_>>()
                .join(", ");
            let message = if missing.len() == 1 {
                format!("missing keyword: {}", named)
            } else {
                format!("missing keywords: {}", named)
            };
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &message,
                crate::lexer::Position::new(0, 0, 0),
            ));
        }
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

        // `**nil` names no parameter at all, so nothing is bound under it.
        let keyword_rest_parameter =
            keyword_rest_parameter.filter(|held| *held != crate::object::NO_KEYWORDS_PARAM);
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

/// Whether an object is an instance of a String, Array, Set, or Hash subclass,
/// which holds the collection it stands for in an instance variable.
fn backs_a_collection(receiver: &crate::object::Object) -> bool {
    use crate::vm::native_methods::{
        array_subclass_value, hash_subclass_value, set_subclass_value, string_subclass_value,
    };
    matches!(receiver, crate::object::Object::Instance(_))
        && (hash_subclass_value(receiver).is_some()
            || array_subclass_value(receiver).is_some()
            || set_subclass_value(receiver).is_some()
            || string_subclass_value(receiver).is_some())
}

/// Whether a method was written by the program rather than by the core
/// library. A library `def self.name` stays behind the native table, where
/// one a program writes stands in front of it.
fn written_by_the_program(method: &std::rc::Rc<crate::object::Method>) -> bool {
    method
        .body
        .first()
        .is_some_and(|statement| !statement.position().prelude)
}

/// Whether `terminal_statement_value` answers for this statement, rather
/// than leaving it to `execute_statement`.
fn answers_its_own_value(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::Expression { .. }
            | Statement::Assignment { .. }
            | Statement::If { .. }
            | Statement::Unless { .. }
            | Statement::Begin { .. }
    )
}
