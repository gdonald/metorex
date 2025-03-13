// Virtual machine core structure for the Metorex AST interpreter.
// This module defines the runtime scaffolding that powers execution.

use super::errors::*;
use super::init::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, GlobalRegistry, Heap};

use crate::ast::{Expression, Statement};
use crate::builtin_classes::BuiltinClasses;
use crate::environment::Environment;
use crate::error::MetorexError;
use crate::object::{BlockStatement, Object};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Core virtual machine responsible for executing Metorex programs.
pub struct VirtualMachine {
    environment: Environment,
    call_stack: Vec<CallFrame>,
    globals: GlobalRegistry,
    heap: Rc<RefCell<Heap>>,
    builtins: BuiltinClasses,
}

impl VirtualMachine {
    /// Construct a new virtual machine instance with all built-ins registered.
    pub fn new() -> Self {
        let mut environment = Environment::new();
        let builtins = BuiltinClasses::new();

        initialize_builtin_methods(&builtins);

        let mut globals = GlobalRegistry::new();
        register_builtin_classes(&mut globals, &builtins);
        register_singletons(&mut globals);
        register_native_functions(&mut globals);

        seed_environment_with_globals(&mut environment, &globals);

        Self {
            environment,
            call_stack: Vec::new(),
            globals,
            heap: Rc::new(RefCell::new(Heap::default())),
            builtins,
        }
    }

    /// Access the environment.
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    /// Mutably access the environment (used by the interpreter).
    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    /// Access the registered built-in classes.
    pub fn builtins(&self) -> &BuiltinClasses {
        &self.builtins
    }

    /// Access the global registry.
    pub fn globals(&self) -> &GlobalRegistry {
        &self.globals
    }

    /// Mutably access the global registry.
    pub fn globals_mut(&mut self) -> &mut GlobalRegistry {
        &mut self.globals
    }

    /// Borrow the heap allocator.
    pub fn heap(&self) -> Rc<RefCell<Heap>> {
        Rc::clone(&self.heap)
    }

    /// Run a closure with a new call frame pushed onto the stack.
    pub fn with_call_frame<F, R>(&mut self, frame: CallFrame, action: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.call_stack.push(frame);
        let result = action(self);
        self.call_stack.pop();
        result
    }

    /// Inspect the current call stack (top is last element).
    pub fn call_stack(&self) -> &[CallFrame] {
        &self.call_stack
    }

    /// Get the name of the current method being executed (from the top of the call stack).
    pub(crate) fn get_current_method_name(&self) -> Option<&str> {
        self.call_stack.last().map(|frame| frame.name())
    }

    /// Execute a sequence of statements and return an optional result (from return statements).
    pub fn execute_program(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        let mut last_value = None;

        for statement in statements {
            // If it's an expression statement, track its value
            if let Statement::Expression {
                expression,
                position,
            } = statement
            {
                let result = self.evaluate_expression(expression)?;

                // Ruby-style auto-call: if expression statement evaluates to a Method
                // and the expression is a bare identifier, auto-call it with zero args
                if matches!(expression, Expression::Identifier { .. })
                    && matches!(result, Object::Method(_))
                {
                    last_value = Some(self.invoke_callable(result, vec![], *position)?);
                    continue;
                }

                last_value = Some(result);
                continue;
            }

            // Match statements also produce values
            if let Statement::Match { .. } = statement {
                match self.execute_statement(statement)? {
                    ControlFlow::Return { value, .. } => {
                        last_value = Some(value);
                        continue;
                    }
                    ControlFlow::Next => {}
                    ControlFlow::Exception {
                        exception,
                        position,
                    } => {
                        return Err(MetorexError::runtime_error(
                            format!("Uncaught exception: {}", format_exception(&exception)),
                            position_to_location(position),
                        ));
                    }
                    ControlFlow::Break { position } => {
                        return Err(loop_control_error("break", position));
                    }
                    ControlFlow::Continue { position } => {
                        return Err(loop_control_error("continue", position));
                    }
                }
                continue;
            }

            // Execute other statements
            match self.execute_statement(statement)? {
                ControlFlow::Next => {}
                ControlFlow::Return { value, .. } => return Ok(Some(value)),
                ControlFlow::Exception {
                    exception,
                    position,
                } => {
                    return Err(MetorexError::runtime_error(
                        format!("Uncaught exception: {}", format_exception(&exception)),
                        position_to_location(position),
                    ));
                }
                ControlFlow::Break { position } => {
                    return Err(loop_control_error("break", position));
                }
                ControlFlow::Continue { position } => {
                    return Err(loop_control_error("continue", position));
                }
            }
        }

        Ok(last_value)
    }

