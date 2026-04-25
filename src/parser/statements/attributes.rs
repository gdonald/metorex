// Attribute accessor parsing (attr_reader, attr_writer, attr_accessor)

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse attr_reader statement: attr_reader :name1, :name2, ...
    pub(crate) fn parse_attr_reader(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::AttrReader, "Expected 'attr_reader'")?
            .position;
        self.skip_whitespace();

        let attributes = self.parse_attribute_list()?;

        Ok(Statement::AttrReader {
            attributes,
            position: start_pos,
        })
    }

    /// Parse attr_writer statement: attr_writer :name1, :name2, ...
    pub(crate) fn parse_attr_writer(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::AttrWriter, "Expected 'attr_writer'")?
            .position;
        self.skip_whitespace();

        let attributes = self.parse_attribute_list()?;

        Ok(Statement::AttrWriter {
            attributes,
            position: start_pos,
        })
    }

    /// Parse attr_accessor statement: attr_accessor :name1, :name2, ...
    pub(crate) fn parse_attr_accessor(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::AttrAccessor, "Expected 'attr_accessor'")?
            .position;
        self.skip_whitespace();

        let attributes = self.parse_attribute_list()?;

        Ok(Statement::AttrAccessor {
            attributes,
            position: start_pos,
        })
    }

    /// Parse a comma-separated list of attribute expressions. Each expression
    /// is evaluated at runtime; symbols and strings resolve directly, other
    /// values are coerced via #to_str.
    fn parse_attribute_list(&mut self) -> Result<Vec<Expression>, MetorexError> {
        let mut attributes = Vec::new();

        attributes.push(self.parse_attribute_expression()?);

        loop {
            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }

            self.skip_whitespace();
            attributes.push(self.parse_attribute_expression()?);
        }

        Ok(attributes)
    }

    /// Parse a single attribute name expression. Recognizes `:keyword` for
    /// reserved-word symbols (e.g. `:include`, `:class`); otherwise defers to
    /// the standard expression parser so strings, identifiers, and arbitrary
    /// expressions all work.
    fn parse_attribute_expression(&mut self) -> Result<Expression, MetorexError> {
        if matches!(self.peek().kind, TokenKind::Colon) {
            let next = self.peek_ahead(1).kind.clone();
            if let Some(name) = reserved_keyword_name(&next) {
                let colon_pos = self.advance().position;
                self.advance();
                return Ok(Expression::Symbol {
                    value: name,
                    position: colon_pos,
                });
            }
        }
        self.parse_expression()
    }
}

fn reserved_keyword_name(kind: &TokenKind) -> Option<String> {
    Some(
        match kind {
            TokenKind::Include => "include",
            TokenKind::Extend => "extend",
            TokenKind::Class => "class",
            TokenKind::Module => "module",
            TokenKind::Def => "def",
            TokenKind::End => "end",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::Do => "do",
            _ => return None,
        }
        .to_string(),
    )
}
