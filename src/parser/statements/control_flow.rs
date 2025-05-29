// Control flow statement parsing (if, while, for, case)

use crate::ast::{ElsifBranch, MatchCase, MatchPattern, Statement};
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse an if statement
    pub(crate) fn parse_if_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::If, "Expected 'if'")?.position;
        self.skip_whitespace();

        let condition = self.parse_expression()?;
        self.skip_whitespace();

        // Parse then branch
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

        // Parse optional elsif branches
        let mut elsif_branches = Vec::new();
        while self.match_token(&[TokenKind::Elsif]) {
            let elsif_pos = self.previous().position;
            self.skip_whitespace();

            let elsif_condition = self.parse_expression()?;
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
                condition: elsif_condition,
                body: elsif_body,
                position: elsif_pos,
            });
        }

        // Parse optional else branch
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

        self.expect(TokenKind::End, "Expected 'end' after if statement")?;

        Ok(Statement::If {
            condition,
            then_branch,
            elsif_branches,
            else_branch,
            position: start_pos,
        })
    }

    /// Parse a while loop
    pub(crate) fn parse_while_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::While, "Expected 'while'")?.position;
        self.skip_whitespace();

        let condition = self.parse_expression()?;
        self.skip_whitespace();

        // Optionally consume 'do'
        self.match_token(&[TokenKind::Do]);
        self.skip_whitespace();

        // Parse loop body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' after while loop")?;

        Ok(Statement::While {
            condition,
            body,
            position: start_pos,
        })
    }

    /// Parse a for loop
    pub(crate) fn parse_for_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::For, "Expected 'for'")?.position;
        self.skip_whitespace();

        // Parse the loop variable
        let variable = if let TokenKind::Ident(name) = &self.peek().kind {
            let var_name = name.clone();
            self.advance();
            var_name
        } else {
            return Err(MetorexError::syntax_error(
                "Expected identifier after 'for'",
                SourceLocation::new(
                    self.peek().position.line,
                    self.peek().position.column,
                    self.peek().position.offset,
                ),
            ));
        };

        self.skip_whitespace();

        // Expect 'in' keyword
        self.expect(TokenKind::In, "Expected 'in' after loop variable")?;
        self.skip_whitespace();

        // Parse the iterable expression
        let iterable = self.parse_expression()?;
        self.skip_whitespace();

        // Optionally consume 'do'
        self.match_token(&[TokenKind::Do]);
        self.skip_whitespace();

        // Parse loop body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' after for loop")?;

        Ok(Statement::For {
            variable,
            iterable,
            body,
            position: start_pos,
        })
    }

    /// Parse a break statement
    pub(crate) fn parse_break_statement(&mut self) -> Result<Statement, MetorexError> {
        let pos = self.expect(TokenKind::Break, "Expected 'break'")?.position;
        Ok(Statement::Break { position: pos })
    }

    /// Parse a continue statement
    pub(crate) fn parse_continue_statement(&mut self) -> Result<Statement, MetorexError> {
        let pos = self
            .expect(TokenKind::Continue, "Expected 'continue'")?
            .position;
        Ok(Statement::Continue { position: pos })
    }

    /// Parse an unless statement
    pub(crate) fn parse_unless_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Unless, "Expected 'unless'")?
            .position;
        self.skip_whitespace();

        let condition = self.parse_expression()?;
        self.skip_whitespace();

        // Parse then branch
        let mut then_branch = Vec::new();
        while !self.check(&[TokenKind::Else, TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::Else, TokenKind::End]) {
                break;
            }
            then_branch.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        // Parse optional else branch
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

        self.expect(TokenKind::End, "Expected 'end' after unless statement")?;

        Ok(Statement::Unless {
            condition,
            then_branch,
            else_branch,
            position: start_pos,
        })
    }

    /// Parse a return statement
    pub(crate) fn parse_return_statement(&mut self) -> Result<Statement, MetorexError> {
        let pos = self
            .expect(TokenKind::Return, "Expected 'return'")?
            .position;
        self.skip_whitespace();

        // Check if there's a return value
        let value = if self.check(&[TokenKind::Newline, TokenKind::Semicolon, TokenKind::EOF])
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Statement::Return {
            value,
            position: pos,
        })
    }

    /// Parse a case statement (Ruby-style case/when)
    /// Syntax:
    ///   case expression
    ///   when pattern1
    ///     body1
    ///   when pattern2
    ///     body2
    ///   else
    ///     else_body
    ///   end
    pub(crate) fn parse_case_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Case, "Expected 'case'")?.position;
        self.skip_whitespace();

        // Parse the expression to match against
        let expression = self.parse_expression()?;
        self.skip_whitespace();

        // Parse when clauses
        let mut cases = Vec::new();
        loop {
            self.skip_whitespace(); // Skip whitespace before checking for when
            if !self.match_token(&[TokenKind::When]) {
                break;
            }
            let when_pos = self.previous().position;
            self.skip_whitespace();

            // Parse the pattern (for basic case statements, this is just a literal)
            let pattern = self.parse_case_pattern()?;
            self.skip_whitespace();

            // Parse optional guard clause (if ...)
            let guard = if self.match_token(&[TokenKind::If]) {
                self.skip_whitespace();
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.skip_whitespace();

            // Parse the body
            let mut body = Vec::new();
            while !self.check(&[TokenKind::When, TokenKind::Else, TokenKind::End])
                && !self.is_at_end()
            {
                self.skip_whitespace();
                if self.check(&[TokenKind::When, TokenKind::Else, TokenKind::End]) {
                    break;
                }
                body.push(self.parse_statement()?);
                self.skip_whitespace();
            }

            cases.push(MatchCase {
                pattern,
                guard,
                body,
                position: when_pos,
            });
        }

        // Parse optional else clause (as a wildcard pattern)
        self.skip_whitespace(); // Skip whitespace before checking for else
        if self.match_token(&[TokenKind::Else]) {
            let else_pos = self.previous().position;
            self.skip_whitespace();

            let mut else_body = Vec::new();
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::End]) {
                    break;
                }
                else_body.push(self.parse_statement()?);
                self.skip_whitespace();
            }

            // Add an else clause as a wildcard case
            cases.push(MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: else_body,
                position: else_pos,
            });
        }

        self.skip_whitespace(); // Skip whitespace before end
        self.expect(TokenKind::End, "Expected 'end' after case statement")?;

        Ok(Statement::Match {
            expression,
            cases,
            position: start_pos,
        })
    }

    /// Parse a pattern for a case statement
    /// Supports:
    /// - Literal patterns (integers, strings, booleans, nil)
    /// - Wildcard pattern (_)
    /// - Variable binding pattern (identifier)
    /// - Array destructuring ([a, b, c] or [first, ...rest])
    /// - Object destructuring ({x, y} or {x: a, y: b})
    fn parse_case_pattern(&mut self) -> Result<MatchPattern, MetorexError> {
        let token = self.peek().clone();

        match &token.kind {
            // Array pattern
            TokenKind::LBracket => {
                self.advance(); // consume '['
                self.skip_whitespace();

                let mut patterns = Vec::new();

                // Parse patterns inside the array
                while !self.check(&[TokenKind::RBracket]) && !self.is_at_end() {
                    self.skip_whitespace();

                    // Check for rest pattern (...)
                    if self.match_token(&[TokenKind::DotDotDot]) {
                        self.skip_whitespace();

                        // Next token should be an identifier for the rest binding
                        if let TokenKind::Ident(name) = &self.peek().kind {
                            let rest_name = name.clone();
                            self.advance();
                            patterns.push(MatchPattern::Rest(rest_name));
                        } else {
                            return Err(MetorexError::syntax_error(
                                "Expected identifier after ... in array pattern".to_string(),
                                SourceLocation::new(
                                    self.peek().position.line,
                                    self.peek().position.column,
                                    self.peek().position.offset,
                                ),
                            ));
                        }
                    } else {
                        // Parse a regular pattern
                        patterns.push(self.parse_case_pattern()?);
                    }

                    self.skip_whitespace();

                    // Check for comma
                    if !self.check(&[TokenKind::RBracket]) {
                        self.expect(TokenKind::Comma, "Expected ',' or ']' in array pattern")?;
                        self.skip_whitespace();
                    }
                }

                self.expect(TokenKind::RBracket, "Expected ']' after array pattern")?;
                Ok(MatchPattern::Array(patterns))
            }

            // Object/Dictionary pattern
            TokenKind::LBrace => {
                self.advance(); // consume '{'
                self.skip_whitespace();

                let mut key_patterns = Vec::new();

                // Parse key-pattern pairs inside the object
                while !self.check(&[TokenKind::RBrace]) && !self.is_at_end() {
                    self.skip_whitespace();

                    // Expect an identifier as the key
                    let key = if let TokenKind::Ident(name) = &self.peek().kind {
                        let k = name.clone();
                        self.advance();
                        k
                    } else if let TokenKind::String(s) = &self.peek().kind {
                        let k = s.clone();
                        self.advance();
                        k
                    } else {
                        return Err(MetorexError::syntax_error(
                            "Expected identifier or string key in object pattern".to_string(),
                            SourceLocation::new(
                                self.peek().position.line,
                                self.peek().position.column,
                                self.peek().position.offset,
                            ),
                        ));
                    };

                    self.skip_whitespace();

                    // Check if there's a colon for explicit pattern (e.g., {x: a, y: b})
                    let pattern = if self.match_token(&[TokenKind::Colon]) {
                        self.skip_whitespace();
                        self.parse_case_pattern()?
                    } else {
                        // Shorthand: {x, y} means {x: x, y: y}
                        MatchPattern::Identifier(key.clone())
                    };

                    key_patterns.push((key, pattern));

                    self.skip_whitespace();

                    // Check for comma
                    if !self.check(&[TokenKind::RBrace]) {
                        self.expect(TokenKind::Comma, "Expected ',' or '}' in object pattern")?;
                        self.skip_whitespace();
                    }
                }

                self.expect(TokenKind::RBrace, "Expected '}' after object pattern")?;
                Ok(MatchPattern::Object(key_patterns))
            }

            // Literal patterns
            TokenKind::Int(n) => {
                let value = *n;
                self.advance();
                Ok(MatchPattern::IntLiteral(value))
            }
            TokenKind::Float(f) => {
                let value = *f;
                self.advance();
                Ok(MatchPattern::FloatLiteral(value))
            }
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(MatchPattern::StringLiteral(value))
            }
            TokenKind::True => {
                self.advance();
                Ok(MatchPattern::BoolLiteral(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(MatchPattern::BoolLiteral(false))
            }
            TokenKind::Nil => {
                self.advance();
                Ok(MatchPattern::NilLiteral)
            }
            // Wildcard pattern
            TokenKind::Ident(name) if name == "_" => {
                self.advance();
                Ok(MatchPattern::Wildcard)
            }
            // Variable binding pattern
            TokenKind::Ident(name) => {
                let var_name = name.clone();
                self.advance();
                Ok(MatchPattern::Identifier(var_name))
            }
            _ => Err(MetorexError::syntax_error(
                format!("Expected pattern, found {:?}", token.kind),
                SourceLocation::new(
                    token.position.line,
                    token.position.column,
                    token.position.offset,
                ),
            )),
        }
    }
}
