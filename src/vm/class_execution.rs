// Class and function definition execution for the Metorex VM.
// This module handles class and function definition statements.

use super::ControlFlow;
use super::core::VirtualMachine;
use super::utils::*;

use crate::ast::{Expression, Statement};
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use std::rc::Rc;

impl VirtualMachine {
    /// Execute class definition - create a Class object and register it in the environment.
    pub(crate) fn execute_class_def(
        &mut self,
        name: &str,
        superclass_name: Option<&str>,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // If we're lexically nested inside a module/class, a bare `class Bar`
        // defines a constant on the enclosing scope (Foo::Bar), not a global
        // reopening.
        let parent_scope = self.def_scope_stack.last().cloned();

        // Resolve superclass if specified. Check enclosing scope's constants first
        // (so `class Bar < Foo` inside `module M` can see `M::Foo`), then the
        // environment, then globals.
        let superclass = if let Some(super_name) = superclass_name {
            let resolved = parent_scope
                .as_ref()
                .and_then(|p| p.get_class_var(super_name))
                .or_else(|| self.environment().get(super_name))
                .or_else(|| self.globals().get(super_name));
            match resolved {
                Some(Object::Class(class)) => Some(class),
                Some(_) => {
                    return Err(MetorexError::runtime_error(
                        format!("Superclass '{}' must be a class", super_name),
                        position_to_location(position),
                    ));
                }
                None => {
                    return Err(MetorexError::runtime_error(
                        format!("Undefined superclass '{}'", super_name),
                        position_to_location(position),
                    ));
                }
            }
        } else {
            None
        };

        // Reopen existing class if it exists (Ruby semantics), otherwise create new
        let class = if superclass.is_none() {
            if let Some(parent) = parent_scope.as_ref() {
                match parent.get_class_var(name) {
                    Some(Object::Class(existing)) => existing,
                    _ => Rc::new(Class::new(name, superclass)),
                }
            } else if let Some(Object::Class(existing)) = self.globals().get(name) {
                existing
            } else if let Some(Object::Class(existing)) = self.environment().get(name) {
                existing
            } else {
                Rc::new(Class::new(name, superclass))
            }
        } else {
            Rc::new(Class::new(name, superclass))
        };

        self.def_scope_stack.push(Rc::clone(&class));
        let body_result = self.apply_class_body(&class, body, position);
        self.def_scope_stack.pop();
        body_result?;

        let class_obj = Object::Class(class);
        if let Some(parent) = parent_scope {
            parent.set_class_var(name, class_obj);
        } else {
            self.environment_mut()
                .define(name.to_string(), class_obj.clone());
            self.globals_mut().set(name.to_string(), class_obj);
        }

        Ok(ControlFlow::Next)
    }

    /// Evaluate a block with `self` bound to the given class/module, executing
    /// the block's body as if it were a class/module definition body.
    pub(crate) fn apply_block_as_class_body(
        &mut self,
        class: &Rc<Class>,
        block: &crate::object::BlockStatement,
        position: Position,
    ) -> Result<(), MetorexError> {
        self.apply_block_as_class_body_with_self(
            class,
            block,
            position,
            Object::Class(Rc::clone(class)),
        )
    }

