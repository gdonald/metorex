// Token dispatch: the main `next_token_inner` switch and operator/punctuation lexing.

use super::{Lexer, Token, TokenKind};

impl<'a> Lexer<'a> {
    /// Internal: produce the next token without updating disambiguation state.
    pub(super) fn next_token_inner(&mut self) -> Token {
        // Skip whitespace (but not newlines). Remember whether any space
        // was actually skipped so the parser can disambiguate `m[...]`
        // (indexing) from `m [...]` (paren-less array argument).
        let had_leading_space = self.skip_whitespace();
        let mut token = self.next_token_body();
        token.had_leading_space = had_leading_space;
        token
    }

    /// The actual per-character dispatch, returning a token without worrying
    /// about leading-space bookkeeping (handled by the caller).
    fn next_token_body(&mut self) -> Token {
        let position = self.current_position();

        let Some(ch) = self.peek() else {
            return Token::new(TokenKind::EOF, position);
        };

        match ch {
            '\n' => {
                self.advance();
                Token::new(TokenKind::Newline, position)
            }
            '#' => {
                let comment = self.read_comment();
                Token::new(TokenKind::Comment(comment), position)
            }
            '0'..='9' => {
                let kind = self.read_number();
                Token::new(kind, position)
            }
            '"' | '\'' => match self.read_string(ch) {
                Ok(kind) => Token::new(kind, position),
                Err(_err) => {
                    // For now, return EOF on error
                    // TODO: Proper error handling will be added later
                    Token::new(TokenKind::EOF, position)
                }
            },
            '@' => {
                let kind = self.read_variable();
                Token::new(kind, position)
            }
            '$' => {
                let kind = self.read_global_variable();
                Token::new(kind, position)
            }
            ch if Self::is_identifier_start(ch) => {
                let kind = self.read_identifier();
                Token::new(kind, position)
            }
            '+' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::PlusEqual, position)
                } else {
                    Token::new(TokenKind::Plus, position)
                }
            }
            '-' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::MinusEqual, position)
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token::new(TokenKind::Arrow, position)
                } else {
                    Token::new(TokenKind::Minus, position)
                }
            }
            '*' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::StarEqual, position)
                } else if self.peek() == Some('*') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::StarStarEqual, position)
                    } else {
                        Token::new(TokenKind::StarStar, position)
                    }
                } else {
                    Token::new(TokenKind::Star, position)
                }
            }
            '/' => {
                if self.slash_is_regex() {
                    self.advance(); // consume opening /
                    let kind = self.read_regex();
                    Token::new(kind, position)
                } else {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::SlashEqual, position)
                    } else {
                        Token::new(TokenKind::Slash, position)
                    }
                }
            }
            '%' => self.lex_percent(position),
            '^' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::CaretEqual, position)
                } else {
                    Token::new(TokenKind::Caret, position)
                }
            }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::TripleEqual, position)
                    } else {
                        Token::new(TokenKind::EqualEqual, position)
                    }
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token::new(TokenKind::FatArrow, position)
                } else if self.peek() == Some('~') {
                    self.advance();
                    Token::new(TokenKind::Match, position)
                } else {
                    Token::new(TokenKind::Equal, position)
                }
            }
            '!' => {
                self.advance();
                if self.peek() == Some('~') {
                    self.advance();
                    Token::new(TokenKind::NotMatch, position)
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::BangEqual, position)
                } else {
                    Token::new(TokenKind::Bang, position)
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        Token::new(TokenKind::Spaceship, position)
                    } else {
                        Token::new(TokenKind::LessEqual, position)
                    }
                } else if self.peek() == Some('<') {
                    self.advance(); // consume second '<'
                    // Heredoc: `<<-IDENT` or `<<~IDENT` (with indent-strip)
                    if let Some(heredoc) = self.try_read_heredoc() {
                        Token::new(heredoc, position)
                    } else if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::ShovelEqual, position)
                    } else {
                        Token::new(TokenKind::Shovel, position)
                    }
                } else {
                    Token::new(TokenKind::Less, position)
                }
            }
            '~' => {
                self.advance();
                Token::new(TokenKind::Tilde, position)
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::GreaterEqual, position)
                } else if self.peek() == Some('>') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::RightShiftEqual, position)
                    } else {
                        Token::new(TokenKind::RightShift, position)
                    }
                } else {
                    Token::new(TokenKind::Greater, position)
                }
            }
            '(' => {
                self.advance();
                Token::new(TokenKind::LParen, position)
            }
            ')' => {
                self.advance();
                Token::new(TokenKind::RParen, position)
            }
            '{' => {
                self.advance();
                Token::new(TokenKind::LBrace, position)
            }
            '}' => {
                self.advance();
                Token::new(TokenKind::RBrace, position)
            }
            '[' => {
                self.advance();
                Token::new(TokenKind::LBracket, position)
            }
            ']' => {
                self.advance();
                Token::new(TokenKind::RBracket, position)
            }
            ',' => {
                self.advance();
                Token::new(TokenKind::Comma, position)
            }
            '.' => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        Token::new(TokenKind::DotDotDot, position)
                    } else {
                        Token::new(TokenKind::DotDot, position)
                    }
                } else {
                    Token::new(TokenKind::Dot, position)
                }
            }
            ':' => {
                self.advance();
                if self.peek() == Some(':') {
                    self.advance();
                    Token::new(TokenKind::ColonColon, position)
                } else if self.peek() == Some('`') {
                    // A backtick after a colon names the command method
                    // rather than opening a command literal.
                    self.advance();
                    Token::new(TokenKind::CommandSymbol, position)
                } else {
                    Token::new(TokenKind::Colon, position)
                }
            }
            ';' => {
                self.advance();
                Token::new(TokenKind::Semicolon, position)
            }
            '|' => {
                self.advance();
                if self.peek() == Some('|') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::LogicalOrAssign, position)
                    } else {
                        Token::new(TokenKind::LogicalOr, position)
                    }
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::PipeEqual, position)
                } else {
                    Token::new(TokenKind::Pipe, position)
                }
            }
            '&' => {
                self.advance();
                if self.peek() == Some('&') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::LogicalAndAssign, position)
                    } else {
                        Token::new(TokenKind::LogicalAnd, position)
                    }
                } else if self.peek() == Some('.') {
                    self.advance();
                    Token::new(TokenKind::SafeDot, position)
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::AmpersandEqual, position)
                } else {
                    Token::new(TokenKind::Ampersand, position)
                }
            }
            '?' => {
                self.advance(); // consume ?
                match self.peek() {
                    // `?\n`, `?\001`, and `?\x41` name a character by an
                    // escape, the same escapes a string literal reads.
                    Some('\\') => {
                        self.advance();
                        let character = self.read_character_escape();
                        Token::new(TokenKind::String(character), position)
                    }
                    // ?x where x is not a space/newline: character literal
                    Some(ch) if !ch.is_whitespace() => {
                        self.advance();
                        Token::new(TokenKind::String(ch.to_string()), position)
                    }
                    // ? followed by space/newline/EOF: ternary operator
                    _ => Token::new(TokenKind::Question, position),
                }
            }
            // A backtick right after `.` or `def` names the command method
            // rather than opening a command literal.
            '`' if matches!(
                self.prev_significant,
                Some(TokenKind::Dot) | Some(TokenKind::Def)
            ) =>
            {
                self.advance();
                Token::new(TokenKind::Ident("`".to_string()), position)
            }
            '`' => match self.read_command_string() {
                Ok(kind) => Token::new(kind, position),
                Err(_) => Token::new(TokenKind::EOF, position),
            },
            _ => {
                // Unknown character, consume and return EOF
                self.advance();
                Token::new(TokenKind::EOF, position)
            }
        }
    }
}

