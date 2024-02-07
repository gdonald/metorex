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

    // Method definition
    MethodDef {
        name: String,
        parameters: Vec<String>,
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
            | Statement::MethodDef { position, .. }
            | Statement::ClassDef { position, .. }
            | Statement::If { position, .. }
            | Statement::While { position, .. }
            | Statement::Return { position, .. }
            | Statement::Break { position, .. }
            | Statement::Continue { position, .. }
            | Statement::Block { position, .. } => *position,
        }
    }

    /// Check if this statement is a definition (method or class)
    pub fn is_definition(&self) -> bool {
        matches!(
            self,
            Statement::MethodDef { .. } | Statement::ClassDef { .. }
        )
    }

    /// Check if this statement is a control flow statement
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Statement::If { .. }
                | Statement::While { .. }
                | Statement::Return { .. }
                | Statement::Break { .. }
                | Statement::Continue { .. }
        )
    }
}
