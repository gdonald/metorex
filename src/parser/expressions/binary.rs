// Binary operator parsing
// Handles parsing of binary operations with proper precedence

use crate::ast::{BinaryOp, Expression};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse equality operators (==, !=)
    pub(crate) fn parse_equality(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_comparison()?;

        while self.check(&[TokenKind::EqualEqual, TokenKind::BangEqual]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
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
        ]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
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
        let mut expr = self.parse_term()?;

        if self.check(&[TokenKind::DotDot, TokenKind::DotDotDot]) {
            let op_token = self.advance();
            let exclusive = op_token.kind == TokenKind::DotDotDot;
            let end = self.parse_term()?;
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
