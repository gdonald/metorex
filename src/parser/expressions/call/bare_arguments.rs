// The arguments a call written without parentheses was given, and
// what may open one.

use super::*;

impl Parser {
    pub(crate) fn can_start_argument_for_call(&mut self, _callee: &Expression) -> bool {
        // A name the program has bound is a variable, so a sign after it
        // carries on the arithmetic rather than opening an argument.
        let names_a_variable = matches!(
            _callee,
            Expression::Identifier { name, .. } if self.bound_names.contains(name)
        );
        // Where arguments written without parentheses are not read at all,
        // as in the value of a `rescue` modifier, a name stands alone.
        if self.refuse_paren_less_args > 0 {
            return false;
        }
        // Don't skip whitespace yet - we need to check if there's a statement
        // terminator first. Newlines, comments, and semicolons all end a
        // paren-less call and must NOT be eaten by skip_whitespace below.
        if matches!(
            self.peek().kind,
            TokenKind::Newline | TokenKind::Comment(_) | TokenKind::Semicolon
        ) {
            return false;
        }

        self.skip_whitespace();

        if self.peek().kind == TokenKind::Colon && !self.colon_starts_symbol_argument() {
            return false;
        }
        // `foo -1` passes a negative number while `foo - 1` subtracts one from
        // what `foo` answers. Ruby tells them apart by the spacing: a sign with
        // a space before it and none after belongs to the argument.
        if self.signed_literal_argument() {
            return !names_a_variable;
        }

        // Don't parse as function call if we see operators or punctuation that
        // indicate we're in a different context (like dictionary key: value)
        // Also check for binary operators that shouldn't start an argument
        if matches!(
            self.peek().kind,
            TokenKind::RBrace
                | TokenKind::Comma
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Equal
                | TokenKind::EqualEqual
                | TokenKind::BangEqual
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::LessEqual
                | TokenKind::GreaterEqual
        ) {
            return false;
        }

        // Check if next token can be an argument
        let can_be_arg = matches!(
            self.peek().kind,
            TokenKind::Ident(_)
                | TokenKind::Int(_)
                | TokenKind::BigInt(_)
                | TokenKind::Rational(_, _)
                | TokenKind::Imaginary(_, _)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::ByteString(_)
                | TokenKind::BinaryString(_)
                | TokenKind::FrozenString(_)
                | TokenKind::MutableString(_)
                | TokenKind::InterpolatedString(_)
                | TokenKind::Regex(_, _)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Nil
                | TokenKind::LBracket
                | TokenKind::LParen
                | TokenKind::InstanceVar(_)
                | TokenKind::ClassVar(_)
                | TokenKind::GlobalVar(_)
                | TokenKind::Bang
                | TokenKind::ColonColon
                | TokenKind::MagicFile
                | TokenKind::SourceEncoding(_)
                | TokenKind::MagicLine
                | TokenKind::MagicDir
                | TokenKind::CommandString(_)
                | TokenKind::CommandSymbol
                | TokenKind::PercentSymbol(_)
                | TokenKind::PercentW(_, _)
                | TokenKind::PercentI(_, _)
                | TokenKind::Ampersand
                | TokenKind::Colon
                | TokenKind::Include
                | TokenKind::Extend
                | TokenKind::Defined
                | TokenKind::Def
                | TokenKind::Yield
        ) || (self.peek().kind == TokenKind::Arrow
            && self.arrow_starts_lambda_argument());

        if !can_be_arg {
            return false;
        }

        // Disambiguate `y & x` (bitwise AND) from `y &x` (block-arg). Both
        // have a space before the `&`; in the first form there's also a
        // space after, while the block-arg form has the operand glued to
        // the `&`. If the token after `&` has leading whitespace, the user
        // meant the binary operator.
        if self.peek().kind == TokenKind::Ampersand && self.peek_ahead(1).had_leading_space {
            return false;
        }

        // Colon could start a symbol arg (:sym) or be a ternary else (:).
        // Outside a ternary, treat it as a paren-less symbol argument.
        if self.peek().kind == TokenKind::Colon && self.ternary_depth > 0 {
            let after_colon = &self.peek_ahead(1).kind;
            let is_dynamic_symbol = matches!(after_colon, TokenKind::InterpolatedString(_));
            if !is_dynamic_symbol && !matches!(self.peek_ahead(2).kind, TokenKind::Comma) {
                return false;
            }
        }

        // Pattern 1: <ident> ':' is a keyword argument (name: value), allow it
        // but only for Ident — not Int/Float/String followed by colon (those are dict-like).
        // A `[` is the exception: `p [:only]` opens an array whose first
        // element is a symbol, not a key.
        if matches!(self.peek_ahead(1).kind, TokenKind::Colon)
            && !matches!(
                self.peek().kind,
                TokenKind::Ident(_) | TokenKind::LBracket | TokenKind::LParen
            )
        {
            return false;
        }

        // Pattern 2: <arg> '}' inside a dict literal — suggests `{x 1}` with
        // missing `:` / `=>`. Only apply this disambiguation when actually
        // parsing a dict key; in a brace block (`{ attr o }`) the `}` simply
        // terminates the block and `attr o` is a paren-less call.
        if self.dict_literal_depth > 0 && matches!(self.peek_ahead(1).kind, TokenKind::RBrace) {
            return false;
        }

        true
    }

