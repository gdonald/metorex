// Virtual machine core structure for the Metorex AST interpreter.
// This module defines the runtime scaffolding that powers execution.

use crate::ast::{BinaryOp, Expression, InterpolationPart, Statement, UnaryOp};
use crate::builtin_classes::{self, BuiltinClasses};
use crate::callable::Callable;
use crate::class::Class;
use crate::environment::Environment;
use crate::error::{MetorexError, SourceLocation, StackFrame};
use crate::lexer::Position;
use crate::object::{BlockStatement, Method, Object};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Lightweight heap placeholder that will evolve with the runtime.
#[derive(Debug, Default)]
pub struct Heap {
    /// Tracks allocated objects for future GC integration.
    allocated: Vec<Object>,
}

impl Heap {
    /// Allocate an object on the heap (no-op stub for now).
    pub fn allocate(&mut self, object: Object) {
        self.allocated.push(object);
    }

    /// Returns number of tracked allocations (for testing/introspection).
    pub fn allocation_count(&self) -> usize {
        self.allocated.len()
    }
}

/// Call frame information stored on the VM call stack for debugging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrame {
    /// Human-readable frame identifier (method/function name).
    name: String,
    /// Optional source location ("file:line") to aid debugging.
    location: Option<String>,
}

impl CallFrame {
    /// Create a new call frame description.
    pub fn new(name: impl Into<String>, location: Option<String>) -> Self {
        Self {
            name: name.into(),
            location,
        }
    }

    /// Return the frame name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the optional source location.
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}

/// Registry that owns global objects accessible throughout the VM.
#[derive(Debug, Default)]
pub struct GlobalRegistry {
    objects: HashMap<String, Object>,
}

