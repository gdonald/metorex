// Literal and identifier parsing helpers used by `parse_primary`.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::{InterpolationPart, Lexer, Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse a `%w[a b c]` percent-word array literal token into an `Array` of `StringLiteral`s.
    pub(super) fn primary_percent_w(&self, value: String, position: Position) -> Expression {
        let elements: Vec<Expression> = value
            .split_whitespace()
            .map(|word| Expression::StringLiteral {
                value: word.to_string(),
                position,
            })
            .collect();
        Expression::Array { elements, position }
    }

    /// Parse a lexer-produced interpolated string into an `Expression::InterpolatedString`.
    pub(super) fn primary_interpolated_string(
        &self,
        parts: Vec<InterpolationPart>,
        position: Position,
    ) -> Result<Expression, MetorexError> {
        let mut ast_parts = Vec::new();
        for part in parts {
            match part {
                InterpolationPart::Text(text) => {
                    ast_parts.push(crate::ast::node::InterpolationPart::Text(text));
                }
                InterpolationPart::Expression(expr_str) => {
                    // Parse the embedded expression as a fresh token stream.
                    let expr_lexer = Lexer::new(&expr_str);
                    let expr_tokens = expr_lexer.tokenize();
                    let mut expr_parser = Parser::new(expr_tokens);
                    let expr = expr_parser.parse_expression()?;
                    ast_parts.push(crate::ast::node::InterpolationPart::Expression(Box::new(
                        expr,
                    )));
                }
            }
        }
        Ok(Expression::InterpolatedString {
            parts: ast_parts,
            position,
        })
    }
}

/// Map a `TokenKind::Int(...)` value to an `IntLiteral` expression.
pub(super) fn int_literal(value: i64, position: Position) -> Expression {
    Expression::IntLiteral { value, position }
}

/// Map a `TokenKind::Float(...)` value to a `FloatLiteral` expression.
pub(super) fn float_literal(value: f64, position: Position) -> Expression {
    Expression::FloatLiteral { value, position }
}

/// Map a `TokenKind::String(...)` value to a `StringLiteral` expression.
pub(super) fn string_literal(value: String, position: Position) -> Expression {
    Expression::StringLiteral { value, position }
}

/// Map a `TokenKind::Regex(...)` value to a `RegexLiteral` expression.
pub(super) fn regex_literal(pattern: String, flags: String, position: Position) -> Expression {
    Expression::RegexLiteral {
        pattern,
        flags,
        position,
    }
}

/// Identifier / variable token to its corresponding expression.
pub(super) fn identifier(name: String, position: Position) -> Expression {
    Expression::Identifier { name, position }
}

pub(super) fn instance_variable(name: String, position: Position) -> Expression {
    Expression::InstanceVariable { name, position }
}

pub(super) fn class_variable(name: String, position: Position) -> Expression {
    Expression::ClassVariable { name, position }
}

pub(super) fn global_variable(name: String, position: Position) -> Expression {
    Expression::GlobalVariable { name, position }
}

pub(super) fn magic_file(position: Position) -> Expression {
    Expression::MagicFile { position }
}

pub(super) fn magic_line(position: Position) -> Expression {
    Expression::MagicLine { position }
}

pub(super) fn bool_literal(value: bool, position: Position) -> Expression {
    Expression::BoolLiteral { value, position }
}

pub(super) fn nil_literal(position: Position) -> Expression {
    Expression::NilLiteral { position }
}

#[allow(dead_code)]
fn _silence_unused_token_kind(_: TokenKind) {}
