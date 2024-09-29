// Class definition parsing

use crate::ast::Statement;
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse a class definition
    pub(crate) fn parse_class_def(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Class, "Expected 'class'")?.position;
        self.skip_whitespace();

        let name = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected class name")),
        };

        self.skip_whitespace();

        // Check for superclass
        let superclass = if self.match_token(&[TokenKind::Less]) {
            self.skip_whitespace();
            match self.advance().kind {
                TokenKind::Ident(parent) => Some(parent),
                _ => return Err(self.error_at_previous("Expected superclass name")),
            }
        } else {
            None
        };

        self.skip_whitespace();

        // Parse class body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' after class body")?;

        Ok(Statement::ClassDef {
            name,
            superclass,
            body,
            position: start_pos,
        })
    }
}
