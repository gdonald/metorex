// Statement parsing module
// Handles parsing of all statement types

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

        let token = self.peek().clone();
        match &token.kind {
            TokenKind::Class => self.parse_class_def(),
            TokenKind::Def => self.parse_function_def(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Begin => self.parse_begin_statement(),
            TokenKind::Raise => self.parse_raise_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Return => self.parse_return_statement(),
            _ => {
                // Try to parse as an expression or assignment (including arrow lambdas)
                let expr = self.parse_expression_with_lambda()?;

                // Check if this is an assignment
                if self.check(&[
                    TokenKind::Equal,
                    TokenKind::PlusEqual,
                    TokenKind::MinusEqual,
                    TokenKind::StarEqual,
                    TokenKind::SlashEqual,
                ]) {
                    let op_token = self.advance();
                    let value = self.parse_expression_with_lambda()?;

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
                        TokenKind::Equal => value,
                        _ => unreachable!(),
                    };

                    Ok(Statement::Assignment {
                        target: expr,
                        value: final_value,
                        position: token.position,
                    })
                } else {
                    // It's just an expression statement
                    Ok(Statement::Expression {
                        expression: expr,
                        position: token.position,
                    })
                }
            }
        }
    }
}
