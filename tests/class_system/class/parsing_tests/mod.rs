// Unit tests for the AST a class definition parses into.

pub(crate) use metorex::ast::{Expression, Parameter, Statement};
pub(crate) use metorex::lexer::Position;

pub(crate) fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

mod bodies;
mod definitions;
