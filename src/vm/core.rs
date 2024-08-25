// Virtual machine core structure for the Metorex AST interpreter.
// This module defines the runtime scaffolding that powers execution.

use super::errors::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, GlobalRegistry, Heap};

use crate::ast::{Expression, Statement};
use crate::builtin_classes::{self, BuiltinClasses};
use crate::callable::Callable;
use crate::class::Class;
use crate::environment::Environment;
use crate::error::{MetorexError, StackFrame};
use crate::lexer::Position;
use crate::object::{BlockStatement, Method, Object};
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

    /// Execute a sequence of statements and return an optional result (from return statements).
    pub fn execute_program(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        let mut last_value = None;

        for statement in statements {
            // If it's an expression statement, track its value
            if let Statement::Expression { expression, .. } = statement {
                last_value = Some(self.evaluate_expression(expression)?);
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
                    for name in names {
                        if let Some(value) = self.environment().get(name) {
                            captured.insert(name.clone(), value);
                        }
                    }
                }
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
                position,
                ..
            } => self.evaluate_method_call(receiver, method, arguments, *position),
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
        }
    }

    /// Evaluate a method call expression on a receiver object.
    fn evaluate_method_call(
        &mut self,
        receiver_expr: &Expression,
        method_name: &str,
        argument_exprs: &[Expression],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let receiver = self.evaluate_expression(receiver_expr)?;
        let mut arguments = Vec::with_capacity(argument_exprs.len());
        for argument in argument_exprs {
            arguments.push(self.evaluate_expression(argument)?);
        }

        match self.lookup_method(&receiver, method_name) {
            Some((class, method)) => {
                self.invoke_method(class, method, receiver, arguments, position)
            }
            None => Err(undefined_method_error(method_name, &receiver, position)),
        }
    }

    /// Look up a method on the receiver and return its class and method definition.
    fn lookup_method(
        &self,
        receiver: &Object,
        method_name: &str,
    ) -> Option<(Rc<Class>, Rc<Method>)> {
        match receiver {
            Object::Instance(instance_rc) => {
                let instance_ref = instance_rc.borrow();
                let class = Rc::clone(&instance_ref.class);
                drop(instance_ref);
                class.find_method(method_name).map(|method| (class, method))
            }
            Object::Class(class_rc) => class_rc
                .find_method(method_name)
                .map(|method| (Rc::clone(class_rc), method)),
            _ => {
                let class = self.builtins.class_of(receiver);
                class.find_method(method_name).map(|method| (class, method))
            }
        }
    }

    /// Invoke a resolved method with evaluated arguments.
    fn invoke_callable(
        &mut self,
        callable: Object,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match callable {
            Object::Block(block) => block.call(self, arguments, position),
            Object::Class(class) => {
                // Create a new instance of the class
                let instance = Rc::new(RefCell::new(crate::object::Instance::new(Rc::clone(
                    &class,
                ))));
                let instance_obj = Object::Instance(Rc::clone(&instance));

                // Look for an 'initialize' method and call it if present
                if let Some(init_method) = class.find_method("initialize") {
                    self.invoke_method(
                        class,
                        init_method,
                        instance_obj.clone(),
                        arguments,
                        position,
                    )?;
                } else if !arguments.is_empty() {
                    // If no initialize method exists, reject non-empty arguments
                    return Err(MetorexError::runtime_error(
                        format!(
                            "No initialize method defined, but {} argument(s) provided",
                            arguments.len()
                        ),
                        position_to_location(position),
                    ));
                }

                Ok(instance_obj)
            }
            other => Err(not_callable_error(&other, position)),
        }
    }

    /// Execute a block callable within the VM, handling scope capture and return semantics.
    pub(crate) fn execute_block_callable(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let expected = block.arity();
        let found = arguments.len();

        if expected != found {
            return Err(callable_argument_error(
                block.name(),
                expected,
                found,
                position,
            ));
        }

        let frame_name = block.name().to_string();
        let frame_location = position_to_location(position);
        let frame_location_string = Some(format!("{}", frame_location));

        let execution_result = self.with_call_frame(
            CallFrame::new(frame_name.clone(), frame_location_string),
            move |vm| vm.execute_block_body(block, arguments),
        );

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(frame_name, frame_location))),
        }
    }

    /// Execute the statements inside a block object with its captured scope.
    fn execute_block_body(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        self.environment.push_scope();

        let result = (|| -> Result<Object, MetorexError> {
            for (name, value) in block.captured_vars() {
                self.environment.define(name.clone(), value.clone());
            }

            for (param, argument) in block.parameters().iter().zip(arguments.into_iter()) {
                self.environment.define(param.clone(), argument);
            }

            let mut last_value = Object::Nil;

            for statement in block.body() {
                if let Statement::Expression { expression, .. } = statement {
                    last_value = self.evaluate_expression(expression)?;
                    continue;
                }

                match self.execute_statement(statement)? {
                    ControlFlow::Next => {}
                    ControlFlow::Return { value, .. } => {
                        last_value = value;
                        break;
                    }
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
        })();

        self.environment.pop_scope();
        result
    }

    /// Invoke a resolved method with evaluated arguments.
    fn invoke_method(
        &mut self,
        class: Rc<Class>,
        method: Rc<Method>,
        receiver: Object,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let method_name = method.name.clone();

        if let Some(result) = self.call_native_method(
            class.as_ref(),
            &receiver,
            &method_name,
            &arguments,
            position,
        )? {
            return Ok(result);
        }

        let expected = method.parameters.len();
        let found = arguments.len();
        if expected != found {
            return Err(method_argument_error(
                &method_name,
                expected,
                found,
                position,
            ));
        }

        let frame_name = format!("{}#{}", class.name(), method_name);
        let frame_location = position_to_location(position);
        let frame_location_string = Some(format!("{}", frame_location));

        let method_for_body = Rc::clone(&method);
        let self_for_body = method
            .receiver()
            .cloned()
            .unwrap_or_else(|| receiver.clone());
        let arguments_for_body = arguments.clone();
        let execution_result = self.with_call_frame(
            CallFrame::new(frame_name.clone(), frame_location_string),
            move |vm| {
                vm.execute_method_body(
                    method_for_body.as_ref(),
                    self_for_body.clone(),
                    arguments_for_body.clone(),
                )
            },
        );

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(frame_name, frame_location))),
        }
    }

    /// Execute the body of a method within a fresh scope.
    fn execute_method_body(
        &mut self,
        method: &Method,
        self_value: Object,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        self.environment.push_scope();

        let result = (|| -> Result<Object, MetorexError> {
            self.environment
                .define("self".to_string(), self_value.clone());

            for (param, value) in method.parameters.iter().zip(arguments.into_iter()) {
                self.environment.define(param.clone(), value);
            }

            match self.execute_statements_internal(method.body())? {
                ControlFlow::Next => Ok(Object::Nil),
                ControlFlow::Return { value, .. } => Ok(value),
                ControlFlow::Exception {
                    exception,
                    position,
                } => Err(MetorexError::runtime_error(
                    format!("Uncaught exception: {}", format_exception(&exception)),
                    position_to_location(position),
                )),
                ControlFlow::Break { position } => Err(loop_control_error("break", position)),
                ControlFlow::Continue { position } => Err(loop_control_error("continue", position)),
            }
        })();

        self.environment.pop_scope();
        result
    }

    /// Attempt to execute a native (built-in) method implementation.
    fn call_native_method(
        &mut self,
        class: &Class,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match class.name() {
            "Object" => match method_name {
                "to_s" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    Ok(Some(Object::string(receiver.to_string())))
                }
                "class" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    Ok(Some(Object::Class(self.builtins.class_of(receiver))))
                }
                "respond_to?" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let method_query = match &arguments[0] {
                        Object::String(name) => name.as_str().to_string(),
                        other => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                other,
                                position,
                            ));
                        }
                    };
                    Ok(Some(Object::Bool(
                        self.lookup_method(receiver, &method_query).is_some(),
                    )))
                }
                _ => Ok(None),
            },
            "String" => match method_name {
                "length" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        Ok(Some(Object::Int(string_value.chars().count() as i64)))
                    } else {
                        Ok(None)
                    }
                }
                "upcase" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        Ok(Some(Object::string(string_value.to_uppercase())))
                    } else {
                        Ok(None)
                    }
                }
                "downcase" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::String(string_value) = receiver {
                        Ok(Some(Object::string(string_value.to_lowercase())))
                    } else {
                        Ok(None)
                    }
                }
                "+" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let (Object::String(lhs), Object::String(rhs)) = (receiver, &arguments[0]) {
                        let mut combined = lhs.as_ref().clone();
                        combined.push_str(rhs);
                        Ok(Some(Object::string(combined)))
                    } else {
                        Err(method_argument_type_error(
                            method_name,
                            "String",
                            &arguments[0],
                            position,
                        ))
                    }
                }
                _ => Ok(None),
            },
            "Array" => match method_name {
                "length" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Array(array_rc) = receiver {
                        Ok(Some(Object::Int(array_rc.borrow().len() as i64)))
                    } else {
                        Ok(None)
                    }
                }
                "push" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Array(array_rc) = receiver {
                        array_rc.borrow_mut().push(arguments[0].clone());
                        Ok(Some(receiver.clone()))
                    } else {
                        Ok(None)
                    }
                }
                "pop" => {
                    if !arguments.is_empty() {
                        return Err(method_argument_error(
                            method_name,
                            0,
                            arguments.len(),
                            position,
                        ));
                    }
                    if let Object::Array(array_rc) = receiver {
                        Ok(Some(array_rc.borrow_mut().pop().unwrap_or(Object::Nil)))
                    } else {
                        Ok(None)
                    }
                }
                "[]" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    Ok(Some(self.evaluate_index_operation(
                        receiver.clone(),
                        arguments[0].clone(),
                        position,
                    )?))
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

fn initialize_builtin_methods(builtins: &BuiltinClasses) {
    builtin_classes::init_object_methods(builtins.object_class.as_ref());
    builtin_classes::init_string_methods(builtins.string_class.as_ref());
    builtin_classes::init_array_methods(builtins.array_class.as_ref());
}

fn register_builtin_classes(globals: &mut GlobalRegistry, builtins: &BuiltinClasses) {
    for (name, class) in builtins.all_classes() {
        globals.set(name, Object::Class(class));
    }
}

fn register_singletons(globals: &mut GlobalRegistry) {
    globals.set("nil", Object::Nil);
    globals.set("true", Object::Bool(true));
    globals.set("false", Object::Bool(false));
}

fn seed_environment_with_globals(environment: &mut Environment, globals: &GlobalRegistry) {
    for (name, value) in globals.iter() {
        environment.define(name.clone(), value.clone());
    }
}
