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

        // `class ::Name` opens the top-level constant `Name` regardless of
        // lexical nesting. Encoded as a `::` prefix on the name; the VM
        // strips it and skips the lexical-scope fallback.
        let top_level = if self.check(&[TokenKind::ColonColon]) {
            self.advance();
            true
        } else {
            false
        };

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
        // a local variable) so `class parent::C < parent` works, plus
        // `::`-qualified names like `Outer::Inner`.
        let superclass = if self.match_token(&[TokenKind::Less]) {
            self.skip_whitespace();
            let mut parent = match self.advance().kind {
                TokenKind::Ident(parent) => parent,
                _ => return Err(self.error_at_previous("Expected superclass name")),
            };
            while self.check(&[TokenKind::ColonColon]) {
                self.advance();
                match self.advance().kind {
                    TokenKind::Ident(segment) => {
                        parent.push_str("::");
                        parent.push_str(&segment);
                    }
                    _ => {
                        return Err(self
                            .error_at_previous("Expected constant name after '::' in superclass"));
                    }
                }
            }
            Some(parent)
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

        if top_level && namespace_expr.is_none() {
            name = format!("::{}", name);
        }

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
    /// Parse `class << target ... end` after the `<<`.
    pub(crate) fn parse_singleton_class_after_shovel(
        &mut self,
        start_pos: Position,
    ) -> Result<Expression, MetorexError> {
        self.skip_whitespace();
        let target = self.parse_expression()?;
        self.skip_whitespace();

        // `class << @receiver = Object.new` assigns first, then opens the
        // singleton class of what was assigned. Assignment is a statement in
        // metorex, so the pair is recorded here and emitted by the caller.
        let assigned_value = if self.check(&[TokenKind::Equal]) {
            self.advance(); // consume =
            self.skip_whitespace();
            Some(self.parse_expression()?)
        } else {
            None
        };
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

        // With an assignment the value is what gets a singleton class, and
        // the name it was written to is where that value is stored.
        Ok(match assigned_value {
            Some(value) => Expression::SingletonClass {
                target: Box::new(value),
                assign_to: Some(Box::new(target)),
                body,
                position: start_pos,
            },
            None => Expression::SingletonClass {
                target: Box::new(target),
                assign_to: None,
                body,
                position: start_pos,
            },
        })
    }

    /// Parse a module definition
    pub(crate) fn parse_module_def(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Module, "Expected 'module'")?
            .position;
        self.skip_whitespace();

        // `module ::Name` opens the top-level constant `Name` regardless of
        // lexical nesting. Encoded as a `::` prefix on the name; the VM
        // strips it and skips the lexical-scope fallback.
        let top_level = if self.check(&[TokenKind::ColonColon]) {
            self.advance();
            true
        } else {
            false
        };

        let first_ident = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected module name")),
        };

        // `module NS::Name` — same approach as `class NS::Name`. The leading
        // segments form a dynamic namespace expression; the trailing one is
        // the module's own name.
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
                        return Err(self.error_at_previous(
                            "Expected constant name after '::' in module path",
                        ));
                    }
                };
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

        if top_level && namespace_expr.is_none() {
            name = format!("::{}", name);
        }

        Ok(Statement::ModuleDef {
            name,
            namespace: namespace_expr,
            body,
            position: start_pos,
        })
    }

    /// Parse an include statement
    pub(crate) fn parse_include(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Include, "Expected 'include'")?
            .position;

        // A bare `include` with no argument is an ArgumentError at runtime,
        // not a syntax error, so hand it to the `include(...)` call path with
        // an empty argument list. Checked before whitespace is skipped, since
        // skipping would consume the newline that ends the statement.
        if matches!(
            self.peek().kind,
            TokenKind::Newline | TokenKind::Semicolon | TokenKind::Comment(_) | TokenKind::EOF
        ) {
            return Ok(self.include_call_statement(Vec::new(), start_pos));
        }
        self.skip_whitespace();

        // `include(Mod)` / `include(Mod.dup, Other)` — a parenthesized
        // argument list. Parse the arguments and dispatch through a regular
        // `include(...)` call so each evaluated module value is mixed in.
        if self.check(&[TokenKind::LParen]) {
            self.advance();
            let arguments = self.parse_arguments()?;
            return Ok(self.include_call_statement(arguments, start_pos));
        }

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

        // `include <constant>.method(...)` — the argument is a runtime
        // expression (e.g. `include Mod.dup`), not a bare constant name.
        // Continue parsing the postfix chain and dispatch through a regular
        // `include(arg)` call so the evaluated module value is mixed in.
        if self.check(&[TokenKind::Dot, TokenKind::LBracket]) {
            let base = self.constant_expression(&module_name, start_pos);
            let argument = self.parse_postfix_calls(base)?;
            return Ok(self.include_call_statement(vec![argument], start_pos));
        }

        // `include A, B, C` — dispatch through the same `include(...)` call
        // path the parenthesized form uses.
        if self.check(&[TokenKind::Comma]) {
            self.advance();
            let mut arguments = vec![self.constant_expression(&module_name, start_pos)];
            arguments.extend(self.parse_arguments_without_parens()?);
            return Ok(self.include_call_statement(arguments, start_pos));
        }

        Ok(Statement::Include {
            module_name,
            position: start_pos,
        })
    }

    /// Build the expression for a possibly qualified constant name such as
    /// `Foo::Bar::Baz`.
    fn constant_expression(&self, qualified_name: &str, position: Position) -> Expression {
        let mut parts = qualified_name.split("::");
        let mut base = Expression::Identifier {
            name: parts.next().unwrap_or_default().to_string(),
            position,
        };
        for part in parts {
            base = Expression::ScopeResolution {
                namespace: Box::new(base),
                name: part.to_string(),
                position,
            };
        }
        base
    }

    /// Build an `include(...)` method-call statement so a runtime expression
    /// argument (e.g. `include Mod.dup` or `include(Mod)`) is evaluated and
    /// mixed in, rather than resolved by bare constant name.
    fn include_call_statement(&self, arguments: Vec<Expression>, position: Position) -> Statement {
        Statement::Expression {
            expression: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "include".to_string(),
                    position,
                }),
                arguments,
                trailing_block: None,
                position,
            },
            position,
        }
    }

    /// Parse an extend statement
    pub(crate) fn parse_extend(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::Extend, "Expected 'extend'")?
            .position;
        self.skip_whitespace();

        let mut module_name = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected module name after 'extend'")),
        };
        // Support qualified names: `extend Foo::Bar::Baz`
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

        Ok(Statement::Extend {
            module_name,
            position: start_pos,
        })
    }

    /// Parse an alias statement: `alias new_name old_name` or `alias :new :old`.
    pub(crate) fn parse_alias(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Alias, "Expected 'alias'")?.position;
        self.skip_whitespace();

        let new_name = self.parse_alias_method_name()?;
        self.skip_whitespace();
        let old_name = self.parse_alias_method_name()?;

        Ok(Statement::Alias {
            new_name,
            old_name,
            position: start_pos,
        })
    }

    /// Parse `undef name, :other` into the equivalent `undef_method` call on
    /// the enclosing class or module.
    pub(crate) fn parse_undef(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.advance().position;
        self.skip_whitespace();

        let mut arguments = Vec::new();
        loop {
            let name = self.parse_alias_method_name()?;
            arguments.push(Expression::Symbol {
                value: name,
                position: start_pos,
            });
            self.skip_whitespace();
            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
            self.skip_whitespace();
        }

        Ok(Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(Expression::SelfExpr {
                    position: start_pos,
                }),
                method: "undef_method".to_string(),
                arguments,
                trailing_block: None,
                position: start_pos,
            },
            position: start_pos,
        })
    }

    /// Parse a method name accepted by `alias`: a bare identifier or a
    /// symbol literal (`:name`). Certain keywords are accepted because they
    /// are valid Ruby method names.
    fn parse_alias_method_name(&mut self) -> Result<String, MetorexError> {
        if self.check(&[TokenKind::Colon]) {
            self.advance();
        }
        match self.advance().kind {
            TokenKind::Ident(name) => Ok(name),
            TokenKind::Include => Ok("include".to_string()),
            TokenKind::Extend => Ok("extend".to_string()),
            TokenKind::Class => Ok("class".to_string()),
            TokenKind::Module => Ok("module".to_string()),
            TokenKind::Def => Ok("def".to_string()),
            TokenKind::End => Ok("end".to_string()),
            TokenKind::If => Ok("if".to_string()),
            TokenKind::Else => Ok("else".to_string()),
            TokenKind::Do => Ok("do".to_string()),
            _ => Err(self.error_at_previous("Expected method name after 'alias'")),
        }
    }
}
