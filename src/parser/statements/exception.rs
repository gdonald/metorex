// Exception handling statement parsing (begin/rescue/raise)

use crate::ast::{Expression, RescueClause, Statement};
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse a begin...rescue...else...ensure...end statement
    pub(crate) fn parse_begin_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Begin, "Expected 'begin'")?.position;
        let parts = self.parse_begin_body()?;

        Ok(Statement::Begin {
            body: parts.body,
            rescue_clauses: parts.rescue_clauses,
            else_clause: parts.else_clause,
            ensure_block: parts.ensure_block,
            position: start_pos,
        })
    }

    /// Parse a `begin ... end` expression (RHS of an assignment, etc.) into
    /// an `Expression::BeginRescue`. Called after the `begin` token has been
    /// consumed by the primary-expression parser.
    pub(crate) fn parse_begin_expression(
        &mut self,
        start_pos: crate::lexer::Position,
    ) -> Result<Expression, MetorexError> {
        let parts = self.parse_begin_body()?;
        Ok(Expression::BeginRescue {
            body: parts.body,
            rescue_clauses: parts.rescue_clauses,
            else_clause: parts.else_clause,
            ensure_block: parts.ensure_block,
            position: start_pos,
        })
    }

    /// Shared parser for the body of a begin block, used by both the
    /// statement and expression forms. The opening `begin` token must
    /// already have been consumed.
    fn parse_begin_body(&mut self) -> Result<BeginParts, MetorexError> {
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

        Ok(BeginParts {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
        })
    }
}

struct BeginParts {
    body: Vec<Statement>,
    rescue_clauses: Vec<RescueClause>,
    else_clause: Option<Vec<Statement>>,
    ensure_block: Option<Vec<Statement>>,
}

impl Parser {
    // Sentinel impl to keep the file's `Parser` impl block contiguous —
    // helpers added below.

    /// Parse a rescue clause
    pub(crate) fn parse_rescue_clause(&mut self) -> Result<RescueClause, MetorexError> {
        let start_pos = self.previous().position;

        // Don't skip whitespace yet - we need to check if there's a newline after rescue
        // If there's a newline, then no exception types are specified
        let has_newline = self.check(&[TokenKind::Newline]);

        self.skip_whitespace();

        // Parse exception types (comma-separated identifiers)
        let mut exception_types = Vec::new();
        let mut variable_name = None;

        // Check if there's an exception type specified
        // An exception type is present if:
        // 1. There's no newline after rescue
        // 2. We see an identifier that's NOT followed by '=' (which would be an assignment)
        if !has_newline && self.check(&[TokenKind::Ident(String::new())]) {
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
                // Parse exception types (may include scope resolution like Errno::ENOENT)
                while let TokenKind::Ident(name) = &self.peek().kind {
                    let mut full_name = name.clone();
                    self.advance();
                    // Handle scope resolution (Errno::ENOENT)
                    while self.match_token(&[TokenKind::ColonColon]) {
                        if let TokenKind::Ident(part) = &self.peek().kind {
                            full_name.push_str("::");
                            full_name.push_str(part);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    exception_types.push(full_name);
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
        // Do NOT skip newlines: a bare `raise` followed by a newline (or
        // postfix `if`/`unless`, or `end` on the next line) means a re-raise
        // with no explicit exception. Skipping newlines first would let
        // `parse_expression` consume the next statement.

        let exception = if self.check(&[
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::End,
            TokenKind::RBrace,
            TokenKind::If,
            TokenKind::Unless,
            TokenKind::While,
            TokenKind::Else,
            TokenKind::Elsif,
            TokenKind::When,
            TokenKind::Rescue,
            TokenKind::Ensure,
        ]) || self.is_at_end()
        {
            None
        } else {
            let expr = self.parse_expression()?;
            // Check for `raise ExceptionClass, message` form
            if self.match_token(&[TokenKind::Comma]) {
                self.skip_whitespace();
                let message = self.parse_expression()?;
                // Transform to ExceptionClass.new(message)
                Some(Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: "new".to_string(),
                    arguments: vec![message],
                    trailing_block: None,
                    position: start_pos,
                })
            } else {
                Some(expr)
            }
        };

        let stmt = Statement::Raise {
            exception,
            position: start_pos,
        };
        self.wrap_with_modifier(stmt)
    }
}
