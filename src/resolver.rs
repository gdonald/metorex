// Variable resolution for Metorex
// This module implements static analysis for variable declarations and usage
// It tracks variable scopes, detects undefined variables, and identifies shadowing

use crate::ast::node::{Expression, MatchCase, MatchPattern, RescueClause, Statement};
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::Position;
use std::collections::HashMap;

/// Convert a Position to SourceLocation
fn pos_to_loc(pos: Position) -> SourceLocation {
    SourceLocation::new(pos.line, pos.column, pos.offset)
}

/// Represents information about a variable declaration
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// The name of the variable
    pub name: String,

    /// The depth at which this variable was declared (0 = global)
    pub depth: usize,

    /// The position where this variable was declared
    pub position: Position,

    /// Whether this variable has been used
    pub used: bool,
}

/// Result of variable resolution
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Variables declared at each scope depth
    pub variables: HashMap<String, VariableInfo>,

    /// Errors encountered during resolution (undefined variables, shadowing, etc.)
    pub errors: Vec<MetorexError>,

    /// Warnings (unused variables, etc.)
    pub warnings: Vec<String>,
}

impl ResolutionResult {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Variable resolver - performs static analysis on AST
pub struct Resolver {
    /// Stack of scopes, each scope is a map of variable names to their info
    scopes: Vec<HashMap<String, VariableInfo>>,

    /// Current depth in the scope stack (0 = global)
    current_depth: usize,

    /// Accumulated errors
    errors: Vec<MetorexError>,

    /// Accumulated warnings
    warnings: Vec<String>,

    /// Whether to treat undefined variables as errors
    strict_mode: bool,
}

impl Resolver {
    /// Creates a new resolver
    pub fn new() -> Self {
        Resolver {
            scopes: vec![HashMap::new()], // Start with global scope
            current_depth: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            strict_mode: true,
        }
    }

    /// Creates a new resolver with strict mode setting
    pub fn with_strict_mode(strict_mode: bool) -> Self {
        let mut resolver = Self::new();
        resolver.strict_mode = strict_mode;
        resolver
    }

    /// Resolves variables in a list of statements
    pub fn resolve(&mut self, statements: &[Statement]) -> ResolutionResult {
        for statement in statements {
            self.resolve_statement(statement);
        }

        // Check for unused variables
        self.check_unused_variables();

        ResolutionResult {
            variables: self.collect_all_variables(),
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
        }
    }

    /// Collects all variables from all scopes
    fn collect_all_variables(&self) -> HashMap<String, VariableInfo> {
        let mut all_vars = HashMap::new();
        for scope in &self.scopes {
            for (name, info) in scope {
                all_vars.insert(name.clone(), info.clone());
            }
        }
        all_vars
    }

    /// Checks for unused variables and generates warnings
    fn check_unused_variables(&mut self) {
        for scope in &self.scopes {
            for (name, info) in scope {
                if !info.used && !name.starts_with('_') {
                    self.warnings.push(format!(
                        "Unused variable '{}' at {}:{}",
                        name, info.position.line, info.position.column
                    ));
                }
            }
        }
    }

    /// Enters a new scope
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_depth += 1;
    }

