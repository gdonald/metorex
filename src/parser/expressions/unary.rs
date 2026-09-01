// Unary operator parsing
// Handles parsing of unary operations (+ and -)

use crate::ast::{BinaryOp, Expression, UnaryOp};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse unary operators (+, -, !, *splat)
    pub(crate) fn parse_unary(&mut self) -> Result<Expression, MetorexError> {
        if self.check(&[TokenKind::Tilde]) {
            // `~x` is a method call, the way Ruby defines it on Integer.
            let op_token = self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::MethodCall {
                receiver: Box::new(operand),
                method: "~".to_string(),
                arguments: Vec::new(),
                trailing_block: None,
                position: op_token.position,
            });
        }
        if self.check(&[TokenKind::Plus, TokenKind::Minus, TokenKind::Bang]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Plus => UnaryOp::Plus,
                TokenKind::Minus => UnaryOp::Minus,
                TokenKind::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };
            // `!` nests (`!!x`), but `-` / `+` go straight to the power level so
            // that `-2**2 == -(2**2)` per Ruby semantics.
            let operand = if matches!(op, UnaryOp::Not) {
                self.parse_unary()?
            } else {
                self.parse_power()?
            };
            Ok(Expression::UnaryOp {
                op,
                operand: Box::new(operand),
                position: op_token.position,
            })
        } else if self.check(&[TokenKind::Star]) {
            // Splat in expression position: `compare = *args`. Wraps the
            // operand in a Splat expression that the VM evaluates to an
            // Array (the operand itself if it's already an Array).
            let star_token = self.advance();
            let operand = self.parse_unary()?;
            Ok(Expression::Splat {
                expression: Box::new(operand),
                position: star_token.position,
            })
        } else {
            self.parse_power()
        }
    }

    /// Parse power operator (right-associative), tighter than unary minus.
    pub(crate) fn parse_power(&mut self) -> Result<Expression, MetorexError> {
        let left = self.parse_call()?;
        if self.check(&[TokenKind::StarStar]) {
            let op_token = self.advance();
            // Right-associative: recurse into parse_unary so `2**-3` works
            // and `2**3**2` groups as `2**(3**2)`.
            let right = self.parse_unary()?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Power,
                left: Box::new(left),
                right: Box::new(right),
                position: op_token.position,
            })
        } else {
            Ok(left)
        }
    }
}
