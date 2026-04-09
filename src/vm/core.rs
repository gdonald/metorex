// Virtual machine core structure for the Metorex AST interpreter.
// This module defines the runtime scaffolding that powers execution.

use super::errors::*;
use super::init::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, GlobalRegistry, Heap};

use crate::ast::{BinaryOp, Expression, Statement};
use crate::builtin_classes::BuiltinClasses;
use crate::environment::Environment;
use crate::error::MetorexError;
use crate::object::{BlockStatement, Object};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

/// Core virtual machine responsible for executing Metorex programs.
pub struct VirtualMachine {
    environment: Environment,
    call_stack: Vec<CallFrame>,
    globals: GlobalRegistry,
    heap: Rc<RefCell<Heap>>,
    builtins: BuiltinClasses,
    current_file: Option<PathBuf>,
    loaded_files: HashSet<PathBuf>,
    /// Trailing block passed to the current call (e.g., `foo() do |x| ... end`).
    /// Set before invoke_method/invoke_callable; taken at method body entry.
    pub(crate) pending_block: Option<Object>,
}

impl VirtualMachine {
    /// Construct a new virtual machine instance with all built-ins registered.
    pub fn new() -> Self {
        let mut environment = Environment::new();
        let builtins = BuiltinClasses::new();

        initialize_builtin_methods(&builtins);

        let mut globals = GlobalRegistry::new();
        register_builtin_classes(&mut globals, &builtins);
        register_builtin_modules(&mut globals);
        register_singletons(&mut globals);
        register_special_globals(&mut globals);
        register_native_functions(&mut globals);

        seed_environment_with_globals(&mut environment, &globals);

        Self {
            environment,
            call_stack: Vec::new(),
            globals,
            heap: Rc::new(RefCell::new(Heap::default())),
            builtins,
            current_file: None,
            loaded_files: HashSet::new(),
            pending_block: None,
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

    /// Set the ARGV global with script arguments.
    pub fn set_argv(&mut self, args: Vec<String>) {
        let elements: Vec<Object> = args
            .into_iter()
            .map(|s| Object::String(Rc::new(s)))
            .collect();
        let argv = Object::Array(Rc::new(RefCell::new(elements)));
        self.globals.set("ARGV", argv.clone());
        self.environment.define("ARGV".to_string(), argv);
    }

    /// Set the current file being executed.
    pub fn set_current_file(&mut self, path: PathBuf) {
        self.current_file = Some(path);
    }

    /// Get the current file being executed.
    pub fn get_current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }

    /// Mark a file as loaded in the registry.
    pub fn mark_file_loaded(&mut self, path: PathBuf) {
        self.loaded_files.insert(path);
    }

    /// Check if a file has already been loaded.
    pub fn is_file_loaded(&self, path: &PathBuf) -> bool {
        self.loaded_files.contains(path)
    }

    /// Prepend a path to the `$LOAD_PATH` (`$:`) global array.
    pub fn prepend_load_path(&mut self, path: String) {
        if let Some(Object::Array(arr)) = self.globals.get(":") {
            arr.borrow_mut().insert(0, Object::String(Rc::new(path)));
        }
    }

    /// Require a library by name, searching `$LOAD_PATH` just like the `require` builtin.
    pub fn require_library(&mut self, name: &str) -> Result<(), MetorexError> {
        use crate::error::SourceLocation;

        let load_path = self.globals().get(":").unwrap_or(Object::Nil);
        let search_dirs: Vec<String> = match &load_path {
            Object::Array(arr) => arr
                .borrow()
                .iter()
                .filter_map(|obj| match obj {
                    Object::String(s) => Some(s.as_ref().clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };

        let mut found_path = None;
        for dir in &search_dirs {
            let base = std::path::PathBuf::from(dir);
            let candidates = [base.join(name), base.join(format!("{}.rb", name))];
            for candidate in &candidates {
                if candidate.exists() {
                    found_path = Some(candidate.clone());
                    break;
                }
            }
            if found_path.is_some() {
                break;
            }
        }

        let resolved = found_path.ok_or_else(|| {
            MetorexError::runtime_error(
                format!(
                    "cannot load such file -- {} (searched in $LOAD_PATH: {:?})",
                    name, search_dirs
                ),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        self.execute_file(&resolved).map_err(|e| {
            MetorexError::runtime_error(
                format!("require('{}') — {}", name, e.message()),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        Ok(())
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

            // Match/CaseIn statements also produce values
            if matches!(
                statement,
                Statement::Match { .. } | Statement::CaseIn { .. }
            ) {
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

    /// Execute a file with automatic deduplication and path tracking.
    ///
    /// This method loads and executes a file, handling:
    /// - File deduplication (files are only executed once)
    /// - Current file path tracking (for require_relative)
    /// - Automatic path canonicalization
    /// - Proper restoration of the previous current file
    ///
    /// # Arguments
    /// * `path` - The path to the file to execute
    ///
    /// # Returns
    /// * `Ok(Object)` - The result of executing the file (or Nil if already loaded)
    /// * `Err(MetorexError)` - If loading, parsing, or execution fails
    pub fn execute_file(&mut self, path: &std::path::Path) -> Result<Object, MetorexError> {
        use crate::error::SourceLocation;
        use crate::file_loader::{find_file_path, load_file_source, parse_file};

        // Find the actual file path (with extension auto-detection)
        let actual_path = find_file_path(path)?;

        // Canonicalize the file path to absolute path for proper deduplication
        let canonical_path = actual_path.canonicalize().map_err(|e| {
            MetorexError::runtime_error(
                format!(
                    "Failed to canonicalize file path '{}': {}",
                    actual_path.display(),
                    e
                ),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        // Check if file is already loaded (deduplication)
        if self.is_file_loaded(&canonical_path) {
            return Ok(Object::Nil);
        }

        // Mark file as loaded before executing to prevent circular dependencies
        self.mark_file_loaded(canonical_path.clone());

        // Save the current file path to restore later
        let previous_file = self.current_file.clone();

        // Load file source with error context
        let source = load_file_source(&canonical_path).map_err(|e| {
            MetorexError::runtime_error(
                format!("Failed to load file '{}': {}", canonical_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        // Parse file with error context
        let statements = parse_file(&source, &canonical_path.to_string_lossy()).map_err(|e| {
            MetorexError::runtime_error(
                format!("Failed to parse file '{}': {}", canonical_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        // Update current file path for require_relative calls within this file
        self.set_current_file(canonical_path.clone());

        // Execute the parsed statements
        let result = self.execute_program(&statements).map_err(|e| {
            MetorexError::runtime_error(
                format!("Error executing file '{}': {}", canonical_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        // Restore previous current file path
        self.current_file = previous_file;

        // Return the result or Nil if no return value
        Ok(result.unwrap_or(Object::Nil))
    }

    /// Evaluate a list of argument expressions, expanding any splat (`*expr`) arguments.
    pub(crate) fn evaluate_arguments(
        &mut self,
        argument_exprs: &[Expression],
    ) -> Result<Vec<Object>, MetorexError> {
        let mut args = Vec::with_capacity(argument_exprs.len());
        for arg in argument_exprs {
            if let Expression::Splat { expression, .. } = arg {
                let value = self.evaluate_expression(expression)?;
                match value {
                    Object::Array(arr) => {
                        args.extend(arr.borrow().iter().cloned());
                    }
                    other => {
                        // Non-array splat: treat as single argument
                        args.push(other);
                    }
                }
            } else {
                args.push(self.evaluate_expression(arg)?);
            }
        }
        Ok(args)
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
            Expression::Symbol { value, .. } => Ok(Object::Symbol(Rc::new(value.clone()))),
            Expression::RegexLiteral { pattern, flags, .. } => Ok(Object::Regex(
                Rc::new(pattern.clone()),
                Rc::new(flags.clone()),
            )),
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
                // Short-circuit evaluation for logical operators
                match op {
                    BinaryOp::And => {
                        let left_value = self.evaluate_expression(left)?;
                        return if !is_truthy(&left_value) {
                            Ok(left_value)
                        } else {
                            self.evaluate_expression(right)
                        };
                    }
                    BinaryOp::Or => {
                        let left_value = self.evaluate_expression(left)?;
                        return if is_truthy(&left_value) {
                            Ok(left_value)
                        } else {
                            self.evaluate_expression(right)
                        };
                    }
                    BinaryOp::Assign => {
                        let value = self.evaluate_expression(right)?;
                        self.assign_value(left, value.clone())?;
                        return Ok(value);
                    }
                    _ => {}
                }
                let left_value = self.evaluate_expression(left)?;
                let right_value = self.evaluate_expression(right)?;
                // Check for user-defined operator methods on instances
                if let (Some(op_name), Object::Instance(instance_rc)) =
                    (binary_op_method_name(op), &left_value)
                {
                    let class = Rc::clone(&instance_rc.borrow().class);
                    if let Some(method) = class.find_method(op_name) {
                        return self.invoke_method(
                            class,
                            method,
                            left_value.clone(),
                            vec![right_value],
                            *position,
                        );
                    }
                }
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
                // Block/Lambda [] call syntax: proc[args]
                if let Object::Block(block) = &collection {
                    return block.call(self, vec![key], *position);
                }
                // Check for user-defined [] method on instances
                if let Object::Instance(instance_rc) = &collection {
                    let class = Rc::clone(&instance_rc.borrow().class);
                    if let Some(method) = class.find_method("[]") {
                        return self.invoke_method(
                            class,
                            method,
                            collection.clone(),
                            vec![key],
                            *position,
                        );
                    }
                }
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
                trailing_block,
                position,
            } => {
                let callable = self.evaluate_expression(callee)?;
                let evaluated_args = self.evaluate_arguments(arguments)?;
                if let Some(block_expr) = trailing_block {
                    self.pending_block = Some(self.evaluate_expression(block_expr)?);
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
            Expression::GlobalVariable { name, .. } => {
                Ok(self.globals.get(name).unwrap_or(Object::Nil))
            }
            Expression::MagicFile { .. } => {
                let path = self
                    .current_file
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(eval)".to_string());
                Ok(Object::String(Rc::new(path)))
            }
            Expression::MagicLine { position, .. } => Ok(Object::Int(position.line as i64)),
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
            Expression::Defined { expression, .. } => {
                let result = match expression.as_ref() {
                    Expression::Identifier { name, .. } => match self.environment.get(name) {
                        Some(Object::Method(_)) => Some("method"),
                        Some(Object::Class(_)) | Some(Object::Module(_)) => Some("constant"),
                        Some(_) => Some("local-variable"),
                        None => {
                            if self.globals.contains(name) {
                                Some("method")
                            } else {
                                None
                            }
                        }
                    },
                    Expression::GlobalVariable { name, .. } => {
                        if self.globals.get(name).is_some_and(|v| v != Object::Nil) {
                            Some("global-variable")
                        } else {
                            None
                        }
                    }
                    Expression::InstanceVariable { name, .. } => {
                        let var_name = if name.starts_with('@') {
                            name.clone()
                        } else {
                            format!("@{}", name)
                        };
                        match self.environment.get("self") {
                            Some(Object::Instance(inst)) => {
                                if inst.borrow().get_var(&var_name).is_some() {
                                    Some("instance-variable")
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }
                    Expression::ClassVariable { .. } => {
                        // Simplified: just check if we can evaluate it
                        if self.evaluate_expression(expression).is_ok() {
                            Some("class variable")
                        } else {
                            None
                        }
                    }
                    Expression::ScopeResolution { .. } => {
                        if self.evaluate_expression(expression).is_ok() {
                            Some("constant")
                        } else {
                            None
                        }
                    }
                    Expression::MethodCall { .. } | Expression::Call { .. } => Some("method"),
                    Expression::Yield { .. } => {
                        if self.environment.get("__block__").is_some() {
                            Some("yield")
                        } else {
                            None
                        }
                    }
                    // Literals are always defined
                    Expression::IntLiteral { .. }
                    | Expression::FloatLiteral { .. }
                    | Expression::StringLiteral { .. }
                    | Expression::BoolLiteral { .. }
                    | Expression::NilLiteral { .. }
                    | Expression::Symbol { .. }
                    | Expression::Array { .. }
                    | Expression::Dictionary { .. }
                    | Expression::RegexLiteral { .. } => Some("expression"),
                    Expression::Super { .. } => Some("super"),
                    Expression::SelfExpr { .. } => Some("self"),
                    // For anything else, try evaluating and check
                    _ => {
                        if self.evaluate_expression(expression).is_ok() {
                            Some("expression")
                        } else {
                            None
                        }
                    }
                };
                match result {
                    Some(desc) => Ok(Object::String(Rc::new(desc.to_string()))),
                    None => Ok(Object::Nil),
                }
            }
            Expression::Splat { expression, .. } => {
                // Outside of argument lists, splat evaluates to the array itself
                let value = self.evaluate_expression(expression)?;
                match value {
                    arr @ Object::Array(_) => Ok(arr),
                    other => Ok(Object::Array(Rc::new(RefCell::new(vec![other])))),
                }
            }
            Expression::Yield {
                arguments,
                position,
            } => {
                let block = self.environment.get("__block__").or_else(|| {
                    // Also check if there's a named block parameter
                    self.environment.get("block_given?").and_then(|bg| {
                        if bg == Object::Bool(true) {
                            // The block was bound to a named parameter — find it
                            None
                        } else {
                            None
                        }
                    })
                });

                let block = match block {
                    Some(Object::Block(b)) => b,
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "no block given (yield)".to_string(),
                            position_to_location(*position),
                        ));
                    }
                };

                let mut evaluated_args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }

                block.call(self, evaluated_args, *position)
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
            Expression::Case {
                expression,
                cases,
                else_case,
                position,
            } => {
                // This will be implemented in Phase 3
                // For now, delegate to the pattern matching module
                self.evaluate_case_expression(expression, cases, else_case.as_deref(), *position)
            }
            Expression::ScopeResolution {
                namespace,
                name,
                position,
            } => {
                let ns_value = self.evaluate_expression(namespace)?;
                match ns_value {
                    Object::Class(class_rc) | Object::Module(class_rc) => {
                        class_rc.get_class_var(name).ok_or_else(|| {
                            MetorexError::runtime_error(
                                format!("Uninitialized constant {}::{}", class_rc.name(), name),
                                position_to_location(*position),
                            )
                        })
                    }
                    _ => Err(MetorexError::runtime_error(
                        "'::' scope resolution requires a class or module as namespace".to_string(),
                        position_to_location(*position),
                    )),
                }
            }

            Expression::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                ..
            } => self.evaluate_if_expression(condition, then_branch, elsif_branches, else_branch),

            Expression::Unless {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.evaluate_unless_expression(condition, then_branch, else_branch),
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a binary operator to its operator method name for user-defined dispatch.
fn binary_op_method_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("+"),
        BinaryOp::Subtract => Some("-"),
        BinaryOp::Multiply => Some("*"),
        BinaryOp::Divide => Some("/"),
        BinaryOp::Modulo => Some("%"),
        BinaryOp::Equal => Some("=="),
        BinaryOp::NotEqual => Some("!="),
        BinaryOp::Less => Some("<"),
        BinaryOp::Greater => Some(">"),
        BinaryOp::LessEqual => Some("<="),
        BinaryOp::GreaterEqual => Some(">="),
        BinaryOp::Spaceship => Some("<=>"),
        _ => None,
    }
}
