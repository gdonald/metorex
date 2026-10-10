// Running a block against a receiver of its own.

use super::*;

impl VirtualMachine {
    /// Run Ruby source with `self` bound to `receiver`, which is what the
    /// String form of `instance_eval` does. The source sees the receiver's
    /// instance variables and defines methods on its singleton class.
    /// Run Ruby source against `receiver`, counted as written in `named` from
    /// `lineno` on. The code sees the locals of the scope it was written in,
    /// so an assignment there reaches the caller's own name.
    pub(crate) fn evaluate_source_named_with_receiver(
        &mut self,
        source: &str,
        receiver: Object,
        named: Option<String>,
        lineno: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let tokens =
            crate::lexer::Lexer::with_start_line(source, lineno.max(1) as usize).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .inside_eval()
            .with_outer_locals(self.visible_local_names())
            .parse()
            .map_err(|errors| {
                let reported = errors
                    .iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                let file = named.clone().unwrap_or_default();
                crate::vm::errors::syntax_error(
                    format!("{file}: {reported}"),
                    Some(&file),
                    position,
                )
            })?;
        let singleton = self.singleton_class_of(&receiver);
        // The code runs in a scope of its own that sees the caller's locals,
        // so a name it assigns is the caller's own name.
        let carried: Vec<(String, std::rc::Rc<std::cell::RefCell<Object>>)> = self
            .environment()
            .binding_variable_names()
            .into_iter()
            .filter_map(|name| self.environment().get_ref(&name).map(|cell| (name, cell)))
            .collect();
        let previous_file = self.current_file.clone();
        let previous_source_file = self.current_source_file.clone();
        // Code with no file named for it is counted as written where the
        // call was made, which is what `__FILE__` answers inside it.
        let named = named.or_else(|| {
            let written_in = previous_source_file
                .clone()
                .map(std::path::PathBuf::from)
                .or_else(|| previous_file.clone())
                .map(|held| self.reported_spelling(&held).display().to_string())
                .unwrap_or_default();
            Some(format!(
                "{}{}:{})",
                crate::vm::EVAL_FILE_PREFIX,
                written_in,
                position.line
            ))
        });
        // The code is a place of its own in a backtrace, named as the code
        // that ran it and standing where the call was made, as Kernel#eval's
        // code is.
        let caller_file = self.file_for_frames();
        let written_in = self
            .call_stack()
            .last()
            .cloned()
            .unwrap_or_else(|| CallFrame::boundary("<main>"));
        self.call_stack_push(
            written_in
                .with_location(Some(format!("{}:{}", position.line, position.column)))
                .with_source_file(caller_file),
        );
        if let Some(file) = &named {
            self.current_file = Some(std::path::PathBuf::from(file));
            self.current_source_file = Some(file.clone());
        }
        self.environment_mut().push_isolated_scope();
        for (name, cell) in carried {
            self.environment_mut().define_inherited(name, cell);
        }
        self.environment_mut()
            .define("self".to_string(), receiver.clone());
        // A line counted from below one is still counted from there, which a
        // backtrace reports rather than the line the lexer could start at.
        let previous_shift = std::mem::replace(&mut self.source_line_shift, lineno.min(1) - 1);
        // The code reads constants from the receiver's singleton class first
        // and from the receiver's own class next, ahead of the scopes the
        // caller was written in.
        let receiver_class = match &receiver {
            Object::Class(held) | Object::Module(held) => Some(std::rc::Rc::clone(held)),
            Object::Instance(held) => Some(std::rc::Rc::clone(&held.borrow().class)),
            _ => None,
        };
        let mut opened = 1;
        let mut nesting = Vec::new();
        nesting.push(std::rc::Rc::clone(&singleton));
        if let Some(held) = receiver_class {
            nesting.push(std::rc::Rc::clone(&held));
            self.def_scope_stack.push(held);
            opened += 1;
        }
        self.def_scope_stack.push(singleton);
        // The scopes open around the code are the receiver's, ahead of the
        // ones the caller was written in, which is the order a name written
        // there is looked up through.
        if let Some(held) = self.method_nesting_stack.last() {
            nesting.extend(held.iter().map(std::rc::Rc::clone));
        }
        self.method_nesting_stack.push(nesting);
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
                match self.execute_statement(statement)? {
                    ControlFlow::Value(value) => last = value,
                    // An exception raised here is the caller's to handle,
                    // rather than something the run swallows.
                    ControlFlow::Exception {
                        exception,
                        position,
                    } => {
                        return Err(MetorexError::UncaughtException {
                            message: crate::vm::utils::format_exception(&exception),
                            exception,
                            location: position_to_location(position),
                        });
                    }
                    _ => {}
                }
            }
            Ok(())
        })();
        // An error the code raised natively is traced while its file is
        // still the one the code was named for.
        if let Err(error) = &result {
            self.trace_error_leaving_frame(error);
        }
        self.call_stack_pop();
        for _ in 0..opened {
            self.def_scope_stack.pop();
        }
        self.method_nesting_stack.pop();
        self.environment_mut().pop_scope();
        self.source_line_shift = previous_shift;
        self.current_file = previous_file;
        self.current_source_file = previous_source_file;
        result?;
        Ok(last)
    }

    /// The frame a block run against another receiver stands in, named as
    /// the block and sitting where the call that ran it was made.
    pub(crate) fn block_frame_at(&self, block: &BlockStatement, position: Position) -> CallFrame {
        let frame_name = block.name().to_string();
        let frame_location_string = Some(format!("{}", position_to_location(position)));
        match block.defining_method.clone() {
            Some((callee, defined)) => {
                CallFrame::method(frame_name.clone(), frame_location_string, callee, defined)
            }
            // A block run against another receiver still sits where the call
            // was made, which is what a backtrace entry for it names.
            None => {
                CallFrame::boundary(frame_name.clone()).with_location(frame_location_string.clone())
            }
        }
        .owned_by(block.defining_owner.clone())
        .nested_in_a_block(
            block
                .written_depth
                .unwrap_or_else(|| self.block_nesting_depth()),
        )
        .written_in_scope(block.written_in.clone())
        .with_source_file(self.file_for_frames())
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
        let frame = self.block_frame_at(block, position);
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);
        // A literal in the body is written in the encoding the block's own
        // file names, not the one the file calling it names.
        let saved_source_encoding = std::mem::replace(
            &mut self.current_source_encoding,
            self.current_source_file
                .as_ref()
                .and_then(|named| self.file_encodings.get(named).cloned()),
        );
        // A `def` in the body defines a method on the receiver alone, and a
        // class variable written there belongs to the class or module the
        // block was written in.
        // The class or module the block was written in. A block written in a
        // method body belongs to the class holding that method, whatever
        // scope was open where the method was called from.
        let lexical = block
            .captured_nesting
            .first()
            .cloned()
            .or_else(|| self.method_owner_stack.last().cloned().flatten())
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
        // A class variable written in the body belongs to the class or
        // module the block was written in, whatever it runs against.
        self.class_var_cref_stack.push(
            block
                .captured_nesting
                .first()
                .cloned()
                .or_else(|| block.captured_def_scope.last().cloned()),
        );
        // The scopes open around the body are the ones the block was written
        // in, whatever the receiver it runs against, which is what
        // `Module.nesting` there reports.
        self.method_nesting_stack
            .push(block.captured_nesting.clone());
        // A `def` or an `alias` written at the top of the body belongs to the
        // receiver alone. Anything the body calls out to keeps its own
        // definee, so this reaches only the statements written here.
        let definee = block
            .body()
            .iter()
            .any(|statement| {
                matches!(
                    statement,
                    Statement::MethodDef { .. }
                        | Statement::FunctionDef { .. }
                        | Statement::Alias { .. }
                )
            })
            .then(|| self.singleton_class_of(&receiver));
        // A value the program cannot hold one copy of has no singleton class
        // to write the method on.
        if definee.is_some()
            && crate::vm::native_methods::object_methods::refuses_a_singleton(&receiver)
        {
            let message = "can't define singleton".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        self.enter_running_code(
            std::rc::Rc::clone(&block.written_within),
            block.opened_at.unwrap_or(position.line),
        );
        // A `def` in the block, or in a block it opens, installs on the
        // receiver's singleton class rather than where the method running
        // the block was defined.
        let running_frame = self.current_method_frame;
        let replaced_definee = match running_frame {
            Some(running)
                if !crate::vm::native_methods::object_methods::refuses_a_singleton(&receiver) =>
            {
                Some(self.method_definees.insert(
                    running,
                    (
                        self.def_scope_stack.len(),
                        crate::vm::core::Definee::SingletonOf(receiver.clone()),
                    ),
                ))
            }
            _ => None,
        };
        let execution_result = self.with_call_frame(frame, move |vm| {
            vm.environment_mut().push_isolated_scope();
            let result = (|| -> Result<Object, MetorexError> {
                vm.environment_mut()
                    .attach_captured(std::rc::Rc::clone(&block.captured_vars));
                vm.environment_mut()
                    .attach_reserved(std::rc::Rc::clone(&block.reserved_names));
                // Override `self` with the instance_exec receiver
                vm.environment_mut().define("self".to_string(), receiver);

                bind_block_params(
                    vm,
                    block.parameters(),
                    &block.parameter_defaults,
                    arguments,
                    Position::new(0, 0, 0),
                )?;

                // Pre-bind syntactically assigned locals to nil (Ruby's
                // parser-level local hoisting) so an `ensure`/`rescue`
                // clause that reads a variable defined later in the body
                // returns nil instead of NameError when execution
                // short-circuits via raise.
                for name in collect_assigned_locals(block.body()) {
                    if vm.environment().assignment_introduces_a_local(&name) {
                        vm.environment_mut().hoist(name);
                    }
                }

                let mut last_value;
                // `redo` runs the block's body again over the same arguments.
                'again: loop {
                    last_value = Object::Nil;
                    for statement in block.body() {
                        // `def self.name` written here names the receiver the
                        // block runs against, and that receiver's singleton
                        // class is the definee, so the method is written there
                        // under its own name rather than as a class method of
                        // the singleton.
                        let written_plainly = statement_for_its_own_receiver(statement);
                        if let Some(definee) = &definee
                            && let Some(plain) = &written_plainly
                        {
                            let definee = std::rc::Rc::clone(definee);
                            last_value = vm.apply_class_body_statements(
                                &definee,
                                std::slice::from_ref(plain),
                                statement.position(),
                            )?;
                            continue;
                        }
                        if let Some(definee) = &definee
                            && written_plainly.is_none()
                            && matches!(
                                statement,
                                Statement::MethodDef { .. }
                                    | Statement::FunctionDef { .. }
                                    | Statement::Alias { .. }
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
                                // The break belongs to the call that was
                                // handed this block, which is one made from
                                // the frame the block was written in.
                                return Err(MetorexError::BlockBreak {
                                    value,
                                    location: position_to_location(position),
                                    home_frame: block.home_frame,
                                });
                            }
                            ControlFlow::Redo { .. } | ControlFlow::Retry { .. } => continue 'again,
                            ControlFlow::Continue { position, .. } => {
                                return Err(loop_control_error("next", position));
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
        self.leave_running_code();
        self.method_nesting_stack.pop();
        self.class_var_cref_stack.pop();
        if carried {
            self.class_var_home.pop();
        }
        self.current_source_file = saved_source_file;
        self.current_source_encoding = saved_source_encoding;
        if let (Some(running), Some(previous)) = (running_frame, replaced_definee) {
            match previous {
                Some(entry) => {
                    self.method_definees.insert(running, entry);
                }
                None => {
                    self.method_definees.remove(&running);
                }
            }
        }

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(
                block.name().to_string(),
                position_to_location(position),
            ))),
        }
    }
}
