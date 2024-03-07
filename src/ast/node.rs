// AST node definitions for Metorex

use crate::lexer::Position;
use std::fmt;

/// Binary operators in Metorex
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic operators
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %

    // Comparison operators
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    // Assignment operators
    Assign,         // =
    AddAssign,      // +=
    SubtractAssign, // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
}

/// Unary operators in Metorex
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Plus,  // +
    Minus, // -
}

/// Expressions in Metorex - values that can be evaluated
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    IntLiteral {
        value: i64,
        position: Position,
    },
    FloatLiteral {
        value: f64,
        position: Position,
    },
    StringLiteral {
        value: String,
        position: Position,
    },
    InterpolatedString {
        parts: Vec<InterpolationPart>,
        position: Position,
    },
    BoolLiteral {
        value: bool,
        position: Position,
    },
    NilLiteral {
        position: Position,
    },

    // Identifiers and variables
    Identifier {
        name: String,
        position: Position,
    },
    InstanceVariable {
        name: String,
        position: Position,
    },
    ClassVariable {
        name: String,
        position: Position,
    },

    // Binary operations
    BinaryOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
        position: Position,
    },

    // Unary operations
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expression>,
        position: Position,
    },

    // Function/method calls
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
        position: Position,
    },

    // Method calls with dot notation
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
        position: Position,
    },

    // Array literals
    Array {
        elements: Vec<Expression>,
        position: Position,
    },

    // Array indexing
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
        position: Position,
    },

    // Dictionary/hash literals
    Dictionary {
        entries: Vec<(Expression, Expression)>,
        position: Position,
    },

    // Lambda/block expressions
    Lambda {
        parameters: Vec<String>,
        body: Vec<Statement>,
        captured_vars: Option<Vec<String>>, // Variables captured from outer scope
        position: Position,
    },

    // Parenthesized expressions
    Grouped {
        expression: Box<Expression>,
        position: Position,
    },

    // Self reference (implicit receiver)
    SelfExpr {
        position: Position,
    },
}

/// Parts of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Text(String),
    Expression(Box<Expression>),
}

/// Pattern for match statement cases
#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    // Literal patterns
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    NilLiteral,

    // Variable binding pattern
    Identifier(String),

    // Wildcard pattern (matches anything)
    Wildcard,

    // Array pattern with optional rest
    Array(Vec<MatchPattern>),

    // Array rest pattern (e.g., [first, ...rest])
    Rest(String), // Variable name to bind remaining elements

    // Object/Dictionary pattern for destructuring
    Object(Vec<(String, MatchPattern)>), // key-pattern pairs

    // Type pattern (for future use)
    Type(String),
}

/// A single case in a match statement
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: MatchPattern,
    pub guard: Option<Expression>, // Optional guard condition (if ...)
    pub body: Vec<Statement>,
    pub position: Position,
}

/// A rescue clause in a begin/rescue/ensure block
#[derive(Debug, Clone, PartialEq)]
pub struct RescueClause {
    pub exception_types: Vec<String>, // Exception types to catch (empty means catch all)
    pub variable_name: Option<String>, // Variable to bind the exception to (e.g., "=> e")
    pub body: Vec<Statement>,
    pub position: Position,
}

/// Function parameter definition
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub default_value: Option<Expression>, // Default value for the parameter
    pub is_variadic: bool,                 // True if this is a *args parameter
    pub is_keyword: bool,                  // True if this is a **kwargs parameter
    pub position: Position,
}

impl Parameter {
    /// Create a new simple parameter (no default, not variadic/keyword)
    pub fn simple(name: String, position: Position) -> Self {
        Parameter {
            name,
            default_value: None,
            is_variadic: false,
            is_keyword: false,
            position,
        }
    }

    /// Create a new parameter with a default value
    pub fn with_default(name: String, default_value: Expression, position: Position) -> Self {
        Parameter {
            name,
            default_value: Some(default_value),
            is_variadic: false,
            is_keyword: false,
            position,
        }
    }

    /// Create a new variadic parameter (*args)
    pub fn variadic(name: String, position: Position) -> Self {
        Parameter {
            name,
            default_value: None,
            is_variadic: true,
            is_keyword: false,
            position,
        }
    }

    /// Create a new keyword parameter (**kwargs)
    pub fn keyword(name: String, position: Position) -> Self {
        Parameter {
            name,
            default_value: None,
            is_variadic: false,
            is_keyword: true,
            position,
        }
    }

    /// Check if this is a simple parameter (no default, not variadic/keyword)
    pub fn is_simple(&self) -> bool {
        self.default_value.is_none() && !self.is_variadic && !self.is_keyword
    }

    /// Check if this parameter has a default value
    pub fn has_default(&self) -> bool {
        self.default_value.is_some()
    }
}

