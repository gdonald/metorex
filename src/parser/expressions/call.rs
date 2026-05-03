// Function and method call parsing
// Handles parsing of function calls, method calls, and array indexing

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse function calls and method calls
    pub(crate) fn parse_call(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&[TokenKind::LParen]) {
                // Function call with parentheses
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenKind::Dot]) {
                // Method call. Trailing-dot continuation: `.foo.\n  .bar` —
                // after a `.` we may skip newlines/comments before the method
                // name. This lets multi-line method chains parse correctly.
                self.skip_whitespace();
                // Allow `.[]` to name the `[]` method explicitly.
                if self.check(&[TokenKind::LBracket])
                    && matches!(self.peek_ahead(1).kind, TokenKind::RBracket)
                {
                    self.advance();
                    self.advance();
                    let position = expr.position();
                    expr = Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: "[]".to_string(),
                        arguments: vec![],
                        trailing_block: None,
                        position,
                    };
                    continue;
                }
                let method_name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    // Allow keywords as method names (e.g., obj.class, obj.if, etc.)
                    TokenKind::Class => "class".to_string(),
                    TokenKind::If => "if".to_string(),
                    TokenKind::Def => "def".to_string(),
                    TokenKind::End => "end".to_string(),
                    TokenKind::Do => "do".to_string(),
                    TokenKind::Else => "else".to_string(),
                    TokenKind::Elsif => "elsif".to_string(),
                    TokenKind::Unless => "unless".to_string(),
                    TokenKind::While => "while".to_string(),
                    TokenKind::For => "for".to_string(),
                    TokenKind::In => "in".to_string(),
                    TokenKind::Begin => "begin".to_string(),
                    TokenKind::Rescue => "rescue".to_string(),
                    TokenKind::Ensure => "ensure".to_string(),
                    TokenKind::Raise => "raise".to_string(),
                    TokenKind::Break => "break".to_string(),
                    TokenKind::Continue => "next".to_string(),
                    TokenKind::Return => "return".to_string(),
                    TokenKind::Lambda => "lambda".to_string(),
                    TokenKind::Super => "super".to_string(),
                    TokenKind::Case => "case".to_string(),
                    TokenKind::When => "when".to_string(),
                    TokenKind::Then => "then".to_string(),
                    TokenKind::Module => "module".to_string(),
                    TokenKind::Include => "include".to_string(),
                    TokenKind::Extend => "extend".to_string(),
                    TokenKind::Yield => "yield".to_string(),
                    TokenKind::Defined => "defined?".to_string(),
                    TokenKind::Nil => "nil".to_string(),
                    TokenKind::True => "true".to_string(),
                    TokenKind::False => "false".to_string(),
                    _ => return Err(self.error_at_previous("Expected method name after '.'")),
                };

                // Check if there are arguments (with or without parens)
                let arguments = if self.match_token(&[TokenKind::LParen]) {
                    self.parse_arguments()?
                } else if self.can_start_argument_for_method_call(&method_name) {
                    self.parse_arguments_without_parens()?
                } else {
                    Vec::new()
                };

                // Check for trailing block (both do...end and {...} syntax)
                let trailing_block = if self.check(&[TokenKind::Do]) {
                    Some(Box::new(self.parse_block()?))
                } else if self.check(&[TokenKind::LBrace]) {
                    Some(Box::new(self.parse_brace_block()?))
                } else {
                    None
                };

                let position = expr.position();
                expr = Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: method_name,
                    arguments,
                    trailing_block,
                    position,
                };
            } else if self.match_token(&[TokenKind::LBracket]) {
                // Array indexing or [] method call
                if self.match_token(&[TokenKind::RBracket]) {
                    // Empty brackets: obj[] — call with no args
                    let position = expr.position();
                    expr = Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: "[]".to_string(),
                        arguments: vec![],
                        trailing_block: None,
                        position,
                    };
                } else {
                    let first_arg = self.parse_expression()?;
                    if self.match_token(&[TokenKind::Comma]) {
                        // Multi-arg bracket: obj[a, b] — method call to []
                        self.skip_whitespace();
                        let mut args = vec![first_arg];
                        args.push(self.parse_expression()?);
                        while self.match_token(&[TokenKind::Comma]) {
                            self.skip_whitespace();
                            args.push(self.parse_expression()?);
                        }
                        self.expect(TokenKind::RBracket, "Expected ']'")?;
                        let position = expr.position();
                        expr = Expression::MethodCall {
                            receiver: Box::new(expr),
                            method: "[]".to_string(),
                            arguments: args,
                            trailing_block: None,
                            position,
                        };
                    } else {
                        self.expect(TokenKind::RBracket, "Expected ']' after array index")?;
                        let position = expr.position();
                        expr = Expression::Index {
                            array: Box::new(expr),
                            index: Box::new(first_arg),
                            position,
                        };
                    }
                }
            } else if self.match_token(&[TokenKind::ColonColon]) {
                // Scope resolution (e.g., Math::PI, Foo::Bar)
                let position = expr.position();
                let name_token = self.advance();
                let name = match name_token.kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected constant name after '::'")),
                };
                expr = Expression::ScopeResolution {
                    namespace: Box::new(expr),
                    name,
                    position,
                };
            } else if self.can_start_argument_for_call(&expr) {
                // Ruby-style function call without parentheses
                // Only parse this if we have an identifier as the callee
                if matches!(expr, Expression::Identifier { .. }) {
                    expr = self.finish_call_without_parens(expr)?;
                } else {
                    break;
                }
            } else if matches!(expr, Expression::Identifier { .. })
                && !matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_))
            {
                // Check for trailing-block-only call: foo { block } or foo do block end
                // (no arguments, but a trailing block on the same line).
                //
                // `{ ... }` binds tightly to this identifier (standard Ruby
                // precedence), but `do ... end` has LOW precedence — if we're
                // currently parsing an argument to a paren-less outer call
                // (e.g. `refine c do ... end`), the `do` block belongs to the
                // outer call, not to this inner identifier.
                self.skip_whitespace();
                if self.check(&[TokenKind::LBrace])
                    || (self.check(&[TokenKind::Do]) && self.paren_less_arg_depth == 0)
                {
                    let position = expr.position();
                    let trailing_block = if self.check(&[TokenKind::Do]) {
                        Some(Box::new(self.parse_block()?))
                    } else {
                        Some(Box::new(self.parse_brace_block()?))
                    };
                    expr = Expression::Call {
                        callee: Box::new(expr),
                        arguments: vec![],
                        trailing_block,
                        position,
                    };
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse any `.method`, `[index]`, or `::Const` postfix operations on an
    /// already-parsed expression. Used so that lambda literals produced outside
    /// of `parse_call` (e.g. `parse_arrow_lambda`) can still have method chains
    /// like `-> { ... }.should raise_error(NameError)`.
    pub(crate) fn parse_postfix_calls(
        &mut self,
        initial: Expression,
    ) -> Result<Expression, MetorexError> {
        // Re-enter the same postfix loop that `parse_call` uses, seeded with
        // the already-parsed expression.
        let mut expr = initial;
        loop {
            if self.match_token(&[TokenKind::Dot]) {
                self.skip_whitespace();
                let method_name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    TokenKind::Class => "class".to_string(),
                    other => {
                        return Err(self.error_at_previous(&format!(
                            "Expected method name after '.', got {:?}",
                            other
                        )));
                    }
                };
                let arguments = if self.match_token(&[TokenKind::LParen]) {
                    self.parse_arguments()?
                } else if self.can_start_argument_for_method_call(&method_name) {
                    self.parse_arguments_without_parens()?
                } else {
                    Vec::new()
                };
                let trailing_block = if self.check(&[TokenKind::Do]) {
                    Some(Box::new(self.parse_block()?))
                } else if self.check(&[TokenKind::LBrace]) {
                    Some(Box::new(self.parse_brace_block()?))
                } else {
                    None
                };
                let position = expr.position();
                expr = Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: method_name,
                    arguments,
                    trailing_block,
                    position,
                };
            } else if self.match_token(&[TokenKind::LParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Finish parsing a function call
    pub(crate) fn finish_call(&mut self, callee: Expression) -> Result<Expression, MetorexError> {
        let arguments = self.parse_arguments()?;

        // Check for trailing block (both do...end and {...} syntax)
        let trailing_block = if self.check(&[TokenKind::Do]) {
            Some(Box::new(self.parse_block()?))
        } else if self.check(&[TokenKind::LBrace]) {
            Some(Box::new(self.parse_brace_block()?))
        } else {
            None
        };

        let position = callee.position();

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            trailing_block,
            position,
        })
    }

    /// Parse function/method arguments (with parentheses)
    pub(crate) fn parse_arguments(&mut self) -> Result<Vec<Expression>, MetorexError> {
        let mut arguments = Vec::new();
        self.skip_whitespace();

        if self.check(&[TokenKind::RParen]) {
            self.advance();
            return Ok(arguments);
        }

        // Collect any keyword args (ident: value) to build a hash at the end
        let mut keyword_pairs: Vec<(String, Expression)> = Vec::new();

        loop {
            self.skip_whitespace();

            // Detect keyword argument: Ident followed by Colon
            if matches!(self.peek().kind, TokenKind::Ident(_))
                && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
            {
                let name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    _ => unreachable!(),
                };
                self.advance(); // consume ':'
                self.skip_whitespace();
                let value = self.parse_expression()?;
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
            } else if self.match_token(&[TokenKind::Star]) {
                // Splat argument: *expr — expand array into individual args
                let position = self.previous().position;
                let expr = self.parse_expression()?;
                arguments.push(Expression::Splat {
                    expression: Box::new(expr),
                    position,
                });
            } else if self.match_token(&[TokenKind::StarStar]) {
                // Double-splat: **expr — pass-through as-is; runtime treats it
                // as a positional hash (good enough for most mspec usages).
                let _position = self.previous().position;
                let expr = self.parse_expression()?;
                arguments.push(expr);
            } else if self.match_token(&[TokenKind::Ampersand]) {
                // `&expr`: convert to a block argument. The runtime drops it
                // entirely if `expr` evaluates to nil and binds it as the
                // pending block otherwise.
                let position = self.previous().position;
                let expr = self.parse_expression()?;
                arguments.push(Expression::BlockArg {
                    expression: Box::new(expr),
                    position,
                });
            } else {
                arguments.push(self.parse_expression()?);
            }

            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        // If there were keyword args, append them as a Dict with a sentinel marker
        // so the runtime can distinguish parser-synthesized kwargs from a user hash.
        if !keyword_pairs.is_empty() {
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
    fn can_start_argument_for_call(&mut self, _callee: &Expression) -> bool {
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

        // Colon starts a symbol when followed by Ident, InstanceVar, ClassVar, or String
        if self.peek().kind == TokenKind::Colon
            && !matches!(
                self.peek_ahead(1).kind,
                TokenKind::Ident(_)
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
                    | TokenKind::String(_)
                    | TokenKind::InterpolatedString(_)
            )
        {
            return false;
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
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::InterpolatedString(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Nil
                | TokenKind::LBracket
                | TokenKind::InstanceVar(_)
                | TokenKind::ClassVar(_)
                | TokenKind::GlobalVar(_)
                | TokenKind::Bang
                | TokenKind::MagicFile
                | TokenKind::MagicLine
                | TokenKind::Ampersand
                | TokenKind::Colon
                | TokenKind::Include
                | TokenKind::Extend
        );

        if !can_be_arg {
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
        // but only for Ident — not Int/Float/String followed by colon (those are dict-like)
        if matches!(self.peek_ahead(1).kind, TokenKind::Colon)
            && !matches!(self.peek().kind, TokenKind::Ident(_))
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
    fn can_start_argument_for_method_call(&mut self, method_name: &str) -> bool {
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

        // Colon starts a symbol when followed by Ident, InstanceVar, ClassVar, or String
        if self.peek().kind == TokenKind::Colon
            && !matches!(
                self.peek_ahead(1).kind,
                TokenKind::Ident(_)
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
                    | TokenKind::String(_)
                    | TokenKind::InterpolatedString(_)
            )
        {
            return false;
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
                    | TokenKind::Float(_)
                    | TokenKind::String(_)
                    | TokenKind::InterpolatedString(_)
                    | TokenKind::True
                    | TokenKind::False
                    | TokenKind::Nil
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
                    | TokenKind::GlobalVar(_)
                    | TokenKind::Bang
                    | TokenKind::MagicFile
                    | TokenKind::MagicLine
                    | TokenKind::Ampersand
                    | TokenKind::Colon
                    | TokenKind::Include
                    | TokenKind::Extend
            );

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

    /// Parse arguments without parentheses, returning the argument list
    fn parse_arguments_without_parens(&mut self) -> Result<Vec<Expression>, MetorexError> {
        // Bump paren-less-arg depth so inner argument expressions don't
        // greedily eat a trailing `do...end` block meant for the outer call.
        self.paren_less_arg_depth += 1;
        let result = self.parse_arguments_without_parens_inner();
        self.paren_less_arg_depth -= 1;
        result
    }

    fn parse_arguments_without_parens_inner(&mut self) -> Result<Vec<Expression>, MetorexError> {
        let mut arguments = Vec::new();
        let mut keyword_pairs: Vec<(String, Expression)> = Vec::new();
        let position = self.peek().position;

        self.skip_whitespace();

        // Parse first argument — detect keyword arg pattern (Ident:).
        // The colon must be glued to the identifier (no space): `key: value` is
        // a kwarg, but `have_method :boom` is a paren-less call whose argument
        // is the symbol `:boom`.
        if matches!(self.peek().kind, TokenKind::Ident(_))
            && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
            && !self.peek_ahead(1).had_leading_space
        {
            let name = match self.advance().kind {
                TokenKind::Ident(n) => n,
                _ => unreachable!(),
            };
            self.advance(); // consume ':'
            self.skip_whitespace();
            let value = self.parse_expression()?;
            keyword_pairs.push((name, value));
        } else {
            // Handle &expr (block-to-proc conversion)
            self.match_token(&[TokenKind::Ampersand]);
            arguments.push(self.parse_expression()?);

            // After parsing the first positional argument, check if we see a colon
            // (which would indicate dict syntax, not a function call).
            if self.check(&[TokenKind::Colon]) {
                return Err(self
                    .error_at_current("Expected function call but found dictionary-like syntax"));
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
            if matches!(self.peek().kind, TokenKind::Ident(_))
                && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
                && !self.peek_ahead(1).had_leading_space
            {
                let name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    _ => unreachable!(),
                };
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
            } else if self.match_token(&[TokenKind::Ampersand]) {
                let position = self.previous().position;
                let expr = self.parse_expression()?;
                arguments.push(Expression::BlockArg {
                    expression: Box::new(expr),
                    position,
                });
            } else {
                arguments.push(self.parse_expression()?);
            }
            // Don't skip newlines here — the newline terminates paren-less args
            // and is needed by wrap_with_modifier to prevent consuming the next line
        }

        // If there were keyword args, append them as a marked kwargs Dict.
        if !keyword_pairs.is_empty() {
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
    fn finish_call_without_parens(
        &mut self,
        callee: Expression,
    ) -> Result<Expression, MetorexError> {
        let position = callee.position();
        let arguments = self.parse_arguments_without_parens()?;

        // Check for trailing block (both do...end and {...} syntax)
        let trailing_block = if self.check(&[TokenKind::Do]) {
            Some(Box::new(self.parse_block()?))
        } else if self.check(&[TokenKind::LBrace]) {
            Some(Box::new(self.parse_brace_block()?))
        } else {
            None
        };

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            trailing_block,
            position,
        })
    }
}
