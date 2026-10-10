// Reading one pattern of a `case/in` clause.

use super::*;

impl Parser {
    /// Parse a single pattern for `case/in`, supporting `pattern => name` binding
    /// at the top level and inside array patterns.
    pub(crate) fn parse_case_in_pattern(&mut self) -> Result<MatchPattern, MetorexError> {
        self.parse_in_pattern()
    }

    /// Parse a pattern for a case statement
    /// Supports:
    /// - Literal patterns (integers, strings, booleans, nil)
    /// - Wildcard pattern (_)
    /// - Variable binding pattern (identifier)
    /// - Array destructuring ([a, b, c] or [first, ...rest])
    /// - Object destructuring ({x, y} or {x: a, y: b})
    ///
    /// This method is public within the parser module so it can be used
    /// by both statement parsing (case statements) and expression parsing (case expressions)
    /// Parse a pattern that may include comma-separated alternatives
    /// Returns a MatchPattern::Multiple if multiple patterns are found, otherwise a single pattern
    pub(in crate::parser) fn parse_when_pattern_with_alternatives(
        &mut self,
    ) -> Result<MatchPattern, MetorexError> {
        self.in_when_clause = true;
        let parsed = self.parse_case_pattern_with_alternatives();
        self.in_when_clause = false;
        parsed
    }

    /// A pattern written inside a `[…]` or `{…}` pattern, where a bare name
    /// binds the element it stands against rather than naming a value to
    /// compare with.
    pub(crate) fn parse_nested_case_pattern(&mut self) -> Result<MatchPattern, MetorexError> {
        let outer = self.in_when_clause;
        self.in_when_clause = false;
        let parsed = self.parse_case_pattern();
        self.in_when_clause = outer;
        parsed
    }

