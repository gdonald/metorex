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

    /// Skip whitespace characters (spaces and tabs, but not newlines).
    /// Also handles line continuations: a `\` immediately followed by a
    /// newline is consumed silently so the two source lines join.
    pub(super) fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('\\') => {
                    // Look one ahead for a newline. If present, eat both
                    // (line continuation). Otherwise leave the backslash
                    // alone for the regular dispatch.
                    let saved_chars = self.chars.clone();
                    let saved_line = self.line;
                    let saved_column = self.column;
                    let saved_offset = self.offset;
                    self.advance(); // consume backslash
                    if self.peek() == Some('\n') {
                        self.advance(); // consume newline
                    } else {
                        // Roll back: not a line continuation.
                        self.chars = saved_chars;
                        self.line = saved_line;
                        self.column = saved_column;
                        self.offset = saved_offset;
                        break;
                    }
                }
                _ => break,
            }
        }
    }
}
