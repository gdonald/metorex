// Reading a `break`, a `next`, a `redo` and a `return`.

use super::*;

impl Parser {
    /// Parse a break statement, optionally with a value (e.g. `break 42`).
    pub(crate) fn parse_break_statement(&mut self) -> Result<Statement, MetorexError> {
        let pos = self.expect(TokenKind::Break, "Expected 'break'")?.position;
        // A value only applies when the very next token is an expression on
        // the same line — stop if we hit a statement terminator or an `if`/
        // `unless`/`while`/`until` modifier.
        let value = if self.is_at_end()
            || self.check(&[
                TokenKind::Newline,
                TokenKind::Semicolon,
                TokenKind::End,
                TokenKind::EOF,
                TokenKind::If,
                TokenKind::Unless,
                TokenKind::While,
                TokenKind::Until,
                TokenKind::RBrace,
                TokenKind::RParen,
                TokenKind::Comment(String::new()),
            ])
            || (self.ternary_depth > 0 && self.check(&[TokenKind::Colon]))
        {
            None
        } else {
            Some(Box::new(self.parse_jump_value()?))
        };
        let stmt = Statement::Break {
            value,
            position: pos,
        };
        self.wrap_with_modifier(stmt)
    }

    /// Parse a continue statement, optionally with a value (e.g. `next 42`).
    pub(crate) fn parse_continue_statement(&mut self) -> Result<Statement, MetorexError> {
        let pos = self
            .expect(TokenKind::Continue, "Expected 'continue'")?
            .position;
        // A `next` in a method body, with no loop or block around it, has
        // nothing to jump to.
        if self.def_body_depth > 0 && self.jump_target_depth == 0 && !self.in_defined_argument {
            return Err(MetorexError::syntax_error(
                "Invalid next".to_string(),
                SourceLocation::new(pos.line, pos.column, pos.offset),
            ));
        }
        let value = if self.is_at_end()
            || self.check(&[
                TokenKind::Newline,
                TokenKind::Semicolon,
                TokenKind::End,
                TokenKind::EOF,
                TokenKind::If,
                TokenKind::Unless,
                TokenKind::While,
                TokenKind::Until,
                TokenKind::RBrace,
                TokenKind::RParen,
                TokenKind::Comment(String::new()),
            ])
            || (self.ternary_depth > 0 && self.check(&[TokenKind::Colon]))
        {
            None
        } else {
            // `next 1, 2, 3` hands the block an Array of what follows.
            let first = self.parse_jump_value()?;
            let mut values = vec![first];
            while self.match_token(&[TokenKind::Comma]) {
                self.skip_whitespace();
                values.push(self.parse_expression()?);
            }
            if values.len() == 1 {
                Some(Box::new(values.remove(0)))
            } else {
                Some(Box::new(crate::ast::Expression::Array {
                    elements: values,
                    position: pos,
                }))
            }
        };
        let stmt = Statement::Continue {
            value,
            position: pos,
        };
        self.wrap_with_modifier(stmt)
    }

    /// The value a `break`, `next`, or `return` carries. `and` and `or` bind
    /// more loosely than the jump, so one written after the value has nothing
    /// on its left to read, which Ruby refuses.
    pub(crate) fn parse_jump_value(&mut self) -> Result<Expression, MetorexError> {
        self.assignment_rhs_depth += 1;
        let parsed = self.parse_expression();
        self.assignment_rhs_depth -= 1;
        let value = parsed?;
        if self.check(&[TokenKind::KeywordAnd, TokenKind::KeywordOr]) {
            return Err(self.error_at_current("void value expression"));
        }
        Ok(value)
    }

    /// Parse a redo statement, which re-runs the enclosing body from the top.
    /// `retry` runs the begin body its rescue clause belongs to again. It is
    /// only written inside a rescue body, and Ruby refuses it anywhere else
    /// while the source is still being read.
    pub(crate) fn parse_retry_statement(&mut self) -> Result<Statement, MetorexError> {
        let pos = self.expect(TokenKind::Retry, "Expected 'retry'")?.position;
        if self.rescue_depth == 0 && !self.in_defined_argument {
            return Err(MetorexError::syntax_error(
                "Invalid retry without rescue",
                crate::error::SourceLocation::new(pos.line, pos.column, pos.offset),
            ));
        }
        self.wrap_with_modifier(Statement::Retry { position: pos })
    }

    pub(crate) fn parse_redo_statement(&mut self) -> Result<Statement, MetorexError> {
        let pos = self.expect(TokenKind::Redo, "Expected 'redo'")?.position;
        self.wrap_with_modifier(Statement::Redo { position: pos })
    }

    /// Parse an unless statement
    pub(crate) fn parse_unless_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Unless, "Expected 'unless'")?
            .position;
        self.skip_whitespace();

        let condition = self.parse_condition()?;
        self.skip_whitespace();
        // `unless cond then` may hold its body on the following line.
        self.match_token(&[TokenKind::Then]);
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
            // `defined?(return)` names the jump without giving it a value.
            TokenKind::RParen,
        ]) || self.is_at_end()
        {
            None
        } else {
            let mut first = self.parse_jump_value()?;
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
}
