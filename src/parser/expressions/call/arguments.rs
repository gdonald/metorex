// The arguments a call written with parentheses was given.

use super::*;

impl Parser {
    /// Parse function/method arguments (with parentheses)
    pub(crate) fn parse_arguments(&mut self) -> Result<Vec<Expression>, MetorexError> {
        // `held.name (value rescue other)` writes the parentheses apart from
        // the name, which makes them a group around one argument rather than
        // the argument list itself.
        let written_apart = self.previous().had_leading_space;
        if !written_apart {
            self.call_argument_depth += 1;
        }
        let read = self.parse_arguments_inside_parens();
        if !written_apart {
            self.call_argument_depth -= 1;
        }
        read
    }

    pub(crate) fn parse_arguments_inside_parens(
        &mut self,
    ) -> Result<Vec<Expression>, MetorexError> {
        let mut arguments = Vec::new();
        self.skip_whitespace();

        if self.check(&[TokenKind::RParen]) {
            self.advance();
            return Ok(arguments);
        }

        // Collect any keyword args (ident: value) to build a hash at the end
        let mut keyword_pairs: Vec<(String, Expression)> = Vec::new();
        // Where each `**held` sat among the keyword arguments, so the two can
        // be put back in the order they were written.
        let mut splat_slots: Vec<(usize, usize)> = Vec::new();
        // `foo(key => value)` with a key that is not a literal symbol, which
        // is the implicit-hash form `Struct.new(name, keyword_init: true)`
        // takes in `struct_class.new(key => 1)`.
        let mut rocket_pairs: Vec<(Expression, Expression)> = Vec::new();

        loop {
            self.skip_whitespace();

            // Detect keyword argument: a name followed by a Colon. A keyword
            // such as `lambda:` names one the same as an identifier does.
            if crate::parser::expressions::primary::groups::keyword_symbol_key(&self.peek().kind)
                .is_some()
                && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
                && !self.peek_ahead(1).had_leading_space
            {
                let name = crate::parser::expressions::primary::groups::keyword_symbol_key(
                    &self.peek().kind,
                )
                .expect("the name was checked")
                .to_string();
                self.advance();
                let named_at = self.previous().position;
                self.advance(); // consume ':'
                self.skip_whitespace();
                // `m(a:, b:)` passes what `a` and `b` name, which is Ruby's
                // shorthand for `m(a: a, b: b)`.
                let value = if matches!(self.peek().kind, TokenKind::Comma | TokenKind::RParen) {
                    Expression::Identifier {
                        name: name.clone(),
                        position: named_at,
                    }
                } else {
                    self.parse_expression()?
                };
                keyword_pairs.push((name, value));
            } else if matches!(self.peek().kind, TokenKind::Colon)
                && matches!(self.peek_ahead(1).kind, TokenKind::Ident(_))
                && matches!(self.peek_ahead(2).kind, TokenKind::FatArrow)
            {
                // Old-style hash arg: `:name => value`. Equivalent to `name: value`.
                self.advance(); // consume ':'
                let name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    _ => unreachable!(),
                };
                self.advance(); // consume '=>'
                self.skip_whitespace();
                let value = self.parse_expression()?;
                keyword_pairs.push((name, value));
            } else if self.check(&[TokenKind::DotDotDot]) && self.dots_are_forwarding() {
                // `foo(...)` passes on everything `def foo(...)` collected. A
                // `...` with an operand after it is a beginless range instead.
                self.advance();
                let position = self.previous().position;
                let named = |name: &str| Expression::Identifier {
                    name: name.to_string(),
                    position,
                };
                arguments.push(Expression::Splat {
                    expression: Box::new(named(crate::parser::ANONYMOUS_SPLAT)),
                    position,
                });
                arguments.push(Expression::KeywordSplat {
                    expression: Box::new(named(crate::parser::ANONYMOUS_KWREST)),
                    position,
                });
                arguments.push(Expression::BlockArg {
                    expression: Box::new(named(crate::parser::ANONYMOUS_BLOCK)),
                    position,
                });
            } else if self.match_token(&[TokenKind::Star]) {
                // Splat argument: *expr — expand array into individual args.
                // A bare `*` forwards the anonymous splat `def foo(*)` bound.
                let position = self.previous().position;
                let expr = if self.check(&[TokenKind::Comma, TokenKind::RParen]) {
                    if self.block_declares_anonymous(|declared| declared.rest) {
                        return Err(self.error_at_previous(
                            "anonymous rest parameter is also used within block",
                        ));
                    }
                    Expression::Identifier {
                        name: crate::parser::ANONYMOUS_SPLAT.to_string(),
                        position,
                    }
                } else {
                    self.parse_expression()?
                };
                arguments.push(Expression::Splat {
                    expression: Box::new(expr),
                    position,
                });
            } else if self.match_token(&[TokenKind::StarStar]) {
                // Double-splat: **expr — a Hash passed as keyword arguments.
                // A bare `**` forwards what `def foo(**)` bound.
                let position = self.previous().position;
                let expr = if self.check(&[TokenKind::Comma, TokenKind::RParen]) {
                    if self.block_declares_anonymous(|declared| declared.keyword_rest) {
                        return Err(self.error_at_previous(
                            "anonymous keyword rest parameter is also used within block",
                        ));
                    }
                    Expression::Identifier {
                        name: crate::parser::ANONYMOUS_KWREST.to_string(),
                        position,
                    }
                } else {
                    self.parse_expression()?
                };
                splat_slots.push((arguments.len(), keyword_pairs.len() + rocket_pairs.len()));
                arguments.push(Expression::KeywordSplat {
                    expression: Box::new(expr),
                    position,
                });
            } else if self.match_token(&[TokenKind::Ampersand]) {
                // `&expr`: convert to a block argument. The runtime drops it
                // entirely if `expr` evaluates to nil and binds it as the
                // pending block otherwise. A bare `&` forwards the anonymous
                // block parameter the enclosing `def foo(&)` bound.
                let position = self.previous().position;
                let expr = if self.check(&[TokenKind::Comma, TokenKind::RParen]) {
                    if self.block_declares_anonymous(|declared| declared.block) {
                        return Err(self.error_at_previous(
                            "anonymous block parameter is also used within block",
                        ));
                    }
                    Expression::Identifier {
                        name: crate::parser::ANONYMOUS_BLOCK.to_string(),
                        position,
                    }
                } else {
                    self.parse_expression()?
                };
                arguments.push(Expression::BlockArg {
                    expression: Box::new(expr),
                    position,
                });
            } else {
                // An argument may assign, which answers what it assigned:
                // `StringIO.new(text = "hello")` binds `text` and passes it.
                let expression = self.parse_expression_with_assignment()?;
                self.skip_whitespace();
                if self.match_token(&[TokenKind::FatArrow]) {
                    self.skip_whitespace();
                    let value = self.parse_expression()?;
                    rocket_pairs.push((expression, value));
                } else {
                    arguments.push(expression);
                }
            }

            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }

