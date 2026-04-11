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
