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
                // A quoted string may run across lines, so a raw newline is
                // content rather than the end of the literal.
                Some('\n') => {
                    current_text.push('\n');
                    self.advance();
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
                        // An octal escape runs to three digits, so `\000` is
                        // one NUL rather than a NUL followed by two zeros. A
                        // run of them names bytes, which together may spell
                        // one character.
                        Some(digit) if ('0'..='7').contains(&digit) => {
                            let mut bytes = Vec::new();
                            loop {
                                let mut digits = String::new();
                                while digits.len() < 3
                                    && self.peek().is_some_and(|next| ('0'..='7').contains(&next))
                                {
                                    digits.push(self.peek().expect("a digit was seen"));
                                    self.advance();
                                }
                                bytes.push(u32::from_str_radix(&digits, 8).unwrap_or(0) as u8);
                                if self.peek() != Some('\\') {
                                    break;
                                }
                                self.advance();
                                if !self.peek().is_some_and(|next| ('0'..='7').contains(&next)) {
                                    // The backslash opened some other escape,
                                    // so it is handed back to the main loop.
                                    self.push_back('\\');
                                    break;
                                }
                            }
                            match binary_run(self.binary_source, &bytes) {
                                Ok(text) => current_text.push_str(&text),
                                Err(_) => {
                                    for byte in bytes {
                                        current_text.push(byte as char);
                                    }
                                }
                            }
                        }
                        Some('s') => {
                            current_text.push(' ');
                            self.advance();
                        }
                        Some('a') => {
                            current_text.push('\u{7}');
                            self.advance();
                        }
                        Some('b') => {
                            current_text.push('\u{8}');
                            self.advance();
                        }
                        Some('f') => {
                            current_text.push('\u{c}');
                            self.advance();
                        }
                        Some('v') => {
                            current_text.push('\u{b}');
                            self.advance();
                        }
                        // `\xNN` names a byte and `\uXXXX` or `\u{...}` a
                        // code point, which is how a spec writes a character
                        // it cannot type.
                        Some('x') => {
                            // A run of `\xNN` escapes names bytes, which
                            // together may spell one character.
                            let mut bytes = Vec::new();
                            loop {
                                self.advance();
                                let mut digits = String::new();
                                while digits.len() < 2
                                    && self.peek().is_some_and(|digit| digit.is_ascii_hexdigit())
                                {
                                    digits.push(self.peek().expect("a digit was seen"));
                                    self.advance();
                                }
                                match u8::from_str_radix(&digits, 16) {
                                    Ok(byte) => bytes.push(byte),
                                    Err(_) => {
                                        current_text.push_str("\\x");
                                        current_text.push_str(&digits);
                                    }
                                }
                                if self.peek() != Some('\\') {
                                    break;
                                }
                                self.advance();
                                if self.peek() != Some('x') {
                                    // The backslash opened some other escape,
                                    // so it is handed back to the main loop.
                                    self.push_back('\\');
                                    break;
                                }
                            }
                            match binary_run(self.binary_source, &bytes) {
                                Ok(text) => current_text.push_str(&text),
                                Err(_) => {
                                    for byte in bytes {
                                        current_text.push(byte as char);
                                    }
                                }
                            }
                        }
                        Some('u') => {
                            self.advance();
                            let mut points = Vec::new();
                            if self.peek() == Some('{') {
                                self.advance();
                                let mut digits = String::new();
                                while let Some(letter) = self.peek() {
                                    if letter == '}' {
                                        self.advance();
                                        break;
                                    }
                                    if letter == ' ' {
                                        points.push(digits.clone());
                                        digits.clear();
                                    } else {
                                        digits.push(letter);
                                    }
                                    self.advance();
                                }
                                points.push(digits);
                            } else {
                                let mut digits = String::new();
                                while digits.len() < 4
                                    && self.peek().is_some_and(|digit| digit.is_ascii_hexdigit())
                                {
                                    digits.push(self.peek().expect("a digit was seen"));
                                    self.advance();
                                }
                                points.push(digits);
                            }
                            for point in points {
                                match u32::from_str_radix(&point, 16)
                                    .ok()
                                    .and_then(char::from_u32)
                                {
                                    Some(letter) => current_text.push(letter),
                                    None => {
                                        current_text.push('\\');
                                        current_text.push('u');
                                        current_text.push_str(&point);
                                    }
                                }
                            }
                        }
                        Some(ch) => {
                            // A backslash before something that opens no escape
                            // stands for the character alone, which is how
                            // `"a\{b"` names three characters. A single-quoted
                            // string keeps the backslash, since only `\'` and
                            // `\\` mean anything inside one.
                            if quote == '\'' {
                                current_text.push('\\');
                            }
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

/// The text a run of numeric escapes spells. In a source written in bytes
/// each byte stands alone, and elsewhere bytes that spell a character in
/// UTF-8 read back as that character.
pub(super) fn binary_run(binary_source: bool, bytes: &[u8]) -> Result<String, ()> {
    if binary_source {
        return Err(());
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| ())
}