/// Statements in Metorex - instructions that can be executed
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // Expression statement (an expression used as a statement)
    Expression {
        expression: Expression,
        position: Position,
    },

    // Variable assignment
    Assignment {
        target: Expression,
        value: Expression,
        position: Position,
    },

    // Function definition (standalone function)
    FunctionDef {
        name: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
        position: Position,
    },

    // Method definition (function within a class)
    MethodDef {
        name: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
        position: Position,
    },

    // Class definition
    ClassDef {
        name: String,
        superclass: Option<String>,
        body: Vec<Statement>,
        position: Position,
    },

    // Conditional statements
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
        position: Position,
    },

    // While loop
    While {
        condition: Expression,
        body: Vec<Statement>,
        position: Position,
    },

    // For loop (iteration over collections)
    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
        position: Position,
    },

    // Match statement (pattern matching)
    Match {
        expression: Expression,
        cases: Vec<MatchCase>,
        position: Position,
    },

    // Return statement
    Return {
        value: Option<Expression>,
        position: Position,
    },

    // Break statement (exit from loop)
    Break {
        position: Position,
    },

    // Continue statement (skip to next iteration)
    Continue {
        position: Position,
    },

    // Block statement
    Block {
        statements: Vec<Statement>,
        position: Position,
    },

    // Exception handling: begin/rescue/else/ensure/end
    Begin {
        body: Vec<Statement>,
        rescue_clauses: Vec<RescueClause>,
        else_clause: Option<Vec<Statement>>, // Runs if no exception occurred
        ensure_block: Option<Vec<Statement>>, // Always runs (like finally)
        position: Position,
    },

    // Raise statement (throw exception)
    Raise {
        exception: Option<Expression>, // None means re-raise current exception
        position: Position,
    },
}

// Implement Display for BinaryOp
impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::LessEqual => write!(f, "<="),
            BinaryOp::GreaterEqual => write!(f, ">="),
            BinaryOp::Assign => write!(f, "="),
            BinaryOp::AddAssign => write!(f, "+="),
            BinaryOp::SubtractAssign => write!(f, "-="),
            BinaryOp::MultiplyAssign => write!(f, "*="),
            BinaryOp::DivideAssign => write!(f, "/="),
        }
    }
}

// Implement Display for UnaryOp
impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Minus => write!(f, "-"),
        }
    }
}

// Implement helper methods for Expression
impl Expression {
    /// Get the position of this expression
    pub fn position(&self) -> Position {
        match self {
            Expression::IntLiteral { position, .. }
            | Expression::FloatLiteral { position, .. }
            | Expression::StringLiteral { position, .. }
            | Expression::InterpolatedString { position, .. }
            | Expression::BoolLiteral { position, .. }
            | Expression::NilLiteral { position, .. }
            | Expression::Identifier { position, .. }
            | Expression::InstanceVariable { position, .. }
            | Expression::ClassVariable { position, .. }
            | Expression::BinaryOp { position, .. }
            | Expression::UnaryOp { position, .. }
            | Expression::Call { position, .. }
            | Expression::MethodCall { position, .. }
            | Expression::Array { position, .. }
            | Expression::Index { position, .. }
            | Expression::Dictionary { position, .. }
            | Expression::Lambda { position, .. }
            | Expression::Grouped { position, .. }
            | Expression::SelfExpr { position, .. } => *position,
        }
    }

    /// Check if this expression is a literal
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expression::IntLiteral { .. }
                | Expression::FloatLiteral { .. }
                | Expression::StringLiteral { .. }
                | Expression::InterpolatedString { .. }
                | Expression::BoolLiteral { .. }
                | Expression::NilLiteral { .. }
        )
    }

    /// Check if this expression is an identifier or variable
    pub fn is_identifier(&self) -> bool {
        matches!(
            self,
            Expression::Identifier { .. }
                | Expression::InstanceVariable { .. }
                | Expression::ClassVariable { .. }
        )
    }
}

// Implement helper methods for Statement
impl Statement {
    /// Get the position of this statement
    pub fn position(&self) -> Position {
        match self {
            Statement::Expression { position, .. }
            | Statement::Assignment { position, .. }
            | Statement::FunctionDef { position, .. }
            | Statement::MethodDef { position, .. }
            | Statement::ClassDef { position, .. }
            | Statement::If { position, .. }
            | Statement::While { position, .. }
            | Statement::For { position, .. }
            | Statement::Match { position, .. }
            | Statement::Return { position, .. }
            | Statement::Break { position, .. }
            | Statement::Continue { position, .. }
            | Statement::Block { position, .. }
            | Statement::Begin { position, .. }
            | Statement::Raise { position, .. } => *position,
        }
    }

    /// Check if this statement is a definition (function, method, or class)
    pub fn is_definition(&self) -> bool {
        matches!(
            self,
            Statement::FunctionDef { .. }
                | Statement::MethodDef { .. }
                | Statement::ClassDef { .. }
        )
    }

    /// Check if this statement is a control flow statement
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Statement::If { .. }
                | Statement::While { .. }
                | Statement::For { .. }
                | Statement::Match { .. }
                | Statement::Return { .. }
                | Statement::Break { .. }
                | Statement::Continue { .. }
                | Statement::Begin { .. }
                | Statement::Raise { .. }
        )
    }
}
