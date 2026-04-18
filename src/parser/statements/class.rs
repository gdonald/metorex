// Class definition parsing

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::{Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse a class definition
    pub(crate) fn parse_class_def(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Class, "Expected 'class'")?.position;
        self.skip_whitespace();

        // `class << <target>` — singleton class. Always parsed as an expression
        // so it composes with both statement and rvalue contexts.
        if self.check(&[TokenKind::Shovel]) {
            self.advance(); // consume <<
            self.skip_whitespace();
            let expr = self.parse_singleton_class_after_shovel(start_pos)?;
            return Ok(Statement::Expression {
                expression: expr,
                position: start_pos,
            });
        }

        // `class NS::Name` — NS is a dynamic expression (local variable,
        // constant, method call, etc.) whose class_vars we install Name on.
        // `class Name` — simple form, installed in the current lexical scope.
        let first_ident = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected class name")),
        };
        let mut namespace_expr: Option<Box<Expression>> = None;
        let mut name = first_ident;
        if self.check(&[TokenKind::ColonColon]) {
            let first_pos = self.previous().position;
            let mut ns_expr = Expression::Identifier {
                name: name.clone(),
                position: first_pos,
            };
            while self.match_token(&[TokenKind::ColonColon]) {
                let segment = match self.advance().kind {
                    TokenKind::Ident(part) => part,
                    _ => {
                        return Err(self
                            .error_at_previous("Expected constant name after '::' in class path"));
                    }
                };
                // If another `::` follows, this segment is another namespace
                // level; otherwise it's the final class name.
                if self.check(&[TokenKind::ColonColon]) {
                    ns_expr = Expression::ScopeResolution {
                        namespace: Box::new(ns_expr),
                        name: segment,
                        position: first_pos,
                    };
                } else {
                    name = segment;
                    namespace_expr = Some(Box::new(ns_expr));
                    break;
                }
            }
        }

        self.skip_whitespace();

        // Check for superclass — allow an arbitrary primary expression (e.g.
        // a local variable) so `class parent::C < parent` works.
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
            namespace: namespace_expr,
            superclass,
            body,
            position: start_pos,
        })
    }

    /// Parse the tail of `class << <target>; body; end` after the `<<` has been
    /// consumed. Used both from statement parsing and from primary-expression
    /// parsing (so `(class << a; self; end)` works as an rvalue).
    pub(crate) fn parse_singleton_class_after_shovel(
        &mut self,
        start_pos: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();
        let target = self.parse_expression()?;
        self.skip_whitespace();

        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }
        self.expect(TokenKind::End, "Expected 'end' after 'class << ...' body")?;

        Ok(Expression::SingletonClass {
            target: Box::new(target),
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
