// Reading a `case` statement, on values and on patterns.

use super::*;

impl Parser {
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
    /// `case` written with no subject, where each `when` holds conditions
    /// rather than patterns. Ruby reads it as the `if` chain it stands for.
    pub(crate) fn parse_subjectless_case(
        &mut self,
        start_pos: crate::lexer::Position,
    ) -> Result<Statement, MetorexError> {
        let mut branches: Vec<crate::ast::ElsifBranch> = Vec::new();
        loop {
            self.skip_whitespace();
            if !self.match_token(&[TokenKind::When]) {
                break;
            }
            let when_pos = self.previous().position;
            self.skip_whitespace();
            let mut condition = when_truth_test(self.parse_expression()?);
            while self.match_token(&[TokenKind::Comma]) {
                self.skip_whitespace();
                let another = when_truth_test(self.parse_expression()?);
                condition = Expression::BinaryOp {
                    op: crate::ast::BinaryOp::Or,
                    left: Box::new(condition),
                    right: Box::new(another),
                    position: when_pos,
                };
            }
            self.skip_whitespace();
            self.match_token(&[TokenKind::Then]);
            branches.push(crate::ast::ElsifBranch {
                condition,
                body: self.parse_clause_body()?,
                position: when_pos,
            });
        }
        self.skip_whitespace();
        let else_branch = if self.match_token(&[TokenKind::Else]) {
            self.skip_whitespace();
            Some(self.parse_clause_body()?)
        } else {
            None
        };
        self.skip_whitespace();
        self.expect(TokenKind::End, "Expected 'end' after case statement")?;

        let mut branches = branches.into_iter();
        let first = branches
            .next()
            .expect("a case with no subject holds at least one when");
        Ok(Statement::If {
            condition: first.condition,
            then_branch: first.body,
            elsif_branches: branches.collect(),
            else_branch,
            position: start_pos,
        })
    }

    /// The statements of one `when` or `else` clause, read up to whatever
    /// closes it.
    pub(crate) fn parse_clause_body(&mut self) -> Result<Vec<Statement>, MetorexError> {
        let mut body = Vec::new();
        loop {
            self.skip_whitespace();
            if self.check(&[TokenKind::When, TokenKind::Else, TokenKind::End]) || self.is_at_end() {
                break;
            }
            body.push(self.parse_statement()?);
        }
        Ok(body)
    }

    pub(crate) fn parse_case_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Case, "Expected 'case'")?.position;
        self.parse_case_after_keyword(start_pos)
    }

    /// A `case` statement from just after its keyword: the subject, if
    /// any, then its `when` or `in` clauses.
    pub(crate) fn parse_case_after_keyword(
        &mut self,
        start_pos: Position,
    ) -> Result<Statement, MetorexError> {
        self.skip_whitespace();

        // `case` with nothing to match against tests each `when` for truth,
        // which is the `if` chain it stands for.
        self.skip_whitespace();
        if self.check(&[TokenKind::When]) {
            return self.parse_subjectless_case(start_pos);
        }
        // Parse the expression to match against. A `case name = value` binds
        // the name in the scope holding the case, so the assignment is part
        // of the subject.
        self.refuse_pattern_test += 1;
        let expression = self.parse_condition();
        self.refuse_pattern_test -= 1;
        let expression = expression?;
        self.skip_whitespace();

        // Detect whether this is case/when or case/in
        if self.check(&[TokenKind::In]) {
            return self.parse_case_in_body(expression, start_pos);
        }

        // A case written with an `else` and no `when` is refused, since Ruby
        // has nothing to compare the subject against.
        self.skip_whitespace();
        if self.check(&[TokenKind::Else]) {
            return Err(MetorexError::syntax_error(
                "else without rescue is useless".to_string(),
                SourceLocation::new(
                    self.peek().position.line,
                    self.peek().position.column,
                    self.peek().position.offset,
                ),
            ));
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
            let pattern = self.parse_when_pattern_with_alternatives()?;

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
    pub(in crate::parser) fn parse_case_in_body(
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
            // A pattern holds no operators of its own, so anything left on
            // the line after one was never part of it.
            if !self.check(&[
                TokenKind::Then,
                TokenKind::If,
                TokenKind::Unless,
                TokenKind::Newline,
                TokenKind::Semicolon,
                TokenKind::Comment(String::new()),
                TokenKind::EOF,
            ]) && !matches!(self.peek().kind, TokenKind::Comment(_))
            {
                return Err(self.error_at_current(
                    "expected a delimiter after the patterns of an `in` clause",
                ));
            }
            self.skip_whitespace();

            // Parse optional guard clause, which may refuse a match as well
            // as ask for one.
            let guard = if self.match_token(&[TokenKind::If]) {
                self.skip_whitespace();
                Some(self.parse_expression()?)
            } else if self.match_token(&[TokenKind::Unless]) {
                let at = self.previous().position;
                self.skip_whitespace();
                let held = self.parse_expression()?;
                Some(Expression::UnaryOp {
                    op: crate::ast::UnaryOp::Not,
                    operand: Box::new(held),
                    position: at,
                })
            } else {
                None
            };
            self.skip_whitespace();
            // `in pattern then body` writes the body on the same line.
            self.match_token(&[TokenKind::Then]);
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
}

/// The test a `when` in a subjectless case stands for. A splat names each of
/// the values it holds as a choice of its own, so the branch is taken when
/// any one of them is true rather than when the list itself is.
pub(crate) fn when_truth_test(condition: Expression) -> Expression {
    if !matches!(condition, Expression::Splat { .. }) {
        return condition;
    }
    let position = condition.position();
    Expression::MethodCall {
        receiver: Box::new(Expression::Array {
            elements: vec![condition],
            position,
        }),
        method: "any?".to_string(),
        arguments: Vec::new(),
        trailing_block: None,
        position,
    }
}
