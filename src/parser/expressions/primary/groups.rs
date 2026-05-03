// Group/collection literal parsing: parenthesised expressions, arrays, hashes.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::{Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse a parenthesised expression after the opening `(` has been consumed.
    /// Also handles parenthesised assignment: `(x = 5)`, `(@x = 5)`, etc.
    pub(super) fn parse_paren_group(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        let expr = self.parse_expression()?;
        // Support parenthesized assignment: `(x = 5)`, `(@x = 5)`, etc.
        let expr = if self.check(&[TokenKind::Equal])
            && matches!(
                expr,
                Expression::Identifier { .. }
                    | Expression::InstanceVariable { .. }
                    | Expression::ClassVariable { .. }
                    | Expression::GlobalVariable { .. }
                    | Expression::Index { .. }
            ) {
            let eq_pos = self.advance().position;
            self.skip_whitespace();
            let value = self.parse_expression()?;
            Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
                position: eq_pos,
            }
        } else {
            expr
        };
        self.expect(TokenKind::RParen, "Expected ')' after expression")?;
        Ok(Expression::Grouped {
            expression: Box::new(expr),
            position: token_position,
        })
    }

    /// Parse an array literal after the opening `[` has been consumed.
    pub(super) fn parse_array_literal(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        let mut elements = Vec::new();
        self.skip_whitespace();

        if !self.check(&[TokenKind::RBracket]) {
            loop {
                self.skip_whitespace();
                // Allow trailing comma: stop the loop if we see `]` here.
                if self.check(&[TokenKind::RBracket]) {
                    break;
                }
                elements.push(self.parse_expression()?);
                self.skip_whitespace();

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.skip_whitespace();
        self.expect(TokenKind::RBracket, "Expected ']' after array elements")?;

        Ok(Expression::Array {
            elements,
            position: token_position,
        })
    }

    /// Parse a hash/dictionary literal after the opening `{` has been consumed.
    pub(super) fn parse_dictionary_literal(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        let mut entries = Vec::new();
        self.skip_whitespace();

        if !self.check(&[TokenKind::RBrace]) {
            loop {
                self.skip_whitespace();
                // Allow trailing comma: stop the loop if we see `}` here.
                if self.check(&[TokenKind::RBrace]) {
                    break;
                }
                // `ident:` shorthand → Symbol key. Check before parse_expression
                // so we don't resolve `ident` as a variable.
                let key = if matches!(self.peek().kind, TokenKind::Ident(_))
                    && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
                    && !matches!(self.peek_ahead(2).kind, TokenKind::Colon)
                {
                    let ident_token = self.advance();
                    let name = match ident_token.kind {
                        TokenKind::Ident(n) => n,
                        _ => unreachable!(),
                    };
                    self.advance(); // consume ':'
                    Expression::Symbol {
                        value: name,
                        position: ident_token.position,
                    }
                } else {
                    self.dict_literal_depth += 1;
                    let key_result = self.parse_expression();
                    self.dict_literal_depth -= 1;
                    let key = key_result?;
                    self.skip_whitespace();
                    // Support both `:` and `=>` for hash syntax
                    if self.check(&[TokenKind::FatArrow]) {
                        self.advance(); // consume =>
                    } else {
                        self.expect(
                            TokenKind::Colon,
                            "Expected ':' or '=>' after dictionary key",
                        )?;
                    }
                    key
                };

                self.skip_whitespace();
                let value = self.parse_expression()?;
                entries.push((key, value));
                self.skip_whitespace();

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.skip_whitespace();
        self.expect(TokenKind::RBrace, "Expected '}' after dictionary entries")?;

        Ok(Expression::Dictionary {
            entries,
            position: token_position,
        })
    }
}
