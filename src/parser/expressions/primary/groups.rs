// Group/collection literal parsing: parenthesized expressions, arrays, hashes.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::{Position, TokenKind};
use crate::parser::Parser;

impl Parser {
    /// Parse a parenthesized expression after the opening `(` has been consumed.
    /// Also handles parenthesized assignment: `(x = 5)`, `(@x = 5)`, etc.
    pub(super) fn parse_paren_group(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        // `()` holds no expression at all, which Ruby reads as nil.
        self.skip_whitespace();
        if self.match_token(&[TokenKind::RParen]) {
            return Ok(Expression::NilLiteral {
                position: token_position,
            });
        }
        // A group may hold a run of statements, whose value is the last one.
        let opened_at = self.stream.current_position();
        if let Ok(held) = self.parse_grouped_statements(token_position) {
            return Ok(held);
        }
        self.stream.restore_position(opened_at);
        let expr = self.parse_expression_with_assignment()?;
        self.skip_whitespace();
        // A group may hold one expression with a trailing modifier, which is
        // what `(123 if true)` and `(count += 1 until done)` spell out.
        if self.check(&[
            TokenKind::If,
            TokenKind::Unless,
            TokenKind::While,
            TokenKind::Until,
        ]) {
            let held = crate::ast::Statement::Expression {
                expression: expr,
                position: token_position,
            };
            let modified = self.wrap_with_modifier(held)?;
            self.skip_whitespace();
            self.expect(TokenKind::RParen, "Expected ')' after expression")?;
            return Ok(Expression::BeginRescue {
                body: vec![modified],
                rescue_clauses: Vec::new(),
                else_clause: None,
                ensure_block: None,
                position: token_position,
            });
        }
        self.expect(TokenKind::RParen, "Expected ')' after expression")?;
        Ok(Expression::Grouped {
            expression: Box::new(expr),
            position: token_position,
        })
    }

    /// The statements a group holds, which `(a; b)` and a group whose body
    /// runs over several lines both spell out. The group answers the last
    /// statement's value.
    fn parse_grouped_statements(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        let mut body = Vec::new();
        loop {
            self.skip_whitespace();
            if self.check(&[TokenKind::RParen]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
            while self.match_token(&[TokenKind::Semicolon]) {
                self.skip_whitespace();
            }
            if self.check(&[TokenKind::RParen]) {
                break;
            }
            if body.len() > 64 {
                return Err(self.error_at_previous("Expected ')' after expression"));
            }
        }
        self.expect(TokenKind::RParen, "Expected ')' after expression")?;
        if body.len() < 2 {
            return Err(self.error_at_previous("Expected ')' after expression"));
        }
        Ok(Expression::BeginRescue {
            body,
            rescue_clauses: Vec::new(),
            else_clause: None,
            ensure_block: None,
            position: token_position,
        })
    }

    /// Parse an expression that may assign, which is what an assignment
    /// written where a value is expected does: `(x = 5)` and `array[i += 1]`
    /// both answer what they assigned.
    pub(crate) fn parse_expression_with_assignment(&mut self) -> Result<Expression, MetorexError> {
        let expr = self.parse_expression()?;
        // `a[1, 3] = x` parses its target as a call to `[]`, which is
        // assignable the same way a single-index one is.
        let assignable = matches!(
            expr,
            Expression::Identifier { .. }
                | Expression::InstanceVariable { .. }
                | Expression::ClassVariable { .. }
                | Expression::GlobalVariable { .. }
                | Expression::Index { .. }
        ) || matches!(&expr, Expression::MethodCall { method, arguments, .. }
            if method == "[]" || arguments.is_empty());
        let compound = [
            (TokenKind::PlusEqual, crate::ast::BinaryOp::Add),
            (TokenKind::MinusEqual, crate::ast::BinaryOp::Subtract),
            (TokenKind::StarEqual, crate::ast::BinaryOp::Multiply),
            (TokenKind::SlashEqual, crate::ast::BinaryOp::Divide),
            (TokenKind::PercentEqual, crate::ast::BinaryOp::Modulo),
            (TokenKind::StarStarEqual, crate::ast::BinaryOp::Power),
            (TokenKind::PipeEqual, crate::ast::BinaryOp::BitwiseOr),
            (TokenKind::AmpersandEqual, crate::ast::BinaryOp::BitwiseAnd),
            (TokenKind::CaretEqual, crate::ast::BinaryOp::Xor),
            (TokenKind::LogicalOrAssign, crate::ast::BinaryOp::Or),
            (TokenKind::LogicalAndAssign, crate::ast::BinaryOp::And),
        ]
        .into_iter()
        .find(|(kind, _)| assignable && self.check(std::slice::from_ref(kind)));
        let expr = if let Some((_, operation)) = compound {
            let operator_position = self.advance().position;
            self.skip_whitespace();
            let value = self.parse_expression()?;
            Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr.clone()),
                right: Box::new(Expression::BinaryOp {
                    op: operation,
                    left: Box::new(expr),
                    right: Box::new(value),
                    position: operator_position,
                }),
                position: operator_position,
            }
        } else if assignable && self.check(&[TokenKind::Equal]) {
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
        Ok(expr)
    }

