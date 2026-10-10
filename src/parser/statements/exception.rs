// Exception handling statement parsing (begin/rescue/raise)

use crate::ast::{Expression, RescueClause, Statement};
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse a begin...rescue...else...ensure...end statement
    pub(crate) fn parse_begin_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Begin, "Expected 'begin'")?.position;
        let parts = self.parse_begin_body()?;

        Ok(Statement::Begin {
            body: parts.body,
            rescue_clauses: parts.rescue_clauses,
            else_clause: parts.else_clause,
            ensure_block: parts.ensure_block,
            position: start_pos,
        })
    }

    /// Parse a `begin ... end` expression (RHS of an assignment, etc.) into
    /// an `Expression::BeginRescue`. Called after the `begin` token has been
    /// consumed by the primary-expression parser.
    pub(crate) fn parse_begin_expression(
        &mut self,
        start_pos: crate::lexer::Position,
    ) -> Result<Expression, MetorexError> {
        let parts = self.parse_begin_body()?;
        Ok(Expression::BeginRescue {
            body: parts.body,
            rescue_clauses: parts.rescue_clauses,
            else_clause: parts.else_clause,
            ensure_block: parts.ensure_block,
            position: start_pos,
        })
    }

    /// Shared parser for the body of a begin block, used by both the
    /// statement and expression forms. The opening `begin` token must
    /// already have been consumed.
    pub(crate) fn parse_begin_body(&mut self) -> Result<BeginParts, MetorexError> {
        self.skip_whitespace();

        // Parse the main body
        let mut body = Vec::new();
        while !self.check(&[
            TokenKind::Rescue,
            TokenKind::Else,
            TokenKind::Ensure,
            TokenKind::End,
        ]) && !self.is_at_end()
        {
            self.skip_whitespace();
            if self.check(&[
                TokenKind::Rescue,
                TokenKind::Else,
                TokenKind::Ensure,
                TokenKind::End,
            ]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        // Parse rescue clauses
        let mut rescue_clauses = Vec::new();
        while self.match_token(&[TokenKind::Rescue]) {
            rescue_clauses.push(self.parse_rescue_clause()?);
            self.skip_whitespace();
        }

        // Parse optional else clause
        let else_clause = if self.match_token(&[TokenKind::Else]) {
            self.skip_whitespace();
            let mut else_body = Vec::new();
            while !self.check(&[TokenKind::Ensure, TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::Ensure, TokenKind::End]) {
                    break;
                }
                else_body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(else_body)
        } else {
            None
        };

        // Parse optional ensure clause
        let ensure_block = if self.match_token(&[TokenKind::Ensure]) {
            self.skip_whitespace();
            let mut ensure_body = Vec::new();
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::End]) {
                    break;
                }
                ensure_body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(ensure_body)
        } else {
            None
        };

        self.expect(TokenKind::End, "Expected 'end' after begin block")?;

        // An `else` answers what runs when nothing was raised, so with no
        // `rescue` beside it there is nothing for it to stand against.
        if else_clause.is_some() && rescue_clauses.is_empty() && ensure_block.is_none() {
            return Err(MetorexError::syntax_error(
                "else without rescue is useless",
                SourceLocation::new(
                    self.previous().position.line,
                    self.previous().position.column,
                    self.previous().position.offset,
                ),
            ));
        }

        Ok(BeginParts {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
        })
    }
}

pub(crate) struct BeginParts {
    body: Vec<Statement>,
    rescue_clauses: Vec<RescueClause>,
    else_clause: Option<Vec<Statement>>,
    ensure_block: Option<Vec<Statement>>,
}

impl Parser {
    // Sentinel impl to keep the file's `Parser` impl block contiguous —
    // helpers added below.