impl GlobalRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a named global object.
    pub fn set(&mut self, name: impl Into<String>, object: Object) {
        self.objects.insert(name.into(), object);
    }

    /// Fetch a named global object if present.
    pub fn get(&self, name: &str) -> Option<Object> {
        self.objects.get(name).cloned()
    }

    /// Determine whether a name exists in the registry.
    pub fn contains(&self, name: &str) -> bool {
        self.objects.contains_key(name)
    }

    /// Iterator over registered globals (useful for seeding environments).
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Object)> {
        self.objects.iter()
    }
}

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

    /// Evaluate a statement and produce control-flow information for the caller.
    fn execute_statement(&mut self, statement: &Statement) -> Result<ControlFlow, MetorexError> {
        match statement {
            Statement::Expression { expression, .. } => {
                self.evaluate_expression(expression)?;
                Ok(ControlFlow::Next)
            }
            Statement::Assignment {
                target,
                value,
                position: _,
            } => {
                let evaluated = self.evaluate_expression(value)?;
                self.assign_value(target, evaluated)?;
                Ok(ControlFlow::Next)
            }
            Statement::Return { value, position } => {
                let result = match value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Object::Nil,
                };
                Ok(ControlFlow::Return {
                    value: result,
                    position: *position,
                })
            }
            Statement::Break { position } => Ok(ControlFlow::Break {
                position: *position,
            }),
            Statement::Continue { position } => Ok(ControlFlow::Continue {
                position: *position,
            }),
            Statement::Block {
                statements,
                position: _,
            } => self.execute_block(statements),
            Statement::If {
                condition,
                then_branch,
                else_branch,
                position: _,
            } => self.execute_if(condition, then_branch, else_branch),
            Statement::While {
                condition,
                body,
                position: _,
            } => self.execute_while(condition, body),
            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => self.execute_for(variable, iterable, body, *position),
            Statement::ClassDef {
                name,
                superclass,
                body,
                position,
            } => self.execute_class_def(name, superclass.as_deref(), body, *position),
            Statement::MethodDef { .. } => {
                // MethodDef should only appear inside ClassDef bodies, not at top level
                Err(unimplemented_statement_error(statement))
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
            Statement::Match { .. } | Statement::FunctionDef { .. } => {
                Err(unimplemented_statement_error(statement))
            }
        }
    }

    /// Execute statements within a new lexical scope.
    fn execute_block(&mut self, statements: &[Statement]) -> Result<ControlFlow, MetorexError> {
        self.environment.push_scope();
        let result = self.execute_statements_internal(statements);
        self.environment.pop_scope();
        result
    }

    /// Core statement execution loop used by program and block execution.
    fn execute_statements_internal(
        &mut self,
        statements: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Next => continue,
                flow => return Ok(flow),
            }
        }
        Ok(ControlFlow::Next)
    }

    /// Execute an if/else statement.
    fn execute_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: &Option<Vec<Statement>>,
    ) -> Result<ControlFlow, MetorexError> {
        let condition_value = self.evaluate_expression(condition)?;

        if is_truthy(&condition_value) {
            self.execute_statements_internal(then_branch)
        } else if let Some(else_stmts) = else_branch {
            self.execute_statements_internal(else_stmts)
        } else {
            Ok(ControlFlow::Next)
        }
    }

    /// Execute a while loop.
    fn execute_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<ControlFlow, MetorexError> {
        loop {
            let condition_value = self.evaluate_expression(condition)?;

            if !is_truthy(&condition_value) {
                break;
            }

            match self.execute_statements_internal(body)? {
                ControlFlow::Next => continue,
                ControlFlow::Break { .. } => break,
                ControlFlow::Continue { .. } => continue,
                ControlFlow::Return { value, position } => {
                    return Ok(ControlFlow::Return { value, position });
                }
                ControlFlow::Exception {
                    exception,
                    position,
                } => {
                    return Ok(ControlFlow::Exception {
                        exception,
                        position,
                    });
                }
            }
        }

        Ok(ControlFlow::Next)
    }

    /// Execute a for loop over an iterable.
    fn execute_for(
        &mut self,
        variable: &str,
        iterable_expr: &Expression,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        let iterable = self.evaluate_expression(iterable_expr)?;

        let elements = match iterable {
            Object::Array(array_rc) => {
                let arr = array_rc.borrow();
                arr.clone()
            }
            other => {
                return Err(MetorexError::type_error(
                    format!(
                        "Cannot iterate over type '{}', expected Array",
                        other.type_name()
                    ),
                    position_to_location(position),
                ));
            }
        };

        for element in elements {
            self.environment.push_scope();
            self.environment.define(variable.to_string(), element);

            let result = self.execute_statements_internal(body);

            self.environment.pop_scope();

            match result? {
                ControlFlow::Next => continue,
                ControlFlow::Break { .. } => break,
                ControlFlow::Continue { .. } => continue,
                ControlFlow::Return { value, position } => {
                    return Ok(ControlFlow::Return { value, position });
                }
                ControlFlow::Exception {
                    exception,
                    position,
                } => {
                    return Ok(ControlFlow::Exception {
                        exception,
                        position,
                    });
                }
            }
        }

        Ok(ControlFlow::Next)
    }

    /// Execute class definition - create a Class object and register it in the environment.
    fn execute_class_def(
        &mut self,
        name: &str,
        superclass_name: Option<&str>,
        body: &[Statement],
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // Resolve superclass if specified
        let superclass = if let Some(super_name) = superclass_name {
            match self.environment.get(super_name) {
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
        self.environment
            .define(name.to_string(), Object::Class(class));

        Ok(ControlFlow::Next)
    }

    /// Execute a raise statement to throw an exception.
    fn execute_raise(
        &mut self,
        exception: &Option<Expression>,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        let exception_obj = if let Some(expr) = exception {
            // Evaluate the exception expression
            let value = self.evaluate_expression(expr)?;

            // If it's already an Exception object, use it directly
            // If it's a String, create a RuntimeError exception
            // If it's a Class (exception class), instantiate it
            match value {
                Object::Exception(_) => value,
                Object::String(message) => {
                    // Create a RuntimeError exception with the string message
                    Object::exception("RuntimeError", (*message).clone())
                }
                Object::Class(class) => {
                    // Instantiate the exception class with an empty message
                    Object::exception(class.name(), String::new())
                }
                _ => {
                    return Err(MetorexError::runtime_error(
                        "Exception must be an Exception object, String, or exception class"
                            .to_string(),
                        position_to_location(position),
                    ));
                }
            }
        } else {
            // Bare raise - re-raise current exception
            // For now, we'll check if there's a $! variable (current exception)
            // If not, it's an error to use bare raise outside a rescue block
            match self.environment.get("$!") {
                Some(Object::Exception(_)) => self.environment.get("$!").unwrap(),
                _ => {
                    return Err(MetorexError::runtime_error(
                        "No exception to re-raise (bare raise only allowed in rescue blocks)"
                            .to_string(),
                        position_to_location(position),
                    ));
                }
            }
        };

        Ok(ControlFlow::Exception {
            exception: exception_obj,
            position,
        })
    }

    /// Execute a begin/rescue/else/ensure block.
    fn execute_begin(
        &mut self,
        body: &[Statement],
        rescue_clauses: &[crate::ast::RescueClause],
        else_clause: &Option<Vec<Statement>>,
        ensure_block: &Option<Vec<Statement>>,
        _position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // Execute the try block
        let body_result = self.execute_statements_internal(body);

        // Track whether an exception was handled
        let mut handled_exception = false;
        let mut final_result = body_result;

        // If an exception occurred, try to match rescue clauses
        if let Ok(ControlFlow::Exception {
            exception,
            position: _ex_pos,
        }) = &final_result
        {
            // Store the current exception in $! for access in rescue blocks
            self.environment.define("$!".to_string(), exception.clone());

            // Try each rescue clause in order
            for rescue_clause in rescue_clauses {
                if self.exception_matches(exception, &rescue_clause.exception_types)? {
                    // Bind exception to variable if specified (=> e)
                    if let Some(var_name) = &rescue_clause.variable_name {
                        self.environment.define(var_name.clone(), exception.clone());
                    }

                    // Execute the rescue block
                    final_result = self.execute_statements_internal(&rescue_clause.body);
                    handled_exception = true;
                    break;
                }
            }

            // If exception wasn't handled, it will propagate
            if !handled_exception {
                // Keep the exception result to propagate it
                // Don't execute else clause
            } else {
                // Clear the $! variable since exception was handled
                self.environment.define("$!".to_string(), Object::Nil);
            }
        } else if final_result.is_ok() && matches!(final_result, Ok(ControlFlow::Next)) {
            // No exception occurred - execute else clause if present
            if let Some(else_stmts) = else_clause {
                final_result = self.execute_statements_internal(else_stmts);
            }
        }

        // Always execute ensure block, regardless of what happened
        if let Some(ensure_stmts) = ensure_block {
            let ensure_result = self.execute_statements_internal(ensure_stmts);

            // If ensure block raises an exception or changes control flow,
            // it overrides the previous result
            match ensure_result {
                Ok(ControlFlow::Exception { .. }) => {
                    final_result = ensure_result;
                }
                Ok(ControlFlow::Next) => {
                    // Ensure completed normally, don't override final_result
                }
                Ok(_) => {
                    // Other control flow (return, break, continue)
                    final_result = ensure_result;
                }
                Err(_) => {
                    // Error in ensure block overrides previous result
                    final_result = ensure_result;
                }
            }
        }

        final_result
    }

    /// Check if an exception matches the given exception type list.
    fn exception_matches(
        &self,
        exception: &Object,
        exception_types: &[String],
    ) -> Result<bool, MetorexError> {
        // Empty exception_types list means catch all exceptions
        if exception_types.is_empty() {
            return Ok(true);
        }

        // Get the exception's type name
        let exception_type_name = match exception {
            Object::Exception(ex) => ex.borrow().exception_type.clone(),
            _ => return Ok(false),
        };

        // Check if the exception's type matches any of the specified types
        for type_name in exception_types {
            // Look up the exception type class in the environment
            if let Some(Object::Class(target_class)) = self.environment.get(type_name) {
                // Get the class for this exception type
                if let Some(Object::Class(exception_class)) =
                    self.environment.get(&exception_type_name)
                {
                    // Check if exception_class is the target_class or a subclass of it
                    if Self::is_class_or_subclass(&exception_class, &target_class) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Check if a class is the same as or a subclass of another class.
    fn is_class_or_subclass(class: &Rc<Class>, target: &Rc<Class>) -> bool {
        if Rc::ptr_eq(class, target) {
            return true;
        }

        // Check superclass chain
        if let Some(superclass) = class.superclass() {
            return Self::is_class_or_subclass(&superclass, target);
        }

        false
    }

    /// Assign a value to the given target expression.
    fn assign_value(&mut self, target: &Expression, value: Object) -> Result<(), MetorexError> {
        match target {
            Expression::Identifier { name, .. } => {
                if !self.environment.set(name, value.clone()) {
                    self.environment.define(name.clone(), value);
                }
                Ok(())
            }
            Expression::InstanceVariable { name, position } => {
                // Instance variables can only be set within a method (where 'self' is defined)
                match self.environment.get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let mut instance = instance_rc.borrow_mut();
                        instance.set_var(name.clone(), value);
                        Ok(())
                    }
                    Some(_) => Err(MetorexError::runtime_error(
                        format!("Cannot set instance variable @{} on non-instance", name),
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
                // Class variables can only be set within a method or class context
                // For now, we'll look for 'self' to get the class
                match self.environment.get("self") {
                    Some(Object::Instance(instance_rc)) => {
                        let instance = instance_rc.borrow();
                        instance.class.set_class_var(name.clone(), value);
                        Ok(())
                    }
                    Some(Object::Class(class)) => {
                        class.set_class_var(name.clone(), value);
                        Ok(())
                    }
                    Some(_) => Err(MetorexError::runtime_error(
                        format!("Cannot set class variable @@{} in this context", name),
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
            Expression::Index { .. } => Err(invalid_assignment_target_error(target)),
            _ => Err(invalid_assignment_target_error(target)),
        }
    }

    /// Evaluate an expression to a runtime value.
    fn evaluate_expression(&mut self, expression: &Expression) -> Result<Object, MetorexError> {
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
            } => self.evaluate_method_call(receiver, method, arguments, *position),
            Expression::Call {
                callee,
                arguments,
                position,
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

    /// Evaluate string interpolation parts into a single owned string.
    fn evaluate_interpolated_string(
        &mut self,
        parts: &[InterpolationPart],
    ) -> Result<String, MetorexError> {
        let mut buffer = String::new();

        for part in parts {
            match part {
                InterpolationPart::Text(text) => buffer.push_str(text),
                InterpolationPart::Expression(expr) => {
                    let value = self.evaluate_expression(expr)?;
                    buffer.push_str(&value.to_string());
                }
            }
        }

        Ok(buffer)
    }

    /// Evaluate a unary operation (`+` or `-`).
    fn evaluate_unary_operation(
        &self,
        op: &UnaryOp,
        value: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match op {
            UnaryOp::Plus => match value {
                Object::Int(_) | Object::Float(_) => Ok(value),
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Minus => match value {
                Object::Int(v) => Ok(Object::Int(-v)),
                Object::Float(v) => Ok(Object::Float(-v)),
                _ => Err(unary_type_error(op, &value, position)),
            },
        }
    }

    /// Evaluate a binary operation across runtime values.
    fn evaluate_binary_operation(
        &self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;

        match op {
            Add => self.evaluate_addition(left, right, position),
            Subtract | Multiply | Divide | Modulo => {
                self.evaluate_numeric_binary(op, left, right, position)
            }
            Equal => Ok(Object::Bool(left.equals(&right))),
            NotEqual => Ok(Object::Bool(!left.equals(&right))),
            Less | Greater | LessEqual | GreaterEqual => {
                self.evaluate_comparison(op, left, right, position)
            }
            Assign | AddAssign | SubtractAssign | MultiplyAssign | DivideAssign => {
                Err(MetorexError::internal_error(format!(
                    "Assignment operation '{:?}' should be handled by statement execution",
                    op
                )))
            }
        }
    }

    /// Handle addition across supported operand types.
    fn evaluate_addition(
        &self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match (left, right) {
            (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a + b)),
            (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a + b)),
            (Object::Int(a), Object::Float(b)) => Ok(Object::Float((a as f64) + b)),
            (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a + (b as f64))),
            (Object::String(a), Object::String(b)) => {
                let mut combined = a.as_ref().clone();
                combined.push_str(b.as_ref());
                Ok(Object::String(Rc::new(combined)))
            }
            (lhs, rhs) => Err(binary_type_error(BinaryOp::Add, &lhs, &rhs, position)),
        }
    }

    /// Evaluate numeric binary operations (`-`, `*`, `/`, `%`).
    fn evaluate_numeric_binary(
        &self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match (left, right) {
            (Object::Int(a), Object::Int(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Int(a - b)),
                BinaryOp::Multiply => Ok(Object::Int(a * b)),
                BinaryOp::Divide => {
                    if b == 0 {
                        Err(divide_by_zero_error(position))
                    } else if a % b == 0 {
                        Ok(Object::Int(a / b))
                    } else {
                        Ok(Object::Float((a as f64) / (b as f64)))
                    }
                }
                BinaryOp::Modulo => {
                    if b == 0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Int(a % b))
                    }
                }
                _ => unreachable!(),
            },
            (Object::Float(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - b)),
                BinaryOp::Multiply => Ok(Object::Float(a * b)),
                BinaryOp::Divide => {
                    if b == 0.0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float(a / b))
                    }
                }
                BinaryOp::Modulo => {
                    if b == 0.0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float(a % b))
                    }
                }
                _ => unreachable!(),
            },
            (Object::Int(a), Object::Float(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float((a as f64) - b)),
                BinaryOp::Multiply => Ok(Object::Float((a as f64) * b)),
                BinaryOp::Divide => {
                    if b == 0.0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float((a as f64) / b))
                    }
                }
                BinaryOp::Modulo => {
                    if b == 0.0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float((a as f64) % b))
                    }
                }
                _ => unreachable!(),
            },
            (Object::Float(a), Object::Int(b)) => match op {
                BinaryOp::Subtract => Ok(Object::Float(a - (b as f64))),
                BinaryOp::Multiply => Ok(Object::Float(a * (b as f64))),
                BinaryOp::Divide => {
                    if b == 0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float(a / (b as f64)))
                    }
                }
                BinaryOp::Modulo => {
                    if b == 0 {
                        Err(divide_by_zero_error(position))
                    } else {
                        Ok(Object::Float(a % (b as f64)))
                    }
                }
                _ => unreachable!(),
            },
            (lhs, rhs) => Err(binary_type_error(op.clone(), &lhs, &rhs, position)),
        }
    }

    /// Evaluate comparison operations on numeric operands.
    fn evaluate_comparison(
        &self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (lhs, rhs) = match (&left, &right) {
            (Object::Int(a), Object::Int(b)) => (*a as f64, *b as f64),
            (Object::Float(a), Object::Float(b)) => (*a, *b),
            (Object::Int(a), Object::Float(b)) => (*a as f64, *b),
            (Object::Float(a), Object::Int(b)) => (*a, *b as f64),
            (lhs, rhs) => {
                return Err(binary_type_error(op.clone(), lhs, rhs, position));
            }
        };

        let result = match op {
            BinaryOp::Less => lhs < rhs,
            BinaryOp::Greater => lhs > rhs,
            BinaryOp::LessEqual => lhs <= rhs,
            BinaryOp::GreaterEqual => lhs >= rhs,
            _ => unreachable!(),
        };

        Ok(Object::Bool(result))
    }

    /// Evaluate array literal expressions.
    fn evaluate_array_literal(&mut self, elements: &[Expression]) -> Result<Object, MetorexError> {
        let mut evaluated = Vec::with_capacity(elements.len());
        for element in elements {
            evaluated.push(self.evaluate_expression(element)?);
        }
        Ok(Object::Array(Rc::new(RefCell::new(evaluated))))
    }

    /// Evaluate dictionary literal expressions.
    fn evaluate_dictionary_literal(
        &mut self,
        entries: &[(Expression, Expression)],
    ) -> Result<Object, MetorexError> {
        let mut map = HashMap::with_capacity(entries.len());

        for (key_expr, value_expr) in entries {
            let key_value = self.evaluate_expression(key_expr)?;
            let key_string = object_to_dict_key(&key_value).ok_or_else(|| {
                MetorexError::type_error(
                    format!(
                        "Dictionary keys must be String, Symbol, Integer, Float, Bool, or Nil, found {}",
                        key_value.type_name()
                    ),
                    position_to_location(key_expr.position()),
                )
            })?;

            let value = self.evaluate_expression(value_expr)?;
            map.insert(key_string, value);
        }

        Ok(Object::Dict(Rc::new(RefCell::new(map))))
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

    /// Evaluate indexing operations on arrays and dictionaries.
    fn evaluate_index_operation(
        &self,
        collection: Object,
        key: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match collection {
            Object::Array(elements_rc) => match key {
                Object::Int(index) => {
                    let elements = elements_rc.borrow();
                    if index < 0 || (index as usize) >= elements.len() {
                        Err(index_out_of_bounds_error(index, elements.len(), position))
                    } else {
                        Ok(elements[index as usize].clone())
                    }
                }
                _ => Err(MetorexError::type_error(
                    format!("Array index must be an Integer, found {}", key.type_name()),
                    position_to_location(position),
                )),
            },
            Object::Dict(dict_rc) => {
                let key_string = object_to_dict_key(&key).ok_or_else(|| {
                    MetorexError::type_error(
                        format!(
                            "Dictionary index must be String, Symbol, Integer, Float, Bool, or Nil, found {}",
                            key.type_name()
                        ),
                        position_to_location(position),
                    )
                })?;

                let dict = dict_rc.borrow();
                dict.get(&key_string)
                    .cloned()
                    .ok_or_else(|| undefined_dictionary_key_error(&key_string, position))
            }

            other => Err(MetorexError::type_error(
                format!("Cannot index into type '{}'", other.type_name()),
                position_to_location(position),
            )),
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

/// Represents control-flow signals produced during statement execution.
#[derive(Debug, Clone, PartialEq)]
enum ControlFlow {
    /// Normal execution, continue with next statement.
    Next,
    /// A return statement was encountered with an associated value.
    Return { value: Object, position: Position },
    /// A break statement was encountered.
    Break { position: Position },
    /// A continue statement was encountered.
    Continue { position: Position },
    /// An exception was raised and is propagating.
    Exception {
        exception: Object,
        position: Position,
    },
}

/// Convert a lexer position into a runtime source location.
fn position_to_location(position: Position) -> SourceLocation {
    SourceLocation::new(position.line, position.column, position.offset)
}

/// Format an exception object for display.
fn format_exception(exception: &Object) -> String {
    match exception {
        Object::Exception(ex) => {
            let exc = ex.borrow();
            format!("{}: {}", exc.exception_type, exc.message)
        }
        _ => format!("{:?}", exception),
    }
}

/// Produce a runtime error for unsupported control-flow usage (e.g., break outside loop).
fn loop_control_error(keyword: &str, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!("{keyword} cannot be used outside of a loop"),
        position_to_location(position),
    )
}

/// Produce a runtime error when attempting to assign to an invalid target.
fn invalid_assignment_target_error(target: &Expression) -> MetorexError {
    MetorexError::runtime_error(
        "Invalid assignment target",
        position_to_location(target.position()),
    )
}

/// Produce a runtime error when accessing `self` outside of a method context.
fn undefined_self_error(position: Position) -> MetorexError {
    MetorexError::runtime_error(
        "Undefined self in current context",
        position_to_location(position),
    )
}

/// Produce a runtime error when invoking an undefined method on a receiver.
fn undefined_method_error(method: &str, receiver: &Object, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!(
            "Undefined method '{}' for type '{}'",
            method,
            receiver.type_name()
        ),
        position_to_location(position),
    )
}

/// Produce a runtime error when a method receives the wrong number of arguments.
fn method_argument_error(
    method: &str,
    expected: usize,
    found: usize,
    position: Position,
) -> MetorexError {
    MetorexError::runtime_error(
        format!(
            "Method '{}' expected {} argument(s) but received {}",
            method, expected, found
        ),
        position_to_location(position),
    )
}

/// Produce a type error for invalid method argument type.
fn method_argument_type_error(
    method: &str,
    expected: &str,
    found: &Object,
    position: Position,
) -> MetorexError {
    MetorexError::type_error(
        format!(
            "Method '{}' expected argument of type '{}' but found '{}'",
            method,
            expected,
            found.type_name()
        ),
        position_to_location(position),
    )
}

/// Produce a runtime error when attempting to call a non-callable object.
fn not_callable_error(value: &Object, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!("Object of type '{}' is not callable", value.type_name()),
        position_to_location(position),
    )
}

