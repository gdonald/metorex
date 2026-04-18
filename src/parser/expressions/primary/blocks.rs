// Block / lambda primary parsing: `lambda { ... }`, `do ... end`, `-> { ... }`.

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::{Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse a `lambda { ... }` / `lambda do ... end` literal after the
    /// `lambda` keyword has been consumed.
    pub(super) fn parse_lambda_literal(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();

        // Check for brace or 'do' keyword
        let use_braces = self.match_token(&[TokenKind::LBrace]);
        if !use_braces {
            self.match_token(&[TokenKind::Do]);
        }
        self.skip_whitespace();

        let parameters = self.parse_pipe_params()?;

        self.skip_whitespace();
        let mut body = Vec::new();
        let end_token = if use_braces {
            TokenKind::RBrace
        } else {
            TokenKind::End
        };

        while !self.check(std::slice::from_ref(&end_token)) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(std::slice::from_ref(&end_token)) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        if use_braces {
            self.expect(TokenKind::RBrace, "Expected '}' after lambda body")?;
        } else {
            self.expect(TokenKind::End, "Expected 'end' after lambda body")?;
        }

        Ok(Expression::Lambda {
            parameters,
            body,
            captured_vars: Some(Vec::new()),
            position: token_position,
        })
    }

    /// Parse a standalone `do ... end` block after the `do` keyword has been consumed.
    pub(super) fn parse_do_block(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();

        let parameters = self.parse_pipe_params()?;

        self.skip_whitespace();
        let body = self.parse_block_body_with_optional_rescue_ensure(token_position)?;
        self.expect(TokenKind::End, "Expected 'end' after block body")?;

        Ok(Expression::Lambda {
            parameters,
            body,
            captured_vars: Some(Vec::new()),
            position: token_position,
        })
    }

    /// Parse a stabby lambda `-> { ... }` / `-> (params) { ... }` after the
    /// `->` token has been consumed.
    pub(super) fn parse_stabby_lambda(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();
        if self.check(&[TokenKind::LBrace]) {
            return self.lambda_with_brace_block();
        }
        if self.check(&[TokenKind::Do]) {
            return self.lambda_with_do_block();
        }
        if self.check(&[TokenKind::LParen]) {
            return self.stabby_lambda_with_params(token_position);
        }
        // Bare `-> expr`
        let expr = self.parse_assignment()?;
        Ok(Expression::Lambda {
            parameters: Vec::new(),
            body: vec![Statement::Expression {
                expression: expr,
                position: token_position,
            }],
            captured_vars: Some(Vec::new()),
            position: token_position,
        })
    }

    /// Parse `|param1, param2, ...|` parameter list, or `||` for an empty
    /// list. Returns an empty `Vec` if no leading `|` is present (no params).
    ///
    /// Accepts (and currently silently strips):
    ///   - splat:        `*args`
    ///   - block param:  `&blk`
    ///   - default value: `name = expr`
    fn parse_pipe_params(&mut self) -> Result<Vec<String>, MetorexError> {
        if self.match_token(&[TokenKind::LogicalOr]) {
            // Empty parameter list: ||
            return Ok(Vec::new());
        }
        if !self.match_token(&[TokenKind::Pipe]) {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();
        self.skip_whitespace();

        if !self.check(&[TokenKind::Pipe]) {
            loop {
                self.skip_whitespace();
                // Allow `*name` (splat) and `&name` (block) modifiers; we
                // strip the modifier and just record the bound name.
                let _ = self.match_token(&[TokenKind::Star, TokenKind::Ampersand]);
                self.skip_whitespace();
                if let TokenKind::Ident(name) = self.peek().kind.clone() {
                    params.push(name);
                    self.advance();
                } else {
                    return Err(self.error_at_current("Expected parameter name"));
                }

                self.skip_whitespace();
                // Allow `name = default_value` — parse and discard the default.
                if self.match_token(&[TokenKind::Equal]) {
                    self.skip_whitespace();
                    let _ = self.parse_expression()?;
                    self.skip_whitespace();
                }
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.skip_whitespace();
        self.expect(TokenKind::Pipe, "Expected '|' after lambda parameters")?;
        Ok(params)
    }

    fn lambda_with_brace_block(&mut self) -> Result<Expression, MetorexError> {
        let block = self.parse_brace_block()?;
        if let Expression::Lambda {
            parameters,
            body,
            position,
            ..
        } = block
        {
            Ok(Expression::Lambda {
                parameters,
                body,
                captured_vars: Some(Vec::new()),
                position,
            })
        } else {
            Ok(block)
        }
    }

    fn lambda_with_do_block(&mut self) -> Result<Expression, MetorexError> {
        let block = self.parse_block()?;
        if let Expression::Lambda {
            parameters,
            body,
            position,
            ..
        } = block
        {
            Ok(Expression::Lambda {
                parameters,
                body,
                captured_vars: Some(Vec::new()),
                position,
            })
        } else {
            Ok(block)
        }
    }

    fn stabby_lambda_with_params(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        self.advance(); // consume (
        let mut params = Vec::new();
        self.skip_whitespace();
        if !self.check(&[TokenKind::RParen]) {
            loop {
                self.skip_whitespace();
                if let TokenKind::Ident(name) = self.peek().kind.clone() {
                    params.push(name);
                    self.advance();
                }
                self.skip_whitespace();
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen, "Expected ')'")?;
        self.skip_whitespace();

        if self.check(&[TokenKind::LBrace]) {
            let block = self.parse_brace_block()?;
            if let Expression::Lambda { body, position, .. } = block {
                return Ok(Expression::Lambda {
                    parameters: params,
                    body,
                    captured_vars: Some(Vec::new()),
                    position,
                });
            }
            return Ok(block);
        }

        let expr = self.parse_expression()?;
        Ok(Expression::Lambda {
            parameters: params,
            body: vec![Statement::Expression {
                expression: expr,
                position: token_position,
            }],
            captured_vars: Some(Vec::new()),
            position: token_position,
        })
    }
}
