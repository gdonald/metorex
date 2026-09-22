// The statements a class body is made of.

use super::*;

impl VirtualMachine {
    pub(crate) fn apply_class_body(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Reset visibility to public at the start of every class body so
        // reopening doesn't carry over a stale state from a prior body.
        class.set_current_visibility("public");
        // A class body is not a method activation, so `__callee__` and
        // `__method__` inside one answer nil rather than reporting whichever
        // method happens to be running further down the stack.
        // Ruby names the body by the class alone, without the namespace it
        // was written in, and a singleton class body by what it is.
        self.call_stack_push(crate::vm::CallFrame::boundary(class_body_label(class)));
        let body_result = self.apply_class_body_statements(class, body, position);
        self.call_stack_pop();
        body_result
    }

    /// Run a `class`, `module`, or `class << x` body. Such a body is a scope
    /// of its own: a local of the method or block holding it is not in scope
    /// inside, so a bare name there reads as a method the way Ruby reads it.
    /// A `class_eval` block is not one of these and keeps its own scope.
    pub(crate) fn apply_written_class_body(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let held_self = self.environment().get("self");
        self.environment_mut().push_isolated_scope();
        if let Some(receiver) = held_self {
            self.environment_mut().define("self".to_string(), receiver);
        }
        let answered = self.apply_class_body(class, body, position);
        self.environment_mut().pop_scope();
        answered
    }

    pub(crate) fn apply_class_body_statements(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A class variable written in this body belongs to this class, not to
        // whatever block the body happens to be running inside.
        let held_home = std::mem::take(&mut self.class_var_home);
        let answer = self.class_body_statements(class, body, position);
        self.class_var_home = held_home;
        refuse_return_from_a_body(answer, position)
    }