    pub(in crate::parser) fn parse_case_pattern_with_alternatives(
        &mut self,
    ) -> Result<MatchPattern, MetorexError> {
        // Parse the first pattern. Do NOT skip newlines after the pattern —
        // the caller relies on being able to distinguish a same-line `if`
        // (a guard clause) from a body `if` statement on the next line.
        let first_pattern = self.parse_case_pattern()?;

        // Check if there's a comma indicating multiple patterns
        if !self.check(&[TokenKind::Comma]) {
            // Single pattern, return as-is
            return Ok(first_pattern);
        }

        // Multiple patterns: collect all comma-separated patterns
        let mut patterns = vec![first_pattern];

        while self.match_token(&[TokenKind::Comma]) {
            self.skip_whitespace();

            // Check if we've hit a terminal token (then, if, newline context)
            // This prevents consuming commas from other constructs
            if self.check(&[
                TokenKind::Then,
                TokenKind::If,
                TokenKind::When,
                TokenKind::Else,
                TokenKind::End,
            ]) {
                break;
            }

            // The line break after the last value ends the list, so an `if`
            // on the next line opens the clause's body.
            patterns.push(self.parse_case_pattern()?);
        }

        // If we only collected one pattern, return it directly
        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            Ok(MatchPattern::Multiple(patterns))
        }
    }

    /// Parse a single case pattern (internal helper)
    pub(in crate::parser) fn parse_case_pattern(&mut self) -> Result<MatchPattern, MetorexError> {
        let token = self.peek().clone();

        match &token.kind {
            // Array pattern
            TokenKind::LBracket => {
                self.advance(); // consume '['
                self.skip_whitespace();

                let mut patterns = Vec::new();

                // Parse patterns inside the array
                while !self.check(&[TokenKind::RBracket]) && !self.is_at_end() {
                    self.skip_whitespace();

                    // Check for rest pattern (...)
                    if self.match_token(&[TokenKind::DotDotDot]) {
                        self.skip_whitespace();

                        // Next token should be an identifier for the rest binding
                        if let TokenKind::Ident(name) = &self.peek().kind {
                            let rest_name = name.clone();
                            self.advance();
                            patterns.push(MatchPattern::Rest(rest_name));
                        } else {
                            return Err(MetorexError::syntax_error(
                                "Expected identifier after ... in array pattern".to_string(),
                                SourceLocation::new(
                                    self.peek().position.line,
                                    self.peek().position.column,
                                    self.peek().position.offset,
                                ),
                            ));
                        }
                    } else {
                        // Parse a regular pattern
                        patterns.push(self.parse_nested_case_pattern()?);
                    }

                    self.skip_whitespace();

                    // Check for comma
                    if !self.check(&[TokenKind::RBracket]) {
                        self.expect(TokenKind::Comma, "Expected ',' or ']' in array pattern")?;
                        self.skip_whitespace();
                    }
                }

                self.expect(TokenKind::RBracket, "Expected ']' after array pattern")?;
                Ok(MatchPattern::Array(patterns))
            }

            // Object/Dictionary pattern
            TokenKind::LBrace => {
                self.advance(); // consume '{'
                self.skip_whitespace();

                let mut key_patterns = Vec::new();

                // Parse key-pattern pairs inside the object
                while !self.check(&[TokenKind::RBrace]) && !self.is_at_end() {
                    self.skip_whitespace();

                    // Expect an identifier as the key
                    let key = if let TokenKind::Ident(name) = &self.peek().kind {
                        let k = name.clone();
                        self.advance();
                        k
                    } else if let TokenKind::String(s)
                    | TokenKind::FrozenString(s)
                    | TokenKind::MutableString(s) = &self.peek().kind
                    {
                        let k = s.clone();
                        self.advance();
                        k
                    } else {
                        return Err(MetorexError::syntax_error(
                            "Expected identifier or string key in object pattern".to_string(),
                            SourceLocation::new(
                                self.peek().position.line,
                                self.peek().position.column,
                                self.peek().position.offset,
                            ),
                        ));
                    };

                    self.skip_whitespace();

                    // Check if there's a colon for explicit pattern (e.g., {x: a, y: b})
                    let pattern = if self.match_token(&[TokenKind::Colon]) {
                        self.skip_whitespace();
                        self.parse_nested_case_pattern()?
                    } else {
                        // Shorthand: {x, y} means {x: x, y: y}
                        MatchPattern::Identifier(key.clone())
                    };

                    key_patterns.push((key, pattern));

                    self.skip_whitespace();

                    // Check for comma
                    if !self.check(&[TokenKind::RBrace]) {
                        self.expect(TokenKind::Comma, "Expected ',' or '}' in object pattern")?;
                        self.skip_whitespace();
                    }
                }

                self.expect(TokenKind::RBrace, "Expected '}' after object pattern")?;
                Ok(MatchPattern::Object(key_patterns))
            }

            // Literal patterns (may be followed by .. or ... to form a range pattern)
            TokenKind::Int(n) => {
                let value = *n;
                self.advance();
                let start = MatchPattern::IntLiteral(value);
                self.parse_range_pattern_suffix(start)
            }
            TokenKind::Float(f) => {
                let value = *f;
                self.advance();
                let start = MatchPattern::FloatLiteral(value);
                self.parse_range_pattern_suffix(start)
            }
            // A source that asked for frozen literals, or outright for ones
            // that change, spells a literal in a pattern the same way.
            TokenKind::String(s) | TokenKind::FrozenString(s) | TokenKind::MutableString(s) => {
                let value = s.clone();
                self.advance();
                let start = MatchPattern::StringLiteral(value);
                self.parse_range_pattern_suffix(start)
            }
            // Symbol pattern, written any way a Symbol literal can be:
            // `:name`, `:"!"`, `:+`, `:return`.
            TokenKind::Colon => {
                let position = self.advance().position;
                match self.parse_symbol_literal(position)? {
                    Expression::Symbol { value, .. } => Ok(MatchPattern::SymbolLiteral(value)),
                    built => Ok(MatchPattern::Expression(Box::new(built))),
                }
            }
            TokenKind::True => {
                self.advance();
                Ok(MatchPattern::BoolLiteral(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(MatchPattern::BoolLiteral(false))
            }
            TokenKind::Nil => {
                self.advance();
                Ok(MatchPattern::NilLiteral)
            }
            // Wildcard pattern
            TokenKind::Ident(name) if name == "_" => {
                self.advance();
                Ok(MatchPattern::Wildcard)
            }
            // A constant named from the top level, `when ::String`. Metorex
            // resolves a bare constant there anyway, so the leading `::` only
            // says where to start looking.
            TokenKind::ColonColon
                if matches!(&self.peek_ahead(1).kind,
                    TokenKind::Ident(name) if name.chars().next().is_some_and(char::is_uppercase)) =>
            {
                self.advance();
                self.parse_case_pattern()
            }
            // Type pattern (capitalized identifiers like Integer, String, Hash, Array)
            TokenKind::Ident(name) if name.chars().next().is_some_and(|c| c.is_uppercase()) => {
                let mut type_name = name.clone();
                self.advance();
                // A constant may be reached through the module holding it, as
                // `when Socket::SOCK_STREAM`, so the whole path is the name.
                while self.check(&[TokenKind::ColonColon]) {
                    let TokenKind::Ident(segment) = &self.peek_ahead(1).kind else {
                        break;
                    };
                    let segment = segment.clone();
                    self.advance();
                    self.advance();
                    type_name.push_str("::");
                    type_name.push_str(&segment);
                }
                Ok(MatchPattern::Type(type_name))
            }
            // A call written where a pattern goes, such as `when klass.new`,
            // stands for what it answers.
            TokenKind::Ident(_)
                if matches!(
                    self.peek_ahead(1).kind,
                    TokenKind::Dot | TokenKind::SafeDot | TokenKind::LParen
                ) =>
            {
                let held = self.parse_expression()?;
                Ok(MatchPattern::Expression(Box::new(held)))
            }
            // `when name` compares against what the name holds. A `when`
            // clause has no binding form, so a bare identifier there is an
            // expression rather than a variable to bind.
            TokenKind::Ident(_) if self.in_when_clause => {
                let held = self.parse_expression()?;
                Ok(MatchPattern::Expression(Box::new(held)))
            }
            // Variable binding pattern
            TokenKind::Ident(name) => {
                let var_name = name.clone();
                self.advance();
                Ok(MatchPattern::Identifier(var_name))
            }
            // `when (expr; expr)` reads a parenthesized run of statements,
            // whose last value is what the case compares against.
            TokenKind::LParen => {
                let held = self.parse_expression()?;
                Ok(MatchPattern::Expression(Box::new(held)))
            }
            // `when *values` names each of the values as a choice.
            TokenKind::Star if self.in_when_clause => {
                let star = self.advance().position;
                let spread = self.parse_expression()?;
                Ok(MatchPattern::Expression(Box::new(Expression::Splat {
                    expression: Box::new(spread),
                    position: star,
                })))
            }
            // A `when` may be written with anything that answers `===`, such
            // as a pattern literal, so what is left is read as an expression.
            TokenKind::Regex(_, _) => {
                let held = self.parse_primary()?;
                Ok(MatchPattern::Expression(Box::new(held)))
            }
            // A `when` compares against any expression, such as `-1` or
            // `!flag`.
            _ if self.in_when_clause => {
                let held = self.parse_expression()?;
                Ok(MatchPattern::Expression(Box::new(held)))
            }
            _ => Err(MetorexError::syntax_error(
                format!("Expected pattern, found {:?}", token.kind),
                SourceLocation::new(
                    token.position.line,
                    token.position.column,
                    token.position.offset,
                ),
            )),
        }
    }

    /// If `..` or `...` follows a literal pattern, wrap it in a Range pattern.
    pub(crate) fn parse_range_pattern_suffix(
        &mut self,
        start: MatchPattern,
    ) -> Result<MatchPattern, MetorexError> {
        if self.match_token(&[TokenKind::DotDotDot]) {
            let end = self.parse_case_pattern()?;
            Ok(MatchPattern::Range {
                start: Box::new(start),
                end: Box::new(end),
                exclusive: true,
            })
        } else if self.match_token(&[TokenKind::DotDot]) {
            let end = self.parse_case_pattern()?;
            Ok(MatchPattern::Range {
                start: Box::new(start),
                end: Box::new(end),
                exclusive: false,
            })
        } else {
            Ok(start)
        }
    }
}
