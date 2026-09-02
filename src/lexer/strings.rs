// String and comment lexing: quoted strings, escape sequences, interpolation.

use super::{InterpolationPart, Lexer, TokenKind};

impl<'a> Lexer<'a> {
    /// Read a comment from # to end of line
    pub(super) fn read_comment(&mut self) -> String {
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

    /// Read a string literal (single or double quoted)
    pub(super) fn read_string(&mut self, quote: char) -> Result<TokenKind, String> {
        self.read_quoted(quote, false)
    }

    /// Read a backtick command literal, which interpolates the way a
    /// double-quoted string does and carries its parts for the parser to turn
    /// into a call.
    pub(super) fn read_command_string(&mut self) -> Result<TokenKind, String> {
        match self.read_quoted('`', true)? {
            TokenKind::String(text) => Ok(TokenKind::CommandString(vec![InterpolationPart::Text(
                text,
            )])),
            TokenKind::InterpolatedString(parts) => Ok(TokenKind::CommandString(parts)),
            other => Ok(other),
        }
    }

    fn read_quoted(&mut self, quote: char, command: bool) -> Result<TokenKind, String> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        // Only double-quoted strings and backticks support interpolation.
        let has_interpolation = quote == '"' || command;

        // Skip the opening quote
        self.advance();

        loop {
            match self.peek() {
                None => {
                    return Err(format!(
                        "Unterminated string starting at line {}",
                        self.line
                    ));
                }
                Some('\n') => {
                    return Err(format!(
                        "Unterminated string starting at line {}",
                        self.line
                    ));
                }
                Some(ch) if ch == quote => {
                    // Found closing quote
                    self.advance();

                    // If we have interpolation parts, return an interpolated string
                    if has_interpolation && !parts.is_empty() {
                        if !current_text.is_empty() {
                            parts.push(InterpolationPart::Text(current_text));
                        }
                        return Ok(TokenKind::InterpolatedString(parts));
                    } else {
                        return Ok(TokenKind::String(current_text));
                    }
                }
                Some('\\') => {
                    // Handle escape sequences
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            current_text.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            current_text.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            current_text.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            current_text.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            current_text.push('"');
                            self.advance();
                        }
                        Some('\'') => {
                            current_text.push('\'');
                            self.advance();
                        }
                        Some('#') => {
                            // Escaped hash - allows literal #{
                            current_text.push('#');
                            self.advance();
                        }
                        Some('e') => {
                            // ESC character (0x1B) for ANSI escape sequences
                            current_text.push('\x1B');
                            self.advance();
                        }
                        Some('0') => {
                            // Null character
                            current_text.push('\0');
                            self.advance();
                        }
                        Some(ch) => {
                            // For unrecognized escape sequences, include the backslash
                            current_text.push('\\');
                            current_text.push(ch);
                            self.advance();
                        }
                        None => {
                            return Err(format!(
                                "Unterminated string starting at line {}",
                                self.line
                            ));
                        }
                    }
                }
                Some('#') if has_interpolation => {
                    // Check if this is the start of interpolation (#{)
                    self.advance();
                    if self.peek() == Some('{') {
                        // Start of interpolation
                        self.advance();

                        // Save current text as a part
                        if !current_text.is_empty() {
                            parts.push(InterpolationPart::Text(current_text.clone()));
                            current_text.clear();
                        }

                        // Read the expression until we find }
                        let mut expr = String::new();
                        let mut depth = 1; // Track nested braces

                        loop {
                            match self.peek() {
                                None => {
                                    return Err(format!(
                                        "Unterminated interpolation starting at line {}",
                                        self.line
                                    ));
                                }
                                Some('\n') => {
                                    return Err(format!(
                                        "Unterminated interpolation starting at line {}",
                                        self.line
                                    ));
                                }
                                Some('{') => {
                                    depth += 1;
                                    expr.push('{');
                                    self.advance();
                                }
                                Some('}') => {
                                    depth -= 1;
                                    if depth == 0 {
                                        self.advance();
                                        parts.push(InterpolationPart::Expression(expr));
                                        break;
                                    } else {
                                        expr.push('}');
                                        self.advance();
                                    }
                                }
                                Some(ch) => {
                                    expr.push(ch);
                                    self.advance();
                                }
                            }
                        }
                    } else {
                        // Not interpolation, just a # character
                        current_text.push('#');
                    }
                }
                Some(ch) => {
                    current_text.push(ch);
                    self.advance();
                }
            }
        }
    }
}
