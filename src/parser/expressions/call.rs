// Function and method call parsing
// Handles parsing of function calls, method calls, and array indexing

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse function calls and method calls
    pub(crate) fn parse_call(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&[TokenKind::LParen]) {
                // Function call
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenKind::Dot]) {
                // Method call
                let method_name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected method name after '.'")),
                };

                // Check if there are arguments
                let arguments = if self.match_token(&[TokenKind::LParen]) {
                    self.parse_arguments()?
                } else {
                    Vec::new()
                };

                let position = expr.position();
                expr = Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: method_name,
                    arguments,
                    trailing_block: None,
                    position,
                };
            } else if self.match_token(&[TokenKind::LBracket]) {
                // Array indexing
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket, "Expected ']' after array index")?;
                let position = expr.position();
                expr = Expression::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                    position,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Finish parsing a function call
    pub(crate) fn finish_call(&mut self, callee: Expression) -> Result<Expression, MetorexError> {
        let arguments = self.parse_arguments()?;
        let position = callee.position();

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            trailing_block: None,
            position,
        })
    }

    /// Parse function/method arguments
    pub(crate) fn parse_arguments(&mut self) -> Result<Vec<Expression>, MetorexError> {
        let mut arguments = Vec::new();
        self.skip_whitespace();

        if self.check(&[TokenKind::RParen]) {
            self.advance();
            return Ok(arguments);
        }

        loop {
            self.skip_whitespace();
            arguments.push(self.parse_expression()?);
            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        self.skip_whitespace();
        self.expect(TokenKind::RParen, "Expected ')' after arguments")?;

        Ok(arguments)
    }
}