    /// Check if the next token can start an argument in a paren-less method call
    pub(crate) fn can_start_argument_for_method_call(&mut self, method_name: &str) -> bool {
        if matches!(
            self.peek().kind,
            TokenKind::Newline | TokenKind::Comment(_) | TokenKind::Semicolon
        ) {
            return false;
        }

        // `foo [x]` (with space) → paren-less array argument.
        // `foo[x]`  (no space)   → index into the result of `foo`.
        // Decide now, before skip_whitespace muddies the state.
        let bracket_with_space =
            self.peek().kind == TokenKind::LBracket && self.peek().had_leading_space;

        self.skip_whitespace();

        if self.peek().kind == TokenKind::Colon && !self.colon_starts_symbol_argument() {
            return false;
        }
        // `foo -1` passes a negative number while `foo - 1` subtracts one from
        // what `foo` answers. Ruby tells them apart by the spacing: a sign with
        // a space before it and none after belongs to the argument.
        if self.signed_literal_argument() {
            return true;
        }
        if matches!(
            self.peek().kind,
            TokenKind::RBrace
                | TokenKind::Comma
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Equal
                | TokenKind::EqualEqual
                | TokenKind::BangEqual
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::LessEqual
                | TokenKind::GreaterEqual
                | TokenKind::Dot
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::Semicolon
        ) {
            return false;
        }

        // For method calls, be more conservative than bare function calls:
        // a bare `[` is ambiguous — with leading space it means a paren-less
        // array argument, without space it means indexing the result.
        let can_be_arg = bracket_with_space
            || matches!(
                self.peek().kind,
                TokenKind::Ident(_)
                    | TokenKind::Int(_)
                    | TokenKind::BigInt(_)
                    | TokenKind::Rational(_, _)
                    | TokenKind::Imaginary(_, _)
                    | TokenKind::Float(_)
                    | TokenKind::String(_)
                    | TokenKind::ByteString(_)
                    | TokenKind::BinaryString(_)
                    | TokenKind::FrozenString(_)
                    | TokenKind::MutableString(_)
                    | TokenKind::InterpolatedString(_)
                    | TokenKind::Regex(_, _)
                    | TokenKind::True
                    | TokenKind::False
                    | TokenKind::Nil
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
                    | TokenKind::GlobalVar(_)
                    | TokenKind::Bang
                    | TokenKind::NotKeyword
                    | TokenKind::ColonColon
                    | TokenKind::MagicFile
                    | TokenKind::SourceEncoding(_)
                    | TokenKind::MagicLine
                    | TokenKind::MagicDir
                    | TokenKind::CommandString(_)
                    | TokenKind::CommandSymbol
                    | TokenKind::PercentSymbol(_)
                    | TokenKind::PercentW(_, _)
                    | TokenKind::PercentI(_, _)
                    | TokenKind::Ampersand
                    | TokenKind::Colon
                    | TokenKind::Include
                    | TokenKind::Extend
                    | TokenKind::Defined
                    | TokenKind::Def
                    | TokenKind::Arrow
                    | TokenKind::Yield
            );

        // Same disambiguation for method-call paren-less args:
        // `obj.foo & x` is `obj.foo() & x`, not `obj.foo(&x)`.
        if can_be_arg
            && self.peek().kind == TokenKind::Ampersand
            && self.peek_ahead(1).had_leading_space
        {
            return false;
        }

        // `obj.foo::Bar` reads Bar out of what `foo` answers, while
        // `obj.foo ::Bar` passes the top-level Bar to `foo`. The space
        // before `::` is what tells the two apart.
        if self.peek().kind == TokenKind::ColonColon && !self.peek().had_leading_space {
            return false;
        }

        if !can_be_arg {
            return false;
        }

        // Colon disambiguation for method calls:
        // - If method name ends with `?` (e.g., `mode?`), the method name already
        //   consumed the `?`. A following `:` could still be the `:` of an outer
        //   ternary (e.g., `cond ? x.failure? : y`), so we must NOT greedily parse
        //   it as a paren-less symbol argument. Require parens for symbol args
        //   on `?`-predicate methods.
        if self.peek().kind == TokenKind::Colon {
            if method_name.ends_with('?') {
                // Inside a ternary, the `:` must be the ternary separator.
                if self.ternary_depth > 0 {
                    return false;
                }
                return true;
            }
            // For other methods, `:` is ambiguous with ternary colon.
            // Outside a ternary, allow `:sym` as a paren-less arg.
            if self.ternary_depth > 0 {
                let after_colon = &self.peek_ahead(1).kind;
                let is_unambiguous = matches!(after_colon, TokenKind::InterpolatedString(_));
                if !is_unambiguous && !matches!(self.peek_ahead(2).kind, TokenKind::Comma) {
                    return false;
                }
            }
        }

        true
    }

