// The statements a module body is made of.

use super::*;

impl VirtualMachine {
    pub(crate) fn module_body_statements(
        &mut self,
        module: &Rc<Class>,
        body: &[Statement],
    ) -> Result<Object, MetorexError> {
        module.set_current_visibility("public");
        let mut last_value = Object::Nil;
        for statement in body {
            if self.coverage.is_some() {
                let line = statement.position().line;
                self.coverage_count(line);
                self.coverage_skip_line = Some(line);
            }
            // Bare `private` / `public` / `protected` toggle the default
            // visibility for the method definitions that follow, the same way
            // they do in a class body.
            if let Statement::Expression {
                expression: Expression::Identifier { name, .. },
                ..
            } = statement
                && matches!(
                    name.as_str(),
                    "private" | "public" | "protected" | MODULE_FUNCTION_VISIBILITY
                )
            {
                module.set_current_visibility(name);
                continue;
            }
            match statement {
                Statement::MethodDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    is_class_method,
                    position: def_position,
                    ..
                } => {
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
                    m.owner = Some(module.name().to_string());
                    m.owner_class = Some(Rc::clone(module));
                    m.definee = Some(Rc::clone(module));
                    let method = Rc::new(m);
                    if *is_class_method {
                        module.define_method(format!("__class__{}", method_name), method);
                        self.invoke_class_hook(
                            module,
                            "singleton_method_added",
                            method_name,
                            statement.position(),
                        )?;
                    } else {
                        self.refuse_frozen_definee(module, statement.position())?;
                        module.define_method(method_name, method);
                        apply_current_visibility(module, method_name);
                        let hook = Self::method_added_hook_for(module);
                        self.invoke_class_hook(module, hook, method_name, statement.position())?;
                        if module_function_is_active(module) {
                            self.copy_to_module_function(
                                module,
                                method_name,
                                statement.position(),
                            )?;
                        }
                    }
                }
                Statement::Assignment {
                    target:
                        Expression::Identifier {
                            name: const_name, ..
                        },
                    value,
                    ..
                } if const_name.starts_with(|c: char| c.is_uppercase()) => {
                    let assign_pos = statement.position();
                    let const_value = self.evaluate_constant_assignment(statement, value)?;
                    if module.get_class_var(const_name).is_some() {
                        let owner = module.ruby_name();
                        let owner = if owner.is_empty() {
                            module.inspect_name()
                        } else {
                            owner
                        };
                        let msg = format!(
                            "warning: already initialized constant {}::{}",
                            owner, const_name
                        );
                        self.emit_warning_to_stderr(&msg, assign_pos);
                    }
                    crate::vm::statement::name_constant_value(module, const_name, &const_value);
                    module.set_class_var(const_name, const_value);
                    let assign_file = self
                        .reported_current_file()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default();
                    module.set_const_location(const_name, assign_file, assign_pos.line as i64);
                    self.trigger_const_added_hook(
                        Object::Module(Rc::clone(module)),
                        const_name,
                        assign_pos,
                    )?;
                }
                // `class << <target>` inside a module body — open the target's
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
                // class << self block — process inner statements at module level
                Statement::Block {
                    statements: inner, ..
                } => {
                    for inner_stmt in inner {
                        match inner_stmt {
                            Statement::AttrReader { attributes, .. }
                            | Statement::AttrWriter { attributes, .. }
                            | Statement::AttrAccessor { attributes, .. } => {
                                let names =
                                    self.resolve_attribute_names(attributes, Position::default())?;
                                for attr in &names {
                                    let getter = Method::new(
                                        attr.clone(),
                                        vec![],
                                        vec![Statement::Expression {
                                            expression: Expression::InstanceVariable {
                                                name: attr.clone(),
                                                position: crate::lexer::Position::default(),
                                            },
                                            position: crate::lexer::Position::default(),
                                        }],
                                    );
                                    module.define_method(attr, Rc::new(getter));
                                }
                            }
                            Statement::MethodDef {
                                name: method_name,
                                parameters,
                                body: method_body,
                                ..
                            } => {
                                let param_names: Vec<String> = parameters
                                    .iter()
                                    .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
                                    .map(|p| p.name.clone())
                                    .collect();
                                let m = Method::new(
                                    method_name.clone(),
                                    param_names,
                                    method_body.clone(),
                                );
                                module.define_method(method_name, Rc::new(m));
                            }
                            _ => {
                                self.execute_statement(inner_stmt)?;
                            }
                        }
                    }
                }
                // ClassDef inside module — execute_class_def attaches the nested
                // class to the enclosing scope directly via parent_scope detection.
                Statement::ClassDef {
                    name: class_name,
                    namespace,
                    superclass,
                    superclass_expression,
                    body: class_body,
                    position: class_pos,
                } => {
                    self.execute_class_def(
                        class_name,
                        namespace.as_deref(),
                        superclass.as_deref(),
                        superclass_expression.as_deref(),
                        class_body,
                        *class_pos,
                    )?;
                }
                // Nested module — execute_module_def attaches it to the enclosing scope.
                Statement::ModuleDef {
                    name: mod_name,
                    namespace: mod_ns,
                    body: mod_body,
                    position: mod_pos,
                } => {
                    self.execute_module_def(mod_name, mod_ns.as_deref(), mod_body, *mod_pos)?;
                }
                // Include inside module body: walk the lexical def-scope
                // stack so nested modules can reference sibling constants
                // (e.g. `module C; include P; end` where P is defined in the
                // enclosing module).
                Statement::Include {
                    module_name: inc_name,
                    position: inc_pos,
                } => {
                    let resolved = self.resolve_constant_with_autoload(inc_name)?;
                    if let Some(Object::Module(inc_module)) = resolved {
                        self.apply_module_include(module, &inc_module, *inc_pos)?;
                    }
                }
                Statement::Extend {
                    module_name: ext_name,
                    position: ext_pos,
                } => {
                    // `extend self` is the idiom that makes a module's
                    // instance methods callable on the module itself.
                    let resolved = if ext_name == "self" {
                        Some(Object::Module(Rc::clone(module)))
                    } else {
                        self.resolve_constant_with_autoload(ext_name)?
                    };
                    if let Some(Object::Module(ext_module)) = resolved {
                        for method_name in ext_module.method_names() {
                            if let Some(method) = ext_module.find_method(&method_name) {
                                module.set_class_var(
                                    format!("__ext__{}", method_name),
                                    Object::Method(method),
                                );
                            }
                        }
                        let target = Object::Module(Rc::clone(module));
                        self.apply_module_extend(&target, &ext_module, *ext_pos)?;
                    }
                }
                // `alias $new $old` names a global, wherever it is written.
                Statement::Alias {
                    new_name,
                    old_name,
                    position: alias_pos,
                } if !new_name.starts_with('$') => {
                    self.install_alias(module, new_name, old_name, *alias_pos)?;
                    self.invoke_class_hook(module, "method_added", new_name, *alias_pos)?;
                }
                // Other statements in module body
                _ => match self.execute_statement(statement)? {
                    ControlFlow::Value(value) => {
                        last_value = value;
                    }
                    ControlFlow::Return { position, .. } => {
                        return Err(return_in_a_body_error(position));
                    }
                    _ => {}
                },
            }
        }
        Ok(last_value)
    }
}