    /// Parse an array literal after the opening `[` has been consumed.
    pub(super) fn parse_array_literal(
        &mut self,
        token_position: Position,
    ) -> Result<Expression, MetorexError> {
        let mut elements = Vec::new();
        // `[1, a: 2, b: 3]` and `[1, "k" => 2]` gather their trailing pairs
        // into one Hash, the last element of the array.
        let mut entries: Vec<(Expression, Expression)> = Vec::new();
        self.skip_whitespace();

        if !self.check(&[TokenKind::RBracket]) {
            loop {
                self.skip_whitespace();
                // Allow trailing comma: stop the loop if we see `]` here.
                if self.check(&[TokenKind::RBracket]) {
                    break;
                }
                match self.parse_implicit_hash_entry()? {
                    Some(entry) => entries.push(entry),
                    // An element may be an assignment, which answers what it
                    // assigned: `[obj = Object.new, "str"]`.
                    None => elements.push(self.parse_expression_with_assignment()?),
                }
                self.skip_whitespace();

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.skip_whitespace();
        self.expect(TokenKind::RBracket, "Expected ']' after array elements")?;

        if !entries.is_empty() {
            elements.push(Expression::Dictionary {
                entries,
                position: token_position,
            });
        }

        Ok(Expression::Array {
            elements,
            position: token_position,
        })
    }

    /// Parse one `key: value` or `key => value` pair when the next tokens form
    /// one. Returns `None` (having consumed nothing) for a plain element.
    fn parse_implicit_hash_entry(
        &mut self,
    ) -> Result<Option<(Expression, Expression)>, MetorexError> {
        // `ident: value` — the shorthand whose key is a Symbol.
        if matches!(self.peek().kind, TokenKind::Ident(_))
            && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
        {
            let ident_token = self.advance();
            let TokenKind::Ident(name) = ident_token.kind else {
                unreachable!()
            };
            self.advance(); // consume ':'
            self.skip_whitespace();
            let key = Expression::Symbol {
                value: name,
                position: ident_token.position,
            };
            return Ok(Some((key, self.parse_expression()?)));
        }

        // `key => value` — parse the key, then commit only if `=>` follows.
        let saved_position = self.stream().current_position();
        let key = self.parse_expression()?;
        self.skip_whitespace();
        if self.match_token(&[TokenKind::FatArrow]) {
            self.skip_whitespace();
            return Ok(Some((key, self.parse_expression()?)));
        }
        self.stream.restore_position(saved_position);
        Ok(None)
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
                let key = if keyword_symbol_key(&self.peek().kind).is_some()
                    && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
                {
                    let ident_token = self.advance();
                    let name = keyword_symbol_key(&ident_token.kind)
                        .expect("the key name was checked")
                        .to_string();
                    self.advance(); // consume ':'
                    self.skip_whitespace();
                    // `{a:, b:}` takes the value from the variable or method
                    // of the same name, which Ruby calls a shorthand value.
                    if self.check(&[TokenKind::Comma, TokenKind::RBrace]) {
                        entries.push((
                            Expression::Symbol {
                                value: name.clone(),
                                position: ident_token.position,
                            },
                            Expression::Identifier {
                                name,
                                position: ident_token.position,
                            },
                        ));
                        self.skip_whitespace();
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                        continue;
                    }
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

/// The symbol name a token stands for when it opens a `name: value` pair. A
/// keyword is a Symbol key the same as an identifier is, so `{nil: 1}` names
/// the symbol `:nil` rather than the keyword.
pub(crate) fn keyword_symbol_key(kind: &TokenKind) -> Option<&str> {
    match kind {
        TokenKind::Ident(name) => Some(name),
        TokenKind::True => Some("true"),
        TokenKind::False => Some("false"),
        TokenKind::Nil => Some("nil"),
        TokenKind::If => Some("if"),
        TokenKind::Unless => Some("unless"),
        TokenKind::While => Some("while"),
        TokenKind::Until => Some("until"),
        TokenKind::For => Some("for"),
        TokenKind::In => Some("in"),
        TokenKind::Do => Some("do"),
        TokenKind::Then => Some("then"),
        TokenKind::Else => Some("else"),
        TokenKind::Elsif => Some("elsif"),
        TokenKind::End => Some("end"),
        TokenKind::Def => Some("def"),
        TokenKind::Class => Some("class"),
        TokenKind::Module => Some("module"),
        TokenKind::Begin => Some("begin"),
        TokenKind::Rescue => Some("rescue"),
        TokenKind::Ensure => Some("ensure"),
        TokenKind::Return => Some("return"),
        TokenKind::Lambda => Some("lambda"),
        TokenKind::Yield => Some("yield"),
        TokenKind::Super => Some("super"),
        TokenKind::Break => Some("break"),
        TokenKind::Case => Some("case"),
        TokenKind::When => Some("when"),
        TokenKind::Alias => Some("alias"),
        _ => None,
    }
}
