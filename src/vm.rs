// Virtual machine core structure for the Metorex AST interpreter.
// This module defines the runtime scaffolding that powers execution.

use crate::ast::{BinaryOp, Expression, InterpolationPart, Statement, UnaryOp};
use crate::builtin_classes::{self, BuiltinClasses};
use crate::environment::Environment;
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::Position;
use crate::object::Object;
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
        match self.execute_statements_internal(statements)? {
            ControlFlow::Next => Ok(None),
            ControlFlow::Return { value, .. } => Ok(Some(value)),
            ControlFlow::Break { position } => Err(loop_control_error("break", position)),
            ControlFlow::Continue { position } => Err(loop_control_error("continue", position)),
        }
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
            Statement::If { .. }
            | Statement::While { .. }
            | Statement::For { .. }
            | Statement::Match { .. }
            | Statement::FunctionDef { .. }
            | Statement::MethodDef { .. }
            | Statement::ClassDef { .. }
            | Statement::Begin { .. }
            | Statement::Raise { .. } => Err(unimplemented_statement_error(statement)),
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

    /// Assign a value to the given target expression.
    fn assign_value(&mut self, target: &Expression, value: Object) -> Result<(), MetorexError> {
        match target {
            Expression::Identifier { name, .. } => {
                if !self.environment.set(name, value.clone()) {
                    self.environment.define(name.clone(), value);
                }
                Ok(())
            }
            Expression::InstanceVariable { .. }
            | Expression::ClassVariable { .. }
            | Expression::Index { .. } => Err(invalid_assignment_target_error(target)),
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
            _ => Err(unsupported_expression_error(expression)),
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
}

/// Convert a lexer position into a runtime source location.
fn position_to_location(position: Position) -> SourceLocation {
    SourceLocation::new(position.line, position.column, position.offset)
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

/// Produce a runtime error for expressions that are not yet supported.
fn unsupported_expression_error(expression: &Expression) -> MetorexError {
    MetorexError::runtime_error(
        format!("Expression execution not implemented for {:?}", expression),
        position_to_location(expression.position()),
    )
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
