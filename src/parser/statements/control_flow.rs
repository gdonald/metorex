// Control flow statement parsing (if, while, for)

use crate::ast::{ElsifBranch, Statement};
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
}
