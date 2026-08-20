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
mod heredoc;
pub(crate) use heredoc::split_interpolation_parts;
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
    /// Characters injected back into the stream (consumed before `chars`).
    /// Stored in reverse order so `pop` yields the next char.
    pub(super) prepend: Vec<char>,
    /// Current position in the source
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) offset: usize,
    /// Last significant token kind (for regex vs division disambiguation)
    pub(super) prev_significant: Option<TokenKind>,
    /// Line number to restore once `prepend` drains. Heredoc lexing rewinds
    /// `line` to the opener's line while the rest of that line is re-lexed
    /// from `prepend`; this puts the counter back at the line following the
    /// heredoc terminator afterwards.
    pub(super) restore_line: Option<usize>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self::with_start_line(source, 1)
    }

    /// Create a lexer whose first line is numbered `start_line`. Used by
    /// `eval`/`class_eval`/`module_eval` so `__LINE__` reflects the optional
    /// `lineno` argument (e.g. `class_eval("...", "file", 102)`).
    pub fn with_start_line(source: &'a str, start_line: usize) -> Self {
        Self {
            chars: source.chars().peekable(),
            prepend: Vec::new(),
            line: start_line,
            column: 1,
            offset: 0,
            prev_significant: None,
            restore_line: None,
        }
    }

    /// Peek at the next token without consuming it
    pub fn peek_token(&mut self) -> Token {
        // Save current state
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;
        let saved_prev = self.prev_significant.clone();
        let saved_restore_line = self.restore_line;

        // Get the next token
        let token = self.next_token();

        // Restore state
        self.chars = saved_chars;
        self.prepend = saved_prepend;
        self.line = saved_line;
        self.column = saved_column;
        self.offset = saved_offset;
        self.prev_significant = saved_prev;
        self.restore_line = saved_restore_line;

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
        match &token.kind {
            // Newlines reset expression context for slash-vs-regex purposes:
            // a `/` at the start of a fresh line should be parsed as a regex
            // literal, not division applied to whatever ended the prior line.
            TokenKind::Newline => {
                self.prev_significant = None;
            }
            // Comments and EOF leave the previous significant token alone.
            TokenKind::Comment(_) | TokenKind::EOF => {}
            other => {
                self.prev_significant = Some(other.clone());
            }
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
