// Lexer module for tokenizing Metorex source code.
//
// Implementation is split across files by responsibility:
//   - cursor:      character cursor primitives (advance, peek, position)
//   - identifiers: identifiers, keywords, instance/class/global vars
//   - numbers:     integer and float literals
//   - strings:     quoted strings, escape sequences, interpolation, comments
//   - regex_lit:   `/.../` regex literals and slash-vs-regex disambiguation
//   - percent:     `%`-prefixed literals (`%r`, `%w`, `%Q`, `%[...]`, etc.)
//   - dispatch:    the main `next_token_inner` switch over leading characters

pub mod token;

mod cursor;
mod dispatch;
mod identifiers;
mod numbers;
mod percent;
mod regex_lit;
mod strings;

pub use token::{InterpolationPart, Position, Token, TokenKind};

use std::iter::Peekable;
use std::str::Chars;

/// The lexer converts source code into a stream of tokens
pub struct Lexer<'a> {
    /// Peekable iterator over the characters
    pub(super) chars: Peekable<Chars<'a>>,
    /// Current position in the source
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) offset: usize,
    /// Last significant token kind (for regex vs division disambiguation)
    pub(super) prev_significant: Option<TokenKind>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
            offset: 0,
            prev_significant: None,
        }
    }

    /// Peek at the next token without consuming it
    pub fn peek_token(&mut self) -> Token {
        // Save current state
        let saved_chars = self.chars.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;
        let saved_prev = self.prev_significant.clone();

        // Get the next token
        let token = self.next_token();

        // Restore state
        self.chars = saved_chars;
        self.line = saved_line;
        self.column = saved_column;
        self.offset = saved_offset;
        self.prev_significant = saved_prev;

        token
    }

    /// Collect all tokens from the lexer
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token.kind == TokenKind::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    /// Get the next token from the source code, updating regex disambiguation state.
    pub fn next_token(&mut self) -> Token {
        let token = self.next_token_inner();
        if !matches!(
            token.kind,
            TokenKind::Newline | TokenKind::Comment(_) | TokenKind::EOF
        ) {
            self.prev_significant = Some(token.kind.clone());
        }
        token
    }
}

/// Iterator implementation for Lexer
/// This allows using the lexer in for loops and with iterator methods
impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::EOF {
            None
        } else {
            Some(token)
        }
    }
}
