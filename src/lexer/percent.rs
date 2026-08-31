// Percent-prefixed literal lexing: %r{...}, %w[...], %Q[...], %[...], %{...}, etc.

use super::{Lexer, Token, TokenKind};
use crate::lexer::Position;

impl<'a> Lexer<'a> {
    /// Handle a `%` token: regex literal, %w word array, %Q/%[]/etc. string,
    /// or fall back to the percent operator.
    pub(super) fn lex_percent(&mut self, position: Position) -> Token {
        self.advance(); // consume %
        if self.peek() == Some('r') {
            return self.lex_percent_r(position);
        }
        if !matches!(self.prev_significant, Some(TokenKind::Def))
            && matches!(self.peek(), Some('w') | Some('i'))
        {
            return self.lex_percent_list(position);
        }
        if !matches!(self.prev_significant, Some(TokenKind::Def))
            && (self.peek() == Some('Q')
                || self.peek() == Some('[')
                || self.peek() == Some('(')
                || self.peek() == Some('{')
                || self.peek() == Some('<'))
        {
            return self.lex_percent_string(position);
        }
        Token::new(TokenKind::Percent, position)
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
        let symbols = self.peek() == Some('i');
        self.advance(); // consume 'w' or 'i'
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
            TokenKind::PercentI(content)
        } else {
            TokenKind::PercentW(content)
        };
        Token::new(kind, position)
    }

    /// Lex `%Q[...]`, `%[...]`, `%(...)`, `%{...}`, `%<...>` string literals.
    fn lex_percent_string(&mut self, position: Position) -> Token {
        if matches!(self.peek(), Some('Q')) {
            self.advance(); // consume Q
        }
        let open = self.peek().unwrap_or('(');
        let close = matching_close(open);
        self.advance(); // consume opening delimiter
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
                        _ => {
                            content.push('\\');
                            content.push(esc);
                        }
                    }
                    self.advance();
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
