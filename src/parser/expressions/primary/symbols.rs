// Symbol literal parsing: `:name`, `:@ivar`, `:@@cvar`, `:keyword`, `:[]`, `:+`, `:"..."`.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::{InterpolationPart, Lexer, Position, TokenKind};
use crate::parser::Parser;

/// Whether `kind` can follow a `:` to form a symbol literal. Keeps the
/// paren-less argument checks in step with what `parse_symbol_literal`
/// actually accepts, so `foo :alias, :meth` parses like `foo :a, :b`.
pub(crate) fn starts_symbol_literal(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Ident(_)
            | TokenKind::InstanceVar(_)
            | TokenKind::ClassVar(_)
            | TokenKind::String(_)
            | TokenKind::InterpolatedString(_)
            | TokenKind::Def
            | TokenKind::Class
            | TokenKind::If
            | TokenKind::Else
            | TokenKind::End
            | TokenKind::Do
            | TokenKind::Nil
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Return
            | TokenKind::Begin
            | TokenKind::Rescue
            | TokenKind::Ensure
            | TokenKind::While
            | TokenKind::For
            | TokenKind::Case
            | TokenKind::When
            | TokenKind::Module
            | TokenKind::Include
            | TokenKind::Yield
            | TokenKind::Super
            | TokenKind::Lambda
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Raise
            | TokenKind::AttrReader
            | TokenKind::AttrWriter
            | TokenKind::AttrAccessor
            | TokenKind::Extend
            | TokenKind::Alias
            | TokenKind::Unless
            | TokenKind::Until
            | TokenKind::Then
            | TokenKind::Elsif
    )
}

