// Literal and identifier parsing helpers used by `parse_primary`.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::{InterpolationPart, Lexer, Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse a `%w[a b c]` percent-word array literal token into an `Array` of `StringLiteral`s.
    pub(super) fn primary_percent_w(
        &self,
        value: String,
        filled: bool,
        position: Position,
    ) -> Expression {
        let elements: Vec<Expression> = split_percent_words(&value)
            .into_iter()
            .map(|word| percent_word(&word, filled, position))
            .collect();
        Expression::Array { elements, position }
    }

    /// Build the Array a `%i[a b c]` symbol-array literal denotes.
    pub(super) fn primary_percent_i(
        &self,
        value: String,
        filled: bool,
        position: Position,
    ) -> Expression {
        let elements: Vec<Expression> = split_percent_words(&value)
            .into_iter()
            .map(|word| match percent_word(&word, filled, position) {
                Expression::StringLiteral { value, position } => {
                    Expression::Symbol { value, position }
                }
                built => Expression::MethodCall {
                    receiver: Box::new(built),
                    method: "to_sym".to_string(),
                    arguments: Vec::new(),
                    trailing_block: None,
                    position,
                },
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
                    // Parse the embedded expression as a fresh token stream,
                    // numbered from the interpolated string's line so `__LINE__`
                    // inside `#{...}` reflects the real source line.
                    let expr_lexer = Lexer::with_start_line(&expr_str, position.line);
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

impl Parser {
    /// Build the expression for a regex literal. A pattern carrying `#{}`
    /// interpolations becomes `Regexp.new(<interpolated source>, flags)`, so
    /// the pattern is assembled where the literal is evaluated.
    pub(super) fn regex_expression(
        &self,
        pattern: String,
        flags: String,
        position: Position,
    ) -> Result<Expression, MetorexError> {
        if !pattern.contains("#{") {
            return Ok(regex_literal(pattern, flags, position));
        }
        let parts = crate::lexer::split_interpolation_parts(&pattern);
        let source = self.primary_interpolated_string(parts, position)?;
        Ok(Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "Regexp".to_string(),
                position,
            }),
            method: "new".to_string(),
            arguments: vec![
                source,
                Expression::StringLiteral {
                    value: flags,
                    position,
                },
            ],
            trailing_block: None,
            position,
        })
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

/// One word of a percent list. A `%W` or `%I` word reads its `#{}` parts the
/// way a double-quoted string does.
fn percent_word(word: &str, filled: bool, position: Position) -> Expression {
    let plain = Expression::StringLiteral {
        value: word.to_string(),
        position,
    };
    if !filled || !word.contains("#{") {
        return plain;
    }
    let source = format!("\"{}\"", word);
    let tokens = crate::lexer::Lexer::new(&source).tokenize();
    let Some(TokenKind::InterpolatedString(parts)) = tokens.first().map(|token| token.kind.clone())
    else {
        return plain;
    };
    let mut built = Vec::new();
    for part in parts {
        match part {
            crate::lexer::InterpolationPart::Text(text) => {
                built.push(crate::ast::InterpolationPart::Text(text));
            }
            crate::lexer::InterpolationPart::Expression(source) => {
                let inner = crate::lexer::Lexer::new(&source).tokenize();
                let Ok(mut statements) = crate::parser::Parser::new(inner).parse() else {
                    return plain;
                };
                let Some(crate::ast::Statement::Expression { expression, .. }) = statements.pop()
                else {
                    return plain;
                };
                built.push(crate::ast::InterpolationPart::Expression(Box::new(
                    expression,
                )));
            }
        }
    }
    Expression::InterpolatedString {
        parts: built,
        position,
    }
}

/// The words a percent list holds. Whitespace separates them unless a
/// backslash escapes it, and the backslash before any character is dropped
/// once the word it belongs to is settled.
fn split_percent_words(value: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(character);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}