    /// Evaluate an expression to a runtime value.
    pub(crate) fn evaluate_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<Object, MetorexError> {
        match expression {
            Expression::IntLiteral { value, .. } => Ok(Object::Int(*value)),
            Expression::FloatLiteral { value, .. } => Ok(Object::Float(*value)),
            Expression::StringLiteral { value, .. } => Ok(Object::String(Rc::new(value.clone()))),
            Expression::InterpolatedString { parts, .. } => self
                .evaluate_interpolated_string(parts)
                .map(|s| Object::String(Rc::new(s))),
            Expression::BoolLiteral { value, .. } => Ok(Object::Bool(*value)),
            Expression::NilLiteral { .. } => Ok(Object::Nil),
            Expression::Identifier { name, position } => self
                .environment
                .get(name)
                .ok_or_else(|| undefined_variable_error(name, *position)),
            Expression::Lambda {
                parameters,
                body,
                captured_vars,
                ..
            } => {
                let mut captured = HashMap::new();
                if let Some(names) = captured_vars {
                    if names.is_empty() {
                        // Empty vec signals automatic capture of all current scope variables
                        //  This is used for true lambdas (lambda do ... end, arrow syntax)
                        captured = self.environment().current_scope_var_refs();
                    } else {
                        // Explicit list of variables to capture
                        for name in names {
                            if let Some(value_ref) = self.environment().get_ref(name) {
                                captured.insert(name.clone(), value_ref);
                            }
                        }
                    }
                }
                // If captured_vars is None, don't capture anything (regular blocks for .each, etc.)
                let block = BlockStatement::new(parameters.clone(), body.clone(), captured);
                Ok(Object::Block(Rc::new(block)))
            }
            Expression::Grouped { expression, .. } => self.evaluate_expression(expression),
            Expression::UnaryOp {
                op,
                operand,
                position,
            } => {
                let value = self.evaluate_expression(operand)?;
                self.evaluate_unary_operation(op, value, *position)
            }
            Expression::BinaryOp {
                op,
                left,
                right,
                position,
            } => {
                let left_value = self.evaluate_expression(left)?;
                let right_value = self.evaluate_expression(right)?;
                self.evaluate_binary_operation(op, left_value, right_value, *position)
            }
            Expression::Array { elements, .. } => self.evaluate_array_literal(elements),
            Expression::Dictionary { entries, .. } => self.evaluate_dictionary_literal(entries),
            Expression::Index {
                array,
                index,
                position,
            } => {
                let collection = self.evaluate_expression(array)?;
                let key = self.evaluate_expression(index)?;
                self.evaluate_index_operation(collection, key, *position)
            }
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                trailing_block,
                position,
            } => self.evaluate_method_call(
                receiver,
                method,
                arguments,
                trailing_block.as_ref().map(|b| b.as_ref()),
                *position,
            ),
            Expression::Call {
                callee,
                arguments,
                position,
                ..
            } => {
                let callable = self.evaluate_expression(callee)?;
                let mut evaluated_args = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    evaluated_args.push(self.evaluate_expression(argument)?);
                }
                self.invoke_callable(callable, evaluated_args, *position)
            }
            Expression::SelfExpr { position } => self
                .environment
                .get("self")
                .ok_or_else(|| undefined_self_error(*position)),
            Expression::InstanceVariable { name, position } => {
                // Instance variables can only be read within a method (where 'self' is defined)
                match self.environment.get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let instance = instance_rc.borrow();
                        Ok(instance.get_var(name).cloned().unwrap_or(Object::Nil))
                    }
                    Some(_) => Err(MetorexError::runtime_error(
                        format!("Cannot read instance variable @{} on non-instance", name),
                        position_to_location(*position),
                    )),
                    None => Err(MetorexError::runtime_error(
                        format!(
                            "Instance variable @{} can only be used within a method",
                            name
                        ),
                        position_to_location(*position),
                    )),
                }
            }
            Expression::ClassVariable { name, position } => {
                // Class variables can be read within a method or class context
                match self.environment.get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let instance = instance_rc.borrow();
                        Ok(instance.class.get_class_var(name).unwrap_or(Object::Nil))
                    }
                    Some(Object::Class(class)) => {
                        Ok(class.get_class_var(name).unwrap_or(Object::Nil))
                    }
                    Some(_) => Err(MetorexError::runtime_error(
                        format!("Cannot read class variable @@{} in this context", name),
                        position_to_location(*position),
                    )),
                    None => Err(MetorexError::runtime_error(
                        format!(
                            "Class variable @@{} can only be used within a class or method",
                            name
                        ),
                        position_to_location(*position),
                    )),
                }
            }
            Expression::Super {
                arguments,
                position,
            } => {
                // Get the current self (must be an instance)
                let instance = match self.environment.get("self") {
                    Some(Object::Instance(instance_rc)) => instance_rc,
                    Some(_) => {
                        return Err(MetorexError::runtime_error(
                            "super can only be called from within an instance method".to_string(),
                            position_to_location(*position),
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            "super can only be called from within a method".to_string(),
                            position_to_location(*position),
                        ));
                    }
                };

                // Get the current method name from the call stack
                // The call stack stores method names as "Class#method", so we need to extract both parts
                let current_frame = self.get_current_method_name().ok_or_else(|| {
                    MetorexError::runtime_error(
                        "super called outside of a method context".to_string(),
                        position_to_location(*position),
                    )
                })?;

                // Extract the class name and method name (format: "Class#method")
                let (class_name, method_name) = if let Some(pos) = current_frame.rfind('#') {
                    (&current_frame[..pos], &current_frame[pos + 1..])
                } else {
                    return Err(MetorexError::runtime_error(
                        "super called in invalid context (no class information)".to_string(),
                        position_to_location(*position),
                    ));
                };

                // Get the instance's class to walk the inheritance chain
                let instance_borrowed = instance.borrow();
                let instance_class = &instance_borrowed.class;

                // Find the class that matches the current frame's class name
                let mut current_class = Some(Rc::clone(instance_class));
                let defining_class = loop {
                    match current_class {
                        Some(ref class) if class.name() == class_name => {
                            break Some(Rc::clone(class));
                        }
                        Some(ref class) => {
                            current_class = class.superclass();
                        }
                        None => break None,
                    }
                };

                let defining_class = defining_class.ok_or_else(|| {
                    MetorexError::runtime_error(
                        format!(
                            "Could not find defining class '{}' in inheritance chain",
                            class_name
                        ),
                        position_to_location(*position),
                    )
                })?;

                // Get the parent class of the defining class
                let parent_class = defining_class.superclass().ok_or_else(|| {
                    MetorexError::runtime_error(
                        format!("Class {} has no superclass", class_name),
                        position_to_location(*position),
                    )
                })?;

                // Look up the method in the parent class
                let method = parent_class.find_method(method_name).ok_or_else(|| {
                    MetorexError::runtime_error(
                        format!(
                            "Superclass {} does not define method '{}'",
                            parent_class.name(),
                            method_name
                        ),
                        position_to_location(*position),
                    )
                })?;

                // Evaluate the arguments
                let mut evaluated_args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }

                // Drop the borrow before invoking the method
                drop(instance_borrowed);

                // Invoke the parent method with self as the receiver
                self.invoke_method(
                    parent_class,
                    method,
                    Object::Instance(Rc::clone(&instance)),
                    evaluated_args,
                    *position,
                )
            }
            Expression::Range {
                start,
                end,
                exclusive,
                ..
            } => {
                let start_value = self.evaluate_expression(start)?;
                let end_value = self.evaluate_expression(end)?;
                Ok(Object::Range {
                    start: Box::new(start_value),
                    end: Box::new(end_value),
                    exclusive: *exclusive,
                })
            }
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}