    /// Same as `apply_block_as_class_body` but explicit about the `self` kind
    /// (use `Object::Module(...)` when the receiver should behave as a module).
    pub(crate) fn apply_block_as_class_body_with_self(
        &mut self,
        class: &Rc<Class>,
        block: &crate::object::BlockStatement,
        position: Position,
        self_obj: Object,
    ) -> Result<(), MetorexError> {
        let prev_self = self.environment().get("self");
        self.environment_mut().define("self".to_string(), self_obj);
        for (name, cell) in block.captured_vars.iter() {
            if self.environment().get(name).is_none() {
                self.environment_mut()
                    .define(name.clone(), cell.borrow().clone());
            }
        }
        self.def_scope_stack.push(Rc::clone(class));
        let result = self.apply_class_body(class, &block.body, position);
        self.def_scope_stack.pop();
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }
        result
    }

    /// Apply a class-body statement list to the given class (also used for
    /// `Class.new { ... }` / anonymous classes).
    pub(crate) fn apply_class_body(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<(), MetorexError> {
        for statement in body {
            match statement {
                Statement::FunctionDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    singleton_class: None,
                    ..
                } => {
                    let param_names: Vec<String> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .map(|p| p.name.clone())
                        .collect();
                    let mut m = Method::new(method_name.clone(), param_names, method_body.clone());
                    m.captured_refinements = self.snapshot_active_refinements();
                    class.define_method(method_name, Rc::new(m));
                }
                Statement::MethodDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    is_class_method,
                    ..
                } => {
                    // Create a Method object
                    let param_names: Vec<String> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
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
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .enumerate()
                        .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
                        .collect();
                    let variadic_param = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .enumerate()
                        .find(|(_, p)| p.is_variadic)
                        .map(|(i, p)| (i, p.name.clone()));
                    let mut m = Method::new(method_name.clone(), param_names, method_body.clone());
                    m.default_parameters = default_params;
                    m.keyword_parameters = keyword_parameters;
                    m.block_parameter = block_parameter;
                    m.variadic_param = variadic_param;
                    m.captured_refinements = self.snapshot_active_refinements();
                    let method = Rc::new(m);
                    if *is_class_method {
                        // def self.method_name — store as class method with __class__ prefix
                        class.define_method(format!("__class__{}", method_name), method);
                    } else {
                        class.define_method(method_name, method);
                    }
                }
                Statement::Assignment {
                    target: Expression::InstanceVariable { name: var_name, .. },
                    ..
                } => {
                    // Declaring an instance variable (e.g., @x = nil in class body)
                    class.declare_instance_var(var_name);
                }
                Statement::Assignment {
                    target: Expression::ClassVariable { name: var_name, .. },
                    value,
                    ..
                } => {
                    // Class variable initialization (e.g., @@count = 0 in class body)
                    let initial_value = self.evaluate_expression(value)?;
                    class.set_class_var(var_name, initial_value);
                }
                Statement::Expression {
                    expression: Expression::InstanceVariable { name: var_name, .. },
                    ..
                } => {
                    // Instance variable declaration without assignment
                    class.declare_instance_var(var_name);
                }
                Statement::AttrReader { attributes, .. } => {
                    // Generate getter methods for each attribute
                    for attr_name in attributes {
                        let getter_body = vec![Statement::Return {
                            value: Some(Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            }),
                            position,
                        }];
                        let method = Rc::new(Method::new(attr_name.clone(), vec![], getter_body));
                        class.define_method(attr_name, method);
                        class.declare_instance_var(attr_name);
                    }
                }
                Statement::AttrWriter { attributes, .. } => {
                    // Generate setter methods for each attribute
                    for attr_name in attributes {
                        let setter_body = vec![Statement::Assignment {
                            target: Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: Expression::Identifier {
                                name: "value".to_string(),
                                position,
                            },
                            position,
                        }];
                        let method = Rc::new(Method::new(
                            format!("{}=", attr_name),
                            vec!["value".to_string()],
                            setter_body,
                        ));
                        class.define_method(format!("{}=", attr_name), method);
                        class.declare_instance_var(attr_name);
                    }
                }
                Statement::AttrAccessor { attributes, .. } => {
                    // Generate both getter and setter methods for each attribute
                    for attr_name in attributes {
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

                        // Setter
                        let setter_body = vec![Statement::Assignment {
                            target: Expression::InstanceVariable {
                                name: attr_name.clone(),
                                position,
                            },
                            value: Expression::Identifier {
                                name: "value".to_string(),
                                position,
                            },
                            position,
                        }];
                        let setter_method = Rc::new(Method::new(
                            format!("{}=", attr_name),
                            vec!["value".to_string()],
                            setter_body,
                        ));
                        class.define_method(format!("{}=", attr_name), setter_method);

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
                } => {
                    // Constant assignment in class body (e.g., PI = 3.14159)
                    let const_value = self.evaluate_expression(value)?;
                    class.set_class_var(const_name, const_value);
                }
                Statement::Include {
                    module_name,
                    position,
                } => {
                    // include ModuleName: add module to mixin chain so methods added
                    // to the module later are still visible (Ruby ancestor-chain semantics).
                    match self.environment().get(module_name) {
                        Some(Object::Module(module)) => {
                            class.add_mixin(Rc::clone(&module));
                        }
                        Some(Object::Class(_)) | Some(_) => {
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
                    // extend ModuleName: add module methods as class-level methods
                    match self.environment().get(module_name) {
                        Some(Object::Module(module)) => {
                            for method_name in module.method_names() {
                                if let Some(method) = module.find_method(&method_name) {
                                    class.set_class_var(
                                        format!("__ext__{}", method_name),
                                        Object::Method(method),
                                    );
                                }
                            }
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
                // class << self block — treat inner statements as class-level
                Statement::Block { statements, .. } => {
                    for inner_stmt in statements {
                        if let Statement::AttrAccessor { attributes, .. } = inner_stmt {
                            for attr_name in attributes {
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
                                        name: "value".to_string(),
                                        position: crate::lexer::Position::default(),
                                    },
                                    position: crate::lexer::Position::default(),
                                }];
                                class.define_method(
                                    format!("{}=", attr_name),
                                    Rc::new(Method::new(
                                        format!("{}=", attr_name),
                                        vec!["value".to_string()],
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
                            Object::String(s) => (*s).clone(),
                            Object::Symbol(s) => (*s).clone(),
                            _ => String::new(),
                        };
                        let old_name = match self.evaluate_expression(&call_args[1])? {
                            Object::String(s) => (*s).clone(),
                            Object::Symbol(s) => (*s).clone(),
                            _ => String::new(),
                        };
                        if !new_name.is_empty() && !old_name.is_empty() {
                            class.alias_method(&new_name, &old_name);
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
                        let method_name_str = if let Some(name_expr) = call_args.first() {
                            match self.evaluate_expression(name_expr)? {
                                Object::String(s) => (*s).clone(),
                                Object::Symbol(s) => (*s).clone(),
                                _ => {
                                    return Err(MetorexError::runtime_error(
                                        "define_method: first argument must be a String or Symbol",
                                        position_to_location(position),
                                    ));
                                }
                            }
                        } else {
                            return Err(MetorexError::runtime_error(
                                "define_method requires at least one argument",
                                position_to_location(position),
                            ));
                        };
                        let block_obj = self.evaluate_expression(block_expr)?;
                        let block = match block_obj {
                            Object::Block(b) => b,
                            _ => {
                                return Err(MetorexError::runtime_error(
                                    "define_method requires a block",
                                    position_to_location(position),
                                ));
                            }
                        };
                        let mut method = Method::new(
                            method_name_str.clone(),
                            block.parameters.clone(),
                            block.body.clone(),
                        );
                        // Capture closure: prefer existing captured_vars, otherwise snap current scope
                        method.captured_vars = Some(if block.captured_vars.is_empty() {
                            self.environment().current_scope_var_refs()
                        } else {
                            block.captured_vars.clone()
                        });
                        class.define_method(&method_name_str, Rc::new(method));
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
                    let _ = handled;
                }
            }
        }
        Ok(())
    }

    /// Execute function definition - create a Method object and register it in the environment as a function.
    pub(crate) fn execute_function_def(
        &mut self,
        name: &str,
        parameters: &[crate::ast::Parameter],
        body: &[Statement],
        position: crate::lexer::Position,
        singleton_class: Option<&str>,
    ) -> Result<ControlFlow, MetorexError> {
        // Extract positional parameter names (exclude named keyword and block params)
        let param_names: Vec<String> = parameters
            .iter()
            .filter(|p| !p.is_named_keyword && !p.is_block)
            .map(|p| p.name.clone())
            .collect();

        // Extract named keyword parameters
        let keyword_parameters: Vec<(String, Option<crate::ast::Expression>)> = parameters
            .iter()
            .filter(|p| p.is_named_keyword)
            .map(|p| (p.name.clone(), p.default_value.clone()))
            .collect();

        // Extract block parameter name
        let block_parameter = parameters
            .iter()
            .find(|p| p.is_block)
            .map(|p| p.name.clone());

        // Extract positional default values
        let default_parameters: Vec<(usize, crate::ast::Expression)> = parameters
            .iter()
            .filter(|p| !p.is_named_keyword && !p.is_block)
            .enumerate()
            .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
            .collect();

        // Create source location from position
        let source_location =
            crate::error::SourceLocation::new(position.line, position.column, position.offset);

        // Extract variadic parameter info
        let variadic_param = parameters
            .iter()
            .filter(|p| !p.is_named_keyword && !p.is_block)
            .enumerate()
            .find(|(_, p)| p.is_variadic)
            .map(|(i, p)| (i, p.name.clone()));

        // Create a Method object to represent the function
        let mut function = Method::with_source_location(
            name.to_string(),
            param_names,
            body.to_vec(),
            source_location,
        );
        function.default_parameters = default_parameters;
        function.keyword_parameters = keyword_parameters;
        function.block_parameter = block_parameter;
        function.variadic_param = variadic_param;
        function.captured_refinements = self.snapshot_active_refinements();
        let function = Rc::new(function);

        // Singleton method: define on the specific class (e.g., TrueClass)
        // or on a specific instance (`def x.foo` where `x` is an Object).
        if let Some(receiver_name) = singleton_class {
            let resolved = self
                .environment()
                .get(receiver_name)
                .or_else(|| self.globals().get(receiver_name));
            match resolved {
                Some(Object::Class(target_class)) => {
                    target_class.define_method(name, Rc::clone(&function));
                }
                Some(Object::Module(target_mod)) => {
                    target_mod.define_method(name, Rc::clone(&function));
                }
                Some(Object::Instance(inst)) => {
                    inst.borrow()
                        .define_singleton_method(name.to_string(), Rc::clone(&function));
                }
                _ => {}
            }
            return Ok(ControlFlow::Next);
        }

        // Register the function in the environment (for immediate local access)
        self.environment_mut()
            .define(name.to_string(), Object::Method(Rc::clone(&function)));

        // Also register as a method on the global Object class (Ruby semantics:
        // top-level `def` defines a method on Object, globally accessible).
        if let Some(Object::Class(object_class)) = self.globals().get("Object") {
            object_class.define_method(name, Rc::clone(&function));
        }

        Ok(ControlFlow::Next)
    }

    /// Execute module definition - create a Module object and register it.
    pub(crate) fn execute_module_def(
        &mut self,
        name: &str,
        body: &[Statement],
        _position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // If we're lexically nested inside a module/class, resolve the name
        // against the parent's constants first — `module Foo; module Bar; end; end`
        // defines `Foo::Bar`, distinct from any top-level `::Bar` of the same name.
        let parent_scope = self.def_scope_stack.last().cloned();
        let (module, existing_as_class) = if let Some(parent) = parent_scope.as_ref() {
            match parent.get_class_var(name) {
                Some(Object::Module(existing)) => (existing, false),
                Some(Object::Class(existing)) => (existing, true),
                _ => (Rc::new(Class::new(name, None)), false),
            }
        } else if let Some(Object::Module(existing)) = self.globals().get(name) {
            (existing, false)
        } else if let Some(Object::Module(existing)) = self.environment().get(name) {
            (existing, false)
        } else if let Some(Object::Class(existing)) = self.globals().get(name) {
            (existing, true)
        } else if let Some(Object::Class(existing)) = self.environment().get(name) {
            (existing, true)
        } else {
            (Rc::new(Class::new(name, None)), false)
        };

        // Set 'self' to the module for instance variable access in module body
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define("self".to_string(), Object::Module(Rc::clone(&module)));
        self.def_scope_stack.push(Rc::clone(&module));

        for statement in body {
            match statement {
                Statement::MethodDef {
                    name: method_name,
                    parameters,
                    body: method_body,
                    is_class_method,
                    ..
                } => {
                    let param_names: Vec<String> = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
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
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .enumerate()
                        .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
                        .collect();
                    let variadic_param = parameters
                        .iter()
                        .filter(|p| !p.is_named_keyword && !p.is_block)
                        .enumerate()
                        .find(|(_, p)| p.is_variadic)
                        .map(|(i, p)| (i, p.name.clone()));
                    let mut m = Method::new(method_name.clone(), param_names, method_body.clone());
                    m.default_parameters = default_params;
                    m.keyword_parameters = keyword_parameters;
                    m.block_parameter = block_parameter;
                    m.variadic_param = variadic_param;
                    m.captured_refinements = self.snapshot_active_refinements();
                    let method = Rc::new(m);
                    if *is_class_method {
                        module.define_method(format!("__class__{}", method_name), method);
                    } else {
                        module.define_method(method_name, method);
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
                    let const_value = self.evaluate_expression(value)?;
                    module.set_class_var(const_name, const_value);
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
                                // For modules, attr_* creates class-level methods
                                for attr in attributes {
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
                                    .filter(|p| !p.is_named_keyword && !p.is_block)
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
                    superclass,
                    body: class_body,
                    position: class_pos,
                } => {
                    self.execute_class_def(
                        class_name,
                        superclass.as_deref(),
                        class_body,
                        *class_pos,
                    )?;
                }
                // Nested module — execute_module_def attaches it to the enclosing scope.
                Statement::ModuleDef {
                    name: mod_name,
                    body: mod_body,
                    position: mod_pos,
                } => {
                    self.execute_module_def(mod_name, mod_body, *mod_pos)?;
                }
                // Include inside module body
                Statement::Include {
                    module_name: inc_name,
                    ..
                } => {
                    if let Some(Object::Module(inc_module)) = self
                        .environment()
                        .get(inc_name)
                        .or_else(|| self.globals().get(inc_name))
                    {
                        module.add_mixin(inc_module);
                    }
                }
                // Other statements in module body
                _ => {
                    self.execute_statement(statement)?;
                }
            }
        }

        self.def_scope_stack.pop();

        // Restore previous self
        if let Some(prev) = prev_self {
            self.environment_mut().define("self".to_string(), prev);
        } else {
            self.environment_mut().undefine("self");
        }

        let module_obj = if existing_as_class {
            Object::Class(module)
        } else {
            Object::Module(module)
        };
        if let Some(parent) = parent_scope {
            // Nested module: attach to parent as a constant; do NOT leak into globals,
            // which would clobber same-named builtins (e.g. ::Module, ::Class).
            parent.set_class_var(name, module_obj);
        } else {
            self.environment_mut()
                .define(name.to_string(), module_obj.clone());
            self.globals_mut().set(name.to_string(), module_obj);
        }

        Ok(ControlFlow::Next)
    }

    /// Execute include at statement level (outside class body).
    /// Top-level `include Mod` adds the module to Object's mixin chain
    /// (Ruby semantics for `include` at the main scope).
    /// Suppressed when running inside `load(path, true)` (wrapped load).
    pub(crate) fn execute_include(
        &mut self,
        module_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        if self.load_wrap_depth > 0 {
            return Ok(ControlFlow::Next);
        }
        let module = self
            .resolve_qualified_constant(module_name)
            .or_else(|| self.environment().get(module_name))
            .or_else(|| self.globals().get(module_name));
        let module_rc = match module {
            Some(Object::Module(m)) => m,
            Some(_) => {
                return Err(MetorexError::runtime_error(
                    format!("'{}' is not a module", module_name),
                    position_to_location(position),
                ));
            }
            None => {
                return Err(MetorexError::runtime_error(
                    format!("Undefined module '{}'", module_name),
                    position_to_location(position),
                ));
            }
        };
        if let Some(Object::Class(object_class)) = self.globals().get("Object") {
            object_class.add_mixin(module_rc);
        }
        Ok(ControlFlow::Next)
    }

    /// Resolve a `Foo::Bar::Baz` qualified constant by walking from the
    /// outermost name through nested module/class constants.
    fn resolve_qualified_constant(&self, qualified: &str) -> Option<Object> {
        let mut parts = qualified.split("::");
        let head = parts.next()?;
        let mut current = self
            .environment()
            .get(head)
            .or_else(|| self.globals().get(head))?;
        for part in parts {
            let class_rc = match &current {
                Object::Class(c) | Object::Module(c) => Rc::clone(c),
                _ => return None,
            };
            current = class_rc.get_class_var(part)?;
        }
        Some(current)
    }

    /// Execute extend at statement level (outside class body - error).
    pub(crate) fn execute_extend(
        &mut self,
        _module_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        Err(MetorexError::runtime_error(
            "extend can only be used inside a class definition",
            position_to_location(position),
        ))
    }
}
