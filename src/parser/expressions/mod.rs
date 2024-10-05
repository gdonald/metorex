// Expression parsing module
// Handles parsing of all expression types

mod binary;
mod call;
mod primary;
mod unary;

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::parser::Parser;

impl Parser {
    /// Parse an expression using operator precedence climbing
    pub(crate) fn parse_expression(&mut self) -> Result<Expression, MetorexError> {
        self.parse_assignment()
    }

    /// Parse assignment (lowest precedence)
    pub(crate) fn parse_assignment(&mut self) -> Result<Expression, MetorexError> {
        self.parse_equality()
    }
}
