// Percent-prefixed literal lexing: %r{...}, %w[...], %Q[...], %[...], %{...}, etc.

use super::{Lexer, Token, TokenKind};
use crate::lexer::Position;

impl<'a> Lexer<'a> {
    /// Handle a `%` token: regex literal, %w word array, %Q/%[]/etc. string,
    /// or fall back to the percent operator.
    pub(super) fn lex_percent(&mut self, position: Position) -> Token {
        // A `%` that stands apart from what precedes it opens a literal
        // rather than dividing, which is how Ruby tells `x % y` from `%(y)`.
        let spaced_before = self.prev_significant_end < self.offset;
        self.advance(); // consume %
        if self.peek() == Some('r') {
            return self.lex_percent_r(position);
        }
        if matches!(self.prev_significant, Some(TokenKind::Def)) {
            return Token::new(TokenKind::Percent, position);
        }
        if matches!(self.peek(), Some('w') | Some('i') | Some('W') | Some('I')) {
            return self.lex_percent_list(position);
        }
        if self.peek() == Some('q') {
            self.advance(); // consume q
            return self.lex_percent_raw(position);
        }
        if self.peek() == Some('x') {
            self.advance(); // consume x
            return self.lex_percent_command(position);
        }
        if self.peek() == Some('Q') {
            self.advance(); // consume Q
            return self.lex_percent_string(position);
        }
        // Any other punctuation may delimit a `%` string, so long as the `%`
        // is not dividing what came before it.
        let opener = self.peek();
        let delimits = matches!(opener, Some(ch) if !ch.is_alphanumeric() && !ch.is_whitespace());
        let after_value = follows_a_value(&self.prev_significant);
        // `x %= 1` divides and assigns, so `=` only delimits where no value
        // precedes the `%`.
        if delimits && (!after_value || (spaced_before && opener != Some('='))) {
            return self.lex_percent_string(position);
        }
        Token::new(TokenKind::Percent, position)
    }

    /// Lex `%x(...)`, which runs its text as a command the way a backtick
    /// literal does.
    fn lex_percent_command(&mut self, position: Position) -> Token {
        let parts = self.read_percent_parts();
        Token::new(TokenKind::CommandString(parts), position)
    }

    /// Lex `%r{pattern}flags` regex literal. Called with `%` already consumed.
    fn lex_percent_r(&mut self, position: Position) -> Token {
        self.advance(); // consume 'r'
        let open = self.peek().unwrap_or('(');
        let close = matching_close(open);
        self.advance(); // consume opening delimiter
        let mut pattern = String::new();
        let mut escaped = false;
        while let Some(ch) = self.peek() {
            if escaped {
                pattern.push('\\');
                pattern.push(ch);
                self.advance();
                escaped = false;
            } else if ch == '\\' {
                self.advance();
                escaped = true;
            } else if ch == close {
                self.advance();
                break;
            } else {
                pattern.push(ch);
                self.advance();
            }
        }
        let mut flags = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                flags.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        Token::new(TokenKind::Regex(pattern, flags), position)
    }