impl Lexer<'_> {
    /// The character an escape after `?` names. Reads the same escapes a
    /// string literal does, so `?\n` is a newline, `?\001` is the byte one,
    /// and `?\x41` is a capital A.
    fn read_character_escape(&mut self) -> String {
        let Some(marker) = self.peek() else {
            return "\\".to_string();
        };
        self.advance();
        match marker {
            'n' => "\n".to_string(),
            't' => "\t".to_string(),
            'r' => "\r".to_string(),
            's' => " ".to_string(),
            '0'..='7' => {
                let mut digits = String::from(marker);
                while digits.len() < 3 && matches!(self.peek(), Some('0'..='7')) {
                    digits.push(self.peek().expect("the guard read a digit"));
                    self.advance();
                }
                let value = u32::from_str_radix(&digits, 8).unwrap_or(0);
                char::from_u32(value).unwrap_or('\0').to_string()
            }
            'x' => {
                let mut digits = String::new();
                while digits.len() < 2 && matches!(self.peek(), Some(c) if c.is_ascii_hexdigit()) {
                    digits.push(self.peek().expect("the guard read a digit"));
                    self.advance();
                }
                let value = u32::from_str_radix(&digits, 16).unwrap_or(0);
                char::from_u32(value).unwrap_or('\0').to_string()
            }
            'e' => "\u{1b}".to_string(),
            'a' => "\u{7}".to_string(),
            'b' => "\u{8}".to_string(),
            'f' => "\u{c}".to_string(),
            'v' => "\u{b}".to_string(),
            other => other.to_string(),
        }
    }
}
