// Reading `a, b = ...` and the targets it names.

use super::*;

impl Parser {
    /// Look ahead to check if this is a multiple assignment (a, b = ...)
    /// by scanning comma-separated identifiers until we find = or something else.
    pub(crate) fn scans_assignment_targets(&self, mut offset: usize, had_first: bool) -> bool {
        let mut seen = had_first;
        loop {
            // Skip whitespace tokens
            let tok = self.peek_ahead(offset);
            if matches!(
                tok.kind,
                TokenKind::Newline | TokenKind::Comment(_) | TokenKind::Semicolon
            ) {
                offset += 1;
                continue;
            }
            // `first, = pair` names one target and destructures anyway, so
            // the list may end on the comma that opened this step.
            if matches!(tok.kind, TokenKind::Equal) {
                return seen;
            }
            // `*rest` takes whatever the targets around it leave, and a
            // splat with no name after it takes them and keeps none.
            let tok = if matches!(tok.kind, TokenKind::Star) {
                offset += 1;
                let next = self.peek_ahead(offset);
                if matches!(next.kind, TokenKind::Equal) {
                    return true;
                }
                if matches!(next.kind, TokenKind::Comma) {
                    seen = true;
                    offset += 1;
                    continue;
                }
                next
            } else {
                tok
            };
            // A group of targets stands where a name may, as the `(y, z)`
            // of `x, (y, z) = held, pair` does.
            if matches!(tok.kind, TokenKind::LParen) {
                // A target list may be written across lines, so a newline
                // inside the group is not the end of it.
                let Some(after) = self.past_balanced(offset, TokenKind::LParen, TokenKind::RParen)
                else {
                    return false;
                };
                offset = after;
                // `(held; object).name` writes the receiver in parentheses,
                // and `(held; object)[key]` subscripts it, so what follows
                // the group names the target on it.
                let Some(after) = self.skip_target_postfix(offset, false) else {
                    return false;
                };
                offset = after;
                match &self.peek_ahead(offset).kind {
                    TokenKind::Equal => return true,
                    TokenKind::Comma => {
                        seen = true;
                        offset += 1;
                        continue;
                    }
                    _ => return false,
                }
            }
            // Expect an identifier (or @ivar, @@cvar, $gvar). `lambda` is a
            // method rather than syntax, so a program may name a local after
            // it and assign to that name.
            if !matches!(
                tok.kind,
                TokenKind::Ident(_)
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
                    | TokenKind::GlobalVar(_)
                    | TokenKind::Lambda
            ) {
                return false;
            }
            let Some(after) = self.skip_target_postfix(offset + 1, true) else {
                return false;
            };
            offset = after;
            // After the target, expect comma or =
            let next = self.peek_ahead(offset);
            if matches!(next.kind, TokenKind::Equal) {
                return true;
            }
            if matches!(next.kind, TokenKind::Comma) {
                seen = true;
                offset += 1;
                continue;
            }
            return false;
        }
    }

    /// The offset past what follows the head of a target: `.name`,
    /// `&.name`, `::Name`, a subscript, and the arguments of a call right
    /// after a name, in any order, as `held.fetch(key)[0].name` writes.
    /// `after_name` says whether the head just read was a name. `None` when
    /// a bracket is never closed or a dot names nothing.
    fn skip_target_postfix(&self, mut offset: usize, mut after_name: bool) -> Option<usize> {
        loop {
            match self.peek_ahead(offset).kind {
                TokenKind::Dot | TokenKind::SafeDot | TokenKind::ColonColon => {
                    offset += 1;
                    if !matches!(self.peek_ahead(offset).kind, TokenKind::Ident(_)) {
                        return None;
                    }
                    offset += 1;
                    after_name = true;
                }
                TokenKind::LBracket => {
                    offset =
                        self.past_balanced(offset, TokenKind::LBracket, TokenKind::RBracket)?;
                    after_name = false;
                }
                TokenKind::LParen if after_name => {
                    offset = self.past_balanced(offset, TokenKind::LParen, TokenKind::RParen)?;
                    after_name = false;
                }
                _ => return Some(offset),
            }
        }
    }

    /// The offset past the bracket that closes the one opening at `offset`.
    fn past_balanced(&self, mut offset: usize, open: TokenKind, close: TokenKind) -> Option<usize> {
        let mut depth = 0;
        loop {
            let kind = &self.peek_ahead(offset).kind;
            if *kind == open {
                depth += 1;
            } else if *kind == close {
                depth -= 1;
                if depth == 0 {
                    return Some(offset + 1);
                }
            } else if matches!(kind, TokenKind::EOF) {
                return None;
            }
            offset += 1;
        }
    }

