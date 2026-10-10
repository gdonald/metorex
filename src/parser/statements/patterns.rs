// The pattern grammar `in` reads, which is its own language rather than the
// `when` clause's list of values. A pattern names the shape a value has to
// have and binds the parts of it the pattern gives names to.

use crate::ast::{Expression, HashPatternRest, MatchPattern};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// A whole `in` clause's pattern. Written without brackets, a run of
    /// comma-separated patterns is an array pattern, so `in a, b` reads the
    /// same as `in [a, b]`.
    pub(in crate::parser) fn parse_in_pattern(&mut self) -> Result<MatchPattern, MetorexError> {
        self.pattern_names.clear();
        // A pattern opening with `*` names the rest of an array from the
        // start, as `in *rest, last` does.
        if self.check(&[TokenKind::Star]) {
            return self.parse_bare_array_pattern(Vec::new());
        }
        // A clause opening with `**` is a hash pattern written without
        // braces, which `in **rest` and `in **nil` both are.
        if self.check(&[TokenKind::StarStar]) {
            return self.parse_bare_hash_pattern(None);
        }
        let first = self.parse_pattern_alternatives()?;
        self.skip_pattern_comments();
        if self.check(&[TokenKind::Comma]) {
            return self.parse_bare_array_pattern(vec![first]);
        }
        // A lone `key:` at the top of a clause is a hash pattern written
        // without braces, which `parse_pattern_unit` already answered as one.
        Ok(first)
    }

    /// The rest of a bracket-less array pattern, given the patterns already
    /// read from it.
    pub(crate) fn parse_bare_array_pattern(
        &mut self,
        parsed: Vec<MatchPattern>,
    ) -> Result<MatchPattern, MetorexError> {
        let mut prefix = parsed;
        let mut rest: Option<Option<String>> = None;
        let mut suffix = Vec::new();
        loop {
            if !prefix.is_empty() || rest.is_some() || !suffix.is_empty() {
                self.skip_pattern_comments();
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
                self.skip_pattern_comments();
                // A trailing comma ends the pattern, and says the value may
                // hold more than the pattern named.
                if self.ends_a_pattern() {
                    rest.get_or_insert(None);
                    break;
                }
            }
            if self.check(&[TokenKind::Star]) {
                if rest.is_some() {
                    return Err(self.error_at_current("Only one rest is allowed in a pattern"));
                }
                rest = Some(self.parse_pattern_rest_name()?);
                continue;
            }
            let held = self.parse_pattern_alternatives()?;
            if rest.is_some() {
                suffix.push(held);
            } else {
                prefix.push(held);
            }
        }
        Ok(MatchPattern::ArrayPattern {
            constant: None,
            prefix,
            rest,
            suffix,
        })
    }

    /// `pattern | pattern | …`, which matches where any one of them does,
    /// and `pattern => name`, which binds the whole value to the name. The
    /// `=>` binds looser than `|`, so `A | B => name` names what either
    /// side matched.
    pub(crate) fn parse_pattern_alternatives(&mut self) -> Result<MatchPattern, MetorexError> {
        let opened_with = self.pattern_names.len();
        let first = self.parse_pattern_unit()?;
        self.skip_pattern_comments();
        let mut held = if self.check(&[TokenKind::Pipe]) {
            let mut choices = vec![first];
            while self.match_token(&[TokenKind::Pipe]) {
                self.skip_whitespace();
                choices.push(self.parse_pattern_unit()?);
                self.skip_pattern_comments();
            }
            // Only one side of an alternative runs, so a name bound in one
            // of them would hold nothing in the others. Ruby refuses the
            // pattern.
            if self.pattern_names.len() != opened_with {
                return Err(self.error_at_current("illegal variable in alternative pattern"));
            }
            MatchPattern::Multiple(choices)
        } else {
            first
        };
        loop {
            self.skip_pattern_comments();
            if !self.match_token(&[TokenKind::FatArrow]) {
                return Ok(held);
            }
            self.skip_whitespace();
            let TokenKind::Ident(name) = self.peek().kind.clone() else {
                return Err(self.error_at_current("Expected a name after '=>' in a pattern"));
            };
            self.advance();
            self.note_bound_name(&name)?;
            held = MatchPattern::Bind {
                pattern: Box::new(held),
                name,
            };
        }
    }

    /// One pattern, without the alternatives or the binding around it.
    pub(crate) fn parse_pattern_unit(&mut self) -> Result<MatchPattern, MetorexError> {
        self.skip_pattern_comments();
        match self.peek().kind.clone() {
            TokenKind::LBracket => self.parse_bracket_array_pattern(None),
            TokenKind::LBrace => self.parse_brace_hash_pattern(None),
            TokenKind::Caret => self.parse_pinned_pattern(),
            // `(pattern)` groups a pattern, which is how an alternative is
            // written inside a larger one.
            TokenKind::LParen => {
                self.advance();
                self.skip_whitespace();
                let held = self.parse_pattern_alternatives()?;
                self.skip_whitespace();
                self.expect(TokenKind::RParen, "Expected ')' after a pattern")?;
                Ok(held)
            }
            TokenKind::Ident(name) if name == "_" || name.starts_with('_') => {
                self.advance();
                // A name beginning with an underscore binds like any other,
                // and `_` alone is the one that names nothing.
                if name == "_" {
                    return Ok(MatchPattern::Wildcard);
                }
                self.note_bound_name(&name)?;
                Ok(MatchPattern::Identifier(name))
            }
            // A constant may be followed by the array or hash pattern it
            // narrows, as `Point[x, y]` and `Point(x:, y:)` are written.
            TokenKind::Ident(name) if name.chars().next().is_some_and(char::is_uppercase) => {
                let _ = &name;
                let named = self.parse_pattern_constant()?;
                match self.peek().kind {
                    TokenKind::LBracket => self.parse_bracket_array_pattern(Some(named)),
                    TokenKind::LParen => self.parse_paren_constant_pattern(named),
                    _ => Ok(self.range_pattern_suffix(MatchPattern::Expression(named))?),
                }
            }
            TokenKind::Ident(name) => {
                // A bare name binds the value it stands against, unless a
                // label follows it, which opens a hash pattern.
                if self.names_a_hash_label() {
                    return self.parse_bare_hash_pattern(None);
                }
                self.advance();
                self.note_bound_name(&name)?;
                Ok(MatchPattern::Identifier(name))
            }
            // A hash pattern written without braces may open with a quoted
            // label, as `in "a": 1` is written.
            TokenKind::String(_) | TokenKind::FrozenString(_) | TokenKind::MutableString(_)
                if self.names_a_hash_label() =>
            {
                self.parse_bare_hash_pattern(None)
            }
            _ => {
                let held = self.parse_pattern_value()?;
                self.range_pattern_suffix(MatchPattern::Expression(held))
            }
        }
    }

    /// `^name`, `^@name`, `^$name`, or `^(expr)`: what the expression answers
    /// is compared against rather than a name being bound.
    pub(crate) fn parse_pinned_pattern(&mut self) -> Result<MatchPattern, MetorexError> {
        let caret = self.advance().position;
        let held = match self.peek().kind.clone() {
            TokenKind::LParen => {
                self.advance();
                self.skip_whitespace();
                let held = self.parse_expression()?;
                self.skip_whitespace();
                self.expect(TokenKind::RParen, "Expected ')' after a pinned expression")?;
                held
            }
            TokenKind::Ident(name) => {
                self.advance();
                if !self.pattern_names.contains(&name) && !self.names_a_local(&name) {
                    return Err(self.error_at_previous(&format!("{name}: no such local variable")));
                }
                Expression::Identifier {
                    name,
                    position: caret,
                }
            }
            TokenKind::InstanceVar(name) => {
                self.advance();
                Expression::InstanceVariable {
                    name,
                    position: caret,
                }
            }
            TokenKind::ClassVar(name) => {
                self.advance();
                Expression::ClassVariable {
                    name,
                    position: caret,
                }
            }
            TokenKind::GlobalVar(name) => {
                self.advance();
                Expression::GlobalVariable {
                    name,
                    position: caret,
                }
            }
            _ => return Err(self.error_at_current("Expected a name or '(' after '^'")),
        };
        Ok(MatchPattern::Pinned(Box::new(held)))
    }

    /// The constant a pattern names, taking the whole `A::B::C` path.
    pub(crate) fn parse_pattern_constant(&mut self) -> Result<Box<Expression>, MetorexError> {
        let token = self.advance();
        let position = token.position;
        let TokenKind::Ident(name) = token.kind else {
            return Err(self.error_at_previous("Expected a constant"));
        };
        let mut held = Expression::Identifier { name, position };
        while self.check(&[TokenKind::ColonColon]) {
            let TokenKind::Ident(segment) = self.peek_ahead(1).kind.clone() else {
                break;
            };
            self.advance();
            self.advance();
            held = Expression::ScopeResolution {
                namespace: Box::new(held),
                name: segment,
                position,
            };
        }
        Ok(Box::new(held))
    }

    /// `Const(…)`, which holds either an array pattern or a hash pattern
    /// depending on what is written inside it.
    pub(crate) fn parse_paren_constant_pattern(
        &mut self,
        constant: Box<Expression>,
    ) -> Result<MatchPattern, MetorexError> {
        self.advance(); // consume '('
        self.skip_whitespace();
        if self.check(&[TokenKind::RParen]) {
            self.advance();
            return Ok(MatchPattern::ArrayPattern {
                constant: Some(constant),
                prefix: Vec::new(),
                rest: None,
                suffix: Vec::new(),
            });
        }
        let holds_keys = self.names_a_hash_label() || self.check(&[TokenKind::StarStar]);
        let held = if holds_keys {
            self.parse_hash_pattern_body(Some(constant), TokenKind::RParen)?
        } else {
            self.parse_array_pattern_body(Some(constant), TokenKind::RParen)?
        };
        self.expect(TokenKind::RParen, "Expected ')' after a pattern")?;
        Ok(held)
    }

    /// `[…]`, with the constant it narrows where one was written.
    pub(crate) fn parse_bracket_array_pattern(
        &mut self,
        constant: Option<Box<Expression>>,
    ) -> Result<MatchPattern, MetorexError> {
        self.advance(); // consume '['
        self.skip_whitespace();
        // `Const[a:, b:]` holds keys rather than elements, which is the hash
        // pattern written with brackets.
        let held = if self.names_a_hash_label() || self.check(&[TokenKind::StarStar]) {
            self.parse_hash_pattern_body(constant, TokenKind::RBracket)?
        } else {
            self.parse_array_pattern_body(constant, TokenKind::RBracket)?
        };
        self.expect(TokenKind::RBracket, "Expected ']' after an array pattern")?;
        Ok(held)
    }

    /// The patterns between the brackets, up to `closing`. Two rests make a
    /// find pattern, which looks for its middle anywhere in the value.
    pub(crate) fn parse_array_pattern_body(
        &mut self,
        constant: Option<Box<Expression>>,
        closing: TokenKind,
    ) -> Result<MatchPattern, MetorexError> {
        let mut prefix = Vec::new();
        let mut rest: Option<Option<String>> = None;
        let mut suffix = Vec::new();
        let mut second_rest: Option<Option<String>> = None;
        loop {
            self.skip_whitespace();
            if self.check(std::slice::from_ref(&closing)) || self.is_at_end() {
                break;
            }
            if self.check(&[TokenKind::Star]) {
                let named = self.parse_pattern_rest_name()?;
                if rest.is_none() {
                    rest = Some(named);
                } else if second_rest.is_none() {
                    second_rest = Some(named);
                } else {
                    return Err(self.error_at_current("Only two rests are allowed in a pattern"));
                }
            } else {
                let held = self.parse_pattern_alternatives()?;
                if rest.is_some() {
                    suffix.push(held);
                } else {
                    prefix.push(held);
                }
            }
            self.skip_whitespace();
            if !self.check(std::slice::from_ref(&closing)) {
                self.expect(TokenKind::Comma, "Expected ',' in an array pattern")?;
                self.skip_whitespace();
                // A comma with nothing after it says the value may hold more
                // than the pattern named, which `in [0, 1, ]` asks for.
                if self.check(std::slice::from_ref(&closing)) && rest.is_none() {
                    rest = Some(None);
                }
            }
        }
        // Two rests around a run make a find pattern, which is matched
        // against every place the run could sit.
        if let (Some(before), Some(after)) = (&rest, &second_rest) {
            if !prefix.is_empty() {
                return Err(self.error_at_current("A find pattern opens with its rest"));
            }
            return Ok(MatchPattern::FindPattern {
                constant,
                before: before.clone(),
                middle: suffix,
                after: after.clone(),
            });
        }
        Ok(MatchPattern::ArrayPattern {
            constant,
            prefix,
            rest,
            suffix,
        })
    }

    /// The name a `*` or `**` binds, or None where it names nothing.
    pub(crate) fn parse_pattern_rest_name(&mut self) -> Result<Option<String>, MetorexError> {
        self.advance(); // consume '*' or '**'
        if let TokenKind::Ident(name) = self.peek().kind.clone()
            && !name.chars().next().is_some_and(char::is_uppercase)
        {
            self.advance();
            self.note_bound_name(&name)?;
            return Ok(Some(name));
        }
        Ok(None)
    }

    /// `{…}`, with the constant it narrows where one was written.
    pub(crate) fn parse_brace_hash_pattern(
        &mut self,
        constant: Option<Box<Expression>>,
    ) -> Result<MatchPattern, MetorexError> {
        self.advance(); // consume '{'
        self.skip_whitespace();
        let held = self.parse_hash_pattern_body(constant, TokenKind::RBrace)?;
        self.expect(TokenKind::RBrace, "Expected '}' after a hash pattern")?;
        Ok(held)
    }

    /// A hash pattern written without braces, as `in a:, b:` is.
    pub(crate) fn parse_bare_hash_pattern(
        &mut self,
        constant: Option<Box<Expression>>,
    ) -> Result<MatchPattern, MetorexError> {
        self.parse_hash_pattern_body(constant, TokenKind::Then)
    }

    /// The `key:` entries up to `closing`, each with the pattern its value
    /// has to match or nothing where the key binds a name of its own.
    pub(crate) fn parse_hash_pattern_body(
        &mut self,
        constant: Option<Box<Expression>>,
        closing: TokenKind,
    ) -> Result<MatchPattern, MetorexError> {
        let mut entries: Vec<(String, Option<MatchPattern>)> = Vec::new();
        let mut rest = HashPatternRest::Silent;
        loop {
            self.skip_whitespace();
            if self.check(std::slice::from_ref(&closing)) || self.ends_a_pattern() {
                break;
            }
            if self.check(&[TokenKind::StarStar]) {
                self.advance();
                if self.check(&[TokenKind::Nil]) {
                    self.advance();
                    rest = HashPatternRest::Refused;
                } else if let TokenKind::Ident(name) = self.peek().kind.clone() {
                    self.advance();
                    self.note_bound_name(&name)?;
                    rest = HashPatternRest::Named(name);
                } else {
                    rest = HashPatternRest::Anonymous;
                }
            } else {
                let key = self.parse_hash_pattern_key()?;
                self.skip_pattern_comments();
                // `key:` on its own binds a local of that name, and anything
                // else after the colon is the pattern the value has to match.
                let held = if self.ends_a_hash_entry(&closing) {
                    self.note_bound_name(&key)?;
                    None
                } else {
                    Some(self.parse_pattern_alternatives()?)
                };
                if entries.iter().any(|(named, _)| *named == key) {
                    return Err(self.error_at_current(&format!("duplicated key name {key}")));
                }
                entries.push((key, held));
            }
            self.skip_pattern_comments();
            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }
        Ok(MatchPattern::HashPattern {
            constant,
            entries,
            rest,
        })
    }

    /// The name a hash pattern's key is written with, without its colon.
    pub(crate) fn parse_hash_pattern_key(&mut self) -> Result<String, MetorexError> {
        let token = self.advance();
        let key = match token.kind {
            TokenKind::Ident(name) => name,
            TokenKind::String(text)
            | TokenKind::FrozenString(text)
            | TokenKind::MutableString(text) => text,
            TokenKind::Nil => "nil".to_string(),
            TokenKind::True => "true".to_string(),
            TokenKind::False => "false".to_string(),
            TokenKind::If => "if".to_string(),
            TokenKind::In => "in".to_string(),
            TokenKind::Then => "then".to_string(),
            _ => {
                return Err(
                    self.error_at_previous("expected a label as the key in the hash pattern")
                );
            }
        };
        if !self.match_token(&[TokenKind::Colon]) {
            return Err(self.error_at_current("expected a label as the key in the hash pattern"));
        }
        Ok(key)
    }

    /// Whether a hash pattern's entry ends here, which means the key binds a
    /// name of its own rather than naming a pattern.
    pub(crate) fn ends_a_hash_entry(&self, closing: &TokenKind) -> bool {
        self.check(std::slice::from_ref(closing))
            || self.check(&[TokenKind::Comma])
            || self.ends_a_pattern()
    }

    /// Whether the clause's pattern ends here rather than carrying on.
    pub(crate) fn ends_a_pattern(&self) -> bool {
        self.check(&[
            TokenKind::Then,
            TokenKind::If,
            TokenKind::Unless,
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::EOF,
        ])
    }

    /// Pass over what stands between the pieces of a pattern without
    /// crossing the line it is written on, since a newline ends a pattern.
    pub(crate) fn skip_pattern_comments(&mut self) {
        while matches!(self.peek().kind, TokenKind::Comment(_)) {
            self.advance();
        }
    }

    /// Whether what stands here is a `key:` rather than a value.
    pub(crate) fn names_a_hash_label(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Ident(_)
                | TokenKind::String(_)
                | TokenKind::FrozenString(_)
                | TokenKind::MutableString(_)
        ) && matches!(self.peek_ahead(1).kind, TokenKind::Colon)
    }
}

