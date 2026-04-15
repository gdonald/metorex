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

        // Handle `class << self` (singleton class)
        if self.check(&[TokenKind::Shovel]) {
            self.advance(); // consume <<
            self.skip_whitespace();
            // Expect `self` (or an expression, but we only support self for now)
            match self.advance().kind {
                TokenKind::Ident(ref n) if n == "self" => {}
                _ => return Err(self.error_at_previous("Expected 'self' after 'class <<'")),
            }
            self.skip_whitespace();

            // Parse the singleton class body — statements get merged into the enclosing class
            let mut body = Vec::new();
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::End]) {
                    break;
                }
                body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            self.expect(TokenKind::End, "Expected 'end' after 'class << self' body")?;

            // Return the body statements wrapped in a Block so the class executor can handle them
            return Ok(Statement::Block {
                statements: body,
                position: start_pos,
            });
        }

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

        // Parse class body - set flag to indicate we're inside a class
        let was_in_class = self.in_class_body;
        self.in_class_body = true;

        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        // Restore the previous state
        self.in_class_body = was_in_class;

        self.expect(TokenKind::End, "Expected 'end' after class body")?;

        Ok(Statement::ClassDef {
            name,
            superclass,
            body,
            position: start_pos,
        })
    }

    /// Parse a module definition
    pub(crate) fn parse_module_def(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Module, "Expected 'module'")?
            .position;
        self.skip_whitespace();

        let name = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected module name")),
        };

        self.skip_whitespace();

        let was_in_class = self.in_class_body;
        self.in_class_body = true;

        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.in_class_body = was_in_class;
        self.expect(TokenKind::End, "Expected 'end' after module body")?;

        Ok(Statement::ModuleDef {
            name,
            body,
            position: start_pos,
        })
    }

    /// Parse an include statement
    pub(crate) fn parse_include(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Include, "Expected 'include'")?
            .position;
        self.skip_whitespace();

        let mut module_name = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected module name after 'include'")),
        };
        // Support qualified names: `include Foo::Bar::Baz`
        while self.check(&[TokenKind::ColonColon]) {
            self.advance();
            match self.advance().kind {
                TokenKind::Ident(part) => {
                    module_name.push_str("::");
                    module_name.push_str(&part);
                }
                _ => {
                    return Err(self.error_at_previous("Expected constant name after '::'"));
                }
            }
        }

        Ok(Statement::Include {
            module_name,
            position: start_pos,
        })
    }

    /// Parse an extend statement
    pub(crate) fn parse_extend(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Extend, "Expected 'extend'")?
            .position;
        self.skip_whitespace();

        let module_name = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected module name after 'extend'")),
        };

        Ok(Statement::Extend {
            module_name,
            position: start_pos,
        })
    }
}
