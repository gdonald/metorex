// Control flow statement parsing (if, while, for, case)

use crate::ast::{ElsifBranch, Expression, MatchCase, MatchPattern, Statement};
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::{Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse an if statement
    pub(crate) fn parse_if_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::If, "Expected 'if'")?.position;
        self.skip_whitespace();

        let condition = self.parse_condition()?;
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

            let elsif_condition = self.parse_condition()?;
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

        let condition = self.parse_condition()?;
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
        let stmt = Statement::Break { position: pos };
        self.wrap_with_modifier(stmt)
    }

    /// Parse a continue statement
    pub(crate) fn parse_continue_statement(&mut self) -> Result<Statement, MetorexError> {
        let pos = self
            .expect(TokenKind::Continue, "Expected 'continue'")?
            .position;
        let stmt = Statement::Continue { position: pos };
        self.wrap_with_modifier(stmt)
    }

    /// Parse an unless statement
    pub(crate) fn parse_unless_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Unless, "Expected 'unless'")?
            .position;
        self.skip_whitespace();

        let condition = self.parse_condition()?;
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
        // Do NOT skip newlines here — bare `return` followed by a newline
        // (or `end`/`}` on the next line) means a value-less return.
        // We can however skip horizontal whitespace, which `skip_whitespace`
        // doesn't see (the lexer already swallowed spaces and tabs).

        // Check if there's a return value
        let value = if self.check(&[
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::EOF,
            TokenKind::If,
            TokenKind::Unless,
            TokenKind::End,
            TokenKind::RBrace,
            TokenKind::Else,
            TokenKind::Elsif,
            TokenKind::When,
            TokenKind::Rescue,
            TokenKind::Ensure,
        ]) || self.is_at_end()
        {
            None
        } else {
            let mut first = self.parse_expression()?;
            // Allow assignment in return value: `return x = value`
            if self.check(&[TokenKind::Equal])
                && matches!(
                    first,
                    Expression::Identifier { .. }
                        | Expression::InstanceVariable { .. }
                        | Expression::ClassVariable { .. }
                        | Expression::GlobalVariable { .. }
                        | Expression::Index { .. }
                )
            {
                let eq_pos = self.advance().position;
                self.skip_whitespace();
                let value = self.parse_expression()?;
                first = Expression::BinaryOp {
                    op: crate::ast::BinaryOp::Assign,
                    left: Box::new(first),
                    right: Box::new(value),
                    position: eq_pos,
                };
            }
            if self.match_token(&[TokenKind::Comma]) {
                // Multiple return values: return a, b, c → return [a, b, c]
                let mut elements = vec![first];
                loop {
                    self.skip_whitespace();
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
                Some(Expression::Array {
                    elements,
                    position: pos,
                })
            } else {
                Some(first)
            }
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

        // Detect whether this is case/when or case/in
        if self.check(&[TokenKind::In]) {
            return self.parse_case_in_body(expression, start_pos);
        }

        // Parse when clauses
        let mut cases = Vec::new();
        loop {
            self.skip_whitespace(); // Skip whitespace before checking for when
            if !self.match_token(&[TokenKind::When]) {
                break;
            }
            let when_pos = self.previous().position;
            self.skip_whitespace();

            // Parse the pattern (may include comma-separated alternatives).
            // Note: do NOT call skip_whitespace here — we need to be able to
            // distinguish a guard on the same line (`when pat if guard`) from
            // an `if` statement on the next line (which is a body statement).
            let pattern = self.parse_case_pattern_with_alternatives()?;

            // Parse optional guard clause (`if expr`) on the SAME line as the
            // pattern. We only consume horizontal whitespace, not newlines.
            let guard = if matches!(self.peek().kind, TokenKind::If) {
                self.advance(); // consume `if`
                self.skip_whitespace();
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.skip_whitespace();
            // Consume optional `then` keyword (allows inline: `when 1, 2 then body`).
            self.match_token(&[TokenKind::Then]);
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

    /// Parse the body of a `case/in` statement (Ruby 2.7+ pattern matching).
    /// Called after `case expr` has been consumed and `in` is the next token.
    fn parse_case_in_body(
        &mut self,
        expression: Expression,
        start_pos: Position,
    ) -> Result<Statement, MetorexError> {
        let mut cases = Vec::new();

        loop {
            self.skip_whitespace();
            if !self.match_token(&[TokenKind::In]) {
                break;
            }
            let in_pos = self.previous().position;
            self.skip_whitespace();

            // Parse the pattern (supports `=> name` binding)
            let pattern = self.parse_case_in_pattern()?;
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
            while !self.check(&[TokenKind::In, TokenKind::Else, TokenKind::End])
                && !self.is_at_end()
            {
                self.skip_whitespace();
                if self.check(&[TokenKind::In, TokenKind::Else, TokenKind::End]) {
                    break;
                }
                body.push(self.parse_statement()?);
                self.skip_whitespace();
            }

            cases.push(MatchCase {
                pattern,
                guard,
                body,
                position: in_pos,
            });
        }

        // Parse optional else clause (as a wildcard pattern)
        self.skip_whitespace();
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

            cases.push(MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: else_body,
                position: else_pos,
            });
        }

        self.skip_whitespace();
        self.expect(TokenKind::End, "Expected 'end' after case/in statement")?;

        Ok(Statement::CaseIn {
            expression,
            cases,
            position: start_pos,
        })
    }

    /// Parse a single pattern for `case/in`, supporting `pattern => name` binding
    /// at the top level and inside array patterns.
    fn parse_case_in_pattern(&mut self) -> Result<MatchPattern, MetorexError> {
        self.parse_case_in_pattern_inner()
    }

    /// Recursive inner parser for `case/in` patterns with `=> name` bind support.
    fn parse_case_in_pattern_inner(&mut self) -> Result<MatchPattern, MetorexError> {
        let token = self.peek().clone();

        let inner = match &token.kind {
            // Array pattern — parse each element with bind support
            TokenKind::LBracket => {
                self.advance();
                self.skip_whitespace();
                let mut patterns = Vec::new();
                while !self.check(&[TokenKind::RBracket]) && !self.is_at_end() {
                    self.skip_whitespace();
                    if self.match_token(&[TokenKind::DotDotDot]) {
                        self.skip_whitespace();
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
                        patterns.push(self.parse_case_in_pattern_inner()?);
                    }
                    self.skip_whitespace();
                    if !self.check(&[TokenKind::RBracket]) {
                        self.expect(TokenKind::Comma, "Expected ',' or ']' in array pattern")?;
                        self.skip_whitespace();
                    }
                }
                self.expect(TokenKind::RBracket, "Expected ']' after array pattern")?;
                MatchPattern::Array(patterns)
            }
            // For all other patterns, delegate to the base parser
            _ => self.parse_case_pattern()?,
        };

        self.skip_whitespace();

        // Check for `=> name` binding
        if self.match_token(&[TokenKind::FatArrow]) {
            self.skip_whitespace();
            let name = if let TokenKind::Ident(n) = &self.peek().kind {
                let n = n.clone();
                self.advance();
                n
            } else {
                return Err(self.error_at_current("Expected identifier after '=>' in pattern"));
            };
            Ok(MatchPattern::Bind {
                pattern: Box::new(inner),
                name,
            })
        } else {
            Ok(inner)
        }
    }

    /// Parse a pattern for a case statement
    /// Supports:
    /// - Literal patterns (integers, strings, booleans, nil)
    /// - Wildcard pattern (_)
    /// - Variable binding pattern (identifier)
    /// - Array destructuring ([a, b, c] or [first, ...rest])
    /// - Object destructuring ({x, y} or {x: a, y: b})
    ///
    /// This method is public within the parser module so it can be used
    /// by both statement parsing (case statements) and expression parsing (case expressions)
    /// Parse a pattern that may include comma-separated alternatives
    /// Returns a MatchPattern::Multiple if multiple patterns are found, otherwise a single pattern
    pub(in crate::parser) fn parse_case_pattern_with_alternatives(
        &mut self,
    ) -> Result<MatchPattern, MetorexError> {
        // Parse the first pattern. Do NOT skip newlines after the pattern —
        // the caller relies on being able to distinguish a same-line `if`
        // (a guard clause) from a body `if` statement on the next line.
        let first_pattern = self.parse_case_pattern()?;

        // Check if there's a comma indicating multiple patterns
        if !self.check(&[TokenKind::Comma]) {
            // Single pattern, return as-is
            return Ok(first_pattern);
        }

        // Multiple patterns: collect all comma-separated patterns
        let mut patterns = vec![first_pattern];

        while self.match_token(&[TokenKind::Comma]) {
            self.skip_whitespace();

            // Check if we've hit a terminal token (then, if, newline context)
            // This prevents consuming commas from other constructs
            if self.check(&[
                TokenKind::Then,
                TokenKind::If,
                TokenKind::When,
                TokenKind::Else,
                TokenKind::End,
            ]) {
                break;
            }

            patterns.push(self.parse_case_pattern()?);
            self.skip_whitespace();
        }

        // If we only collected one pattern, return it directly
        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            Ok(MatchPattern::Multiple(patterns))
        }
    }

    /// Parse a single case pattern (internal helper)
    pub(in crate::parser) fn parse_case_pattern(&mut self) -> Result<MatchPattern, MetorexError> {
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

            // Literal patterns (may be followed by .. or ... to form a range pattern)
            TokenKind::Int(n) => {
                let value = *n;
                self.advance();
                let start = MatchPattern::IntLiteral(value);
                self.parse_range_pattern_suffix(start)
            }
            TokenKind::Float(f) => {
                let value = *f;
                self.advance();
                let start = MatchPattern::FloatLiteral(value);
                self.parse_range_pattern_suffix(start)
            }
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                let start = MatchPattern::StringLiteral(value);
                self.parse_range_pattern_suffix(start)
            }
            // Symbol pattern (:name)
            TokenKind::Colon => {
                self.advance();
                let name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    TokenKind::InstanceVar(n) => format!("@{}", n),
                    TokenKind::ClassVar(n) => format!("@@{}", n),
                    _ => {
                        return Err(
                            self.error_at_previous("Expected identifier after ':' in pattern")
                        );
                    }
                };
                Ok(MatchPattern::SymbolLiteral(name))
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
            // Type pattern (capitalized identifiers like Integer, String, Hash, Array)
            TokenKind::Ident(name) if name.chars().next().is_some_and(|c| c.is_uppercase()) => {
                let type_name = name.clone();
                self.advance();
                Ok(MatchPattern::Type(type_name))
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

    /// If `..` or `...` follows a literal pattern, wrap it in a Range pattern.
    fn parse_range_pattern_suffix(
        &mut self,
        start: MatchPattern,
    ) -> Result<MatchPattern, MetorexError> {
        if self.match_token(&[TokenKind::DotDotDot]) {
            let end = self.parse_case_pattern()?;
            Ok(MatchPattern::Range {
                start: Box::new(start),
                end: Box::new(end),
                exclusive: true,
            })
        } else if self.match_token(&[TokenKind::DotDot]) {
            let end = self.parse_case_pattern()?;
            Ok(MatchPattern::Range {
                start: Box::new(start),
                end: Box::new(end),
                exclusive: false,
            })
        } else {
            Ok(start)
        }
    }
}
