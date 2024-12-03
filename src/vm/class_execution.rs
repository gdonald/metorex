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
                    let param_names: Vec<String> =
                        parameters.iter().map(|p| p.name.clone()).collect();
                    let method = Rc::new(Method::new(
                        method_name.clone(),
                        param_names,
                        method_body.clone(),
                    ));
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
                _ => {
                    // For now, we ignore other statements in the class body
                    // In the future, we might support class-level code execution
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
    ) -> Result<ControlFlow, MetorexError> {
        // Extract parameter names from the parameter definitions
        let param_names: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();

        // Create a Method object to represent the function
        // (Method objects can represent both class methods and standalone functions)
        let function = Rc::new(Method::new(name.to_string(), param_names, body.to_vec()));

        // Register the function in the environment
        self.environment_mut()
            .define(name.to_string(), Object::Method(function));

        Ok(ControlFlow::Next)
    }
}