impl Parser {
    /// Parse a symbol literal after a leading `:` token has been consumed.
    pub(super) fn parse_symbol_literal(
        &mut self,
        symbol_position: Position,
    ) -> Result<Expression, MetorexError> {
        let next = self.advance();
        match next.kind {
            TokenKind::Ident(name) => {
                if self.check(&[TokenKind::Equal])
                    && !matches!(self.peek_ahead(1).kind, TokenKind::Equal)
                {
                    self.advance(); // consume '='
                    Ok(symbol(format!("{}=", name), symbol_position))
                } else {
                    Ok(symbol(name, symbol_position))
                }
            }
            TokenKind::InstanceVar(name) => Ok(symbol(format!("@{}", name), symbol_position)),
            TokenKind::ClassVar(name) => Ok(symbol(format!("@@{}", name), symbol_position)),

            // Keyword names as symbols
            TokenKind::Def => Ok(symbol("def", symbol_position)),
            TokenKind::Class => Ok(symbol("class", symbol_position)),
            TokenKind::If => Ok(symbol("if", symbol_position)),
            TokenKind::Else => Ok(symbol("else", symbol_position)),
            TokenKind::End => Ok(symbol("end", symbol_position)),
            TokenKind::Do => Ok(symbol("do", symbol_position)),
            TokenKind::Nil => Ok(symbol("nil", symbol_position)),
            TokenKind::True => Ok(symbol("true", symbol_position)),
            TokenKind::False => Ok(symbol("false", symbol_position)),
            TokenKind::Return => Ok(symbol("return", symbol_position)),
            TokenKind::Begin => Ok(symbol("begin", symbol_position)),
            TokenKind::Rescue => Ok(symbol("rescue", symbol_position)),
            TokenKind::Ensure => Ok(symbol("ensure", symbol_position)),
            TokenKind::While => Ok(symbol("while", symbol_position)),
            TokenKind::For => Ok(symbol("for", symbol_position)),
            TokenKind::Case => Ok(symbol("case", symbol_position)),
            TokenKind::When => Ok(symbol("when", symbol_position)),
            TokenKind::Module => Ok(symbol("module", symbol_position)),
            TokenKind::Include => Ok(symbol("include", symbol_position)),
            TokenKind::Yield => Ok(symbol("yield", symbol_position)),
            TokenKind::Super => Ok(symbol("super", symbol_position)),
            TokenKind::Lambda => Ok(symbol("lambda", symbol_position)),
            TokenKind::Break => Ok(symbol("break", symbol_position)),
            TokenKind::Continue => Ok(symbol("next", symbol_position)),
            TokenKind::Raise => Ok(symbol("raise", symbol_position)),
            TokenKind::AttrReader => Ok(symbol("attr_reader", symbol_position)),
            TokenKind::AttrWriter => Ok(symbol("attr_writer", symbol_position)),
            TokenKind::AttrAccessor => Ok(symbol("attr_accessor", symbol_position)),
            TokenKind::Extend => Ok(symbol("extend", symbol_position)),
            TokenKind::Alias => Ok(symbol("alias", symbol_position)),
            TokenKind::Unless => Ok(symbol("unless", symbol_position)),
            TokenKind::Until => Ok(symbol("until", symbol_position)),
            TokenKind::Then => Ok(symbol("then", symbol_position)),
            TokenKind::Elsif => Ok(symbol("elsif", symbol_position)),

            // :[] and :[]= operator symbols
            TokenKind::LBracket => {
                self.expect(TokenKind::RBracket, "Expected ']' after '[' in symbol")?;
                if self.match_token(&[TokenKind::Equal]) {
                    Ok(symbol("[]=", symbol_position))
                } else {
                    Ok(symbol("[]", symbol_position))
                }
            }

            // Operator symbols
            TokenKind::Plus => Ok(symbol("+", symbol_position)),
            TokenKind::Minus => Ok(symbol("-", symbol_position)),
            TokenKind::Star => Ok(symbol("*", symbol_position)),
            TokenKind::Slash => Ok(symbol("/", symbol_position)),
            TokenKind::Percent => Ok(symbol("%", symbol_position)),
            TokenKind::EqualEqual => Ok(symbol("==", symbol_position)),
            TokenKind::BangEqual => Ok(symbol("!=", symbol_position)),
            TokenKind::Less => Ok(symbol("<", symbol_position)),
            TokenKind::Greater => Ok(symbol(">", symbol_position)),
            TokenKind::LessEqual => Ok(symbol("<=", symbol_position)),
            TokenKind::GreaterEqual => Ok(symbol(">=", symbol_position)),
            TokenKind::Spaceship => Ok(symbol("<=>", symbol_position)),
            TokenKind::Shovel => Ok(symbol("<<", symbol_position)),
            TokenKind::Caret => Ok(symbol("^", symbol_position)),
            TokenKind::Match => Ok(symbol("=~", symbol_position)),
            TokenKind::NotMatch => Ok(symbol("!~", symbol_position)),

            // `:$stdout` — the sigil is part of the symbol's name, as it
            // already is for `:@name` and `:@@count` above.
            TokenKind::GlobalVar(name) => Ok(symbol(format!("${}", name), symbol_position)),

            // :"string" syntax — symbol from string literal
            TokenKind::String(s) => Ok(symbol(s, symbol_position)),

            // :"#{interpolated}" syntax — dynamic symbol. Returned as an
            // InterpolatedString (a runtime `to_sym` would be needed for true
            // Symbol semantics; this works for mspec's usages).
            TokenKind::InterpolatedString(parts) => {
                let mut ast_parts = Vec::new();
                for part in parts {
                    match part {
                        InterpolationPart::Text(text) => {
                            ast_parts.push(crate::ast::node::InterpolationPart::Text(text));
                        }
                        InterpolationPart::Expression(expr_str) => {
                            let expr_lexer = Lexer::new(&expr_str);
                            let expr_tokens = expr_lexer.tokenize();
                            let mut expr_parser = Parser::new(expr_tokens);
                            let expr = expr_parser.parse_expression()?;
                            ast_parts.push(crate::ast::node::InterpolationPart::Expression(
                                Box::new(expr),
                            ));
                        }
                    }
                }
                Ok(Expression::InterpolatedString {
                    parts: ast_parts,
                    position: symbol_position,
                })
            }

            _ => Err(self.error_at_previous("Expected identifier after ':' for symbol")),
        }
    }
}

fn symbol(value: impl Into<String>, position: Position) -> Expression {
    Expression::Symbol {
        value: value.into(),
        position,
    }
}
