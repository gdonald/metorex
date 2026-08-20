// Function definition parsing

use crate::ast::{Parameter, Statement};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

/// Marks a `def (nil).name` style singleton receiver whose sole instance
/// makes its singleton class the class itself, so the method belongs in the
/// class's instance method table. Shared with the VM's function-def path.
pub(crate) const SOLE_INSTANCE_RECEIVER: &str = "sole-instance:";

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
                        TokenKind::Plus => "+".to_string(),
                        TokenKind::Minus => "-".to_string(),
                        TokenKind::Star => "*".to_string(),
                        TokenKind::StarStar => "**".to_string(),
                        TokenKind::Slash => "/".to_string(),
                        TokenKind::Percent => "%".to_string(),
                        TokenKind::EqualEqual => "==".to_string(),
                        TokenKind::TripleEqual => "===".to_string(),
                        TokenKind::BangEqual => "!=".to_string(),
                        TokenKind::Less => "<".to_string(),
                        TokenKind::Greater => ">".to_string(),
                        TokenKind::LessEqual => "<=".to_string(),
                        TokenKind::GreaterEqual => ">=".to_string(),
                        TokenKind::Spaceship => "<=>".to_string(),
                        TokenKind::Shovel => "<<".to_string(),
                        TokenKind::Pipe => "|".to_string(),
                        TokenKind::Ampersand => "&".to_string(),
                        TokenKind::Match => "=~".to_string(),
                        TokenKind::LBracket => {
                            self.expect(
                                TokenKind::RBracket,
                                "Expected ']' after '[' in method name",
                            )?;
                            if self.match_token(&[TokenKind::Equal]) {
                                "[]=".to_string()
                            } else {
                                "[]".to_string()
                            }
                        }
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
            TokenKind::TripleEqual => "===".to_string(),
            TokenKind::Match => "=~".to_string(),
            TokenKind::NotMatch => "!~".to_string(),
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
            TokenKind::Include => "include".to_string(),
            TokenKind::Extend => "extend".to_string(),
            TokenKind::Module => "module".to_string(),
            TokenKind::Class => "class".to_string(),
            TokenKind::Raise => "raise".to_string(),
            TokenKind::Begin => "begin".to_string(),
            TokenKind::End => "end".to_string(),
            TokenKind::Lambda => "lambda".to_string(),
            TokenKind::Yield => "yield".to_string(),
            TokenKind::Return => "return".to_string(),
            TokenKind::Break => "break".to_string(),
            TokenKind::Defined => "defined?".to_string(),
            TokenKind::True => "true".to_string(),
            TokenKind::False => "false".to_string(),
            TokenKind::Nil => "nil".to_string(),
            // [] and []= operator method names
            TokenKind::LBracket => {
                self.expect(TokenKind::RBracket, "Expected ']' after '[' in method name")?;
                if self.match_token(&[TokenKind::Equal]) {
                    "[]=".to_string()
                } else {
                    "[]".to_string()
                }
            }
            // def (expr).method_name — singleton method on expression result.
            TokenKind::LParen => {
                let receiver_expr = self.parse_expression()?;
                self.expect(TokenKind::RParen, "Expected ')' after singleton receiver")?;
                self.expect(TokenKind::Dot, "Expected '.' after singleton receiver")?;
                let method_name = match self.advance().kind {
                    TokenKind::Ident(method_name) => method_name,
                    _ => return Err(self.error_at_previous("Expected method name after '.'")),
                };
                // true, false, and nil each have exactly one instance, so
                // their singleton class is the class itself and `def
                // (nil).foo` defines an instance method on NilClass. The
                // marker prefix tells the VM to define it that way rather
                // than as a method on the class object.
                _singleton_receiver = match &receiver_expr {
                    crate::ast::Expression::BoolLiteral { value: true, .. } => {
                        Some(format!("{}TrueClass", SOLE_INSTANCE_RECEIVER))
                    }
                    crate::ast::Expression::BoolLiteral { value: false, .. } => {
                        Some(format!("{}FalseClass", SOLE_INSTANCE_RECEIVER))
                    }
                    crate::ast::Expression::NilLiteral { .. } => {
                        Some(format!("{}NilClass", SOLE_INSTANCE_RECEIVER))
                    }
                    _ => None,
                };
                method_name
            }
            _ => return Err(self.error_at_previous("Expected function name")),
        };

        // Parse parameters (optional parentheses). Parens for params must come
        // immediately after the method name — do NOT skip newlines here, or
        // `def foo\n(@x = y)` would eat the grouped expression as a param list.
        let parameters = if self.match_token(&[TokenKind::LParen]) {
            self.parse_parameters()?
        } else if self.can_start_no_paren_params() {
            // `def name arg, *rest, &block` — Ruby's paren-less param list.
            // Stops at newline/semicolon (statement terminator).
            self.parse_parameters_no_parens()?
        } else {
            self.skip_whitespace();
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
                singleton_class: _singleton_receiver,
            })
        }
    }

    /// Parse function parameters
    /// Whether the next token can begin a paren-less `def` parameter list.
    /// Distinguishes `def foo bar` (paren-less param `bar`) from `def foo;`
    /// or `def foo\n` (no params). Newline / Semicolon / End / RBrace etc.
    /// are all parameter-list terminators.
    fn can_start_no_paren_params(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Ident(_) | TokenKind::Star | TokenKind::Ampersand
        )
    }

    /// Parse a paren-less parameter list: `arg1, *rest, &block`. Mirrors
    /// `parse_parameters` but stops at the statement terminator (Newline /
    /// Semicolon) instead of expecting a closing RParen.
    fn parse_parameters_no_parens(&mut self) -> Result<Vec<Parameter>, MetorexError> {
        let mut params = Vec::new();

        loop {
            let param_pos = self.peek().position;

            if self.match_token(&[TokenKind::Ampersand]) {
                let name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected parameter name after '&'")),
                };
                params.push(Parameter::block(name, param_pos));
            } else if self.match_token(&[TokenKind::Star]) {
                if self.check(&[TokenKind::Comma, TokenKind::Newline, TokenKind::Semicolon]) {
                    params.push(Parameter::variadic("__anon_splat".to_string(), param_pos));
                } else {
                    let name = match self.advance().kind {
                        TokenKind::Ident(name) => name,
                        _ => {
                            return Err(self.error_at_previous("Expected parameter name after '*'"));
                        }
                    };
                    params.push(Parameter::variadic(name, param_pos));
                }
            } else {
                let name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected parameter name")),
                };

                if self.match_token(&[TokenKind::Colon]) {
                    let default = if self.check(&[
                        TokenKind::Comma,
                        TokenKind::Newline,
                        TokenKind::Semicolon,
                    ]) {
                        None
                    } else {
                        Some(self.parse_expression()?)
                    };
                    params.push(Parameter::named_keyword(name, default, param_pos));
                } else if self.match_token(&[TokenKind::Equal]) {
                    let default = self.parse_expression()?;
                    params.push(Parameter::with_default(name, default, param_pos));
                } else {
                    params.push(Parameter::simple(name, param_pos));
                }
            }

            // Comma separator — allow optional whitespace around it. A bare
            // newline/semicolon ends the parameter list.
            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
            self.skip_whitespace();
        }

        Ok(params)
    }

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
            // Check for variadic parameter (*args). A bare `*` with no name is
            // an anonymous splat, which discards the remaining positional args.
            else if self.match_token(&[TokenKind::Star]) {
                if self.check(&[TokenKind::Comma, TokenKind::RParen, TokenKind::Pipe]) {
                    params.push(Parameter::variadic("__anon_splat".to_string(), param_pos));
                } else {
                    let name = match self.advance().kind {
                        TokenKind::Ident(name) => name,
                        _ => {
                            return Err(self.error_at_previous("Expected parameter name after '*'"));
                        }
                    };
                    params.push(Parameter::variadic(name, param_pos));
                }
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
