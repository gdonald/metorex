// Function definition parsing

use crate::ast::{Parameter, Statement};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse a function definition
    pub(crate) fn parse_function_def(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Def, "Expected 'def'")?.position;
        self.skip_whitespace();

        let name = match self.advance().kind {
            TokenKind::Ident(name) => name,
            // Operator method names
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::Percent => "%".to_string(),
            TokenKind::EqualEqual => "==".to_string(),
            TokenKind::BangEqual => "!=".to_string(),
            TokenKind::Less => "<".to_string(),
            TokenKind::Greater => ">".to_string(),
            TokenKind::LessEqual => "<=".to_string(),
            TokenKind::GreaterEqual => ">=".to_string(),
            TokenKind::Pipe => "|".to_string(),
            TokenKind::Ampersand => "&".to_string(),
            // [] and []= operator method names
            TokenKind::LBracket => {
                self.expect(TokenKind::RBracket, "Expected ']' after '[' in method name")?;
                if self.match_token(&[TokenKind::Equal]) {
                    "[]=".to_string()
                } else {
                    "[]".to_string()
                }
            }
            _ => return Err(self.error_at_previous("Expected function name")),
        };

        self.skip_whitespace();

        // Parse parameters (optional parentheses)
        let parameters = if self.match_token(&[TokenKind::LParen]) {
            self.parse_parameters()?
        } else {
            Vec::new()
        };

        self.skip_whitespace();

        // Parse function body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' after function body")?;

        // Return MethodDef if we're inside a class, otherwise FunctionDef
        if self.in_class_body {
            Ok(Statement::MethodDef {
                name,
                parameters,
                body,
                position: start_pos,
            })
        } else {
            Ok(Statement::FunctionDef {
                name,
                parameters,
                body,
                position: start_pos,
            })
        }
    }

    /// Parse function parameters
    pub(crate) fn parse_parameters(&mut self) -> Result<Vec<Parameter>, MetorexError> {
        let mut params = Vec::new();
        self.skip_whitespace();

        if self.check(&[TokenKind::RParen]) {
            self.advance();
            return Ok(params);
        }

        loop {
            self.skip_whitespace();

            let param_pos = self.peek().position;

            // Check for block parameter (&block)
            if self.match_token(&[TokenKind::Ampersand]) {
                let name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected parameter name after '&'")),
                };
                params.push(Parameter::block(name, param_pos));
            }
            // Check for variadic parameter (*args)
            else if self.match_token(&[TokenKind::Star]) {
                let name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected parameter name after '*'")),
                };
                params.push(Parameter::variadic(name, param_pos));
            } else {
                let name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected parameter name")),
                };

                // Check for `name:` or `name: default` (named keyword argument)
                if self.match_token(&[TokenKind::Colon]) {
                    // `name:` alone means required keyword arg (no default)
                    // `name: expr` means optional keyword arg with default
                    let default =
                        if self.check(&[TokenKind::Comma, TokenKind::RParen, TokenKind::Newline]) {
                            None
                        } else {
                            Some(self.parse_expression()?)
                        };
                    params.push(Parameter::named_keyword(name, default, param_pos));
                // Check for positional default value
                } else if self.match_token(&[TokenKind::Equal]) {
                    let default = self.parse_expression()?;
                    params.push(Parameter::with_default(name, default, param_pos));
                } else {
                    params.push(Parameter::simple(name, param_pos));
                }
            }

            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        self.skip_whitespace();
        self.expect(TokenKind::RParen, "Expected ')' after parameters")?;

        Ok(params)
    }
}