    /// Whether a statement opening with `(` is a multiple assignment whose
    /// first target is a group, as `(a, b), c = pair, held` is.
    pub(crate) fn scans_grouped_targets(&self) -> bool {
        let mut offset = 0;
        let mut depth = 0;
        let mut commas = 0;
        loop {
            match &self.peek_ahead(offset).kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Comma if depth == 1 => commas += 1,
                // `(*a) = held` names one target and takes the value apart
                // anyway, which is what the splat says.
                TokenKind::Star if depth == 1 => commas += 1,
                TokenKind::EOF | TokenKind::Newline => return false,
                _ => {}
            }
            offset += 1;
        }
        offset += 1;
        while matches!(
            self.peek_ahead(offset).kind,
            TokenKind::Comment(_) | TokenKind::Semicolon
        ) {
            offset += 1;
        }
        match &self.peek_ahead(offset).kind {
            TokenKind::Equal => commas > 0,
            TokenKind::Comma => self.scans_assignment_targets(offset + 1, true),
            _ => false,
        }
    }

    /// One target in a multiple assignment, which may carry a leading `*`
    /// marking it as the one that takes everything the others leave.
    pub(crate) fn parse_assignment_target(&mut self) -> Result<Expression, MetorexError> {
        // `(a, b), c = pair, held` groups targets, and a group may hold
        // further groups, which is what takes a nested Array apart.
        if self.check(&[TokenKind::LParen]) {
            // `(held; object).name = value` writes the receiver in
            // parentheses rather than grouping targets, so a group that does
            // not stand on its own is read again as an expression.
            let saved = self.stream().current_position();
            let opened = self.advance();
            let grouped = (|| -> Result<Vec<Expression>, MetorexError> {
                let mut grouped = Vec::new();
                loop {
                    self.skip_whitespace();
                    if self.check(&[TokenKind::RParen]) {
                        break;
                    }
                    grouped.push(self.parse_assignment_target()?);
                    self.skip_whitespace();
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen, "Expected ')' after grouped targets")?;
                Ok(grouped)
            })();
            let stands_alone = grouped.is_ok()
                && !self.check(&[
                    TokenKind::Dot,
                    TokenKind::SafeDot,
                    TokenKind::LBracket,
                    TokenKind::ColonColon,
                ]);
            if stands_alone {
                return Ok(Expression::Array {
                    elements: grouped.expect("the group was read"),
                    position: opened.position,
                });
            }
            self.stream.restore_position(saved);
        }
        if self.check(&[TokenKind::Star]) {
            let star = self.advance();
            self.skip_whitespace();
            // A splat with no name after it takes the values the other
            // targets leave and keeps none of them.
            if self.check(&[TokenKind::Equal, TokenKind::Comma, TokenKind::RParen]) {
                return Ok(Expression::Splat {
                    expression: Box::new(Expression::Array {
                        elements: Vec::new(),
                        position: star.position,
                    }),
                    position: star.position,
                });
            }
            let expression = self.parse_expression_with_lambda()?;
            return Ok(Expression::Splat {
                expression: Box::new(expression),
                position: star.position,
            });
        }
        self.parse_expression_with_lambda()
    }

    /// The rest of a multiple assignment, given the first target when one has
    /// already been read as an expression.
    pub(crate) fn finish_multiple_assignment(
        &mut self,
        first: Option<Expression>,
        position: crate::lexer::Position,
    ) -> Result<Statement, MetorexError> {
        let first = match first {
            Some(first) => first,
            None => self.parse_assignment_target()?,
        };
        let mut targets = vec![first];
        while self.match_token(&[TokenKind::Comma]) {
            self.skip_whitespace();
            // A trailing comma before the `=` names no further target.
            if self.check(&[TokenKind::Equal]) {
                break;
            }
            targets.push(self.parse_assignment_target()?);
        }
        // `(a, b) = pair` wraps the whole list in a group, which names the
        // same targets as writing them bare does.
        if targets.len() == 1
            && let Expression::Array { elements, .. } = &targets[0]
        {
            targets = elements.clone();
        }
        for target in &targets {
            self.refuse_fixed_target(target)?;
            self.refuse_dynamic_constant_assignment(target)?;
        }
        self.expect(TokenKind::Equal, "Expected '=' in multiple assignment")?;
        self.skip_whitespace();
        let mut values = vec![self.parse_expression_with_lambda()?];
        while self.match_token(&[TokenKind::Comma]) {
            self.skip_whitespace();
            values.push(self.parse_expression_with_lambda()?);
        }
        // `a, b = raise rescue [1, 2]` assigns what the rescue answered, so
        // the modifier belongs to the value rather than to the assignment.
        if values.len() == 1
            && let Some(last) = values.pop()
        {
            values.push(self.wrap_with_rescue_modifier(last)?);
        }
        Ok(Statement::MultipleAssignment {
            targets,
            values,
            position,
        })
    }
}
