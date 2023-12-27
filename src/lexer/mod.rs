// Lexer module for tokenizing Metorex source code

pub mod token;

pub use token::{Position, Token, TokenKind};

use std::iter::Peekable;
use std::str::Chars;

/// The lexer converts source code into a stream of tokens
pub struct Lexer<'a> {
    /// Peekable iterator over the characters
    chars: Peekable<Chars<'a>>,
    /// Current position in the source
    line: usize,
    column: usize,
    offset: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Get the current position
    fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.offset)
    }

    /// Advance to the next character and return it
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.next() {
            self.offset += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    /// Peek at the next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Skip whitespace characters (spaces and tabs, but not newlines)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read a comment from # to end of line
    fn read_comment(&mut self) -> String {
        let mut comment = String::new();
        // Skip the # character
        self.advance();

        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        comment.trim().to_string()
    }

    /// Get the next token from the source code
    pub fn next_token(&mut self) -> Token {
        // Skip whitespace (but not newlines)
        self.skip_whitespace();

        let position = self.current_position();

        // Check for end of input
        if let Some(ch) = self.peek() {
            match ch {
                '\n' => {
                    self.advance();
                    Token::new(TokenKind::Newline, position)
                }
                '#' => {
                    let comment = self.read_comment();
                    Token::new(TokenKind::Comment(comment), position)
                }
                _ => {
                    // For now, just consume the character and return EOF
                    // This skeleton will be expanded in later roadmap items
                    self.advance();
                    Token::new(TokenKind::EOF, position)
                }
            }
        } else {
            Token::new(TokenKind::EOF, position)
        }
    }
}