    /// Parse a rescue clause
    pub(crate) fn parse_rescue_clause(&mut self) -> Result<RescueClause, MetorexError> {
        let start_pos = self.previous().position;

        // Don't skip whitespace yet - we need to check if there's a newline after rescue
        // If there's a newline, then no exception types are specified
        let has_newline = self.check(&[TokenKind::Newline, TokenKind::Semicolon])
            || matches!(self.peek().kind, TokenKind::Comment(_));

        self.skip_whitespace();

        // Parse exception types (comma-separated identifiers)
        let mut exception_types = Vec::new();
        let mut variable_name = None;

        // Check if there's an exception type specified
        // An exception type is present if:
        // 1. There's no newline after rescue
        // 2. We see an identifier that's NOT followed by '=' (which would be an assignment)
        // `rescue *handled` names the classes through a splat, which is read
        // where the clause is reached rather than written out here.
        let mut splatted_types = Vec::new();
        // An identifier followed by `=` starts an assignment in the body
        // rather than naming a class.
        let starts_an_assignment = matches!(self.peek().kind, TokenKind::Ident(_))
            && matches!(self.peek_ahead(1).kind, TokenKind::Equal);
        let names_classes = !has_newline
            && !starts_an_assignment
            && !self.check(&[TokenKind::FatArrow, TokenKind::Then]);
        if names_classes {
            loop {
                if self.match_token(&[TokenKind::Star]) {
                    self.skip_whitespace();
                    // The classes stand on the `rescue` line, so the walk stops
                    // at the end of it rather than reading the body as part of
                    // them.
                    splatted_types.push(self.parse_range()?);
                } else if let Some(name) = self.written_class_name() {
                    exception_types.push(name);
                } else {
                    // Any other expression names one class, read where the
                    // clause is reached: `defined?(Held) ? Held : IOError`.
                    let position = self.peek().position;
                    let element = self.parse_assignment()?;
                    splatted_types.push(Expression::Array {
                        elements: vec![element],
                        position,
                    });
                }
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
                self.skip_whitespace();
            }
        }

        // Check for variable binding (=> var)
        let mut variable_target = None;
        if self.match_token(&[TokenKind::FatArrow]) {
            self.skip_whitespace();
            // A plain name reaches a local, and one written with a sigil the
            // global of that name. Anything else — `held.error`,
            // `held&.error`, `held[:error]` — is a target of its own.
            let names_a_local = matches!(
                &self.peek().kind,
                TokenKind::Ident(name)
                    if name.starts_with(|first: char| first.is_lowercase() || first == '_')
            ) && !matches!(
                self.peek_ahead(1).kind,
                TokenKind::Dot | TokenKind::SafeDot | TokenKind::LBracket | TokenKind::ColonColon
            );
            if names_a_local {
                match self.advance().kind {
                    TokenKind::Ident(name) => {
                        self.declare_local(&name);
                        variable_name = Some(name)
                    }
                    _ => unreachable!("only a name reaches here"),
                }
                self.skip_whitespace();
            } else if matches!(
                self.peek().kind,
                TokenKind::Ident(_)
                    | TokenKind::GlobalVar(_)
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
            ) {
                variable_target = Some(self.parse_expression()?);
                self.skip_whitespace();
            } else {
                return Err(MetorexError::syntax_error(
                    "Expected variable name after '=>'",
                    SourceLocation::new(
                        self.peek().position.line,
                        self.peek().position.column,
                        self.peek().position.offset,
                    ),
                ));
            }
        }

        // Parse rescue body. `retry` is only allowed here, which the depth
        // counter is what says.
        self.rescue_depth += 1;
        let mut body = Vec::new();
        while !self.check(&[
            TokenKind::Rescue,
            TokenKind::Else,
            TokenKind::Ensure,
            TokenKind::End,
        ]) && !self.is_at_end()
        {
            self.skip_whitespace();
            if self.check(&[
                TokenKind::Rescue,
                TokenKind::Else,
                TokenKind::Ensure,
                TokenKind::End,
            ]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.rescue_depth -= 1;
        Ok(RescueClause {
            exception_types,
            variable_name,
            variable_target,
            splatted_types,
            body,
            position: start_pos,
        })
    }

    /// A class written by name in a rescue list, as `Errno::ENOENT`, taken
    /// when nothing but the next entry, the variable or the body follows it.
    fn written_class_name(&mut self) -> Option<String> {
        let mut offset = 0;
        let mut full_name = String::new();
        loop {
            let TokenKind::Ident(part) = &self.peek_ahead(offset).kind else {
                return None;
            };
            full_name.push_str(part);
            offset += 1;
            if !matches!(self.peek_ahead(offset).kind, TokenKind::ColonColon) {
                break;
            }
            full_name.push_str("::");
            offset += 1;
        }
        let ends_the_entry = matches!(
            self.peek_ahead(offset).kind,
            TokenKind::Comma
                | TokenKind::FatArrow
                | TokenKind::Then
                | TokenKind::Semicolon
                | TokenKind::Newline
                | TokenKind::Comment(_)
                | TokenKind::EOF
        );
        if !ends_the_entry {
            return None;
        }
        for _ in 0..offset {
            self.advance();
        }
        Some(full_name)
    }

    /// Parse a raise statement
    pub(crate) fn parse_raise_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Raise, "Expected 'raise'")?.position;
        // Do NOT skip newlines: a bare `raise` followed by a newline (or
        // postfix `if`/`unless`, or `end` on the next line) means a re-raise
        // with no explicit exception. Skipping newlines first would let
        // `parse_expression` consume the next statement.

        // A trailing comment ends the statement the way a newline does, so
        // `raise # note` is still the bare re-raise form.
        let exception = if matches!(self.peek().kind, TokenKind::Comment(_))
            || self.check(&[
                TokenKind::Newline,
                TokenKind::Semicolon,
                TokenKind::End,
                TokenKind::RBrace,
                TokenKind::If,
                TokenKind::Unless,
                TokenKind::While,
                // `raise until cond` re-raises under a modifier, the way
                // `raise while cond` does.
                TokenKind::Until,
                TokenKind::RParen,
                TokenKind::Else,
                TokenKind::Elsif,
                TokenKind::When,
                TokenKind::Rescue,
                TokenKind::Ensure,
            ])
            || self.is_at_end()
        {
            None
        } else if self.check(&[TokenKind::LParen]) {
            // `raise(Class, "message")` — a parenthesized argument list, which
            // reading the first argument as a grouped expression would choke
            // on at the comma.
            self.advance();
            let arguments = self.parse_arguments()?;
            // `raise(*args)` names what to raise in a list built at run time,
            // and `cause:` alongside the rest is more than a class and a
            // message, so either is handed to Kernel's own `raise` rather
            // than read here as a single exception.
            let handed_over = arguments
                .iter()
                .any(|given| matches!(given, Expression::Splat { .. }))
                || arguments.len() > 1
                || arguments
                    .iter()
                    .any(|given| matches!(given, Expression::Dictionary { .. }));
            if handed_over {
                let call = Expression::Call {
                    callee: Box::new(Expression::Identifier {
                        name: "raise".to_string(),
                        position: start_pos,
                    }),
                    arguments,
                    trailing_block: None,
                    position: start_pos,
                };
                return self.wrap_with_modifier(Statement::Expression {
                    expression: call,
                    position: start_pos,
                });
            }
            arguments.into_iter().next()
        } else if crate::parser::expressions::primary::groups::keyword_symbol_key(&self.peek().kind)
            .is_some()
            && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
            && !self.peek_ahead(1).had_leading_space
        {
            // `raise cause: held` names no exception of its own, so the whole
            // list goes to Kernel's own `raise`, which says as much.
            let arguments = self.parse_arguments_without_parens()?;
            let call = Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "raise".to_string(),
                    position: start_pos,
                }),
                arguments,
                trailing_block: None,
                position: start_pos,
            };
            return self.wrap_with_modifier(Statement::Expression {
                expression: call,
                position: start_pos,
            });
        } else {
            let expr = self.parse_expression()?;
            // Check for `raise ExceptionClass, message` form
            if self.match_token(&[TokenKind::Comma]) {
                self.skip_whitespace();
                let rest = self.parse_arguments_without_parens()?;
                // A message, a backtrace or a `cause:` goes to Kernel's own
                // `raise`, which asks the first argument for the exception
                // through `exception`, as a class or an exception answers it.
                {
                    let mut arguments = vec![expr];
                    arguments.extend(rest);
                    let call = Expression::Call {
                        callee: Box::new(Expression::Identifier {
                            name: "raise".to_string(),
                            position: start_pos,
                        }),
                        arguments,
                        trailing_block: None,
                        position: start_pos,
                    };
                    return self.wrap_with_modifier(Statement::Expression {
                        expression: call,
                        position: start_pos,
                    });
                }
            } else {
                Some(expr)
            }
        };

        let stmt = Statement::Raise {
            exception,
            position: start_pos,
        };
        self.wrap_with_modifier(stmt)
    }
}
