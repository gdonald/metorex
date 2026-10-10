// The `if` and `unless` written after a statement rather than in
// front of it.

use super::*;

impl Parser {
    /// Check for postfix if/unless modifiers and wrap the statement.
    /// Only matches if the modifier is on the same line (no newline before it).
    /// A control-flow form read as a statement, unless a `.` follows its
    /// `end`. That marks the form as being read for its value, so it is
    /// parsed again as an expression and the chained call lands on what it
    /// answers.
    pub(crate) fn control_flow_statement(
        &mut self,
        position: crate::lexer::Position,
        parse: fn(&mut Self) -> Result<Statement, MetorexError>,
    ) -> Result<Statement, MetorexError> {
        let opened_at = self.stream.current_position();
        let parsed = parse(self)?;
        if !self.check(&[TokenKind::Dot]) {
            // `begin ... end while cond` and the rest of the modifiers read
            // the block they follow.
            return self.wrap_with_modifier(parsed);
        }
        self.stream.restore_position(opened_at);
        let expression = self.parse_expression_with_lambda()?;
        let stmt = Statement::Expression {
            expression,
            position,
        };
        self.wrap_with_modifier(stmt)
    }

    /// Apply the modifiers written after a statement, left to right, so
    /// `p 1 if ready unless stopped` tests `ready` inside `stopped`.
    pub(crate) fn wrap_with_modifier(
        &mut self,
        stmt: Statement,
    ) -> Result<Statement, MetorexError> {
        let mut wrapped = stmt;
        loop {
            let before = self.stream.current_position();
            wrapped = self.wrap_with_one_modifier(wrapped)?;
            if self.stream.current_position() == before {
                return Ok(wrapped);
            }
        }
    }

    fn wrap_with_one_modifier(&mut self, stmt: Statement) -> Result<Statement, MetorexError> {
        // Don't consume newlines — modifier must be on the same line
        if matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_)) {
            return Ok(stmt);
        }
        // `stmt rescue fallback` runs the fallback when the statement raises a
        // StandardError. A modifier binds to the statement it follows, so
        // nothing may come between them: a `rescue` on its own line opens a
        // clause, and so does one after a semicolon, even on the same line.
        if self.check(&[TokenKind::Rescue])
            && self.peek().position.line == self.previous().position.line
            && !matches!(
                self.previous().kind,
                TokenKind::Semicolon | TokenKind::Newline
            )
        {
            let position = self.advance().position;
            self.skip_whitespace();
            let fallback = self.parse_expression()?;
            return Ok(Statement::Expression {
                expression: crate::ast::Expression::BeginRescue {
                    body: vec![stmt],
                    rescue_clauses: vec![crate::ast::RescueClause {
                        exception_types: vec!["StandardError".to_string()],
                        variable_name: None,
                        variable_target: None,
                        splatted_types: Vec::new(),
                        body: vec![Statement::Expression {
                            expression: fallback,
                            position,
                        }],
                        position,
                    }],
                    else_clause: None,
                    ensure_block: None,
                    position,
                },
                position,
            });
        }
        if self.check(&[TokenKind::If]) {
            let position = self.advance().position; // consume 'if'
            self.skip_whitespace();
            let condition = self.parse_modifier_condition()?;
            Ok(Statement::If {
                condition,
                then_branch: vec![stmt],
                elsif_branches: vec![],
                else_branch: None,
                position,
            })
        } else if self.check(&[TokenKind::Unless]) {
            let position = self.advance().position; // consume 'unless'
            self.skip_whitespace();
            let condition = self.parse_modifier_condition()?;
            Ok(Statement::Unless {
                condition,
                then_branch: vec![stmt],
                else_branch: None,
                position,
            })
        } else if self.check(&[TokenKind::While]) {
            let position = self.advance().position; // consume 'while'
            // The statement is a loop body now, which a `redo`, `break` or
            // `next` in it belongs to.
            let body_start = stmt.position().offset;
            self.unlooped_jumps.retain(|(at, _)| at.offset < body_start);
            self.skip_whitespace();
            let condition = self.parse_modifier_condition()?;
            // `begin ... end while cond` reads its condition after the body
            // has run, so the body runs at least once.
            if runs_before_the_test(&stmt) {
                return Ok(Statement::DoWhile {
                    condition,
                    body: vec![stmt],
                    position,
                });
            }
            Ok(Statement::While {
                condition,
                body: vec![stmt],
                position,
            })
        } else if self.check(&[TokenKind::Until]) {
            let position = self.advance().position; // consume 'until'
            // The statement is a loop body now, which a `redo`, `break` or
            // `next` in it belongs to.
            let body_start = stmt.position().offset;
            self.unlooped_jumps.retain(|(at, _)| at.offset < body_start);
            self.skip_whitespace();
            let condition = self.parse_modifier_condition()?;
            let condition = crate::ast::Expression::UnaryOp {
                op: crate::ast::UnaryOp::Not,
                operand: Box::new(condition),
                position,
            };
            if runs_before_the_test(&stmt) {
                return Ok(Statement::DoWhile {
                    condition,
                    body: vec![stmt],
                    position,
                });
            }
            Ok(Statement::While {
                condition,
                body: vec![stmt],
                position,
            })
        } else {
            Ok(stmt)
        }
    }

    /// Parse a condition expression that may contain an assignment (`x = expr`).
    /// In Ruby, `if x = foo()` assigns and tests truthiness.
    pub(crate) fn parse_condition_expression(
        &mut self,
    ) -> Result<crate::ast::Expression, crate::error::MetorexError> {
        let expr = self.parse_condition_operands()?;
        if self.match_token(&[crate::lexer::TokenKind::Equal]) {
            self.skip_whitespace();
            let value = self.parse_expression()?;
            let position = expr.position();
            Ok(crate::ast::Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
                position,
            })
        } else {
            Ok(expr)
        }
    }

    /// The condition after a modifier `if`, `unless`, `while`, or `until`. A
    /// `rescue` after it belongs to the whole statement, so the condition
    /// leaves it alone.
    fn parse_modifier_condition(
        &mut self,
    ) -> Result<crate::ast::Expression, crate::error::MetorexError> {
        self.modifier_condition_depth += 1;
        let condition = self.parse_condition_expression();
        self.modifier_condition_depth -= 1;
        if let Ok(read) = &condition {
            self.warn_literal_condition(read);
        }
        condition
    }

    /// The condition a modifier reads, where `and` and `or` join the tests the
    /// way they do in a statement of their own.
    pub(crate) fn parse_condition_operands(
        &mut self,
    ) -> Result<crate::ast::Expression, crate::error::MetorexError> {
        use crate::lexer::TokenKind as Kind;
        let mut held = self.parse_expression()?;
        loop {
            let op = if self.check(&[Kind::KeywordAnd]) {
                crate::ast::BinaryOp::And
            } else if self.check(&[Kind::KeywordOr]) {
                crate::ast::BinaryOp::Or
            } else {
                return Ok(held);
            };
            let position = self.advance().position;
            self.skip_whitespace();
            let right = self.parse_expression()?;
            held = crate::ast::Expression::BinaryOp {
                op,
                left: Box::new(held),
                right: Box::new(right),
                position,
            };
        }
    }
}
