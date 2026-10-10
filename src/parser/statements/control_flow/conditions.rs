// Reading an `if`, an `unless`, and the loops written with a test.

use super::*;

impl Parser {
    /// Parse an if statement
    pub(crate) fn parse_if_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::If, "Expected 'if'")?.position;
        self.skip_whitespace();

        let condition = self.parse_tested_condition()?;
        self.skip_whitespace();
        // `if cond then` may hold its body on the following line.
        self.match_token(&[TokenKind::Then]);
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

            let elsif_condition = self.parse_tested_condition()?;
            self.skip_whitespace();
            // `elsif cond then` may hold its body on the same line.
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

        self.expect_closing(
            TokenKind::End,
            "Expected 'end' after if statement",
            start_pos,
        )?;

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

        let condition = self.parse_tested_condition()?;
        self.skip_whitespace();

        // Optionally consume 'do'
        self.match_token(&[TokenKind::Do]);
        self.skip_whitespace();

        // Parse loop body
        let mut body = Vec::new();
        self.jump_target_depth += 1;
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }
        self.jump_target_depth -= 1;

        self.expect(TokenKind::End, "Expected 'end' after while loop")?;

        Ok(Statement::While {
            condition,
            body,
            position: start_pos,
        })
    }

    /// Parse `until cond ... end`, compiled to a `while !cond` loop.
    pub(crate) fn parse_until_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Until, "Expected 'until'")?.position;
        self.skip_whitespace();

        let condition = self.parse_tested_condition()?;
        self.skip_whitespace();
        self.match_token(&[TokenKind::Do]);
        self.skip_whitespace();

        let mut body = Vec::new();
        self.jump_target_depth += 1;
        let collected = (|| -> Result<(), MetorexError> {
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::End]) {
                    break;
                }
                body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Ok(())
        })();
        self.jump_target_depth -= 1;
        collected?;
        self.expect(TokenKind::End, "Expected 'end' after until loop")?;

        Ok(Statement::While {
            condition: crate::ast::Expression::UnaryOp {
                op: crate::ast::UnaryOp::Not,
                operand: Box::new(condition),
                position: start_pos,
            },
            body,
            position: start_pos,
        })
    }

    /// Parse a for loop
    pub(crate) fn parse_for_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::For, "Expected 'for'")?.position;
        self.skip_whitespace();

        // Parse the loop variables. Ruby names one or more assignment
        // targets here, and a comma with nothing after it is a target list
        // that takes the first element alone.
        let mut targets = vec![self.parse_for_target()?];
        let mut destructures = false;
        loop {
            self.skip_whitespace();
            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
            destructures = true;
            self.skip_whitespace();
            if self.check(&[TokenKind::In]) {
                break;
            }
            targets.push(self.parse_for_target()?);
        }

        self.skip_whitespace();

        // Expect 'in' keyword
        self.expect(TokenKind::In, "Expected 'in' after loop variable")?;
        for target in &targets {
            self.declare_target(target);
        }
        self.skip_whitespace();

        // Parse the iterable expression
        let iterable = self.parse_expression()?;
        self.skip_whitespace();

        // Optionally consume 'do'
        self.match_token(&[TokenKind::Do]);
        self.skip_whitespace();

        // Parse loop body
        let mut body = Vec::new();
        self.jump_target_depth += 1;
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }
        self.jump_target_depth -= 1;

        self.expect(TokenKind::End, "Expected 'end' after for loop")?;

        // A loop naming one plain local keeps its own node, which the
        // bytecode compiler reads. The rest stand for the `each` call that
        // binds their targets.
        if !destructures && let [Expression::Identifier { name, .. }] = targets.as_slice() {
            return Ok(Statement::For {
                variable: name.clone(),
                iterable,
                body,
                position: start_pos,
            });
        }
        Ok(crate::ast::for_loop::for_over_each(
            targets,
            destructures,
            iterable,
            body,
            start_pos,
        ))
    }

    /// One assignment target named between `for` and `in`. A bare `*` stands
    /// for the elements no other target takes, under a name nothing reads.
    pub(crate) fn parse_for_target(&mut self) -> Result<Expression, MetorexError> {
        if self.check(&[TokenKind::Star]) {
            let star = self.advance();
            self.skip_whitespace();
            let discards = self.check(&[TokenKind::In, TokenKind::Comma]);
            let expression = if discards {
                Expression::Identifier {
                    name: "_".to_string(),
                    position: star.position,
                }
            } else {
                self.parse_expression_with_lambda()?
            };
            return Ok(Expression::Splat {
                expression: Box::new(expression),
                position: star.position,
            });
        }
        let target = self.parse_expression_with_lambda()?;
        // Only a form a value can be stored in names a loop variable.
        if !matches!(
            target,
            Expression::Identifier { .. }
                | Expression::InstanceVariable { .. }
                | Expression::ClassVariable { .. }
                | Expression::GlobalVariable { .. }
                | Expression::MethodCall { .. }
                | Expression::Index { .. }
                | Expression::ScopeResolution { .. }
        ) {
            return Err(self.error_at_previous("Cannot assign to this expression"));
        }
        Ok(target)
    }
}
