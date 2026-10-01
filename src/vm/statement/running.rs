// Running one statement, and the statements of a scope.

use super::*;

impl VirtualMachine {
    /// Evaluate a statement and produce control-flow information for the caller.
    pub(crate) fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, MetorexError> {
        let flow = self.execute_statement_body(statement)?;
        // A local assigned inside a branch or a loop is a local from there
        // on whether or not the assignment ran, so a block written later
        // closes over it.
        if matches!(
            statement,
            Statement::If { .. }
                | Statement::Unless { .. }
                | Statement::While { .. }
                | Statement::DoWhile { .. }
                | Statement::For { .. }
                | Statement::Match { .. }
                | Statement::CaseIn { .. }
                | Statement::Begin { .. }
        ) {
            for name in crate::ast::collect_assigned_locals(std::slice::from_ref(statement)) {
                self.environment_mut().unhoist(&name);
            }
        }
        Ok(flow)
    }

    fn execute_statement_body(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, MetorexError> {
        // Every statement is a `:line` event for whatever is tracing, which
        // costs a check on an empty list when nothing is.
        if !self.tracepoints.is_empty() {
            self.fire_line_event(statement.position())?;
        }
        self.deliver_pending_signals(statement.position())?;
        let line = statement.position().line;
        if self.coverage_skip_line.take() != Some(line) && self.coverage.is_some() {
            self.coverage_count(line);
        }
        match statement {
            Statement::Expression {
                expression,
                position,
            } => {
                let result = self.evaluate_expression(expression)?;

                // Ruby-style auto-call: if expression statement evaluates to a Method
                // and the expression is a bare identifier, auto-call it with zero args
                if matches!(expression, Expression::Identifier { .. })
                    && matches!(result, Object::Method(_))
                {
                    let called = self.invoke_callable(result, vec![], *position)?;
                    return Ok(ControlFlow::Value(called));
                }

                // The value travels with the flow, so the last expression in
                // an `if` branch is what the `if` answers.
                Ok(ControlFlow::Value(result))
            }
            Statement::Assignment {
                target,
                value,
                position: _,
            } => {
                // Ruby's parser introduces the local where the assignment is
                // written, ahead of the value, so a block on the right-hand
                // side closes over the name being assigned. That is what
                // lets `walk = -> { walk.call }` call itself.
                if let crate::ast::Expression::Identifier { name, .. } = target {
                    self.environment_mut().unhoist(name);
                }
                let evaluated = self.evaluate_assignment(target, value)?;
                // An assignment answers the value it assigned, so a block or
                // an `if` branch ending in one has that as its value.
                Ok(ControlFlow::Value(evaluated))
            }
            Statement::MultipleAssignment {
                targets,
                values,
                position,
            } => {
                // Ruby evaluates the receivers and subscripts the targets
                // name, left to right, before anything on the right.
                let prepared = self.prepare_targets(targets)?;
                // One value on the right is spread across the targets when it
                // is an Array or converts to one with `to_ary`, and otherwise
                // reaches the first target alone. Several values, or a
                // splat, are gathered the way an Array literal gathers them.
                let lone_value = match values.as_slice() {
                    [single] if !matches!(single, Expression::Splat { .. }) => Some(single),
                    _ => None,
                };
                let (answer, source): (Object, Vec<Object>) = match lone_value {
                    Some(single) => {
                        let single = self.evaluate_expression(single)?;
                        let spread = self
                            .block_argument_spread(&single, *position)?
                            .unwrap_or_else(|| vec![single.clone()]);
                        (single, spread)
                    }
                    None => {
                        let gathered = self.evaluate_array_literal(values)?;
                        let each = match &gathered {
                            Object::Array(elements) => elements.borrow().clone(),
                            held => vec![held.clone()],
                        };
                        (gathered, each)
                    }
                };
                self.spread_into_prepared(&prepared, &source)?;
                // The assignment answers the right-hand side as it was
                // written, which is what `(a, b = 1, 2)` reads back as.
                Ok(ControlFlow::Value(answer))
            }
            Statement::Return { value, position } => {
                // A `return` written at the top level ends the file, and the
                // value written after it goes nowhere, which Ruby says so.
                if value.is_some() && self.call_stack.is_empty() {
                    let written_in = self
                        .current_source_file
                        .clone()
                        .or_else(|| {
                            self.reported_current_file()
                                .map(|path| path.display().to_string())
                        })
                        .unwrap_or_else(|| "-".to_string());
                    let notice = format!(
                        "{}: warning: argument of top-level return is ignored",
                        written_in
                    );
                    self.emit_warning_to_stderr(&notice, *position);
                }
                let result = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Object::Nil,
                };
                Ok(ControlFlow::Return {
                    value: result,
                    position: *position,
                })
            }
            Statement::Break { value, position } => {
                let resolved = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Object::Nil,
                };
                Ok(ControlFlow::Break {
                    value: resolved,
                    position: *position,
                })
            }
            Statement::Continue { value, position } => {
                let resolved = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Object::Nil,
                };
                Ok(ControlFlow::Continue {
                    value: resolved,
                    position: *position,
                })
            }
            // `retry` inside a rescue body runs the begin body again, which
            // the begin handler does when it sees this.
            Statement::Retry { position } => Ok(ControlFlow::Retry {
                position: *position,
            }),
            Statement::Redo { position } => Ok(ControlFlow::Redo {
                position: *position,
            }),
            Statement::Block {
                statements,
                position: _,
            } => self.execute_block(statements),
            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position: _,
            } => self.execute_if(condition, then_branch, elsif_branches, else_branch),
            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                position: _,
            } => self.execute_unless(condition, then_branch, else_branch),
            Statement::While {
                condition,
                body,
                position: _,
            } => self.inside_loop(|vm| vm.execute_while(condition, body)),
            Statement::DoWhile {
                condition,
                body,
                position: _,
            } => self.inside_loop(|vm| vm.execute_do_while(condition, body)),
            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => self.inside_loop(|vm| vm.execute_for(variable, iterable, body, *position)),
            // A `BEGIN` body already ran before the rest of its code unit,
            // so reaching it where it was written does nothing.
            Statement::BeginBlock { .. } => Ok(ControlFlow::Next),
            Statement::DeclareLocals { names, .. } => {
                for name in names {
                    if self.environment().get(name).is_none() {
                        self.environment_mut().define(name.clone(), Object::Nil);
                    } else {
                        // A `for` loop names what its body binds so the scope
                        // holding the loop still reads it afterwards, which
                        // means the block the loop runs closes over the name.
                        self.environment_mut().unhoist(name);
                    }
                }
                Ok(ControlFlow::Next)
            }
            Statement::ClassDef {
                name,
                namespace,
                superclass,
                superclass_expression,
                body,
                position,
            } => self.execute_class_def(
                name,
                namespace.as_deref(),
                superclass.as_deref(),
                superclass_expression.as_deref(),
                body,
                *position,
            ),
            // A `def` nested inside a block or a `begin` within a class body
            // reaches here rather than the class-body walk, so it installs on
            // the innermost lexical class the same way that walk would.
            Statement::MethodDef {
                name,
                parameters,
                body,
                is_class_method,
                position,
                ..
            } => {
                // Run from a method, such as in a default value, it installs
                // on the class the method was written in.
                let from_method_body = self.running_method_def_scope();
                let Some(target) = from_method_body
                    .clone()
                    .or_else(|| self.def_scope_stack.last().cloned())
                    .or_else(|| {
                        self.method_nesting_stack
                            .last()
                            .and_then(|nesting| nesting.first().cloned())
                    })
                else {
                    return Err(unimplemented_statement_error(statement));
                };
                let _ = (parameters, body);
                self.apply_class_body_statements(
                    &target,
                    std::slice::from_ref(statement),
                    *position,
                )?;
                // Run from a method body, it is public whatever visibility
                // the class body left in force.
                if from_method_body.is_some() && !is_class_method {
                    target.clear_method_visibility(name);
                }
                Ok(ControlFlow::Value(Object::symbol(name.clone())))
            }
            Statement::Begin {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                position,
            } => self.execute_begin(body, rescue_clauses, else_clause, ensure_block, *position),
            Statement::Raise {
                exception,
                position,
            } => self.execute_raise(exception, *position),
            Statement::Match {
                expression,
                cases,
                position,
            } => self.execute_match(expression, cases, *position),
            Statement::CaseIn {
                expression,
                cases,
                position,
            } => self.execute_case_in(expression, cases, *position),
            Statement::FunctionDef {
                name,
                parameters,
                body,
                position,
                singleton_class,
            } => self.execute_function_def(
                name,
                parameters,
                body,
                *position,
                singleton_class.as_deref(),
            ),
            Statement::AttrReader { position, .. }
            | Statement::AttrWriter { position, .. }
            | Statement::AttrAccessor { position, .. } => {
                // These are only processed during class definition, not as standalone statements
                Err(MetorexError::runtime_error(
                    "attr_reader, attr_writer, and attr_accessor can only be used inside a class definition",
                    position_to_location(*position),
                ))
            }
            Statement::ModuleDef {
                name,
                namespace,
                body,
                position,
            } => self.execute_module_def(name, namespace.as_deref(), body, *position),
            Statement::Include {
                module_name,
                position,
            } => self.execute_include(module_name, *position),
            Statement::Extend {
                module_name,
                position,
            } => self.execute_extend(module_name, *position),
            Statement::Alias {
                new_name,
                old_name,
                position,
            } => self.execute_alias(new_name, old_name, *position),
        }
    }

    /// Run a loop, during which a `break` ends the loop rather than
    /// returning from the call a surrounding block was attached to.
    fn inside_loop(
        &mut self,
        run: impl FnOnce(&mut Self) -> Result<ControlFlow, MetorexError>,
    ) -> Result<ControlFlow, MetorexError> {
        self.running_block_breaks.push(None);
        let result = run(self);
        self.running_block_breaks.pop();
        result
    }

    /// Execute statements within a new lexical scope.
    /// Ruby reports a constant given a second value, naming the class or
    /// module the constant is bound on.
    pub(crate) fn warn_already_initialized(
        &mut self,
        owner: &Rc<crate::class::Class>,
        name: &str,
        position: crate::lexer::Position,
    ) {
        // `$VERBOSE = nil` silences warnings of every kind.
        if matches!(self.globals().get("VERBOSE"), Some(Object::Nil)) {
            return;
        }
        let named = owner.ruby_name();
        let named = if named.is_empty() {
            owner.inspect_name()
        } else {
            named
        };
        // A constant at the top level is bound on Object, which Ruby leaves
        // out of the name it reports.
        let message = if named == "Object" {
            format!("warning: already initialized constant {}", name)
        } else {
            format!("warning: already initialized constant {}::{}", named, name)
        };
        self.emit_warning_to_stderr(&message, position);
    }

    pub(crate) fn execute_block(
        &mut self,
        statements: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        self.environment_mut().push_scope();
        let result = self.execute_statements_internal(statements);
        self.environment_mut().pop_scope();
        result
    }

    /// Core statement execution loop used by program and block execution.
    pub(crate) fn execute_statements_internal(
        &mut self,
        statements: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        let mut last = ControlFlow::Next;
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Next => last = ControlFlow::Next,
                // A statement that produced a value does not end the run; the
                // last one to produce one is what the group answers.
                ControlFlow::Value(value) => last = ControlFlow::Value(value),
                flow => return Ok(flow),
            }
        }
        Ok(last)
    }
}
