// Binary operator parsing
// Handles parsing of binary operations with proper precedence

use crate::ast::{BinaryOp, Expression};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse logical OR (||)
    pub(crate) fn parse_logical_or(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_logical_and()?;

        while self.check(&[TokenKind::LogicalOr]) {
            let op_token = self.advance();
            self.skip_whitespace();
            let right = self.parse_logical_and()?;
            expr = Expression::BinaryOp {
                op: BinaryOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse logical AND (&&)
    pub(crate) fn parse_logical_and(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_equality()?;

        while self.check(&[TokenKind::LogicalAnd]) {
            let op_token = self.advance();
            self.skip_whitespace();
            let right = self.parse_equality()?;
            expr = Expression::BinaryOp {
                op: BinaryOp::And,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse equality operators (==, !=)
    pub(crate) fn parse_equality(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_comparison()?;

        while self.check(&[
            TokenKind::EqualEqual,
            TokenKind::BangEqual,
            TokenKind::TripleEqual,
            TokenKind::Match,
            TokenKind::NotMatch,
        ]) {
            let op_token = self.advance();
            if matches!(op_token.kind, TokenKind::Match | TokenKind::NotMatch) {
                // =~ and !~ are dispatched as method calls
                self.skip_whitespace();
                let right = self.parse_comparison()?;
                let method_call = Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: if op_token.kind == TokenKind::Match {
                        "=~".to_string()
                    } else {
                        "!~".to_string()
                    },
                    arguments: vec![right],
                    trailing_block: None,
                    position: op_token.position,
                };
                expr = method_call;
                continue;
            }
            let op = match op_token.kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::TripleEqual => BinaryOp::CaseEqual,
                TokenKind::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            self.skip_whitespace();
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse comparison operators (<, >, <=, >=)
    pub(crate) fn parse_comparison(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_range()?;

        while self.check(&[
            TokenKind::Less,
            TokenKind::Greater,
            TokenKind::LessEqual,
            TokenKind::GreaterEqual,
            TokenKind::Spaceship,
            TokenKind::Shovel,
            TokenKind::Caret,
            TokenKind::Pipe,
            TokenKind::Ampersand,
        ]) {
            let op_token = self.advance();
            if op_token.kind == TokenKind::Shovel {
                // << is a method call (e.g., array.<<(value))
                self.skip_whitespace();
                let right = self.parse_range()?;
                expr = Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: "<<".to_string(),
                    arguments: vec![right],
                    trailing_block: None,
                    position: op_token.position,
                };
                continue;
            }
            let op = match op_token.kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                TokenKind::Spaceship => BinaryOp::Spaceship,
                TokenKind::Caret => BinaryOp::Xor,
                TokenKind::Pipe => BinaryOp::BitwiseOr,
                TokenKind::Ampersand => BinaryOp::BitwiseAnd,
                _ => unreachable!(),
            };
            let right = self.parse_range()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse range operators (.., ...)
    pub(crate) fn parse_range(&mut self) -> Result<Expression, MetorexError> {
        // Beginless range: `..expr` or `...expr`
        if self.check(&[TokenKind::DotDot, TokenKind::DotDotDot]) {
            let op_token = self.advance();
            let exclusive = op_token.kind == TokenKind::DotDotDot;
            let end = self.parse_term()?;
            return Ok(Expression::Range {
                start: Box::new(Expression::NilLiteral {
                    position: op_token.position,
                }),
                end: Box::new(end),
                exclusive,
                position: op_token.position,
            });
        }

        let mut expr = self.parse_term()?;

        if self.check(&[TokenKind::DotDot, TokenKind::DotDotDot]) {
            let op_token = self.advance();
            let exclusive = op_token.kind == TokenKind::DotDotDot;
            // Endless range: `x..` followed by `)`, `]`, `,`, `}`, newline, or EOF.
            let is_endless = self.check(&[
                TokenKind::RParen,
                TokenKind::RBracket,
                TokenKind::RBrace,
                TokenKind::Comma,
                TokenKind::Newline,
                TokenKind::Semicolon,
            ]) || self.is_at_end();
            let end = if is_endless {
                Expression::NilLiteral {
                    position: op_token.position,
                }
            } else {
                self.parse_term()?
            };
            expr = Expression::Range {
                start: Box::new(expr),
                end: Box::new(end),
                exclusive,
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse addition and subtraction
    pub(crate) fn parse_term(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_factor()?;

        while self.check(&[TokenKind::Plus, TokenKind::Minus]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse multiplication, division, and modulo
    pub(crate) fn parse_factor(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_unary()?;

        while self.check(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }
}
