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

        let mut _singleton_receiver: Option<String> = None;
        let name = match self.advance().kind {
            TokenKind::Ident(name) => {
                // Check for singleton method: def obj.method_name
                if self.check(&[TokenKind::Dot]) {
                    self.advance(); // consume .
                    _singleton_receiver = Some(name);
                    let method_name = match self.advance().kind {
                        TokenKind::Ident(method_name) => method_name,
                        _ => return Err(self.error_at_previous("Expected method name after '.'")),
                    };
                    // Check for setter: def obj.name=(...)
                    if self.check(&[TokenKind::Equal])
                        && matches!(self.peek_ahead(1).kind, TokenKind::LParen)
                    {
                        self.advance(); // consume =
                        format!("{}=", method_name)
                    } else {
                        method_name
                    }
                } else if self.check(&[TokenKind::Equal])
                    && matches!(self.peek_ahead(1).kind, TokenKind::LParen)
                {
                    // Setter method: def name=(value)
                    self.advance(); // consume =
                    format!("{}=", name)
                } else {
                    name
                }
            }
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
            TokenKind::Spaceship => "<=>".to_string(),
            TokenKind::Pipe => "|".to_string(),
            TokenKind::Ampersand => "&".to_string(),
            TokenKind::Shovel => "<<".to_string(),
            // Allow keywords as method names
            TokenKind::Continue => "next".to_string(),
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
        while !self.check(&[TokenKind::End, TokenKind::Rescue, TokenKind::Ensure])
            && !self.is_at_end()
        {
            self.skip_whitespace();
            if self.check(&[TokenKind::End, TokenKind::Rescue, TokenKind::Ensure]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        // Check for method-level rescue/ensure (implicit begin)
        if self.check(&[TokenKind::Rescue, TokenKind::Ensure]) {
            let mut rescue_clauses = Vec::new();
            while self.match_token(&[TokenKind::Rescue]) {
                rescue_clauses.push(self.parse_rescue_clause()?);
                self.skip_whitespace();
            }

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

            // Wrap the body in a Begin statement
            body = vec![Statement::Begin {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                position: start_pos,
            }];
        }

        self.expect(TokenKind::End, "Expected 'end' after function body")?;

        // Return MethodDef if we're inside a class, otherwise FunctionDef
        if self.in_class_body {
            let is_class_method = _singleton_receiver.is_some();
            Ok(Statement::MethodDef {
                name,
                parameters,
                body,
                is_class_method,
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
