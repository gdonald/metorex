// Expression parsing module
// Handles parsing of all expression types

mod binary;
mod call;
mod primary;
mod unary;

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::TokenKind;
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

    /// Parse a block: `do |param1, param2| ... end`
    pub(crate) fn parse_block(&mut self) -> Result<Expression, MetorexError> {
        let start_pos = self.peek().position;

        // Expect 'do' keyword
        self.expect(TokenKind::Do, "Expected 'do' to start block")?;
        self.skip_whitespace();

        // Parse block parameters (e.g., |x, y|)
        let parameters = if self.match_token(&[TokenKind::Pipe]) {
            let mut params = Vec::new();
            self.skip_whitespace();

            if !self.check(&[TokenKind::Pipe]) {
                loop {
                    self.skip_whitespace();
                    let param_token = self.advance();
                    match param_token.kind {
                        TokenKind::Ident(name) => params.push(name),
                        _ => return Err(self.error_at_previous("Expected parameter name")),
                    }
                    self.skip_whitespace();

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.skip_whitespace();
            self.expect(TokenKind::Pipe, "Expected '|' after block parameters")?;
            params
        } else {
            Vec::new()
        };

        self.skip_whitespace();

        // Parse block body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' to close block")?;

        Ok(Expression::Lambda {
            parameters,
            body,
            captured_vars: None, // Will be filled by semantic analysis
            position: start_pos,
        })
    }
}
