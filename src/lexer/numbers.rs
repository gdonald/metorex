// Numeric literal lexing: integers and floats with float-vs-method-call disambiguation.

use super::{Lexer, TokenKind};

impl<'a> Lexer<'a> {
    /// Read a number (integer or float)
    pub(super) fn read_number(&mut self) -> TokenKind {
        let mut number = String::new();
        let mut is_float = false;

        // Read digits before decimal point
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' {
                // Need to peek ahead to see if this is a float or a range/method call
                // Save current state to peek ahead
                let saved_chars = self.chars.clone();
                let saved_prepend = self.prepend.clone();
                let saved_line = self.line;
                let saved_column = self.column;
                let saved_offset = self.offset;

                self.advance(); // consume the dot
                let next_ch = self.peek();

                // Restore state
                self.chars = saved_chars;
                self.prepend = saved_prepend;
                self.line = saved_line;
                self.column = saved_column;
                self.offset = saved_offset;

                // Check if next character is a digit (float) or not (method/range)
                if let Some(next_ch) = next_ch {
                    if next_ch.is_ascii_digit() {
                        // It's a float literal
                        self.advance(); // consume the dot for real
                        is_float = true;
                        number.push('.');
                        // Read digits after decimal point
                        while let Some(digit_ch) = self.peek() {
                            if digit_ch.is_ascii_digit() {
                                number.push(digit_ch);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        break;
                    } else {
                        // Not a float - dot will be lexed separately
                        break;
                    }
                }
                break;
            } else {
                break;
            }
        }

        if is_float {
            TokenKind::Float(number.parse().unwrap_or(0.0))
        } else {
            TokenKind::Int(number.parse().unwrap_or(0))
        }
    }
}
