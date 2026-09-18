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
        let elements: Vec<Expression> = split_percent_words(&value, filled)
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
        let elements: Vec<Expression> = split_percent_words(&value, filled)
            .into_iter()
            .map(|word| match percent_word(&word, filled, position) {
                Expression::StringLiteral {
                    value, position, ..
                } => Expression::Symbol { value, position },
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
    pub(crate) fn primary_interpolated_string(
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
                // `"#{}"` interpolates nothing at all, which reads as the
                // empty string rather than as an expression.
                InterpolationPart::Expression(expr_str, _) if expr_str.trim().is_empty() => {
                    ast_parts.push(crate::ast::node::InterpolationPart::Text(String::new()));
                }
                InterpolationPart::Expression(expr_str, written_on) => {
                    // Parse the embedded expression as a fresh token stream,
                    // numbered from the interpolated string's line so `__LINE__`
                    // inside `#{...}` reflects the real source line.
                    let expr_lexer = Lexer::with_start_line(
                        &expr_str,
                        if written_on > 0 {
                            written_on
                        } else {
                            position.line
                        },
                    );
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

/// A whole number written out as base-ten digits, which a literal wider than
/// an i64 has to be carried as.
pub(super) fn whole_number(digits: &str, position: Position) -> Expression {
    match digits.parse::<i64>() {
        Ok(value) => Expression::IntLiteral { value, position },
        Err(_) => Expression::BigIntLiteral {
            digits: digits.to_string(),
            position,
        },
    }
}

/// Map a `TokenKind::Float(...)` value to a `FloatLiteral` expression.
pub(super) fn float_literal(value: f64, position: Position) -> Expression {
    Expression::FloatLiteral { value, position }
}

/// Map a `TokenKind::String(...)` value to a `StringLiteral` expression.
pub(super) fn string_literal(value: String, position: Position) -> Expression {
    Expression::StringLiteral { value, position }
}

/// Map a `TokenKind::ByteString(...)` value to a String that says it holds
/// bytes. The characters stand for the bytes the escapes named, which is what
/// tagging the literal as bytes says.
/// Map a `TokenKind::BinaryString(...)` value to a String written in bytes,
/// which is what a literal in a source written in bytes stands for.
pub(super) fn binary_string_literal(value: String, position: Position) -> Expression {
    Expression::MethodCall {
        receiver: Box::new(Expression::StringLiteral { value, position }),
        method: "__binary_literal__".to_string(),
        arguments: Vec::new(),
        trailing_block: None,
        position,
    }
}

pub(super) fn byte_string_literal(value: String, position: Position) -> Expression {
    Expression::MethodCall {
        receiver: Box::new(Expression::StringLiteral { value, position }),
        method: "__holds_bytes__".to_string(),
        arguments: Vec::new(),
        trailing_block: None,
        position,
    }
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
        // Only these letters name an option a pattern may be written with.
        if let Some(unknown) = flags.chars().find(|held| !"imxonesu".contains(*held)) {
            return Err(MetorexError::syntax_error(
                format!("unknown regexp option - {unknown}"),
                crate::error::SourceLocation::new(position.line, position.column, position.offset),
            ));
        }
        if !pattern.contains("#{") {
            // A pattern written out is read where it is written, so one the
            // engine cannot make sense of is not a program.
            let written = crate::regexp::Flags {
                folded: flags.contains('i'),
                dot_reads_newline: flags.contains('m'),
                extended: flags.contains('x'),
                ..crate::regexp::Flags::default()
            };
            if let Err(trouble) = crate::regexp::Pattern::compile(&pattern, written) {
                return Err(MetorexError::syntax_error(
                    format!("{}: /{}/", trouble.0, pattern),
                    crate::error::SourceLocation::new(
                        position.line,
                        position.column,
                        position.offset,
                    ),
                ));
            }
            return Ok(regex_literal(pattern, flags.replace('o', ""), position));
        }
        let parts = crate::lexer::split_interpolation_parts(&pattern);
        let source = self.primary_interpolated_string(parts, position)?;
        Ok(Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "Regexp".to_string(),
                position,
            }),
            method: "__literal__".to_string(),
            arguments: {
                let mut given = vec![
                    source,
                    Expression::StringLiteral {
                        value: flags.clone(),
                        position,
                    },
                ];
                // `o` says the pattern is built the first time it is reached
                // and stands for every time after, so the site it was written
                // at is what the built one is kept under.
                if flags.contains('o') {
                    given.push(Expression::StringLiteral {
                        value: format!("{}:{}", position.line, position.column),
                        position,
                    });
                }
                given
            },
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
    // `%W` fills in `#{}` and reads escape sequences, which a double-quoted
    // string already does, so the word is read back as one.
    if !filled || !(word.contains("#{") || word.contains('\\')) {
        return plain;
    }
    let source = format!("\"{}\"", quoted_percent_word(word));
    let tokens = crate::lexer::Lexer::new(&source).tokenize();
    let read = tokens.first().map(|token| token.kind.clone());
    // A word with escapes but no interpolation reads back as a plain string.
    if let Some(TokenKind::String(text)) = read {
        return Expression::StringLiteral {
            value: text,
            position,
        };
    }
    let Some(TokenKind::InterpolatedString(parts)) = read else {
        return plain;
    };
    let mut built = Vec::new();
    for part in parts {
        match part {
            crate::lexer::InterpolationPart::Text(text) => {
                built.push(crate::ast::InterpolationPart::Text(text));
            }
            crate::lexer::InterpolationPart::Expression(source, _) => {
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
/// Whether a character escaped in a `%w` list keeps the backslash in front of
/// it. Only the delimiters and the backslash itself stand alone.
fn keeps_its_backslash(character: char) -> bool {
    !matches!(
        character,
        '\\' | '[' | ']' | '(' | ')' | '{' | '}' | '<' | '>' | '|' | '!' | '/'
    )
}

fn split_percent_words(value: &str, filled: bool) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            // `%W` reads `\t` as a tab, so the escape travels with the word.
            // An escaped space is the word's own space either way, and in
            // `%w` every other character keeps the backslash in front of it.
            if !character.is_whitespace() && (filled || keeps_its_backslash(character)) {
                current.push('\\');
            }
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

/// A percent-list word written as the body of a double-quoted string. A quote
/// standing in the text is escaped, where one inside an interpolation belongs
/// to the code there and is left alone.
fn quoted_percent_word(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut depth = 0usize;
    let mut letters = word.chars().peekable();
    while let Some(character) = letters.next() {
        match character {
            '#' if depth == 0 && letters.peek() == Some(&'{') => {
                out.push('#');
                out.push('{');
                letters.next();
                depth = 1;
            }
            '{' if depth > 0 => {
                depth += 1;
                out.push('{');
            }
            '}' if depth > 0 => {
                depth -= 1;
                out.push('}');
            }
            '"' if depth == 0 => out.push_str("\\\""),
            character => out.push(character),
        }
    }
    out
}