impl Parser {
    /// Say that a pattern binds this name, so the rest of its scope reads it
    /// as a local rather than as a call.
    pub(crate) fn note_bound_name(&mut self, name: &str) -> Result<(), MetorexError> {
        self.note_pattern_binding(name);
        // One pattern names each of its parts once, so a repeat has nothing
        // of its own to hold. A name opening with an underscore says it is
        // not read, so it may stand as often as the pattern needs it.
        if name.starts_with('_') {
            return Ok(());
        }
        if !self.pattern_names.insert(name.to_string()) {
            return Err(self.error_at_current(&format!("duplicated variable name {name}")));
        }
        Ok(())
    }

    /// The value a pattern is written with: a literal, a constant, a regexp,
    /// a lambda, or a parenthesized expression. A pattern holds no operators
    /// of its own beyond the ranges read after it.
    pub(crate) fn parse_pattern_value(&mut self) -> Result<Box<Expression>, MetorexError> {
        // A range with no left side, as `in ..5` is written, opens here.
        if self.check(&[TokenKind::DotDot, TokenKind::DotDotDot]) {
            let exclusive = matches!(self.peek().kind, TokenKind::DotDotDot);
            let position = self.advance().position;
            let end = self.parse_pattern_primary()?;
            return Ok(Box::new(Expression::Range {
                // A range with no left side reaches from as far down as the
                // values go, which nil stands for.
                start: Box::new(Expression::NilLiteral { position }),
                end: Box::new(end),
                exclusive,
                position,
            }));
        }
        Ok(Box::new(self.parse_pattern_primary()?))
    }