/// Produce a runtime error when a callable receives the wrong number of arguments.
fn callable_argument_error(
    callable_name: &str,
    expected: usize,
    found: usize,
    position: Position,
) -> MetorexError {
    MetorexError::runtime_error(
        format!(
            "Callable '{}' expected {} argument(s) but received {}",
            callable_name, expected, found
        ),
        position_to_location(position),
    )
}

/// Produce a runtime error for referencing an undefined variable.
fn undefined_variable_error(name: &str, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!("Undefined variable '{name}'"),
        position_to_location(position),
    )
}

/// Produce an internal error for statements that are not yet implemented.
fn unimplemented_statement_error(statement: &Statement) -> MetorexError {
    MetorexError::internal_error(format!(
        "Statement execution not implemented for {:?}",
        statement
    ))
}

/// Produce a type error for unary operations.
fn unary_type_error(op: &UnaryOp, value: &Object, position: Position) -> MetorexError {
    MetorexError::type_error(
        format!(
            "Cannot apply unary operator '{:?}' to type '{}'",
            op,
            value.type_name()
        ),
        position_to_location(position),
    )
}

/// Produce a type error for binary operations.
fn binary_type_error(
    op: BinaryOp,
    left: &Object,
    right: &Object,
    position: Position,
) -> MetorexError {
    MetorexError::type_error(
        format!(
            "Cannot apply operator '{:?}' to types '{}' and '{}'",
            op,
            left.type_name(),
            right.type_name()
        ),
        position_to_location(position),
    )
}

