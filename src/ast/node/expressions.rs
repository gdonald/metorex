// The expressions a program is written with.

use super::*;

/// Binary operators in Metorex
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic operators
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Power,    // **

    // Comparison operators
    Equal,        // ==
    CaseEqual,    // ===
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Spaceship,    // <=>

    // Bitwise operators
    BitwiseAnd, // &
    BitwiseOr,  // |
    Xor,        // ^

    // Logical operators
    And, // &&
    Or,  // ||

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
    Not,   // !
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
    RegexLiteral {
        pattern: String,
        flags: String,
        position: Position,
    },

    /// `value => pattern` and `value in pattern`, written on their own rather
    /// than inside a `case`. The first refuses a value the pattern does not
    /// cover, and the second answers whether it does.
    PatternTest {
        value: Box<Expression>,
        pattern: Box<MatchPattern>,
        /// True for `=>`, which raises where the pattern does not match.
        refuses: bool,
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
    Symbol {
        value: String,
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
    GlobalVariable {
        name: String,
        position: Position,
    },
    MagicFile {
        position: Position,
    },
    MagicLine {
        position: Position,
    },
    MagicDir {
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
        /// Optional trailing block (e.g., `foo(x) do |y| ... end`)
        trailing_block: Option<Box<Expression>>,
        position: Position,
    },

    // Method calls with dot notation
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
        /// Optional trailing block (e.g., `foo.bar(x) do |y| ... end`)
        trailing_block: Option<Box<Expression>>,
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
        /// Default values for optional parameters, keyed by index into
        /// `parameters` (e.g. `{ |a, b = 1| }` records `(1, <1>)`).
        parameter_defaults: Vec<(usize, Expression)>,
        body: Vec<Statement>,
        captured_vars: Option<Vec<String>>, // Variables captured from outer scope
        /// True for `-> {}` and `lambda {}`, false for `proc {}` and every
        /// ordinary block. Lambdas check arity strictly and return from
        /// themselves; procs do neither.
        is_lambda: bool,
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

    // `class << <target>; body; end` — enter the singleton class of `target`.
    // Evaluates to the value of the final body statement (or nil for an empty body).
    SingletonClass {
        target: Box<Expression>,
        /// Where to store `target` before opening its singleton class, for
        /// `class << @receiver = Object.new`.
        assign_to: Option<Box<Expression>>,
        body: Vec<Statement>,
        position: Position,
    },

    // Super call - calls parent class method
    Super {
        arguments: Vec<Expression>,
        /// True iff this was written as bare `super` (no parens, no args) —
        /// Ruby forwards the enclosing method's arguments in that case.
        /// `super()` has this false (explicit zero args).
        forward_args: bool,
        /// A `do`/`{` block written on the super call, which the parent
        /// method receives in place of the one the caller supplied.
        trailing_block: Option<Box<Expression>>,
        position: Position,
    },

    // An integer literal too large for an i64, kept as its base-ten digits.
    BigIntLiteral {
        digits: String,
        position: Position,
    },

    // Splat expression (*expr) — expands an array into individual arguments
    Splat {
        expression: Box<Expression>,
        position: Position,
    },

    // Keyword-splat expression (**expr) — passes a Hash as keyword arguments.
    // An empty Hash contributes no argument at all, the way Ruby's does.
    KeywordSplat {
        expression: Box<Expression>,
        position: Position,
    },

    // `::Name` — a constant taken from the top level, skipping the lexical
    // chain the way Ruby's leading `::` does.
    TopLevelConstant {
        name: String,
        position: Position,
    },

    // Block-arg expression (&expr) — converts the value to a block and binds
    // it as the receiver method's `pending_block`. If the value is nil the
    // call is treated as if no block were given.
    BlockArg {
        expression: Box<Expression>,
        position: Position,
    },

    // begin/rescue/else/ensure used as an expression — yields the value of
    // the last successfully-executed statement (in body, rescue, or else).
    BeginRescue {
        body: Vec<Statement>,
        rescue_clauses: Vec<RescueClause>,
        else_clause: Option<Vec<Statement>>,
        ensure_block: Option<Vec<Statement>>,
        position: Position,
    },

    // defined?(expr) — returns a description string or nil
    Defined {
        expression: Box<Expression>,
        position: Position,
    },

    // Yield - invoke the block passed to the current method
    Yield {
        arguments: Vec<Expression>,
        position: Position,
    },

    // Range literals
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        exclusive: bool, // true for ..., false for ..
        position: Position,
    },

    // Case expression (pattern matching in expression context)
    // Unlike Statement::Match which is used for statement context,
    // Expression::Case can be used anywhere an expression is expected
    // (e.g., assignments, method arguments, return values)
    Case {
        expression: Box<Expression>,        // Value to match against
        cases: Vec<ExprMatchCase>,          // When branches
        else_case: Option<Box<Expression>>, // Optional else branch
        position: Position,
    },

    // If expression (if...end in expression context)
    // Evaluates to the last expression in the matching branch, or nil.
    If {
        condition: Box<Expression>,
        then_branch: Vec<Statement>,
        elsif_branches: Vec<ElsifBranch>,
        else_branch: Option<Vec<Statement>>,
        position: Position,
    },

    // Unless expression (unless...end in expression context)
    // Evaluates to the last expression in the matching branch, or nil.
    Unless {
        condition: Box<Expression>,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
        position: Position,
    },

    // Scope resolution (e.g., Math::PI, Foo::Bar)
    ScopeResolution {
        namespace: Box<Expression>,
        name: String,
        position: Position,
    },
}

/// Parts of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Text(String),
    Expression(Box<Expression>),
}
