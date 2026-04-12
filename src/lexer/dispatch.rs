// Token dispatch: the main `next_token_inner` switch and operator/punctuation lexing.

use super::{Lexer, Token, TokenKind};

impl<'a> Lexer<'a> {
    /// Internal: produce the next token without updating disambiguation state.
    pub(super) fn next_token_inner(&mut self) -> Token {
        // Skip whitespace (but not newlines)
        self.skip_whitespace();

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
                    Token::new(TokenKind::StarStar, position)
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
                Token::new(TokenKind::Caret, position)
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
                    } else {
                        Token::new(TokenKind::Shovel, position)
                    }
                } else {
                    Token::new(TokenKind::Less, position)
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::GreaterEqual, position)
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
                } else {
                    Token::new(TokenKind::Ampersand, position)
                }
            }
            '?' => {
                self.advance(); // consume ?
                match self.peek() {
                    // ?x where x is not a space/newline: character literal
                    Some(ch) if !ch.is_whitespace() => {
                        self.advance();
                        Token::new(TokenKind::String(ch.to_string()), position)
                    }
                    // ? followed by space/newline/EOF: ternary operator
                    _ => Token::new(TokenKind::Question, position),
                }
            }
            '`' => {
                self.advance(); // consume opening `
                let mut content = String::new();
                while let Some(ch) = self.peek() {
                    if ch == '`' {
                        self.advance();
                        break;
                    }
                    content.push(ch);
                    self.advance();
                }
                Token::new(TokenKind::String(content), position)
            }
            _ => {
                // Unknown character, consume and return EOF
                self.advance();
                Token::new(TokenKind::EOF, position)
            }
        }
    }
}
