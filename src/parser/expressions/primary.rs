// Primary expression parsing
// Handles parsing of literals, identifiers, and compound expressions

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse primary expressions (literals, identifiers, groups)
    pub(crate) fn parse_primary(&mut self) -> Result<Expression, MetorexError> {
        let token = self.advance();

        match token.kind {
            // Literals
            TokenKind::Int(value) => Ok(Expression::IntLiteral {
                value,
                position: token.position,
            }),
            TokenKind::Float(value) => Ok(Expression::FloatLiteral {
                value,
                position: token.position,
            }),
            TokenKind::String(value) => Ok(Expression::StringLiteral {
                value,
                position: token.position,
            }),
            TokenKind::InterpolatedString(parts) => {
                // Convert token interpolation parts to AST interpolation parts
                let mut ast_parts = Vec::new();
                for part in parts {
                    match part {
                        crate::lexer::InterpolationPart::Text(text) => {
                            ast_parts.push(crate::ast::node::InterpolationPart::Text(text));
                        }
                        crate::lexer::InterpolationPart::Expression(expr_str) => {
                            // Parse the expression string
                            // For now, we'll create a simple parser for the embedded expression
                            let expr_lexer = crate::lexer::Lexer::new(&expr_str);
                            let expr_tokens = expr_lexer.tokenize();
                            let mut expr_parser = Parser::new(expr_tokens);
                            let expr = expr_parser.parse_expression()?;
                            ast_parts.push(crate::ast::node::InterpolationPart::Expression(
                                Box::new(expr),
                            ));
                        }
                    }
                }
                Ok(Expression::InterpolatedString {
                    parts: ast_parts,
                    position: token.position,
                })
            }
            TokenKind::True => Ok(Expression::BoolLiteral {
                value: true,
                position: token.position,
            }),
            TokenKind::False => Ok(Expression::BoolLiteral {
                value: false,
                position: token.position,
            }),
            TokenKind::Nil => Ok(Expression::NilLiteral {
                position: token.position,
            }),

            // Identifiers and variables
            TokenKind::Ident(name) => Ok(Expression::Identifier {
                name,
                position: token.position,
            }),
            TokenKind::InstanceVar(name) => Ok(Expression::InstanceVariable {
                name,
                position: token.position,
            }),
            TokenKind::ClassVar(name) => Ok(Expression::ClassVariable {
                name,
                position: token.position,
            }),

            // Symbol literal (:name)
            TokenKind::Colon => {
                let symbol_position = token.position;
                match self.advance().kind {
                    TokenKind::Ident(name) => Ok(Expression::Symbol {
                        value: name,
                        position: symbol_position,
                    }),
                    _ => Err(self.error_at_previous("Expected identifier after ':' for symbol")),
                }
            }

            // Grouped expression
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen, "Expected ')' after expression")?;
                Ok(Expression::Grouped {
                    expression: Box::new(expr),
                    position: token.position,
                })
            }

            // Array literal
            TokenKind::LBracket => {
                let mut elements = Vec::new();
                self.skip_whitespace();

                if !self.check(&[TokenKind::RBracket]) {
                    loop {
                        self.skip_whitespace();
                        elements.push(self.parse_expression()?);
                        self.skip_whitespace();

                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.skip_whitespace();
                self.expect(TokenKind::RBracket, "Expected ']' after array elements")?;

                Ok(Expression::Array {
                    elements,
                    position: token.position,
                })
            }

            // Dictionary literal
            TokenKind::LBrace => {
                let mut entries = Vec::new();
                self.skip_whitespace();

                if !self.check(&[TokenKind::RBrace]) {
                    loop {
                        self.skip_whitespace();
                        let key = self.parse_expression()?;
                        self.skip_whitespace();

                        // Support both `:` and `=>` for hash syntax
                        if self.check(&[TokenKind::FatArrow]) {
                            self.advance(); // consume =>
                        } else {
                            self.expect(
                                TokenKind::Colon,
                                "Expected ':' or '=>' after dictionary key",
                            )?;
                        }

                        self.skip_whitespace();
                        let value = self.parse_expression()?;
                        entries.push((key, value));
                        self.skip_whitespace();

                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.skip_whitespace();
                self.expect(TokenKind::RBrace, "Expected '}' after dictionary entries")?;

                Ok(Expression::Dictionary {
                    entries,
                    position: token.position,
                })
            }

            // Lambda literal: lambda do |params| ... end or lambda |params| ... end
            TokenKind::Lambda => {
                self.skip_whitespace();

                // Check for 'do' keyword (optional for compact syntax)
                let _has_do = self.match_token(&[TokenKind::Do]);
                self.skip_whitespace();

                // Parse parameters: |param1, param2, ...|
                let parameters = if self.match_token(&[TokenKind::Pipe]) {
                    let mut params = Vec::new();
                    self.skip_whitespace();

                    if !self.check(&[TokenKind::Pipe]) {
                        loop {
                            self.skip_whitespace();
                            if let TokenKind::Ident(name) = self.peek().kind.clone() {
                                params.push(name);
                                self.advance();
                            } else {
                                return Err(self.error_at_current("Expected parameter name"));
                            }

                            self.skip_whitespace();
                            if !self.match_token(&[TokenKind::Comma]) {
                                break;
                            }
                        }
                    }

                    self.skip_whitespace();
                    self.expect(TokenKind::Pipe, "Expected '|' after lambda parameters")?;
                    params
                } else {
                    Vec::new()
                };

                // Parse body statements
                self.skip_whitespace();
                let mut body = Vec::new();

                while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                    self.skip_whitespace();
                    if self.check(&[TokenKind::End]) {
                        break;
                    }
                    body.push(self.parse_statement()?);
                    self.skip_whitespace();
                }

                self.expect(TokenKind::End, "Expected 'end' after lambda body")?;

                Ok(Expression::Lambda {
                    parameters,
                    body,
                    captured_vars: Some(Vec::new()), // Empty vec signals automatic capture
                    position: token.position,
                })
            }

            // Standalone block: do ... end
            TokenKind::Do => {
                self.skip_whitespace();

                // Parse optional parameters: |param1, param2, ...|
                let parameters = if self.match_token(&[TokenKind::Pipe]) {
                    let mut params = Vec::new();
                    self.skip_whitespace();

                    if !self.check(&[TokenKind::Pipe]) {
                        loop {
                            self.skip_whitespace();
                            if let TokenKind::Ident(name) = self.peek().kind.clone() {
                                params.push(name);
                                self.advance();
                            } else {
                                return Err(self.error_at_current("Expected parameter name"));
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

                // Parse body statements
                self.skip_whitespace();
                let mut body = Vec::new();

                while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                    self.skip_whitespace();
                    if self.check(&[TokenKind::End]) {
                        break;
                    }
                    body.push(self.parse_statement()?);
                    self.skip_whitespace();
                }

                self.expect(TokenKind::End, "Expected 'end' after block body")?;

                // A standalone block is essentially a lambda with no parameters
                // that gets evaluated immediately (in this parser representation)
                Ok(Expression::Lambda {
                    parameters,
                    body,
                    captured_vars: Some(Vec::new()), // Empty vec signals automatic capture
                    position: token.position,
                })
            }

            // Super call: super() or super(args)
            TokenKind::Super => {
                self.skip_whitespace();
                let position = token.position;

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

            _ => Err(self.error_at_previous(&format!("Unexpected token: {:?}", token.kind))),
        }
    }
}