    /// Whether the `->` at the cursor introduces a lambda literal that is an
    /// argument to the identifier just parsed, as in `guard -> { cond } do`.
    /// Metorex also accepts `name -> expr` as a one-parameter lambda, so the
    /// two are told apart by looking past the parameter list for the `{` or
    /// `do` that opens a lambda body.
    pub(crate) fn arrow_starts_lambda_argument(&mut self) -> bool {
        let saved_position = self.stream().current_position();
        self.advance(); // consume '->'
        self.skip_whitespace();

        let opens_body = if self.check(&[TokenKind::LBrace, TokenKind::Do, TokenKind::LParen]) {
            true
        } else {
            loop {
                self.skip_whitespace();
                self.match_token(&[TokenKind::Star]);
                self.skip_whitespace();
                if !matches!(self.peek().kind, TokenKind::Ident(_)) {
                    break;
                }
                self.advance();
                self.skip_whitespace();
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
            self.skip_whitespace();
            self.check(&[TokenKind::LBrace, TokenKind::Do])
        };

        self.stream.restore_position(saved_position);
        opens_body
    }

    /// Parse arguments without parentheses, returning the argument list
    pub(crate) fn parse_arguments_without_parens(
        &mut self,
    ) -> Result<Vec<Expression>, MetorexError> {
        // Bump paren-less-arg depth so inner argument expressions don't
        // greedily eat a trailing `do...end` block meant for the outer call.
        self.paren_less_arg_depth += 1;
        let result = self.parse_arguments_without_parens_inner();
        self.paren_less_arg_depth -= 1;
        result
    }

    pub(crate) fn parse_arguments_without_parens_inner(
        &mut self,
    ) -> Result<Vec<Expression>, MetorexError> {
        let mut arguments = Vec::new();
        let mut keyword_pairs: Vec<(String, Expression)> = Vec::new();
        // Where each `**held` sat among the keyword arguments, so the two can
        // be put back in the order they were written.
        let mut splat_slots: Vec<(usize, usize)> = Vec::new();
        let mut rocket_pairs: Vec<(Expression, Expression)> = Vec::new();
        let position = self.peek().position;

        self.skip_whitespace();

        // Parse first argument — detect keyword arg pattern (Ident:).
        // The colon must be glued to the identifier (no space): `key: value` is
        // a kwarg, but `have_method :boom` is a paren-less call whose argument
        // is the symbol `:boom`.
        if crate::parser::expressions::primary::groups::keyword_symbol_key(&self.peek().kind)
            .is_some()
            && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
            && !self.peek_ahead(1).had_leading_space
        {
            let name =
                crate::parser::expressions::primary::groups::keyword_symbol_key(&self.peek().kind)
                    .expect("the name was checked")
                    .to_string();
            self.advance();
            self.advance(); // consume ':'
            self.skip_whitespace();
            let value = self.parse_expression()?;
            keyword_pairs.push((name, value));
        } else {
            // Handle &expr (block-to-proc conversion)
            self.match_token(&[TokenKind::Ampersand]);
            // An argument may assign, which answers what it assigned:
            // `puts text[0] = "y"` passes "y".
            let first = self.parse_expression_with_assignment()?;
            // A `key => value` pair gathers into a Hash the same way a
            // `name: value` one does, which is what `held.update "a" => 1`
            // passes.
            if self.match_token(&[TokenKind::FatArrow]) {
                self.skip_whitespace();
                let value = self.parse_expression()?;
                rocket_pairs.push((first, value));
            } else {
                arguments.push(first);

                // After parsing the first positional argument, check if we see a colon
                // (which would indicate dict syntax, not a function call).
                if self.check(&[TokenKind::Colon]) {
                    return Err(self.error_at_current(
                        "Expected function call but found dictionary-like syntax",
                    ));
                }
            }
        }

        // Parse remaining arguments if there are commas
        while self.match_token(&[TokenKind::Comma]) {
            self.skip_whitespace();

            // Stop if we hit a statement terminator or keyword
            if self.check(&[
                TokenKind::Newline,
                TokenKind::Semicolon,
                TokenKind::End,
                TokenKind::Else,
                TokenKind::Rescue,
                TokenKind::Ensure,
                TokenKind::Do,
            ]) || self.is_at_end()
            {
                break;
            }

            // Detect keyword argument — require the colon to be glued to the
            // identifier (no leading space) so `foo bar :baz` parses as
            // `foo(bar, :baz)`, not `foo(bar: :baz)`.
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
                self.advance(); // consume ':'
                self.skip_whitespace();
                let value = self.parse_expression()?;
                keyword_pairs.push((name, value));
            } else if self.match_token(&[TokenKind::Star]) {
                let position = self.previous().position;
                let expr = self.parse_expression()?;
                arguments.push(Expression::Splat {
                    expression: Box::new(expr),
                    position,
                });
            } else if self.match_token(&[TokenKind::StarStar]) {
                // Double-splat: `**held` passes a Hash as keyword arguments,
                // and a bare `**` forwards what `def name(**)` bound.
                let position = self.previous().position;
                let expr =
                    if self.check(&[TokenKind::Comma, TokenKind::Newline]) || self.is_at_end() {
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
                let position = self.previous().position;
                let expr = self.parse_expression()?;
                arguments.push(Expression::BlockArg {
                    expression: Box::new(expr),
                    position,
                });
            } else {
                let held = self.parse_expression_with_assignment()?;
                if self.match_token(&[TokenKind::FatArrow]) {
                    self.skip_whitespace();
                    let value = self.parse_expression()?;
                    rocket_pairs.push((held, value));
                } else {
                    arguments.push(held);
                }
            }
            // Don't skip newlines here — the newline terminates paren-less args
            // and is needed by wrap_with_modifier to prevent consuming the next line
        }

        // If there were keyword args, append them as a marked kwargs Dict.
        if !keyword_pairs.is_empty() || !rocket_pairs.is_empty() {
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

        Ok(arguments)
    }

    /// Finish parsing a function call without parentheses (Ruby-style)
    pub(crate) fn finish_call_without_parens(
        &mut self,
        callee: Expression,
    ) -> Result<Expression, MetorexError> {
        let position = callee.position();
        let arguments = self.parse_arguments_without_parens()?;

        // Check for trailing block (both do...end and {...} syntax)
        let trailing_block = if self.starts_do_block() && self.paren_less_arg_depth == 0 {
            Some(Box::new(self.parse_block()?))
        } else if self.starts_brace_block() {
            Some(Box::new(self.parse_brace_block()?))
        } else {
            None
        };
        refuse_two_blocks(&arguments, trailing_block.is_some())?;

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            trailing_block,
            position,
        })
    }
}
