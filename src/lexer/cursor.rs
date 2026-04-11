// Cursor primitives for the lexer: advance, peek, position, whitespace skipping.

use super::{Lexer, Position};

impl<'a> Lexer<'a> {
    /// Get the current position
    pub(super) fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.offset)
    }

    /// Advance to the next character and return it
    pub(super) fn advance(&mut self) -> Option<char> {
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
    pub(super) fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Skip whitespace characters (spaces and tabs, but not newlines)
    pub(super) fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
}
