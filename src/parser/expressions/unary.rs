// Unary operator parsing
// Handles parsing of unary operations (+ and -)

use crate::ast::{Expression, UnaryOp};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse unary operators (+, -)
    pub(crate) fn parse_unary(&mut self) -> Result<Expression, MetorexError> {
        if self.check(&[TokenKind::Plus, TokenKind::Minus]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Plus => UnaryOp::Plus,
                TokenKind::Minus => UnaryOp::Minus,
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            Ok(Expression::UnaryOp {
                op,
                operand: Box::new(operand),
                position: op_token.position,
            })
        } else {
            self.parse_call()
        }
    }
}