            // Allow trailing comma before closing paren: foo(a, b,)
            self.skip_whitespace();
            if self.check(&[TokenKind::RParen]) {
                break;
            }
        }

        // If there were keyword args, append them as a Dict with a sentinel marker
        // so the runtime can distinguish parser-synthesized kwargs from a user hash.
        if !keyword_pairs.is_empty() || !rocket_pairs.is_empty() {
            let position = self.peek().position;
            let mut entries: Vec<(Expression, Expression)> = keyword_pairs
                .into_iter()
                .map(|(k, v)| {
                    (
                        Expression::StringLiteral {
                            value: format!(":{}", k),
                            position,
                        },
                        v,
                    )
                })
                .collect();
            entries.extend(rocket_pairs);
            fold_keyword_splats(&mut arguments, &mut entries, &splat_slots, position);
            // Marker entry the runtime recognizes (see split_keyword_args).
            entries.push((
                Expression::StringLiteral {
                    value: "__MX_KWARGS__".to_string(),
                    position,
                },
                Expression::BoolLiteral {
                    value: true,
                    position,
                },
            ));
            arguments.push(Expression::Dictionary { entries, position });
        }

        self.skip_whitespace();
        self.expect(TokenKind::RParen, "Expected ')' after arguments")?;

        Ok(arguments)
    }

    /// Check if the next token can start an argument in a parentheses-less call
    /// Also checks if this looks like a dictionary context (value followed by colon)
    /// Whether the sign at the cursor opens a signed numeric argument rather
    /// than continuing an arithmetic expression.
    pub(crate) fn signed_literal_argument(&mut self) -> bool {
        matches!(self.peek().kind, TokenKind::Minus | TokenKind::Plus)
            && self.peek().had_leading_space
            && !self.peek_ahead(1).had_leading_space
            && matches!(
                self.peek_ahead(1).kind,
                TokenKind::Int(_) | TokenKind::Float(_) | TokenKind::BigInt(_)
            )
    }

    /// Whether the `*` or `**` at the cursor opens a splatted argument rather
    /// than multiplying. Ruby reads `name *list` and `name **options` as
    /// arguments, by the space before the operator and none after it.
    pub(crate) fn spaced_splat_argument(&mut self) -> bool {
        matches!(self.peek().kind, TokenKind::Star | TokenKind::StarStar)
            && self.peek().had_leading_space
            && !self.peek_ahead(1).had_leading_space
    }

    /// Whether the `:` at the cursor opens a symbol argument. A name follows
    /// it either way, but an operator name only counts when it is glued to the
    /// colon, since `condition ? value : -1` puts one there too.
    pub(crate) fn colon_starts_symbol_argument(&mut self) -> bool {
        use crate::parser::expressions::primary::symbols;
        // A symbol never carries a space between its colon and its name, so
        // `flag ? held : "text"` is a ternary rather than a call passing the
        // symbol `:"text"`.
        let next = self.peek_ahead(1);
        if next.had_leading_space {
            return false;
        }
        symbols::starts_symbol_literal(&next.kind) || symbols::starts_operator_symbol(&next.kind)
    }

    /// `p [1, 2]` passes an array while `values[0]` indexes one. Ruby tells
    /// them apart by the space before the bracket and by whether the name is
    /// bound as a variable, which is what this repeats.
    pub(crate) fn bracket_starts_array_argument(&self, callee: &Expression) -> bool {
        let Expression::Identifier { name, .. } = callee else {
            return false;
        };
        self.peek().had_leading_space && !self.bound_names.contains(name)
    }

    /// `p (1..3).to_a` passes what the parentheses hold along with the calls
    /// that follow, where `p(1..3).to_a` calls those on what `p` answers.
    /// Ruby tells them apart by the space before the parenthesis and by what
    /// follows the matching one.
    pub(crate) fn spaced_paren_opens_argument(&mut self, callee: &Expression) -> bool {
        let Expression::Identifier { name, .. } = callee else {
            return false;
        };
        if !self.peek().had_leading_space || self.bound_names.contains(name) {
            return false;
        }
        let mut depth = 0;
        let mut offset = 0;
        loop {
            match self.peek_ahead(offset).kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return matches!(
                            self.peek_ahead(offset + 1).kind,
                            TokenKind::Dot | TokenKind::ColonColon
                        );
                    }
                }
                TokenKind::EOF => return false,
                _ => {}
            }
            offset += 1;
        }
    }

    /// Whether the brackets open a Hash rather than a subscript, which they do
    /// when the first thing inside is a `name:` pair.
    pub(crate) fn bracket_opens_hash(&mut self) -> bool {
        matches!(self.peek().kind, TokenKind::Ident(_))
            && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
            && !self.peek_ahead(1).had_leading_space
    }

    /// One `name: value` pair inside brackets, or None when what follows is
    /// an ordinary subscript.
    pub(crate) fn parse_bracket_hash_pair(
        &mut self,
    ) -> Result<Option<(Expression, Expression)>, MetorexError> {
        if !self.bracket_opens_hash() {
            return Ok(None);
        }
        let position = self.peek().position;
        let TokenKind::Ident(name) = self.advance().kind else {
            unreachable!("the name was checked")
        };
        self.advance();
        self.skip_whitespace();
        let value = self.parse_expression()?;
        Ok(Some((
            Expression::Symbol {
                value: name,
                position,
            },
            value,
        )))
    }
}
