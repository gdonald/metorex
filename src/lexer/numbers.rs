// Numeric literal lexing: integers and floats with float-vs-method-call disambiguation.

use super::{Lexer, TokenKind};

impl<'a> Lexer<'a> {
    /// Consume a trailing `r`, which makes the literal a Rational: `5r` is
    /// (5/1) and `1.5r` is (3/2). The `r` only counts when no identifier
    /// character follows it, so `5rescue` still lexes as `5` then `rescue`.
    fn read_rational_suffix(&mut self, number: &str, is_float: bool) -> Option<TokenKind> {
        if self.peek() != Some('r') {
            return None;
        }

        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;

        self.advance(); // consume the 'r'
        if self
            .peek()
            .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
        {
            self.chars = saved_chars;
            self.prepend = saved_prepend;
            self.line = saved_line;
            self.column = saved_column;
            self.offset = saved_offset;
            return None;
        }

        if is_float {
            // `1.25r` is exactly 125/100, not the binary float nearest 1.25.
            let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
            let digits = format!("{}{}", whole, fraction);
            let numerator = digits.parse::<i64>().unwrap_or(0);
            let denominator = 10i64.checked_pow(fraction.len() as u32).unwrap_or(1);
            Some(TokenKind::Rational(numerator, denominator))
        } else {
            Some(TokenKind::Rational(number.parse().unwrap_or(0), 1))
        }
    }

    /// Consume a trailing `i`, which makes the literal imaginary: `1.3i` is
    /// (0+1.3i). As with the rational suffix, the `i` only counts when no
    /// identifier character follows it, so `2if x` still lexes as `2` then
    /// `if`.
    fn read_imaginary_suffix(&mut self, number: &str) -> Option<TokenKind> {
        if self.peek() != Some('i') {
            return None;
        }

        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;

        self.advance(); // consume the 'i'
        if self
            .peek()
            .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
        {
            self.chars = saved_chars;
            self.prepend = saved_prepend;
            self.line = saved_line;
            self.column = saved_column;
            self.offset = saved_offset;
            return None;
        }

        Some(TokenKind::Imaginary(number.parse().unwrap_or(0.0)))
    }

    /// Whether the `_` at the cursor sits between digits, as in `1_000`. A
    /// trailing `_` is left for the next token instead of being swallowed.
    fn underscore_joins_digits(&mut self) -> bool {
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;

        self.advance(); // consume the '_'
        let joins = self.peek().is_some_and(|ch| ch.is_ascii_digit());

        self.chars = saved_chars;
        self.prepend = saved_prepend;
        self.line = saved_line;
        self.column = saved_column;
        self.offset = saved_offset;
        joins
    }

    /// Read `0x1f`, `0b1010`, `0o17`, `0d99`, or the bare-leading-zero octal
    /// `017`. Answers None (leaving the cursor untouched) when what follows
    /// the zero is not a radix literal, so `0`, `0.5`, and `0e3` fall through
    /// to the decimal reader.
    fn read_radix_literal(&mut self) -> Option<TokenKind> {
        if self.peek() != Some('0') {
            return None;
        }

        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;
        let restore = |lexer: &mut Self| {
            lexer.chars = saved_chars.clone();
            lexer.prepend = saved_prepend.clone();
            lexer.line = saved_line;
            lexer.column = saved_column;
            lexer.offset = saved_offset;
        };

        self.advance(); // consume the leading '0'
        let (radix, has_prefix_letter) = match self.peek() {
            Some('x') | Some('X') => (16, true),
            Some('b') | Some('B') => (2, true),
            Some('o') | Some('O') => (8, true),
            Some('d') | Some('D') => (10, true),
            Some(ch) if ch.is_ascii_digit() || ch == '_' => (8, false),
            _ => {
                restore(self);
                return None;
            }
        };
        if has_prefix_letter {
            self.advance();
        }

        let mut digits = String::new();
        while let Some(ch) = self.peek() {
            if ch == '_' {
                self.advance();
                continue;
            }
            if ch.to_digit(radix).is_none() {
                break;
            }
            digits.push(ch);
            self.advance();
        }

        if digits.is_empty() {
            restore(self);
            return None;
        }
        Some(TokenKind::Int(
            i64::from_str_radix(&digits, radix).unwrap_or(0),
        ))
    }

    /// Read a number (integer or float)
    pub(super) fn read_number(&mut self) -> TokenKind {
        if let Some(token) = self.read_radix_literal() {
            return token;
        }

        let mut number = String::new();
        let mut is_float = false;

        // Read digits before decimal point
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '_' && !number.is_empty() && self.underscore_joins_digits() {
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
                            } else if digit_ch == '_' && self.underscore_joins_digits() {
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

        // Scientific notation: `2e3`, `1.5e-3`, `2E+3`. The `e` is only part
        // of the literal when digits follow it, so a trailing identifier such
        // as `2.times` or a bare `2` before a name is left alone.
        if matches!(self.peek(), Some('e') | Some('E')) {
            let saved_chars = self.chars.clone();
            let saved_prepend = self.prepend.clone();
            let saved_line = self.line;
            let saved_column = self.column;
            let saved_offset = self.offset;

            let mut exponent = String::new();
            if let Some(marker) = self.advance() {
                exponent.push(marker);
            }
            if matches!(self.peek(), Some('+') | Some('-'))
                && let Some(sign) = self.advance()
            {
                exponent.push(sign);
            }
            let mut has_digits = false;
            while let Some(digit_ch) = self.peek() {
                if digit_ch.is_ascii_digit() {
                    has_digits = true;
                    exponent.push(digit_ch);
                    self.advance();
                } else {
                    break;
                }
            }

            if has_digits {
                number.push_str(&exponent);
                is_float = true;
            } else {
                self.chars = saved_chars;
                self.prepend = saved_prepend;
                self.line = saved_line;
                self.column = saved_column;
                self.offset = saved_offset;
            }
        }

        if let Some(token) = self.read_rational_suffix(&number, is_float) {
            return token;
        }

        if let Some(token) = self.read_imaginary_suffix(&number) {
            return token;
        }

        if is_float {
            TokenKind::Float(number.parse().unwrap_or(0.0))
        } else {
            TokenKind::Int(number.parse().unwrap_or(0))
        }
    }
}
