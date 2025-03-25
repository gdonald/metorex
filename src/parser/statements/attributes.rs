// Attribute accessor parsing (attr_reader, attr_writer, attr_accessor)

use crate::ast::Statement;
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

        let attributes = self.parse_symbol_list()?;

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

        let attributes = self.parse_symbol_list()?;

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

        let attributes = self.parse_symbol_list()?;

        Ok(Statement::AttrAccessor {
            attributes,
            position: start_pos,
        })
    }

    /// Parse a comma-separated list of symbols (:name1, :name2, ...)
    fn parse_symbol_list(&mut self) -> Result<Vec<String>, MetorexError> {
        let mut attributes = Vec::new();

        // Parse first symbol
        self.expect(TokenKind::Colon, "Expected ':' before attribute name")?;
        self.skip_whitespace();

        match self.advance().kind {
            TokenKind::Ident(name) => attributes.push(name),
            _ => return Err(self.error_at_previous("Expected attribute name after ':'")),
        }

        // Parse remaining symbols
        loop {
            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }

            self.skip_whitespace();
            self.expect(TokenKind::Colon, "Expected ':' before attribute name")?;
            self.skip_whitespace();

            match self.advance().kind {
                TokenKind::Ident(name) => attributes.push(name),
                _ => return Err(self.error_at_previous("Expected attribute name after ':'")),
            }
        }

        Ok(attributes)
    }
}