/// Produce a divide-by-zero runtime error.
fn divide_by_zero_error(position: Position) -> MetorexError {
    MetorexError::runtime_error("Division by zero", position_to_location(position))
}

/// Produce an index out of bounds runtime error.
fn index_out_of_bounds_error(index: i64, length: usize, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!(
            "Index {} is out of bounds for array of length {}",
            index, length
        ),
        position_to_location(position),
    )
}

/// Produce a runtime error when a dictionary key is missing.
fn undefined_dictionary_key_error(key: &str, position: Position) -> MetorexError {
    MetorexError::runtime_error(
        format!("Key '{}' not found in dictionary", key),
        position_to_location(position),
    )
}

/// Convert an object into a dictionary key string representation.
fn object_to_dict_key(value: &Object) -> Option<String> {
    match value {
        Object::String(s) => Some((**s).clone()),
        Object::Int(i) => Some(i.to_string()),
        Object::Float(f) => Some(f.to_string()),
        Object::Bool(b) => Some(b.to_string()),
        Object::Nil => Some("nil".to_string()),
        _ => None,
    }
}

/// Determine if a value is truthy for conditional statements.
/// In Metorex, only `false` and `nil` are falsy; everything else is truthy.
fn is_truthy(value: &Object) -> bool {
    !matches!(value, Object::Bool(false) | Object::Nil)
}