    /// Lex a `%w[a b c]` word array or a `%i[a b c]` symbol array. The token
    /// carries the raw whitespace-separated source for the parser to split.
    fn lex_percent_list(&mut self, position: Position) -> Token {
        let marker = self.peek().unwrap_or('w');
        let symbols = marker == 'i' || marker == 'I';
        let interpolates = marker == 'W' || marker == 'I';
        self.advance(); // consume the list marker
        let open = self.peek().unwrap_or('(');
        let close = matching_close(open);
        self.advance(); // consume opening delimiter
        let mut content = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\\' {
                self.advance();
                if let Some(esc) = self.peek() {
                    content.push(esc);
                    self.advance();
                }
            } else if ch == close {
                self.advance();
                break;
            } else {
                content.push(ch);
                self.advance();
            }
        }
        let kind = if symbols {
            TokenKind::PercentI(content, interpolates)
        } else {
            TokenKind::PercentW(content, interpolates)
        };
        Token::new(kind, position)
    }

    /// Lex `%Q[...]`, `%[...]`, `%(...)`, `%{...}`, `%<...>` string literals.
    fn lex_percent_string(&mut self, position: Position) -> Token {
        let parts = self.read_percent_parts();
        if parts.len() == 1
            && let super::InterpolationPart::Text(text) = &parts[0]
        {
            return Token::new(TokenKind::String(text.clone()), position);
        }
        if parts.is_empty() {
            return Token::new(TokenKind::String(String::new()), position);
        }
        Token::new(TokenKind::InterpolatedString(parts), position)
    }

    /// The text and `#{}` parts a percent literal holds, read up to the
    /// delimiter that closes it.
    fn read_percent_parts(&mut self) -> Vec<super::InterpolationPart> {
        let open = self.peek().unwrap_or('(');
        let close = matching_close(open);
        self.advance(); // consume opening delimiter
        let mut parts = Vec::new();
        let mut content = String::new();
        let mut depth = 1;
        while let Some(ch) = self.peek() {
            if ch == '\\' {
                self.advance();
                if let Some(esc) = self.peek() {
                    match esc {
                        'n' => content.push('\n'),
                        't' => content.push('\t'),
                        '\\' => content.push('\\'),
                        other if other == close || other == open => content.push(other),
                        _ => {
                            content.push('\\');
                            content.push(esc);
                        }
                    }
                    self.advance();
                }
            } else if ch == '#' && ch != close {
                self.advance();
                if self.peek() != Some('{') {
                    content.push('#');
                    continue;
                }
                self.advance();
                if !content.is_empty() {
                    parts.push(super::InterpolationPart::Text(std::mem::take(&mut content)));
                }
                parts.push(super::InterpolationPart::Expression(
                    self.read_percent_hole(),
                ));
            } else if ch == open && open != close {
                depth += 1;
                content.push(ch);
                self.advance();
            } else if ch == close {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    break;
                }
                content.push(ch);
                self.advance();
            } else {
                content.push(ch);
                self.advance();
            }
        }
        if !content.is_empty() || parts.is_empty() {
            parts.push(super::InterpolationPart::Text(content));
        }
        parts
    }

    /// The source of one `#{}` hole, read up to the brace that closes it.
    fn read_percent_hole(&mut self) -> String {
        let mut held = String::new();
        let mut depth = 1;
        while let Some(ch) = self.peek() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    break;
                }
            }
            held.push(ch);
            self.advance();
        }
        held
    }
}

/// Whether the token before a `%` produces a value, which makes the `%` a
/// division rather than the start of a literal.
fn follows_a_value(previous: &Option<TokenKind>) -> bool {
    matches!(
        previous,
        Some(
            TokenKind::Ident(_)
                | TokenKind::Int(_)
                | TokenKind::BigInt(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::InterpolatedString(_)
                | TokenKind::InstanceVar(_)
                | TokenKind::ClassVar(_)
                | TokenKind::GlobalVar(_)
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::End
        )
    )
}

/// Map an opening percent-literal delimiter to its closing counterpart.
fn matching_close(open: char) -> char {
    match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        _ => open,
    }
}

impl<'a> Lexer<'a> {
    /// Lex `%q{...}`, which reads no `#{}` part. Only the delimiter and a
    /// backslash may be escaped.
    fn lex_percent_raw(&mut self, position: Position) -> Token {
        let open = self.peek().unwrap_or('(');
        let close = matching_close(open);
        self.advance(); // consume opening delimiter
        let mut content = String::new();
        let mut depth = 1;
        while let Some(ch) = self.peek() {
            if ch == '\\' {
                self.advance();
                match self.peek() {
                    Some(next) if next == close || next == open || next == '\\' => {
                        content.push(next);
                        self.advance();
                    }
                    Some(next) => {
                        content.push('\\');
                        content.push(next);
                        self.advance();
                    }
                    None => break,
                }
            } else if ch == open && open != close {
                depth += 1;
                content.push(ch);
                self.advance();
            } else if ch == close {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    break;
                }
                content.push(ch);
                self.advance();
            } else {
                content.push(ch);
                self.advance();
            }
        }
        Token::new(TokenKind::String(content), position)
    }
}
