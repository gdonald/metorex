// Keyword-led primary parsing: `super`, `defined?`, `yield`, leading `::`.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::{Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse a `super` call after the `super` keyword has been consumed.
    ///
    /// Three forms:
    ///   - `super`             — implicit forwarding (no args here)
    ///   - `super(a, b, c)`    — explicit (possibly empty) arg list
    ///   - `super a, b, c`     — paren-less arg list
    pub(super) fn parse_super_call(
        &mut self,
        position: Position,
    ) -> Result<Expression, MetorexError> {
        let mut forward_args = false;
        let arguments = if self.check(&[TokenKind::LParen]) {
            self.advance(); // consume (
            let mut args = Vec::new();
            self.skip_whitespace();

            if !self.check(&[TokenKind::RParen]) {
                loop {
                    self.skip_whitespace();
                    if self.match_token(&[TokenKind::Ampersand]) {
                        let arg_position = self.previous().position;
                        let expression = Box::new(self.parse_expression()?);
                        args.push(Expression::BlockArg {
                            expression,
                            position: arg_position,
                        });
                    } else if self.match_token(&[TokenKind::Star]) {
                        let arg_position = self.previous().position;
                        let expression = Box::new(self.parse_expression()?);
                        args.push(Expression::Splat {
                            expression,
                            position: arg_position,
                        });
                    } else {
                        // `**expr` reaches the callee as a trailing hash.
                        self.match_token(&[TokenKind::StarStar]);
                        args.push(self.parse_expression()?);
                    }
                    self.skip_whitespace();

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.skip_whitespace();
            self.expect(TokenKind::RParen, "Expected ')' after super arguments")?;
            args
        } else if self.check(&[
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::EOF,
            TokenKind::End,
            TokenKind::RBrace,
            TokenKind::RParen,
            TokenKind::RBracket,
            TokenKind::Comma,
            TokenKind::If,
            TokenKind::Unless,
            TokenKind::Do,
            TokenKind::Then,
            TokenKind::Else,
            TokenKind::Elsif,
            TokenKind::When,
            TokenKind::Rescue,
            TokenKind::Ensure,
            // Binary operators after `super` apply to the result of the
            // bare super call: `super + 1` is `super() + 1`, not `super(+1)`.
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::EqualEqual,
            TokenKind::TripleEqual,
            TokenKind::BangEqual,
            TokenKind::Less,
            TokenKind::Greater,
            TokenKind::LessEqual,
            TokenKind::GreaterEqual,
            TokenKind::Spaceship,
            TokenKind::LogicalAnd,
            TokenKind::LogicalOr,
            TokenKind::Pipe,
            TokenKind::Ampersand,
            TokenKind::Caret,
            TokenKind::Shovel,
            TokenKind::Match,
            TokenKind::NotMatch,
            TokenKind::Equal,
            TokenKind::PlusEqual,
            TokenKind::MinusEqual,
            TokenKind::StarEqual,
            TokenKind::SlashEqual,
            TokenKind::LogicalOrAssign,
            TokenKind::LogicalAndAssign,
            TokenKind::Question,
            TokenKind::Dot,
            TokenKind::ColonColon,
            TokenKind::DotDot,
            TokenKind::DotDotDot,
            TokenKind::FatArrow,
        ]) || self.is_at_end()
        {
            // Bare `super` with no arguments and no parens. Also covers
            // `super`-followed-by-binary-op which applies to the result.
            // Ruby semantics: forward the enclosing method's arguments.
            forward_args = true;
            Vec::new()
        } else {
            // Paren-less arg list: `super a, b, c`.
            let mut args = vec![self.parse_expression()?];
            while self.match_token(&[TokenKind::Comma]) {
                self.skip_whitespace();
                args.push(self.parse_expression()?);
            }
            args
        };

        Ok(Expression::Super {
            arguments,
            forward_args,
            position,
        })
    }

    /// Parse a `defined?(...)` expression after the `defined?` keyword has been consumed.
    pub(super) fn parse_defined_expression(
        &mut self,
        position: Position,
    ) -> Result<Expression, MetorexError> {
        let expression = if self.match_token(&[TokenKind::LParen]) {
            let expr = self.parse_expression()?;
            self.expect(TokenKind::RParen, "Expected ')' after defined? argument")?;
            expr
        } else {
            self.parse_expression()?
        };
        Ok(Expression::Defined {
            expression: Box::new(expression),
            position,
        })
    }

    /// Parse a `yield` expression after the `yield` keyword has been consumed.
    pub(super) fn parse_yield_expression(
        &mut self,
        position: Position,
    ) -> Result<Expression, MetorexError> {
        let arguments = if self.check(&[TokenKind::LParen]) {
            self.advance(); // consume (
            let mut args = Vec::new();
            self.skip_whitespace();

            if !self.check(&[TokenKind::RParen]) {
                loop {
                    self.skip_whitespace();
                    args.push(self.parse_expression()?);
                    self.skip_whitespace();

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.skip_whitespace();
            self.expect(TokenKind::RParen, "Expected ')' after yield arguments")?;
            args
        } else if !self.check(&[
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::EOF,
            TokenKind::End,
            TokenKind::RBrace,
            TokenKind::RParen,
            TokenKind::If,
            TokenKind::Unless,
            TokenKind::Do,
        ]) && !self.is_at_end()
        {
            // yield expr, expr — paren-less arguments
            let mut args = vec![self.parse_expression()?];
            while self.match_token(&[TokenKind::Comma]) {
                self.skip_whitespace();
                args.push(self.parse_expression()?);
            }
            args
        } else {
            Vec::new()
        };

        Ok(Expression::Yield {
            arguments,
            position,
        })
    }

    /// Parse a leading `::Name` top-level constant access. We don't have a
    /// dedicated top-level namespace distinct from the global one, so treat
    /// `::Name` exactly as `Name`.
    pub(super) fn parse_leading_coloncolon(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            _ => {
                return Err(self.error_at_previous("Expected identifier after leading '::'"));
            }
        };
        Ok(Expression::TopLevelConstant {
            name,
            position: token_position,
        })
    }
}
