// Control-flow expression parsing: `if`, `unless`, `case` as expressions.

use crate::ast::Expression;
use crate::ast::node::{ElsifBranch, ExprMatchCase};
use crate::error::MetorexError;
use crate::lexer::{Position, TokenKind};
use crate::parser::Parser;

impl Parser {
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
        start_pos: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();

        // `case` with nothing to match against tests each `when` for truth,
        // which is the `if` chain it stands for.
        if self.check(&[TokenKind::When]) {
            let held = self.parse_subjectless_case(start_pos)?;
            return Ok(Expression::BeginRescue {
                body: vec![held],
                rescue_clauses: Vec::new(),
                else_clause: None,
                ensure_block: None,
                position: start_pos,
            });
        }
        // Parse the expression to match against
        self.refuse_pattern_test += 1;
        let subject = self.parse_expression();
        self.refuse_pattern_test -= 1;
        let expression = Box::new(subject?);
        self.skip_whitespace();

        // A `case` written with `in` matches patterns rather than comparing
        // values, which the statement form already reads.
        if self.check(&[TokenKind::In]) {
            let held = self.parse_case_in_body(*expression, start_pos)?;
            return Ok(Expression::BeginRescue {
                body: vec![held],
                rescue_clauses: Vec::new(),
                else_clause: None,
                ensure_block: None,
                position: start_pos,
            });
        }

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
            let pattern = self.parse_when_pattern_with_alternatives()?;

            // A guard stands on the line of the values it guards, so an `if`
            // on the next line opens the clause's body instead.
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
            self.match_token(&[TokenKind::Then]);
            self.skip_whitespace();
            // A clause with nothing written under it answers nil.
            let body = if self.check(&[TokenKind::When, TokenKind::Else, TokenKind::End]) {
                Expression::NilLiteral { position: when_pos }
            } else {
                self.parse_clause_value(when_pos)?
            };

            cases.push(ExprMatchCase {
                pattern,
                guard,
                body,
                position: when_pos,
            });

            self.skip_whitespace();
        }

        // Parse optional else clause, which answers nil when nothing is
        // written under it.
        let else_case = if self.match_token(&[TokenKind::Else]) {
            let else_pos = self.previous().position;
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                Some(Box::new(Expression::NilLiteral { position: else_pos }))
            } else {
                Some(Box::new(self.parse_clause_value(else_pos)?))
            }
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

    /// What a `when` or `else` clause of a `case` expression answers: its
    /// one expression, or the statements under it run as a `begin` body.
    fn parse_clause_value(&mut self, position: Position) -> Result<Expression, MetorexError> {
        let first = self.parse_expression_with_assignment()?;
        self.skip_whitespace();
        if self.check(&[TokenKind::When, TokenKind::Else, TokenKind::End]) || self.is_at_end() {
            return Ok(first);
        }
        let first_position = first.position();
        let mut body = vec![crate::ast::Statement::Expression {
            expression: first,
            position: first_position,
        }];
        body.extend(self.parse_clause_body()?);
        Ok(Expression::BeginRescue {
            body,
            rescue_clauses: Vec::new(),
            else_clause: None,
            ensure_block: None,
            position,
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

impl crate::parser::Parser {
    /// A loop read for its value. The loop stays a statement, held in a body
    /// so what surrounds it can chain onto what the loop answered.
    /// `for ... end` read as an expression, which is what a call written on
    /// the loop's own `end` reads it as.
    pub(super) fn parse_for_expression(
        &mut self,
        position: crate::lexer::Position,
    ) -> Result<Expression, MetorexError> {
        let keyword = self.stream.current_position().saturating_sub(1);
        self.stream.restore_position(keyword);
        let looped = self.parse_for_statement()?;
        Ok(Expression::BeginRescue {
            body: vec![looped],
            rescue_clauses: Vec::new(),
            else_clause: None,
            ensure_block: None,
            position,
        })
    }

    pub(super) fn parse_loop_expression(
        &mut self,
        position: crate::lexer::Position,
    ) -> Result<Expression, MetorexError> {
        // The keyword was already stepped over on the way here, so the walk
        // goes back to it and the statement parser reads the whole loop.
        let keyword = self.stream.current_position().saturating_sub(1);
        let until = matches!(self.previous().kind, TokenKind::Until);
        self.stream.restore_position(keyword);
        let looped = if until {
            self.parse_until_statement()?
        } else {
            self.parse_while_statement()?
        };
        Ok(Expression::BeginRescue {
            body: vec![looped],
            rescue_clauses: Vec::new(),
            else_clause: None,
            ensure_block: None,
            position,
        })
    }
}
