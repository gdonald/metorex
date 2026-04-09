// Primary expression parsing
// Handles parsing of literals, identifiers, and compound expressions

use crate::ast::Expression;
use crate::ast::node::{ElsifBranch, ExprMatchCase};
use crate::error::MetorexError;
use crate::lexer::{Position, TokenKind};
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
            TokenKind::Regex(pattern, flags) => Ok(Expression::RegexLiteral {
                pattern,
                flags,
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
            TokenKind::GlobalVar(name) => Ok(Expression::GlobalVariable {
                name,
                position: token.position,
            }),
            TokenKind::MagicFile => Ok(Expression::MagicFile {
                position: token.position,
            }),
            TokenKind::MagicLine => Ok(Expression::MagicLine {
                position: token.position,
            }),

            // Symbol literal (:name, :@ivar, :@@cvar, :keyword)
            TokenKind::Colon => {
                let symbol_position = token.position;
                match self.advance().kind {
                    TokenKind::Ident(name) => Ok(Expression::Symbol {
                        value: name,
                        position: symbol_position,
                    }),
                    TokenKind::InstanceVar(name) => Ok(Expression::Symbol {
                        value: format!("@{}", name),
                        position: symbol_position,
                    }),
                    TokenKind::ClassVar(name) => Ok(Expression::Symbol {
                        value: format!("@@{}", name),
                        position: symbol_position,
                    }),
                    // Allow keywords as symbol names
                    TokenKind::Def => Ok(Expression::Symbol {
                        value: "def".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Class => Ok(Expression::Symbol {
                        value: "class".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::If => Ok(Expression::Symbol {
                        value: "if".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Else => Ok(Expression::Symbol {
                        value: "else".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::End => Ok(Expression::Symbol {
                        value: "end".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Do => Ok(Expression::Symbol {
                        value: "do".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Nil => Ok(Expression::Symbol {
                        value: "nil".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::True => Ok(Expression::Symbol {
                        value: "true".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::False => Ok(Expression::Symbol {
                        value: "false".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Return => Ok(Expression::Symbol {
                        value: "return".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Begin => Ok(Expression::Symbol {
                        value: "begin".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Rescue => Ok(Expression::Symbol {
                        value: "rescue".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Ensure => Ok(Expression::Symbol {
                        value: "ensure".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::While => Ok(Expression::Symbol {
                        value: "while".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::For => Ok(Expression::Symbol {
                        value: "for".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Case => Ok(Expression::Symbol {
                        value: "case".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::When => Ok(Expression::Symbol {
                        value: "when".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Module => Ok(Expression::Symbol {
                        value: "module".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Include => Ok(Expression::Symbol {
                        value: "include".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Yield => Ok(Expression::Symbol {
                        value: "yield".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Super => Ok(Expression::Symbol {
                        value: "super".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Lambda => Ok(Expression::Symbol {
                        value: "lambda".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Break => Ok(Expression::Symbol {
                        value: "break".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Continue => Ok(Expression::Symbol {
                        value: "next".to_string(),
                        position: symbol_position,
                    }),
                    TokenKind::Raise => Ok(Expression::Symbol {
                        value: "raise".to_string(),
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

            // Lambda literal: lambda do |params| ... end or lambda { |params| ... }
            TokenKind::Lambda => {
                self.skip_whitespace();

                // Check for brace or 'do' keyword
                let use_braces = self.match_token(&[TokenKind::LBrace]);
                if !use_braces {
                    self.match_token(&[TokenKind::Do]);
                }
                self.skip_whitespace();

                // Parse parameters: |param1, param2, ...| or || for empty params
                let parameters = if self.match_token(&[TokenKind::LogicalOr]) {
                    // Empty parameter list: ||
                    Vec::new()
                } else if self.match_token(&[TokenKind::Pipe]) {
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

            // defined?(expr) — check if something is defined
            TokenKind::Defined => {
                let position = token.position;
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

            // Yield: yield or yield(args) or yield args
            TokenKind::Yield => {
                let position = token.position;

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

            // Case expression: case value when pattern then expr ... end
            TokenKind::Case => self.parse_case_expression(token.position),

            // If expression: if cond then ... else ... end
            TokenKind::If => self.parse_if_expression(token.position),

            // Unless expression: unless cond then ... else ... end
            TokenKind::Unless => self.parse_unless_expression(token.position),

            _ => Err(self.error_at_previous(&format!("Unexpected token: {:?}", token.kind))),
        }
    }

    /// Parse a case expression (pattern matching in expression context)
    ///
    /// Supports two syntaxes:
    ///
    /// # Block syntax
    /// ```text
    /// case expression
    /// when pattern
    ///   expr
    /// when pattern
    ///   expr
    /// else
    ///   expr
    /// end
    /// ```
    ///
    /// # Inline syntax
    /// ```text
    /// case expression when pattern then expr when pattern then expr else expr end
    /// ```
    ///
    /// # Guard clauses
    /// ```text
    /// when pattern if guard_expr then expr
    /// ```
    pub(crate) fn parse_case_expression(
        &mut self,
        start_pos: crate::lexer::Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();

        // Parse the expression to match against
        let expression = Box::new(self.parse_expression()?);
        self.skip_whitespace();

        // Parse when clauses
        let mut cases = Vec::new();
        loop {
            self.skip_whitespace();
            if !self.match_token(&[TokenKind::When]) {
                break;
            }
            let when_pos = self.previous().position;
            self.skip_whitespace();

            // Parse the pattern using the shared pattern parser (may include comma-separated alternatives)
            let pattern = self.parse_case_pattern_with_alternatives()?;
            self.skip_whitespace();

            // Parse optional guard clause (if ...)
            let guard = if self.match_token(&[TokenKind::If]) {
                self.skip_whitespace();
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.skip_whitespace();

            // Parse the body expression
            // Two syntaxes supported:
            // 1. Inline: when pattern then expression
            // 2. Block: when pattern newline expression(s)
            let body = if self.match_token(&[TokenKind::Then]) {
                // Inline syntax: parse expression after 'then'
                self.skip_whitespace();
                self.parse_expression()?
            } else {
                // Block syntax: parse expression after whitespace
                self.skip_whitespace();
                self.parse_expression()?
            };

            cases.push(ExprMatchCase {
                pattern,
                guard,
                body,
                position: when_pos,
            });

            self.skip_whitespace();
        }

        // Parse optional else clause
        let else_case = if self.match_token(&[TokenKind::Else]) {
            self.skip_whitespace();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.skip_whitespace();
        self.expect(TokenKind::End, "Expected 'end' after case expression")?;

        Ok(Expression::Case {
            expression,
            cases,
            else_case,
            position: start_pos,
        })
    }

    /// Parse an if expression: `if cond [then] body [elsif cond body]* [else body] end`
    pub(crate) fn parse_if_expression(
        &mut self,
        start_pos: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();
        let condition = Box::new(self.parse_expression()?);
        self.skip_whitespace();
        self.match_token(&[TokenKind::Then]); // optional `then`
        self.skip_whitespace();

        let mut then_branch = Vec::new();
        while !self.check(&[TokenKind::Elsif, TokenKind::Else, TokenKind::End]) && !self.is_at_end()
        {
            self.skip_whitespace();
            if self.check(&[TokenKind::Elsif, TokenKind::Else, TokenKind::End]) {
                break;
            }
            then_branch.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        let mut elsif_branches = Vec::new();
        while self.match_token(&[TokenKind::Elsif]) {
            let elsif_pos = self.previous().position;
            self.skip_whitespace();
            let elsif_cond = self.parse_expression()?;
            self.skip_whitespace();
            self.match_token(&[TokenKind::Then]);
            self.skip_whitespace();
            let mut elsif_body = Vec::new();
            while !self.check(&[TokenKind::Elsif, TokenKind::Else, TokenKind::End])
                && !self.is_at_end()
            {
                self.skip_whitespace();
                if self.check(&[TokenKind::Elsif, TokenKind::Else, TokenKind::End]) {
                    break;
                }
                elsif_body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            elsif_branches.push(ElsifBranch {
                condition: elsif_cond,
                body: elsif_body,
                position: elsif_pos,
            });
        }

        let else_branch = if self.match_token(&[TokenKind::Else]) {
            self.skip_whitespace();
            let mut else_stmts = Vec::new();
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::End]) {
                    break;
                }
                else_stmts.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(else_stmts)
        } else {
            None
        };

        self.skip_whitespace();
        self.expect(TokenKind::End, "Expected 'end' after if expression")?;

        Ok(Expression::If {
            condition,
            then_branch,
            elsif_branches,
            else_branch,
            position: start_pos,
        })
    }

    /// Parse an unless expression: `unless cond [then] body [else body] end`
    pub(crate) fn parse_unless_expression(
        &mut self,
        start_pos: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();
        let condition = Box::new(self.parse_expression()?);
        self.skip_whitespace();
        self.match_token(&[TokenKind::Then]);
        self.skip_whitespace();

        let mut then_branch = Vec::new();
        while !self.check(&[TokenKind::Else, TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::Else, TokenKind::End]) {
                break;
            }
            then_branch.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        let else_branch = if self.match_token(&[TokenKind::Else]) {
            self.skip_whitespace();
            let mut else_stmts = Vec::new();
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::End]) {
                    break;
                }
                else_stmts.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(else_stmts)
        } else {
            None
        };

        self.skip_whitespace();
        self.expect(TokenKind::End, "Expected 'end' after unless expression")?;

        Ok(Expression::Unless {
            condition,
            then_branch,
            else_branch,
            position: start_pos,
        })
    }
}