    pub(crate) fn class_body_statements(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut last_value = Object::Nil;
        for statement in body {
            if self.coverage.is_some() {
                let line = statement.position().line;
                self.coverage_count(line);
                self.coverage_skip_line = Some(line);
            }
            // Bare `private` / `public` / `protected` statements (no args)
            // toggle the default visibility for subsequent method defs.
            if let Statement::Expression {
                expression: Expression::Identifier { name, .. },
                ..
            } = statement
                && matches!(
                    name.as_str(),
                    "private" | "public" | "protected" | MODULE_FUNCTION_VISIBILITY
                )
            {
                class.set_current_visibility(name);
                continue;
            }
            // A bare `freeze` in a class body freezes the class itself.
            if let Statement::Expression {
                expression: Expression::Identifier { name, .. },
                ..
            } = statement
                && name == "freeze"
            {
                class.freeze();
                continue;
            }
            match statement {
                Statement::FunctionDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    singleton_class: None,
                    position: def_position,
                } => {
                    let mut m = build_method_from_params(
                        method_name.clone(),
                        parameters,
                        method_body.clone(),
                        self.snapshot_active_refinements(),
                        self.snapshot_lexical_nesting(),
                    );
                    m.owner = Some(class.name().to_string());
                    m.owner_class = Some(Rc::clone(class));
                    m.source_location = Some(self.source_location_for(*def_position));
                    self.warn_redefined_optimized_method(class.name(), method_name, *def_position)?;
                    class.define_method(method_name, Rc::new(m));
                    apply_current_visibility(class, method_name);
                    let hook = Self::method_added_hook_for(class);
                    self.invoke_class_hook(class, hook, method_name, *def_position)?;
                    if module_function_is_active(class) {
                        self.copy_to_module_function(class, method_name, *def_position)?;
                    }
                    last_value = Object::symbol(method_name.clone());
                }
                // `def self.name` inside a `Class.new do ... end` block: the parser
                // emits a FunctionDef (not MethodDef, because `in_class_body` is
                // false inside a do-block). Route it through the same path as
                // `def self.name` in a regular class body — i.e. store as a
                // class-level method under the `__class__` prefix.
                Statement::FunctionDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    singleton_class: Some(receiver),
                    position: def_position,
                } if receiver == "self" => {
                    let mut m = build_method_from_params(
                        method_name.clone(),
                        parameters,
                        method_body.clone(),
                        self.snapshot_active_refinements(),
                        self.snapshot_lexical_nesting(),
                    );
                    m.source_location = Some(self.source_location_for(*def_position));
                    class.define_method(format!("__class__{}", method_name), Rc::new(m));
                    last_value = Object::symbol(method_name.clone());
                }
                // `def Named.method` inside a body names the object it says
                // rather than the class the body opens, so the ordinary route
                // resolves the receiver.
                Statement::FunctionDef {
                    singleton_class: Some(_),
                    ..
                } => {
                    if let ControlFlow::Value(value) = self.execute_statement(statement)? {
                        last_value = value;
                    }
                }
                Statement::MethodDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    is_class_method,
                    position: def_position,
                    end_position: def_end,
                } => {
                    // Create a Method object
                    let param_names: Vec<String> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
                        .map(|p| p.name.clone())
                        .collect();
                    let keyword_parameters: Vec<(String, Option<crate::ast::Expression>)> =
                        parameters
                            .iter()
                            .filter(|p| p.is_named_keyword)
                            .map(|p| (p.name.clone(), p.default_value.clone()))
                            .collect();
                    let block_parameter = parameters
                        .iter()
                        .find(|p| p.is_block)
                        .map(|p| p.name.clone());
                    let default_params: Vec<(usize, crate::ast::Expression)> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
                        .enumerate()
                        .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
                        .collect();
                    let variadic_param = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
                        .enumerate()
                        .find(|(_, p)| p.is_variadic)
                        .map(|(i, p)| (i, p.name.clone()));
                    let mut m = Method::new(method_name.clone(), param_names, method_body.clone());
                    m.source_location = Some(self.source_location_for(*def_position));
                    m.default_parameters = default_params;
                    m.keyword_parameters = keyword_parameters;
                    m.keyword_rest_parameter = parameters
                        .iter()
                        .find(|p| p.is_keyword)
                        .map(|p| p.name.clone());
                    m.block_parameter = block_parameter;
                    m.variadic_param = variadic_param;
                    m.captured_refinements = self.snapshot_active_refinements();
                    m.captured_nesting = self.snapshot_lexical_nesting();
                    m.owner = Some(class.name().to_string());
                    m.owner_class = Some(Rc::clone(class));
                    let method = Rc::new(m);
                    if *is_class_method {
                        // def self.method_name — store as class method with __class__ prefix
                        class.define_method(format!("__class__{}", method_name), method);
                        self.invoke_class_hook(
                            class,
                            "singleton_method_added",
                            method_name,
                            position,
                        )?;
                    } else {
                        self.warn_redefined_optimized_method(
                            class.name(),
                            method_name,
                            *def_position,
                        )?;
                        class.define_method(method_name, method);
                        apply_current_visibility(class, method_name);
                        // A method defined in a `class << obj` body is a
                        // singleton method of the attached object, so it
                        // fires the singleton hook rather than the plain one.
                        let hook = Self::method_added_hook_for(class);
                        self.invoke_class_hook(class, hook, method_name, position)?;
                        if module_function_is_active(class) {
                            self.copy_to_module_function(class, method_name, position)?;
                        }
                    }
                    if self.coverage.is_some() {
                        self.coverage_note_method(
                            Object::Class(Rc::clone(class)),
                            method_name,
                            *def_position,
                            *def_end,
                        );
                    }
                    last_value = Object::symbol(method_name.clone());
                }
                Statement::Assignment {
                    target: Expression::InstanceVariable { name: var_name, .. },
                    value,
                    ..
                } => {
                    // Declare the ivar and bind the class-level value so
                    // `@body = self` in a class body sets a real class-level
                    // instance variable (read back by `class.get_class_var`
                    // with the `@` prefix), not just a declared-but-unset name.
                    class.declare_instance_var(var_name);
                    let initial_value = self.evaluate_expression(value)?;
                    class.set_class_var(format!("@{}", var_name), initial_value);
                }
                Statement::Assignment {
                    target: Expression::ClassVariable { name: var_name, .. },
                    value,
                    ..
                } => {
                    // Class variable initialization (e.g., @@count = 0 in class body)
                    let initial_value = self.evaluate_expression(value)?;
                    // Inside a singleton class body (`class << self`) a class
                    // variable attaches to the lexical enclosing class, not the
                    // singleton itself. Redirect to the attached class/module.
                    if class.get_class_var("__singleton__").is_some()
                        && let Some(Object::Class(attached) | Object::Module(attached)) =
                            class.get_class_var("__attached__")
                    {
                        attached.set_class_var(var_name, initial_value);
                    } else {
                        class.set_class_var(var_name, initial_value);
                    }
                }
                Statement::Expression {
                    expression: Expression::InstanceVariable { name: var_name, .. },
                    ..
                } => {
                    // Instance variable declaration without assignment
                    class.declare_instance_var(var_name);
                }
                Statement::AttrReader { attributes, .. } => {
                    let names = self.resolve_attribute_names(attributes, position)?;
                    for attr_name in &names {
                        let getter_body = vec![Statement::Return {
                            value: Some(Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            }),
                            position,
                        }];
                        let method = Rc::new(Method::new(attr_name.clone(), vec![], getter_body));
                        class.define_method(attr_name, method);
                        apply_current_visibility(class, attr_name);
                        class.declare_instance_var(attr_name);
                    }
                }
                Statement::AttrWriter { attributes, .. } => {
                    let names = self.resolve_attribute_names(attributes, position)?;
                    for attr_name in &names {
                        let setter_body = vec![Statement::Assignment {
                            target: Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: Expression::Identifier {
                                name: crate::object::UNNAMED_PARAMETER.to_string(),
                                position,
                            },
                            position,
                        }];
                        let setter_name = format!("{}=", attr_name);
                        let method = Rc::new(Method::new(
                            setter_name.clone(),
                            vec![crate::object::UNNAMED_PARAMETER.to_string()],
                            setter_body,
                        ));
                        class.define_method(&setter_name, method);
                        apply_current_visibility(class, &setter_name);
                        class.declare_instance_var(attr_name);
                    }
                }
                Statement::AttrAccessor { attributes, .. } => {
                    let names = self.resolve_attribute_names(attributes, position)?;
                    for attr_name in &names {
                        // Getter
                        let getter_body = vec![Statement::Return {
                            value: Some(Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            }),
                            position,
                        }];
                        let getter_method =
                            Rc::new(Method::new(attr_name.clone(), vec![], getter_body));
                        class.define_method(attr_name, getter_method);
                        apply_current_visibility(class, attr_name);

                        // Setter
                        let setter_body = vec![Statement::Assignment {
                            target: Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: Expression::Identifier {
                                name: crate::object::UNNAMED_PARAMETER.to_string(),
                                position,
                            },
                            position,
                        }];
                        let setter_name = format!("{}=", attr_name);
                        let setter_method = Rc::new(Method::new(
                            setter_name.clone(),
                            vec![crate::object::UNNAMED_PARAMETER.to_string()],
                            setter_body,
                        ));
                        class.define_method(&setter_name, setter_method);
                        apply_current_visibility(class, &setter_name);

                        class.declare_instance_var(attr_name);
                    }
                }
                Statement::Assignment {
                    target:
                        Expression::Identifier {
                            name: const_name, ..
                        },
                    value,
                    ..
                } if const_name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase()) =>
                {
                    // Constant assignment in class body (e.g., PI = 3.14159).
                    // Lowercase identifiers fall through to the `_` arm and are
                    // treated as normal local-variable assignments.
                    let assign_pos = statement.position();
                    let const_value = self.evaluate_constant_assignment(statement, value)?;
                    if class.get_class_var(const_name).is_some() {
                        let owner = class.ruby_name();
                        let owner = if owner.is_empty() {
                            class.inspect_name()
                        } else {
                            owner
                        };
                        let msg = format!(
                            "warning: already initialized constant {}::{}",
                            owner, const_name
                        );
                        self.emit_warning_to_stderr(&msg, assign_pos);
                    }
                    class.set_class_var(const_name, const_value.clone());
                    let assign_file = self
                        .reported_current_file()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default();
                    class.set_const_location(const_name, assign_file, assign_pos.line as i64);
                    self.trigger_const_added_hook(
                        Object::Class(Rc::clone(class)),
                        const_name,
                        assign_pos,
                    )?;
                    last_value = const_value;
                }
                Statement::Include {
                    module_name,
                    position,
                } => {
                    // include ModuleName: dispatch through `append_features`
                    // so user overrides on the module's singleton class fire,
                    // and so cyclic/frozen checks run.
                    let resolved = self.resolve_constant_with_autoload(module_name)?;
                    match resolved {
                        Some(Object::Module(module)) => {
                            self.apply_module_include(class, &module, *position)?;
                        }
                        Some(_) => {
                            return Err(MetorexError::runtime_error(
                                format!("'{}' is not a module", module_name),
                                position_to_location(*position),
                            ));
                        }
                        None => {
                            return Err(MetorexError::runtime_error(
                                format!("Undefined module '{}'", module_name),
                                position_to_location(*position),
                            ));
                        }
                    }
                }
                Statement::Extend {
                    module_name,
                    position,
                } => {
                    // extend ModuleName: add module methods as class-level
                    // methods, and record the mixin on the singleton class so
                    // `kind_of?` reports it the way an `obj.extend` does.
                    // `extend self` names the module being defined, which is
                    // how a module makes its instance methods callable on
                    // itself.
                    let resolved = if module_name == "self" {
                        Some(if class.is_module() {
                            Object::Module(Rc::clone(class))
                        } else {
                            Object::Class(Rc::clone(class))
                        })
                    } else {
                        self.resolve_constant_in_scope(module_name)
                    };
                    match resolved {
                        Some(Object::Module(module)) => {
                            for method_name in module.method_names() {
                                if let Some(method) = module.find_method(&method_name) {
                                    class.set_class_var(
                                        format!("__ext__{}", method_name),
                                        Object::Method(method),
                                    );
                                }
                            }
                            let target = Object::Class(Rc::clone(class));
                            self.apply_module_extend(&target, &module, *position)?;
                        }
                        Some(_) => {
                            return Err(MetorexError::runtime_error(
                                format!("'{}' is not a module", module_name),
                                position_to_location(*position),
                            ));
                        }
                        None => {
                            return Err(MetorexError::runtime_error(
                                format!("Undefined module '{}'", module_name),
                                position_to_location(*position),
                            ));
                        }
                    }
                }
                Statement::Alias {
                    new_name,
                    old_name,
                    position: alias_pos,
                } => {
                    self.install_alias(class, new_name, old_name, *alias_pos)?;
                    let hook = Self::method_added_hook_for(class);
                    self.invoke_class_hook(class, hook, new_name, *alias_pos)?;
                }
                // `class << <target>` inside a class body — open the target's
                // singleton class and apply the inner body there.
                Statement::Expression {
                    expression:
                        Expression::SingletonClass {
                            target,
                            body: inner_body,
                            position: sc_pos,
                            ..
                        },
                    ..
                } => {
                    self.evaluate_singleton_class_expression(target, None, inner_body, *sc_pos)?;
                }
                // class << self block — treat inner statements as class-level
                Statement::Block { statements, .. } => {
                    for inner_stmt in statements {
                        if let Statement::AttrAccessor { attributes, .. } = inner_stmt {
                            let names = self.resolve_attribute_names(attributes, position)?;
                            for attr_name in &names {
                                // Getter
                                let getter_body = vec![Statement::Expression {
                                    expression: Expression::InstanceVariable {
                                        name: format!("@{}", attr_name),
                                        position: crate::lexer::Position::default(),
                                    },
                                    position: crate::lexer::Position::default(),
                                }];
                                class.define_method(
                                    attr_name,
                                    Rc::new(Method::new(
                                        attr_name.to_string(),
                                        vec![],
                                        getter_body,
                                    )),
                                );
                                // Setter
                                let setter_body = vec![Statement::Assignment {
                                    target: Expression::InstanceVariable {
                                        name: format!("@{}", attr_name),
                                        position: crate::lexer::Position::default(),
                                    },
                                    value: Expression::Identifier {
                                        name: crate::object::UNNAMED_PARAMETER.to_string(),
                                        position: crate::lexer::Position::default(),
                                    },
                                    position: crate::lexer::Position::default(),
                                }];
                                class.define_method(
                                    format!("{}=", attr_name),
                                    Rc::new(Method::new(
                                        format!("{}=", attr_name),
                                        vec![crate::object::UNNAMED_PARAMETER.to_string()],
                                        setter_body,
                                    )),
                                );
                            }
                        }
                    }
                }
                _ => {
                    let mut handled = false;
                    // Handle alias_method in class body
                    if let Statement::Expression {
                        expression:
                            Expression::Call {
                                callee,
                                arguments: call_args,
                                ..
                            },
                        ..
                    } = statement
                        && let Expression::Identifier {
                            name: callee_name, ..
                        } = callee.as_ref()
                        && callee_name == "alias_method"
                        && call_args.len() == 2
                    {
                        handled = true;
                        let new_name = match self.evaluate_expression(&call_args[0])? {
                            Object::String(s) => s.as_str().to_string(),
                            Object::Symbol(s) => s.as_str().to_string(),
                            _ => String::new(),
                        };
                        let old_name = match self.evaluate_expression(&call_args[1])? {
                            Object::String(s) => s.as_str().to_string(),
                            Object::Symbol(s) => s.as_str().to_string(),
                            _ => String::new(),
                        };
                        if !new_name.is_empty() && !old_name.is_empty() {
                            // A name the class holds no entry for, such as one
                            // the interpreter answers natively, is aliased
                            // through the full method so the stub that reaches
                            // it is made.
                            if class.alias_method(&new_name, &old_name) {
                                let hook = Self::method_added_hook_for(class);
                                self.invoke_class_hook(class, hook, &new_name, position)?;
                            } else {
                                let given = vec![
                                    Object::symbol(new_name.clone()),
                                    Object::symbol(old_name.clone()),
                                ];
                                self.call_class_methods(class, "alias_method", &given, position)?;
                            }
                        }
                    }
                    // Handle define_method(:name) { |args| body } calls in class body
                    else if let Statement::Expression {
                        expression:
                            Expression::Call {
                                callee,
                                arguments: call_args,
                                trailing_block: Some(block_expr),
                                ..
                            },
                        ..
                    } = statement
                        && let Expression::Identifier {
                            name: callee_name, ..
                        } = callee.as_ref()
                        && callee_name == "define_method"
                    {
                        handled = true;
                        let mut define_args: Vec<Object> = Vec::new();
                        for arg_expr in call_args {
                            define_args.push(self.evaluate_expression(arg_expr)?);
                        }
                        self.pending_block = Some(self.evaluate_expression(block_expr)?);
                        self.pending_block_from_ampersand = false;
                        // The method stands where the call was written, not
                        // where the body it sits in opened.
                        last_value =
                            self.module_define_method(class, &define_args, statement.position())?;
                    }
                    // `refine(target) { body }` inside a module body — dispatch
                    // to the module's refine method, preserving the block.
                    else if let Statement::Expression {
                        expression:
                            Expression::Call {
                                callee,
                                arguments: call_args,
                                trailing_block,
                                ..
                            },
                        ..
                    } = statement
                        && let Expression::Identifier {
                            name: callee_name, ..
                        } = callee.as_ref()
                        && callee_name == "refine"
                    {
                        handled = true;
                        self.evaluate_method_call(
                            &Expression::SelfExpr { position },
                            "refine",
                            call_args,
                            trailing_block.as_deref(),
                            position,
                        )?;
                    }
                    // Fall through to general statement execution for arbitrary
                    // expressions and lowercase-identifier assignments (locals,
                    // method calls, @ivars on the class, etc.). Ruby evaluates
                    // any expression inside a class body with `self` bound to
                    // the class; we've already pushed that self in
                    // `apply_block_as_class_body_with_self` / `execute_class_def`.
                    if !handled {
                        match statement {
                            // A bare expression statement (e.g. `1 + 1`, `self`,
                            // a constant reference) is the common tail of a
                            // `class_eval`/`module_eval` string or block — capture
                            // its value so it becomes the eval result.
                            Statement::Expression {
                                expression,
                                position: expr_pos,
                            } => {
                                let v = self.evaluate_expression(expression)?;
                                if matches!(expression, Expression::Identifier { .. })
                                    && matches!(v, Object::Method(_))
                                {
                                    last_value = self.invoke_callable(v, vec![], *expr_pos)?;
                                } else {
                                    last_value = v;
                                }
                            }
                            _ => match self.execute_statement(statement)? {
                                ControlFlow::Value(v) => {
                                    last_value = v;
                                }
                                // Ruby's parser refuses a `return` written in
                                // a class or module body outright.
                                ControlFlow::Return { position, .. } => {
                                    return Err(return_in_a_body_error(position));
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
        }
        Ok(last_value)
    }
}
