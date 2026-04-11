// Keyword-led primary parsing: `super`, `defined?`, `yield`, leading `::`.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::{Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse a `super` call after the `super` keyword has been consumed.
    pub(super) fn parse_super_call(
        &mut self,
        position: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();

        // Parse optional arguments
        let arguments = if self.check(&[TokenKind::LParen]) {
            self.advance(); // consume (
            let mut args = Vec::new();
            self.skip_whitespace();

            if !self.check(&[TokenKind::RParen]) {
                loop {
                    self.skip_whitespace();
                    args.push(self.parse_expression()?);
                    self.skip_whitespace();

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.skip_whitespace();
            self.expect(TokenKind::RParen, "Expected ')' after super arguments")?;
            args
        } else {
            // super without parentheses - no arguments
            Vec::new()
        };

        Ok(Expression::Super {
            arguments,
            position,
        })
    }

    /// Parse a `defined?(...)` expression after the `defined?` keyword has been consumed.
    pub(super) fn parse_defined_expression(
        &mut self,
        position: Position,
    ) -> Result<Expression, MetorexError> {
        let expression = if self.match_token(&[TokenKind::LParen]) {
            let expr = self.parse_expression()?;
            self.expect(TokenKind::RParen, "Expected ')' after defined? argument")?;
            expr
        } else {
            self.parse_expression()?
        };
        Ok(Expression::Defined {
            expression: Box::new(expression),
            position,
        })
    }

    /// Parse a `yield` expression after the `yield` keyword has been consumed.
    pub(super) fn parse_yield_expression(
        &mut self,
        position: Position,
    ) -> Result<Expression, MetorexError> {
        let arguments = if self.check(&[TokenKind::LParen]) {
            self.advance(); // consume (
            let mut args = Vec::new();
            self.skip_whitespace();

            if !self.check(&[TokenKind::RParen]) {
                loop {
                    self.skip_whitespace();
                    args.push(self.parse_expression()?);
                    self.skip_whitespace();

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.skip_whitespace();
            self.expect(TokenKind::RParen, "Expected ')' after yield arguments")?;
            args
        } else if !self.check(&[
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::EOF,
            TokenKind::End,
            TokenKind::RBrace,
            TokenKind::RParen,
            TokenKind::If,
            TokenKind::Unless,
            TokenKind::Do,
        ]) && !self.is_at_end()
        {
            // yield expr, expr — paren-less arguments
            let mut args = vec![self.parse_expression()?];
            while self.match_token(&[TokenKind::Comma]) {
                self.skip_whitespace();
                args.push(self.parse_expression()?);
            }
            args
        } else {
            Vec::new()
        };

        Ok(Expression::Yield {
            arguments,
            position,
        })
    }

    /// Parse a leading `::Name` top-level constant access. We don't have a
    /// dedicated top-level namespace distinct from the global one, so treat
    /// `::Name` exactly as `Name`.
    pub(super) fn parse_leading_coloncolon(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            _ => {
                return Err(self.error_at_previous("Expected identifier after leading '::'"));
            }
        };
        Ok(Expression::Identifier {
            name,
            position: token_position,
        })
    }
}
