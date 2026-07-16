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

        let (parameters, parameter_defaults) = self.parse_block_pipe_params()?;

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
            parameter_defaults,
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

        let (parameters, parameter_defaults) = self.parse_block_pipe_params()?;

        self.skip_whitespace();
        let body = self.parse_block_body_with_optional_rescue_ensure(token_position)?;
        self.expect(TokenKind::End, "Expected 'end' after block body")?;

        Ok(Expression::Lambda {
            parameters,
            parameter_defaults,
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
        // Paren-less params: `-> e { ... }` / `-> a, b { ... }`. Collect a
        // comma-separated identifier list; if a `{` or `do` follows, those
        // are the lambda's parameters. Otherwise backtrack to the bare
        // `-> expr` form below.
        if matches!(self.peek().kind, TokenKind::Ident(_)) {
            let saved_position = self.stream().current_position();
            let mut params = Vec::new();
            loop {
                if let TokenKind::Ident(name) = self.peek().kind.clone() {
                    params.push(name);
                    self.advance();
                } else {
                    params.clear();
                    break;
                }
                self.skip_whitespace();
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
                self.skip_whitespace();
            }
            if !params.is_empty() && self.check(&[TokenKind::LBrace, TokenKind::Do]) {
                let block = if self.check(&[TokenKind::LBrace]) {
                    self.parse_brace_block()?
                } else {
                    self.parse_block()?
                };
                if let Expression::Lambda { body, position, .. } = block {
                    return Ok(Expression::Lambda {
                        parameters: params,
                        parameter_defaults: Vec::new(),
                        body,
                        captured_vars: Some(Vec::new()),
                        position,
                    });
                }
                return Ok(block);
            }
            self.stream.restore_position(saved_position);
        }
        // Bare `-> expr`
        let expr = self.parse_assignment()?;
        Ok(Expression::Lambda {
            parameters: Vec::new(),
            parameter_defaults: Vec::new(),
            body: vec![Statement::Expression {
                expression: expr,
                position: token_position,
            }],
            captured_vars: Some(Vec::new()),
            position: token_position,
        })
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
                parameter_defaults: Vec::new(),
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
                parameter_defaults: Vec::new(),
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
                    parameter_defaults: Vec::new(),
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
            parameter_defaults: Vec::new(),
            body: vec![Statement::Expression {
                expression: expr,
                position: token_position,
            }],
            captured_vars: Some(Vec::new()),
            position: token_position,
        })
    }
}
