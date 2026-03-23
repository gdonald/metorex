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
        // Resolve superclass if specified
        let superclass = if let Some(super_name) = superclass_name {
            match self.environment().get(super_name) {
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

        // Create the class object
        let class = Rc::new(Class::new(name, superclass));

        // Process the class body to extract methods and instance variable declarations
        for statement in body {
            match statement {
                Statement::MethodDef {
                    name: method_name,
                    parameters,
                    body: method_body,
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
                    let mut m = Method::new(method_name.clone(), param_names, method_body.clone());
                    m.keyword_parameters = keyword_parameters;
                    m.block_parameter = block_parameter;
                    let method = Rc::new(m);
                    class.define_method(method_name, method);
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
                    // include ModuleName: copy module instance methods into this class
                    match self.environment().get(module_name) {
                        Some(Object::Module(module)) => {
                            for method_name in module.method_names() {
                                if let Some(method) = module.find_method(&method_name) {
                                    class.define_method(&method_name, method);
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
                _ => {
                    // Handle define_method(:name) { |args| body } calls in class body
                    if let Statement::Expression {
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
                }
            }
        }

        // Register the class in the environment
        self.environment_mut()
            .define(name.to_string(), Object::Class(class));

        Ok(ControlFlow::Next)
    }

    /// Execute function definition - create a Method object and register it in the environment as a function.
    pub(crate) fn execute_function_def(
        &mut self,
        name: &str,
        parameters: &[crate::ast::Parameter],
        body: &[Statement],
        position: crate::lexer::Position,
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

        // Create source location from position
        let source_location =
            crate::error::SourceLocation::new(position.line, position.column, position.offset);

        // Create a Method object to represent the function
        let mut function = Method::with_source_location(
            name.to_string(),
            param_names,
            body.to_vec(),
            source_location,
        );
        function.keyword_parameters = keyword_parameters;
        function.block_parameter = block_parameter;
        let function = Rc::new(function);

        // Register the function in the environment
        self.environment_mut()
            .define(name.to_string(), Object::Method(function));

        Ok(ControlFlow::Next)
    }

    /// Execute module definition - create a Module object and register it.
    pub(crate) fn execute_module_def(
        &mut self,
        name: &str,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        let module = Rc::new(Class::new(name, None));

        for statement in body {
            match statement {
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
                    let mut m = Method::new(method_name.clone(), param_names, method_body.clone());
                    m.keyword_parameters = keyword_parameters;
                    m.block_parameter = block_parameter;
                    module.define_method(method_name, Rc::new(m));
                }
                _ => {
                    return Err(MetorexError::runtime_error(
                        "Unsupported statement in module body".to_string(),
                        position_to_location(position),
                    ));
                }
            }
        }

        self.environment_mut()
            .define(name.to_string(), Object::Module(module));

        Ok(ControlFlow::Next)
    }

    /// Execute include at statement level (outside class body - error).
    pub(crate) fn execute_include(
        &mut self,
        _module_name: &str,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        Err(MetorexError::runtime_error(
            "include can only be used inside a class definition",
            position_to_location(position),
        ))
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
