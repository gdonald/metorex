// Statement parsing module
// Handles parsing of all statement types

mod attributes;
mod class;
mod control_flow;
mod exception;
mod function;

use crate::ast::{BinaryOp, Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse a single statement
    pub(crate) fn parse_statement(&mut self) -> Result<Statement, MetorexError> {
        // Skip leading whitespace
        self.skip_whitespace();

        // Recognise `private def …` / `public def …` / `protected def …` /
        // `module_function def …` as a special two-keyword form. The
        // visibility modifier is parsed and discarded; the wrapped `def`
        // becomes a normal method definition.
        if let TokenKind::Ident(name) = &self.peek().kind
            && matches!(
                name.as_str(),
                "private" | "public" | "protected" | "module_function"
            )
            && matches!(self.peek_ahead(1).kind, TokenKind::Def)
        {
            self.advance(); // consume the visibility ident
            self.skip_whitespace();
            return self.parse_function_def();
        }

        let token = self.peek().clone();
        match &token.kind {
            TokenKind::Class => {
                // `class << target; …; end` is an expression whose value is the
                // singleton class, so it may be chained (`.ancestors`, `==`, …).
                // Route it through the expression parser; `class Name` remains
                // a statement-level definition.
                if matches!(self.peek_ahead(1).kind, TokenKind::Shovel) {
                    let expr = self.parse_expression_with_lambda()?;
                    let stmt = Statement::Expression {
                        expression: expr,
                        position: token.position,
                    };
                    self.wrap_with_modifier(stmt)
                } else {
                    self.parse_class_def()
                }
            }
            TokenKind::Def => self.parse_function_def(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::Unless => self.parse_unless_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Until => self.parse_until_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Case => self.parse_case_statement(),
            TokenKind::Begin => self.parse_begin_statement(),
            TokenKind::Raise => self.parse_raise_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Return => {
                let stmt = self.parse_return_statement()?;
                self.wrap_with_modifier(stmt)
            }
            TokenKind::AttrReader => self.parse_attr_reader(),
            TokenKind::AttrWriter => self.parse_attr_writer(),
            TokenKind::AttrAccessor => self.parse_attr_accessor(),
            TokenKind::Module => self.parse_module_def(),
            TokenKind::Include => self.parse_include(),
            TokenKind::Extend => self.parse_extend(),
            TokenKind::Alias => self.parse_alias(),
            _ => {
                // Try to parse as an expression or assignment (including arrow lambdas)
                let expr = self.parse_expression_with_lambda()?;

                // Check for multiple assignment: a, b, c = ...
                // Look ahead to verify there's an = after the comma-separated identifiers
                if matches!(
                    expr,
                    Expression::Identifier { .. }
                        | Expression::Index { .. }
                        | Expression::InstanceVariable { .. }
                        | Expression::ClassVariable { .. }
                        | Expression::GlobalVariable { .. }
                ) && self.check(&[TokenKind::Comma])
                    && self.is_multiple_assignment()
                {
                    let mut targets = vec![expr];
                    while self.match_token(&[TokenKind::Comma]) {
                        self.skip_whitespace();
                        targets.push(self.parse_expression_with_lambda()?);
                    }
                    self.expect(TokenKind::Equal, "Expected '=' in multiple assignment")?;
                    self.skip_whitespace();
                    let mut values = vec![self.parse_expression_with_lambda()?];
                    while self.match_token(&[TokenKind::Comma]) {
                        self.skip_whitespace();
                        values.push(self.parse_expression_with_lambda()?);
                    }
                    let stmt = Statement::MultipleAssignment {
                        targets,
                        values,
                        position: token.position,
                    };
                    return self.wrap_with_modifier(stmt);
                }

                // Check if this is an assignment
                if self.check(&[
                    TokenKind::Equal,
                    TokenKind::PlusEqual,
                    TokenKind::MinusEqual,
                    TokenKind::StarEqual,
                    TokenKind::SlashEqual,
                    TokenKind::LogicalOrAssign,
                    TokenKind::LogicalAndAssign,
                ]) {
                    let op_token = self.advance();
                    let value = self.parse_assignment_rhs()?;

                    // Convert compound assignment to regular assignment with binary op
                    let final_value = match op_token.kind {
                        TokenKind::PlusEqual => Expression::BinaryOp {
                            op: BinaryOp::Add,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::MinusEqual => Expression::BinaryOp {
                            op: BinaryOp::Subtract,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::StarEqual => Expression::BinaryOp {
                            op: BinaryOp::Multiply,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::SlashEqual => Expression::BinaryOp {
                            op: BinaryOp::Divide,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::LogicalOrAssign => Expression::BinaryOp {
                            op: BinaryOp::Or,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::LogicalAndAssign => Expression::BinaryOp {
                            op: BinaryOp::And,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::Equal => value,
                        _ => unreachable!(),
                    };

                    let stmt = Statement::Assignment {
                        target: expr,
                        value: final_value,
                        position: token.position,
                    };
                    self.wrap_with_modifier(stmt)
                } else {
                    // It's just an expression statement
                    let stmt = Statement::Expression {
                        expression: expr,
                        position: token.position,
                    };
                    self.wrap_with_modifier(stmt)
                }
            }
        }
    }

    /// Look ahead to check if this is a multiple assignment (a, b = ...)
    /// by scanning comma-separated identifiers until we find = or something else.
    fn is_multiple_assignment(&self) -> bool {
        let mut offset = 1; // start after the first comma
        loop {
            // Skip whitespace tokens
            let tok = self.peek_ahead(offset);
            if matches!(
                tok.kind,
                TokenKind::Newline | TokenKind::Comment(_) | TokenKind::Semicolon
            ) {
                offset += 1;
                continue;
            }
            // Expect an identifier (or @ivar, @@cvar, $gvar)
            if !matches!(
                tok.kind,
                TokenKind::Ident(_)
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
                    | TokenKind::GlobalVar(_)
            ) {
                return false;
            }
            offset += 1;
            // Skip bracket indexing (e.g., a[0])
            if matches!(self.peek_ahead(offset).kind, TokenKind::LBracket) {
                offset += 1; // skip [
                let mut depth = 1;
                while depth > 0 {
                    let inner = &self.peek_ahead(offset).kind;
                    if matches!(inner, TokenKind::LBracket) {
                        depth += 1;
                    } else if matches!(inner, TokenKind::RBracket) {
                        depth -= 1;
                    } else if matches!(inner, TokenKind::EOF) {
                        return false;
                    }
                    offset += 1;
                }
            }
            // Skip dot+method chains (e.g., obj.field)
            while matches!(self.peek_ahead(offset).kind, TokenKind::Dot) {
                offset += 1; // skip .
                if matches!(self.peek_ahead(offset).kind, TokenKind::Ident(_)) {
                    offset += 1; // skip method name
                } else {
                    return false;
                }
            }
            // After the target, expect comma or =
            let next = self.peek_ahead(offset);
            if matches!(next.kind, TokenKind::Equal) {
                return true;
            }
            if matches!(next.kind, TokenKind::Comma) {
                offset += 1;
                continue;
            }
            return false;
        }
    }

    /// Check for postfix if/unless modifiers and wrap the statement.
    /// Only matches if the modifier is on the same line (no newline before it).
    fn wrap_with_modifier(&mut self, stmt: Statement) -> Result<Statement, MetorexError> {
        // Don't consume newlines — modifier must be on the same line
        if matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_)) {
            return Ok(stmt);
        }
        if self.check(&[TokenKind::If]) {
            let position = self.advance().position; // consume 'if'
            self.skip_whitespace();
            let condition = self.parse_condition_expression()?;
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
            let condition = self.parse_condition_expression()?;
            Ok(Statement::Unless {
                condition,
                then_branch: vec![stmt],
                else_branch: None,
                position,
            })
        } else if self.check(&[TokenKind::While]) {
            let position = self.advance().position; // consume 'while'
            self.skip_whitespace();
            let condition = self.parse_condition_expression()?;
            Ok(Statement::While {
                condition,
                body: vec![stmt],
                position,
            })
        } else if self.check(&[TokenKind::Until]) {
            let position = self.advance().position; // consume 'until'
            self.skip_whitespace();
            let condition = self.parse_condition_expression()?;
            Ok(Statement::While {
                condition: crate::ast::Expression::UnaryOp {
                    op: crate::ast::UnaryOp::Not,
                    operand: Box::new(condition),
                    position,
                },
                body: vec![stmt],
                position,
            })
        } else {
            Ok(stmt)
        }
    }

    /// Parse a condition expression that may contain an assignment (`x = expr`).
    /// In Ruby, `if x = foo()` assigns and tests truthiness.
    fn parse_condition_expression(
        &mut self,
    ) -> Result<crate::ast::Expression, crate::error::MetorexError> {
        let expr = self.parse_expression()?;
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

    /// Parse the right-hand side of an assignment, supporting chained assignments
    /// like `@a = @b = value`.
    fn parse_assignment_rhs(
        &mut self,
    ) -> Result<crate::ast::Expression, crate::error::MetorexError> {
        let expr = self.parse_expression_with_lambda()?;
        // Check for chained assignment: if the parsed expression is followed by `=`
        // and the expression is an assignable target, parse as nested assignment.
        if self.check(&[crate::lexer::TokenKind::Equal])
            && matches!(
                expr,
                crate::ast::Expression::Identifier { .. }
                    | crate::ast::Expression::InstanceVariable { .. }
                    | crate::ast::Expression::ClassVariable { .. }
                    | crate::ast::Expression::GlobalVariable { .. }
                    | crate::ast::Expression::Index { .. }
                    | crate::ast::Expression::MethodCall { .. }
            )
        {
            let position = self.advance().position;
            self.skip_whitespace();
            let value = self.parse_assignment_rhs()?;
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
}
