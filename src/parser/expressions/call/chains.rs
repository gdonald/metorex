// Reading a call chain: the `.method`, `[index]` and `::Const`
// that follow an expression already parsed.

use super::*;

impl Parser {
    /// Parse function calls and method calls
    pub(crate) fn parse_call(&mut self) -> Result<Expression, MetorexError> {
        let primary = self.parse_primary()?;
        self.parse_call_from(primary)
    }

    /// Continue a call chain from an expression already parsed, which is how a
    /// negative numeric literal takes the method calls that follow it.
    pub(crate) fn parse_call_from(
        &mut self,
        primary: Expression,
    ) -> Result<Expression, MetorexError> {
        let mut expr = primary;

        loop {
            self.step_over_fluent_break();
            if self.check(&[TokenKind::LParen]) && !self.spaced_paren_opens_argument(&expr) {
                self.advance();
                // Function call with parentheses
                expr = self.finish_call(expr)?;
            } else if self.check(&[TokenKind::Dot, TokenKind::SafeDot]) {
                // A call written with `&.` answers nil for a nil receiver
                // without running the method at all.
                let safe = self.check(&[TokenKind::SafeDot]);
                self.advance();
                // Method call. Trailing-dot continuation: `.foo.\n  .bar` —
                // after a `.` we may skip newlines/comments before the method
                // name. This lets multi-line method chains parse correctly.
                self.skip_whitespace();
                // `held.(args)` calls `call` on what stands to its left.
                if self.check(&[TokenKind::LParen]) {
                    let position = expr.position();
                    self.advance();
                    let arguments = self.parse_arguments()?;
                    // `held.(args) { }` hands the block to `call` the way any
                    // other method call written out would.
                    let trailing_block = if self.starts_do_block() && self.paren_less_arg_depth == 0
                    {
                        Some(Box::new(self.parse_block()?))
                    } else if self.starts_brace_block() {
                        Some(Box::new(self.parse_brace_block()?))
                    } else {
                        None
                    };
                    refuse_two_blocks(&arguments, trailing_block.is_some())?;
                    expr = Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: "call".to_string(),
                        arguments,
                        trailing_block,
                        position,
                    };
                    continue;
                }
                // Allow `.[]` to name the `[]` method explicitly.
                if self.check(&[TokenKind::LBracket])
                    && matches!(self.peek_ahead(1).kind, TokenKind::RBracket)
                {
                    self.advance();
                    self.advance();
                    // `.[]=` names the writer, which takes its subscripts and
                    // the value together in the parentheses that follow.
                    let name = if self.match_token(&[TokenKind::Equal]) {
                        "[]=".to_string()
                    } else {
                        "[]".to_string()
                    };
                    let position = expr.position();
                    // `.[](index)` carries its index in the parentheses that
                    // follow, the same as any other named method call.
                    let arguments = if self.match_token(&[TokenKind::LParen]) {
                        self.parse_arguments()?
                    } else {
                        Vec::new()
                    };
                    expr = Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: name,
                        arguments,
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
                    TokenKind::Alias => "alias".to_string(),
                    TokenKind::Until => "until".to_string(),
                    TokenKind::AttrReader => "attr_reader".to_string(),
                    TokenKind::AttrWriter => "attr_writer".to_string(),
                    TokenKind::AttrAccessor => "attr_accessor".to_string(),
                    TokenKind::Yield => "yield".to_string(),
                    TokenKind::Defined => "defined?".to_string(),
                    TokenKind::Nil => "nil".to_string(),
                    TokenKind::True => "true".to_string(),
                    TokenKind::False => "false".to_string(),
                    // Operators named as methods: `obj.<=>(other)`, `a.+(b)`.
                    TokenKind::Plus => "+".to_string(),
                    TokenKind::Minus => "-".to_string(),
                    TokenKind::Star => "*".to_string(),
                    TokenKind::StarStar => "**".to_string(),
                    TokenKind::Slash => "/".to_string(),
                    TokenKind::Percent => "%".to_string(),
                    TokenKind::EqualEqual => "==".to_string(),
                    TokenKind::TripleEqual => "===".to_string(),
                    TokenKind::BangEqual => "!=".to_string(),
                    TokenKind::Less => "<".to_string(),
                    TokenKind::Greater => ">".to_string(),
                    TokenKind::LessEqual => "<=".to_string(),
                    TokenKind::GreaterEqual => ">=".to_string(),
                    TokenKind::Spaceship => "<=>".to_string(),
                    TokenKind::Shovel => "<<".to_string(),
                    TokenKind::Pipe => "|".to_string(),
                    TokenKind::Ampersand => "&".to_string(),
                    TokenKind::Bang => "!".to_string(),
                    TokenKind::Tilde => "~".to_string(),
                    TokenKind::Match => "=~".to_string(),
                    TokenKind::NotMatch => "!~".to_string(),
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
                let trailing_block = if self.starts_do_block() && self.paren_less_arg_depth == 0 {
                    Some(Box::new(self.parse_block()?))
                } else if self.starts_brace_block() {
                    Some(Box::new(self.parse_brace_block()?))
                } else {
                    None
                };
                refuse_two_blocks(&arguments, trailing_block.is_some())?;

                let position = expr.position();
                expr = if safe {
                    let mut relayed = vec![Expression::Symbol {
                        value: method_name,
                        position,
                    }];
                    relayed.extend(arguments);
                    Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: SAFE_CALL.to_string(),
                        arguments: relayed,
                        trailing_block,
                        position,
                    }
                } else {
                    // `"text".freeze` stands for one frozen string shared by
                    // every place the same literal is written.
                    let method_name = if method_name == "freeze"
                        && arguments.is_empty()
                        && trailing_block.is_none()
                        && names_a_string_literal(&expr)
                    {
                        "__frozen_literal__".to_string()
                    } else {
                        method_name
                    };
                    Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: method_name,
                        arguments,
                        trailing_block,
                        position,
                    }
                };
            } else if self.check(&[TokenKind::LBracket])
                && !self.bracket_starts_array_argument(&expr)
            {
                self.advance();
                // A subscript may be written across several lines, so a
                // newline after the bracket carries no meaning.
                self.skip_whitespace();
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
                    // A `name:` or `key =>` opening the brackets makes the
                    // whole subscript a Hash, which is what `Hash[a: 1]` and
                    // `Hash[1 => 2]` pass.
                    let opens_hash = self.bracket_opens_hash();
                    // An index may be written as an assignment, which answers
                    // what it assigned: `array[i += 1]`.
                    let first_arg = if opens_hash {
                        None
                    } else {
                        Some(self.parse_bracket_argument()?)
                    };
                    if first_arg.is_some() {
                        self.skip_whitespace();
                    }
                    // `held[&block]` hands `[]` a block, which makes the
                    // subscript a call rather than a plain index.
                    let carries_a_block = matches!(first_arg, Some(Expression::BlockArg { .. }));
                    if opens_hash
                        || carries_a_block
                        || self.check(&[TokenKind::FatArrow])
                        || self.check(&[TokenKind::Comma])
                    {
                        let mut args: Vec<Expression> = first_arg.into_iter().collect();
                        // Trailing `key => value` and `name: value` pairs
                        // gather into one Hash, which is what
                        // `Array[1, 2, 3 => 4]` passes.
                        let mut pairs: Vec<(Expression, Expression)> = Vec::new();
                        if let Some(pair) = self.parse_bracket_hash_pair()? {
                            pairs.push(pair);
                        } else if self.match_token(&[TokenKind::FatArrow]) {
                            self.skip_whitespace();
                            let key = args.pop().expect("a key was parsed");
                            let value = self.parse_expression()?;
                            pairs.push((key, value));
                        }
                        while self.match_token(&[TokenKind::Comma]) {
                            self.skip_whitespace();
                            if self.check(&[TokenKind::RBracket]) {
                                break;
                            }
                            if let Some(pair) = self.parse_bracket_hash_pair()? {
                                pairs.push(pair);
                                continue;
                            }
                            let argument = self.parse_bracket_argument()?;
                            self.skip_whitespace();
                            if self.match_token(&[TokenKind::FatArrow]) {
                                self.skip_whitespace();
                                let value = self.parse_expression()?;
                                pairs.push((argument, value));
                            } else {
                                args.push(argument);
                            }
                        }
                        if !pairs.is_empty() {
                            let position = self.peek().position;
                            args.push(Expression::Dictionary {
                                entries: pairs,
                                position,
                            });
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
                            index: Box::new(first_arg.expect("an index was parsed")),
                            position,
                        };
                    }
                }
            } else if self.check(&[TokenKind::ColonColon])
                && !(self.peek().had_leading_space
                    && matches!(&expr, Expression::Identifier { name, .. } if !self.bound_names.contains(name)))
            {
                // `Foo::Bar` reads a name out of Foo, and `take ::Bar` passes
                // the top-level Bar to `take`. The space before `::` is what
                // tells them apart, and only for a name the file never binds,
                // which is the one that can be a paren-less call.
                self.advance();
                // Scope resolution (e.g., Math::PI, Foo::Bar)
                let position = expr.position();
                let name_token = self.advance();
                let name = match name_token.kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected constant name after '::'")),
                };
                // A constant is capitalized, so a lowercase name after `::`
                // names a method rather than something in the namespace.
                if name.starts_with(|first: char| first.is_lowercase() || first == '_') {
                    let arguments = if self.match_token(&[TokenKind::LParen]) {
                        self.parse_arguments()?
                    } else if self.can_start_argument_for_method_call(&name) {
                        self.parse_arguments_without_parens()?
                    } else {
                        Vec::new()
                    };
                    let trailing_block = if self.starts_do_block() && self.paren_less_arg_depth == 0
                    {
                        Some(Box::new(self.parse_block()?))
                    } else if self.starts_brace_block() {
                        Some(Box::new(self.parse_brace_block()?))
                    } else {
                        None
                    };
                    refuse_two_blocks(&arguments, trailing_block.is_some())?;
                    expr = Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: name,
                        arguments,
                        trailing_block,
                        position,
                    };
                    continue;
                }
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
                    || (self.starts_do_block() && self.paren_less_arg_depth == 0)
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
    /// Whether a `...` in an argument list is argument forwarding rather than
    /// the start of a beginless range: forwarding stands alone, so the next
    /// token closes the list or separates it from the next argument.
    pub(crate) fn dots_are_forwarding(&self) -> bool {
        matches!(
            self.peek_ahead(1).kind,
            TokenKind::RParen | TokenKind::Comma
        )
    }

    /// of `parse_call` (e.g. `parse_arrow_lambda`) can still have method chains
    /// like `-> { ... }.should raise_error(NameError)`.
    /// Whether a `do` here opens a block. Inside a loop's condition it closes
    /// the condition instead, which is what `while x do y end` reads it as.
    /// One argument inside a subscript. A `&` hands `[]` a block, the way it
    /// does in any other argument list.
    pub(crate) fn parse_bracket_argument(&mut self) -> Result<Expression, MetorexError> {
        if !self.check(&[TokenKind::Ampersand]) {
            return self.parse_expression_with_assignment();
        }
        let position = self.peek().position;
        self.advance();
        self.skip_whitespace();
        let expression = if self.check(&[TokenKind::Comma, TokenKind::RBracket]) {
            Expression::Identifier {
                name: crate::parser::ANONYMOUS_BLOCK.to_string(),
                position,
            }
        } else {
            self.parse_expression()?
        };
        Ok(Expression::BlockArg {
            expression: Box::new(expression),
            position,
        })
    }

    pub(crate) fn starts_do_block(&self) -> bool {
        self.check(&[TokenKind::Do]) && self.condition_depth == 0
    }

    pub(crate) fn parse_postfix_calls(
        &mut self,
        initial: Expression,
    ) -> Result<Expression, MetorexError> {
        // Re-enter the same postfix loop that `parse_call` uses, seeded with
        // the already-parsed expression.
        let mut expr = initial;
        loop {
            self.step_over_fluent_break();
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
                let trailing_block = if self.starts_do_block() && self.paren_less_arg_depth == 0 {
                    Some(Box::new(self.parse_block()?))
                } else if self.starts_brace_block() {
                    Some(Box::new(self.parse_brace_block()?))
                } else {
                    None
                };
                refuse_two_blocks(&arguments, trailing_block.is_some())?;
                let position = expr.position();
                // `"text".freeze` stands for one frozen string shared by
                // every place the same literal is written.
                let method_name = if method_name == "freeze"
                    && arguments.is_empty()
                    && trailing_block.is_none()
                    && names_a_string_literal(&expr)
                {
                    "__frozen_literal__".to_string()
                } else {
                    method_name
                };
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
        let trailing_block = if self.starts_do_block() && self.paren_less_arg_depth == 0 {
            Some(Box::new(self.parse_block()?))
        } else if self.starts_brace_block() {
            Some(Box::new(self.parse_brace_block()?))
        } else {
            None
        };
        refuse_two_blocks(&arguments, trailing_block.is_some())?;

        let position = callee.position();

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            trailing_block,
            position,
        })
    }

    /// A chain may be written with the dot leading the next line, so the
    /// newlines and comments between one call and that dot are stepped over.
    /// Anything else leaves the walk where it was.
    pub(crate) fn step_over_fluent_break(&mut self) {
        let resume = self.stream.current_position();
        let mut walked = 0;
        while matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_)) {
            self.advance();
            walked += 1;
            if walked > 64 {
                break;
            }
        }
        if !self.check(&[TokenKind::Dot]) {
            self.stream.restore_position(resume);
        }
    }
}