    /// One value written in a pattern. A `-` in front of a number belongs to
    /// the number, which is the only sign a pattern may carry.
    pub(crate) fn parse_pattern_primary(&mut self) -> Result<Expression, MetorexError> {
        if self.check(&[TokenKind::Minus]) {
            let position = self.advance().position;
            return match self.advance().kind {
                TokenKind::Int(held) => Ok(Expression::IntLiteral {
                    value: -held,
                    position,
                }),
                TokenKind::Float(held) => Ok(Expression::FloatLiteral {
                    value: -held,
                    position,
                }),
                _ => Err(self.error_at_previous("Expected a number after '-' in a pattern")),
            };
        }
        self.parse_primary()
    }

    /// A range written after a value pattern, which may name no end at all.
    pub(crate) fn range_pattern_suffix(
        &mut self,
        held: MatchPattern,
    ) -> Result<MatchPattern, MetorexError> {
        if !self.check(&[TokenKind::DotDot, TokenKind::DotDotDot]) {
            return Ok(held);
        }
        let exclusive = matches!(self.peek().kind, TokenKind::DotDotDot);
        let position = self.advance().position;
        let MatchPattern::Expression(start) = held else {
            return Err(self.error_at_previous("Expected a value before '..' in a pattern"));
        };
        // `in 1..` names everything from one upward, so what follows the dots
        // is an end only where a value stands there.
        let end = if self.opens_a_pattern_value() {
            self.parse_pattern_primary()?
        } else {
            Expression::NilLiteral { position }
        };
        Ok(MatchPattern::Expression(Box::new(Expression::Range {
            start,
            end: Box::new(end),
            exclusive,
            position,
        })))
    }

    /// Whether a value stands here, which is what tells a range's end from
    /// the end of the pattern holding it.
    pub(crate) fn opens_a_pattern_value(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::FrozenString(_)
                | TokenKind::MutableString(_)
                | TokenKind::Ident(_)
                | TokenKind::Minus
                | TokenKind::Colon
        )
    }
}