    /// Exits the current scope
    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
            self.current_depth -= 1;
        }
    }

    /// Declares a variable in the current scope
    fn declare(&mut self, name: String, position: Position) {
        // Check for shadowing in current scope first
        if let Some(existing) = self.scopes.last().unwrap().get(&name) {
            self.errors.push(MetorexError::syntax_error(
                format!(
                    "Variable '{}' is already declared in this scope at {}:{}",
                    name, existing.position.line, existing.position.column
                ),
                pos_to_loc(position),
            ));
            return;
        }

        // Check for shadowing from outer scopes
        let scope_count = self.scopes.len();
        if scope_count > 1 {
            for scope in self.scopes.iter().take(scope_count - 1).rev() {
                if let Some(existing) = scope.get(&name) {
                    self.warnings.push(format!(
                        "Variable '{}' at {}:{} shadows variable from outer scope at {}:{}",
                        name,
                        position.line,
                        position.column,
                        existing.position.line,
                        existing.position.column
                    ));
                    break;
                }
            }
        }

        // Now insert into current scope
        self.scopes.last_mut().unwrap().insert(
            name.clone(),
            VariableInfo {
                name,
                depth: self.current_depth,
                position,
                used: false,
            },
        );
    }

    /// Looks up a variable in the scope chain
    fn resolve_variable(&mut self, name: &str, position: Position) -> Option<usize> {
        // Search from innermost to outermost scope
        for (depth, scope) in self.scopes.iter_mut().enumerate().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                var_info.used = true;
                return Some(depth);
            }
        }

        // Variable not found
        if self.strict_mode {
            self.errors.push(MetorexError::syntax_error(
                format!("Undefined variable '{}'", name),
                pos_to_loc(position),
            ));
        }

        None
    }

    /// Resolves a statement
    fn resolve_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression { expression, .. } => {
                self.resolve_expression(expression);
            }

            Statement::Assignment { target, value, .. } => {
                // Resolve the value first
                self.resolve_expression(value);

                // Then handle the target - this declares or updates a variable
                match target {
                    Expression::Identifier { name, position } => {
                        // Check if variable exists in CURRENT scope only
                        let exists_in_current_scope =
                            self.scopes.last().unwrap().contains_key(name);

                        if exists_in_current_scope {
                            // Update existing variable in current scope - mark it as used
                            if let Some(var_info) = self.scopes.last_mut().unwrap().get_mut(name) {
                                var_info.used = true;
                            }
                        } else {
                            // Declare new variable in current scope
                            // This might shadow a variable from an outer scope
                            self.declare(name.clone(), *position);
                        }
                    }
                    Expression::InstanceVariable { .. } | Expression::ClassVariable { .. } => {
                        // Instance and class variables don't need resolution
                    }
                    Expression::Index { array, index, .. } => {
                        // Resolve array and index expressions
                        self.resolve_expression(array);
                        self.resolve_expression(index);
                    }
                    _ => {
                        // Resolve target as expression
                        self.resolve_expression(target);
                    }
                }
            }

            Statement::FunctionDef {
                name,
                parameters,
                body,
                position,
            } => {
                // Declare function name in current scope
                self.declare(name.clone(), *position);

                // Enter function scope
                self.push_scope();

                // Declare parameters
                for param in parameters {
                    self.declare(param.name.clone(), param.position);
                    if let Some(default) = &param.default_value {
                        self.resolve_expression(default);
                    }
                }

                // Resolve function body
                for stmt in body {
                    self.resolve_statement(stmt);
                }

                // Exit function scope
                self.pop_scope();
            }

            Statement::MethodDef {
                parameters, body, ..
            } => {
                // Methods are similar to functions but don't declare a name in outer scope
                self.push_scope();

                // Declare parameters
                for param in parameters {
                    self.declare(param.name.clone(), param.position);
                    if let Some(default) = &param.default_value {
                        self.resolve_expression(default);
                    }
                }

                // Resolve method body
                for stmt in body {
                    self.resolve_statement(stmt);
                }

                self.pop_scope();
            }

            Statement::ClassDef { name, body, .. } => {
                // Class definitions create their own scope
                self.push_scope();

                // Resolve class body
                for stmt in body {
                    self.resolve_statement(stmt);
                }

                self.pop_scope();

                // Declare class name after resolving body
                self.declare(name.clone(), Position::default());
            }

            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                ..
            } => {
                self.resolve_expression(condition);

                self.push_scope();
                for stmt in then_branch {
                    self.resolve_statement(stmt);
                }
                self.pop_scope();

                for elsif in elsif_branches {
                    self.resolve_expression(&elsif.condition);
                    self.push_scope();
                    for stmt in &elsif.body {
                        self.resolve_statement(stmt);
                    }
                    self.pop_scope();
                }

                if let Some(else_body) = else_branch {
                    self.push_scope();
                    for stmt in else_body {
                        self.resolve_statement(stmt);
                    }
                    self.pop_scope();
                }
            }

            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_expression(condition);

                self.push_scope();
                for stmt in then_branch {
                    self.resolve_statement(stmt);
                }
                self.pop_scope();

                if let Some(else_body) = else_branch {
                    self.push_scope();
                    for stmt in else_body {
                        self.resolve_statement(stmt);
                    }
                    self.pop_scope();
                }
            }

            Statement::While {
                condition, body, ..
            } => {
                self.resolve_expression(condition);
                self.push_scope();
                for stmt in body {
                    self.resolve_statement(stmt);
                }
                self.pop_scope();
            }

            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => {
                self.resolve_expression(iterable);
                self.push_scope();
                self.declare(variable.clone(), *position);
                for stmt in body {
                    self.resolve_statement(stmt);
                }
                self.pop_scope();
            }

            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.resolve_expression(expr);
                }
            }

            Statement::Break { .. } | Statement::Continue { .. } => {
                // Nothing to resolve
            }

            Statement::Match {
                expression, cases, ..
            } => {
                self.resolve_expression(expression);

                for case in cases {
                    self.resolve_match_case(case);
                }
            }

            Statement::Begin {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                ..
            } => {
                self.push_scope();
                for stmt in body {
                    self.resolve_statement(stmt);
                }
                self.pop_scope();

                for rescue in rescue_clauses {
                    self.resolve_rescue_clause(rescue);
                }

                if let Some(else_body) = else_clause {
                    self.push_scope();
                    for stmt in else_body {
                        self.resolve_statement(stmt);
                    }
                    self.pop_scope();
                }

                if let Some(ensure_body) = ensure_block {
                    self.push_scope();
                    for stmt in ensure_body {
                        self.resolve_statement(stmt);
                    }
                    self.pop_scope();
                }
            }

            Statement::Raise { exception, .. } => {
                if let Some(expr) = exception {
                    self.resolve_expression(expr);
                }
            }

            Statement::Block { statements, .. } => {
                self.push_scope();
                for stmt in statements {
                    self.resolve_statement(stmt);
                }
                self.pop_scope();
            }

            Statement::AttrReader { .. }
            | Statement::AttrWriter { .. }
            | Statement::AttrAccessor { .. } => {
                // These are class-level declarations, no variable resolution needed
            }
        }
    }

    /// Resolves a match case
    fn resolve_match_case(&mut self, case: &MatchCase) {
        self.push_scope();

        // Declare variables from pattern
        self.resolve_pattern(&case.pattern);

        // Resolve guard condition
        if let Some(guard) = &case.guard {
            self.resolve_expression(guard);
        }

        // Resolve case body
        for stmt in &case.body {
            self.resolve_statement(stmt);
        }

        self.pop_scope();
    }

    /// Resolves a rescue clause
    fn resolve_rescue_clause(&mut self, rescue: &RescueClause) {
        self.push_scope();

        // Declare exception variable if present
        if let Some(var_name) = &rescue.variable_name {
            self.declare(var_name.clone(), rescue.position);
        }

        // Resolve rescue body
        for stmt in &rescue.body {
            self.resolve_statement(stmt);
        }

        self.pop_scope();
    }

    /// Resolves variables declared in a pattern
    fn resolve_pattern(&mut self, pattern: &MatchPattern) {
        match pattern {
            MatchPattern::Identifier(name) => {
                self.declare(name.clone(), Position::default());
            }
            MatchPattern::Array(patterns) => {
                for p in patterns {
                    self.resolve_pattern(p);
                }
            }
            MatchPattern::Rest(name) => {
                self.declare(name.clone(), Position::default());
            }
            MatchPattern::Object(fields) => {
                for (_, pattern) in fields {
                    self.resolve_pattern(pattern);
                }
            }
            _ => {
                // Literals and wildcards don't declare variables
            }
        }
    }

    /// Resolves an expression
    fn resolve_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Identifier { name, position } => {
                self.resolve_variable(name, *position);
            }

            Expression::InstanceVariable { .. } | Expression::ClassVariable { .. } => {
                // Instance and class variables don't need resolution
            }

            Expression::BinaryOp { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }

            Expression::UnaryOp { operand, .. } => {
                self.resolve_expression(operand);
            }

            Expression::Call {
                callee, arguments, ..
            } => {
                self.resolve_expression(callee);
                for arg in arguments {
                    self.resolve_expression(arg);
                }
            }

            Expression::MethodCall {
                receiver,
                arguments,
                ..
            } => {
                self.resolve_expression(receiver);
                for arg in arguments {
                    self.resolve_expression(arg);
                }
            }

            Expression::Array { elements, .. } => {
                for element in elements {
                    self.resolve_expression(element);
                }
            }

            Expression::Index { array, index, .. } => {
                self.resolve_expression(array);
                self.resolve_expression(index);
            }

            Expression::Dictionary { entries, .. } => {
                for (key, value) in entries {
                    self.resolve_expression(key);
                    self.resolve_expression(value);
                }
            }

            Expression::Lambda {
                parameters, body, ..
            } => {
                self.push_scope();

                // Declare parameters
                for param in parameters {
                    self.declare(param.clone(), Position::default());
                }

                // Resolve lambda body
                for stmt in body {
                    self.resolve_statement(stmt);
                }

                self.pop_scope();
            }

            Expression::Grouped { expression, .. } => {
                self.resolve_expression(expression);
            }

            Expression::Range { start, end, .. } => {
                self.resolve_expression(start);
                self.resolve_expression(end);
            }

            Expression::InterpolatedString { parts, .. } => {
                for part in parts {
                    if let crate::ast::node::InterpolationPart::Expression(expr) = part {
                        self.resolve_expression(expr);
                    }
                }
            }

            // Literals don't need resolution
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::Symbol { .. }
            | Expression::BoolLiteral { .. }
            | Expression::NilLiteral { .. }
            | Expression::SelfExpr { .. }
            | Expression::Super { .. } => {}
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
