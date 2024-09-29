// Exception handling statement parsing (begin/rescue/raise)

use crate::ast::{RescueClause, Statement};
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse a begin...rescue...else...ensure...end statement
    pub(crate) fn parse_begin_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Begin, "Expected 'begin'")?.position;
        self.skip_whitespace();

        // Parse the main body
        let mut body = Vec::new();
        while !self.check(&[
            TokenKind::Rescue,
            TokenKind::Else,
            TokenKind::Ensure,
            TokenKind::End,
        ]) && !self.is_at_end()
        {
            self.skip_whitespace();
            if self.check(&[
                TokenKind::Rescue,
                TokenKind::Else,
                TokenKind::Ensure,
                TokenKind::End,
            ]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        // Parse rescue clauses
        let mut rescue_clauses = Vec::new();
        while self.match_token(&[TokenKind::Rescue]) {
            rescue_clauses.push(self.parse_rescue_clause()?);
            self.skip_whitespace();
        }

        // Parse optional else clause
        let else_clause = if self.match_token(&[TokenKind::Else]) {
            self.skip_whitespace();
            let mut else_body = Vec::new();
            while !self.check(&[TokenKind::Ensure, TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::Ensure, TokenKind::End]) {
                    break;
                }
                else_body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(else_body)
        } else {
            None
        };

        // Parse optional ensure clause
        let ensure_block = if self.match_token(&[TokenKind::Ensure]) {
            self.skip_whitespace();
            let mut ensure_body = Vec::new();
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::End]) {
                    break;
                }
                ensure_body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(ensure_body)
        } else {
            None
        };

        self.expect(TokenKind::End, "Expected 'end' after begin block")?;

        Ok(Statement::Begin {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            position: start_pos,
        })
    }

    /// Parse a rescue clause
    pub(crate) fn parse_rescue_clause(&mut self) -> Result<RescueClause, MetorexError> {
        let start_pos = self.previous().position;
        self.skip_whitespace();

        // Parse exception types (comma-separated identifiers)
        let mut exception_types = Vec::new();
        let mut variable_name = None;

        // Check if there's an exception type specified
        // An exception type is present if we see an identifier that's NOT followed by '='
        // (which would indicate an assignment statement in the rescue body)
        if self.check(&[TokenKind::Ident(String::new())]) {
            // Peek ahead to see if this looks like an exception type or an assignment
            // If the next token after the identifier is '=', it's an assignment, not an exception type
            let current_pos = self.stream().current_position();
            let next_is_assignment = if let Some(next) = self.stream().tokens().get(current_pos + 1)
            {
                matches!(next.kind, TokenKind::Equal)
            } else {
                false
            };

            if !next_is_assignment {
                // Parse exception types
                while let TokenKind::Ident(name) = &self.peek().kind {
                    exception_types.push(name.clone());
                    self.advance();
                    self.skip_whitespace();

                    // Check for comma (multiple exception types)
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                    self.skip_whitespace();
                }
            }
        }

        // Check for variable binding (=> var)
        if self.match_token(&[TokenKind::FatArrow]) {
            self.skip_whitespace();
            if let TokenKind::Ident(name) = &self.peek().kind {
                variable_name = Some(name.clone());
                self.advance();
                self.skip_whitespace();
            } else {
                return Err(MetorexError::syntax_error(
                    "Expected variable name after '=>'",
                    SourceLocation::new(
                        self.peek().position.line,
                        self.peek().position.column,
                        self.peek().position.offset,
                    ),
                ));
            }
        }

        // Parse rescue body
        let mut body = Vec::new();
        while !self.check(&[
            TokenKind::Rescue,
            TokenKind::Else,
            TokenKind::Ensure,
            TokenKind::End,
        ]) && !self.is_at_end()
        {
            self.skip_whitespace();
            if self.check(&[
                TokenKind::Rescue,
                TokenKind::Else,
                TokenKind::Ensure,
                TokenKind::End,
            ]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        Ok(RescueClause {
            exception_types,
            variable_name,
            body,
            position: start_pos,
        })
    }

    /// Parse a raise statement
    pub(crate) fn parse_raise_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Raise, "Expected 'raise'")?.position;
        self.skip_whitespace();

        // Check if there's an exception expression
        // If the next token is a newline, semicolon, or end, it's a bare raise
        let exception = if self.check(&[TokenKind::Newline, TokenKind::Semicolon, TokenKind::End])
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Statement::Raise {
            exception,
            position: start_pos,
        })
    }
}
