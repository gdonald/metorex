// The statements a program is made of.

use super::*;

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

    // `begin ... end while cond`, which runs its body before it first reads
    // the condition.
    DoWhile {
        condition: Expression,
        body: Vec<Statement>,
        position: Position,
    },

    // Names a `for` loop binds in the scope holding the loop, which is where
    // Ruby puts them. A name already bound keeps the value it had.
    DeclareLocals {
        names: Vec<String>,
        position: Position,
    },

    /// `BEGIN { ... }`, whose body runs before the rest of the code unit it
    /// was written in and shares that unit's own scope.
    BeginBlock {
        body: Vec<Statement>,
        position: Position,
    },

    // Multiple assignment (a, b, c = expr)
    MultipleAssignment {
        targets: Vec<Expression>,
        values: Vec<Expression>,
        position: Position,
    },

    // Function definition (standalone function)
    FunctionDef {
        name: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
        position: Position,
        /// For `def (expr).method_name`, the singleton receiver class name
        singleton_class: Option<String>,
    },

    // Method definition (function within a class)
    MethodDef {
        name: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
        is_class_method: bool,
        position: Position,
        /// Where the `end` closing the definition sits, which is what
        /// `Coverage`'s methods mode reports as the definition's extent.
        end_position: Position,
    },

    // Class definition
    ClassDef {
        name: String,
        /// Optional namespace for `class Foo::Bar < X` forms — a dynamic
        /// expression evaluated to the enclosing module/class where the
        /// constant should be installed. When `None`, the class is installed
        /// on the current lexical scope (existing behavior).
        namespace: Option<Box<Expression>>,
        superclass: Option<String>,
        /// A superclass written as an expression rather than a constant path,
        /// which is what `class C < Struct.new(:a)` names. When set, it is
        /// evaluated and the class it answers becomes the parent.
        superclass_expression: Option<Box<Expression>>,
        body: Vec<Statement>,
        position: Position,
    },

    ModuleDef {
        name: String,
        /// Optional namespace for `module Foo::Bar` forms — same shape as
        /// `ClassDef::namespace`. None means install in current lexical scope.
        namespace: Option<Box<Expression>>,
        body: Vec<Statement>,
        position: Position,
    },

    Include {
        module_name: String,
        position: Position,
    },

    Extend {
        module_name: String,
        position: Position,
    },

    Alias {
        new_name: String,
        old_name: String,
        position: Position,
    },

    // Conditional statements
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        elsif_branches: Vec<ElsifBranch>,
        else_branch: Option<Vec<Statement>>,
        position: Position,
    },

    // Unless statement (inverted if)
    Unless {
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

    // Match statement (case/when pattern matching)
    Match {
        expression: Expression,
        cases: Vec<MatchCase>,
        position: Position,
    },

    // CaseIn statement (Ruby 2.7+ case/in pattern matching)
    // Raises NoMatchingPatternError if no arm matches and no else is present.
    CaseIn {
        expression: Expression,
        cases: Vec<MatchCase>,
        position: Position,
    },

    // Return statement
    Return {
        value: Option<Expression>,
        position: Position,
    },

    // Break statement (exit from loop). `value` carries the optional
    // expression that follows `break` (e.g. `break 42`) — None when bare.
    Break {
        value: Option<Box<Expression>>,
        position: Position,
    },

    // Continue statement (skip to next iteration). `value` carries the
    // optional expression that follows `next` (e.g. `next 42`).
    Continue {
        value: Option<Box<Expression>>,
        position: Position,
    },

    // Redo statement (re-run the enclosing block/method body from the top)
    Redo {
        position: Position,
    },

    /// `retry` inside a rescue body, which runs the begin body again.
    Retry {
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

    // attr_reader - creates getter methods for instance variables
    AttrReader {
        attributes: Vec<Expression>, // Each expression evaluates to a Symbol or to_str-able value
        position: Position,
    },

    // attr_writer - creates setter methods for instance variables
    AttrWriter {
        attributes: Vec<Expression>, // Each expression evaluates to a Symbol or to_str-able value
        position: Position,
    },

    // attr_accessor - creates both getter and setter methods for instance variables
    AttrAccessor {
        attributes: Vec<Expression>, // Each expression evaluates to a Symbol or to_str-able value
        position: Position,
    },
}
