// The patterns a `case/in` clause is written with.

use super::*;

/// Pattern for match statement cases
#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    // Literal patterns
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    SymbolLiteral(String),
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

    // Multiple patterns (OR matching) - matches if any pattern matches
    // Used for: when 1, 2, 3 then "small"
    Multiple(Vec<MatchPattern>),

    // Bind pattern: matches inner pattern and binds the whole value to a name
    // Used in case/in: `in Integer => n`
    Bind {
        pattern: Box<MatchPattern>,
        name: String,
    },

    // Anything else a `when` may be written with, such as a pattern literal
    // or a call, which the case compares against with `===`.
    Expression(Box<Expression>),

    // Range pattern: matches if value falls within start..end or start...end
    // Used in case/when: `when 1..10 then`
    Range {
        start: Box<MatchPattern>,
        end: Box<MatchPattern>,
        exclusive: bool, // true for ..., false for ..
    },

    /// `^name`, `^@name`, `^$name`, or `^(expr)`: the value the expression
    /// answers is compared against, rather than a name being bound.
    Pinned(Box<Expression>),

    /// An array pattern written for `in`: the names before a `*rest`, the
    /// rest itself where one is written, and the names after it. A constant
    /// written in front narrows what the value may be first.
    ArrayPattern {
        constant: Option<Box<Expression>>,
        prefix: Vec<MatchPattern>,
        /// `Some(None)` for a bare `*`, `Some(Some(name))` for `*name`, and
        /// None where the pattern names every element.
        rest: Option<Option<String>>,
        suffix: Vec<MatchPattern>,
    },

    /// `[*, a, b, *]`: a run to find anywhere in the value, with the parts
    /// before and after it bound where they are named.
    FindPattern {
        constant: Option<Box<Expression>>,
        before: Option<String>,
        middle: Vec<MatchPattern>,
        after: Option<String>,
    },

    /// A hash pattern written for `in`: each key with the pattern its value
    /// has to match, or None where the key binds a local of its own name.
    HashPattern {
        constant: Option<Box<Expression>>,
        entries: Vec<(String, Option<MatchPattern>)>,
        rest: HashPatternRest,
    },
}

/// What a hash pattern says about the keys it did not name.
#[derive(Debug, Clone, PartialEq)]
pub enum HashPatternRest {
    /// The pattern says nothing, so other keys are allowed and dropped.
    Silent,
    /// `**name` binds the keys the pattern did not name.
    Named(String),
    /// `**` allows other keys without naming them.
    Anonymous,
    /// `**nil` says the value may hold no other key.
    Refused,
}

/// A single case in a match statement
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: MatchPattern,
    pub guard: Option<Expression>, // Optional guard condition (if ...)
    pub body: Vec<Statement>,
    pub position: Position,
}

/// A single case in a case expression (expression context)
/// Unlike MatchCase which uses Vec<Statement> for body,
/// ExprMatchCase uses a single Expression for the body.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprMatchCase {
    pub pattern: MatchPattern,
    pub guard: Option<Expression>, // Optional guard condition (if ...)
    pub body: Expression,          // Single expression (NOT Vec<Statement>)
    pub position: Position,
}

/// A rescue clause in a begin/rescue/ensure block
#[derive(Debug, Clone, PartialEq)]
pub struct RescueClause {
    pub exception_types: Vec<String>, // Exception types to catch (empty means catch all)
    pub variable_name: Option<String>, // Variable to bind the exception to (e.g., "=> e")
    /// Where the exception is stored when it is not a plain local or global:
    /// `rescue E => held.error` and `rescue E => held[:error]` both name one.
    pub variable_target: Option<Expression>,
    /// Classes named by a splat rather than written out, as
    /// `rescue *handled` does. They are read where the clause is reached.
    pub splatted_types: Vec<Expression>,
    pub body: Vec<Statement>,
    pub position: Position,
}

/// An elsif branch in an if statement
#[derive(Debug, Clone, PartialEq)]
pub struct ElsifBranch {
    pub condition: Expression,
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
    pub is_named_keyword: bool,            // True if this is a name: style keyword parameter
    pub is_block: bool,                    // True if this is a &block parameter
    pub position: Position,
}
