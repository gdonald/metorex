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
            // A command whose escapes name bytes runs those bytes, so the
            // text stands for them rather than for characters.
            TokenKind::ByteString(text) => Ok(TokenKind::ByteCommandString(text)),
            other => Ok(other),
        }
    }

    fn read_quoted(&mut self, quote: char, command: bool) -> Result<TokenKind, String> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        // A run of numeric escapes that spells no text leaves the literal
        // standing for bytes rather than for characters.
        let mut holds_bytes = false;
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
                    } else if self.frozen_literals && !command {
                        // The source asked for its literals to be frozen, and
                        // one written in escapes still stands for its bytes.
                        if holds_bytes {
                            return Ok(TokenKind::ByteString(current_text));
                        }
                        return Ok(TokenKind::FrozenString(current_text));
                    } else if self.mutable_literals && !command && !holds_bytes {
                        // The source asked outright for literals that change,
                        // so one carries no notice that it will be frozen.
                        return Ok(TokenKind::MutableString(current_text));
                    } else if holds_bytes {
                        // A source written in bytes spells its literals in
                        // bytes; anywhere else the characters stand for the
                        // bytes the escapes named, in the source's encoding.
                        if self.binary_source {
                            return Ok(TokenKind::BinaryString(current_text));
                        }
                        return Ok(TokenKind::ByteString(current_text));
                    } else {
                        return Ok(TokenKind::String(current_text));
                    }
                }
                Some('\\') => {
                    // Handle escape sequences
                    self.advance();
                    // A single-quoted string reads only `\\` and `\'` as
                    // escapes, so every other backslash stands for itself.
                    if quote == '\'' {
                        match self.peek() {
                            Some(next @ ('\\' | '\'')) => {
                                current_text.push(next);
                                self.advance();
                            }
                            _ => current_text.push('\\'),
                        }
                        continue;
                    }
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
                                    holds_bytes = true;
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
                                    holds_bytes = true;
                                    for byte in bytes {
                                        current_text.push(byte as char);
                                    }
                                }
                            }
                        }
                        // `\cX`, `\C-X`, and `\M-X` name a control or meta
                        // character by the one that follows, and `\M-\C-X`
                        // names both at once.
                        Some('c') | Some('C') | Some('M') => match self.read_control_escape() {
                            Some(byte) if byte.is_ascii() => {
                                current_text.push(byte as char);
                            }
                            Some(byte) => {
                                holds_bytes = true;
                                current_text.push(byte as char);
                            }
                            None => {
                                if quote == '\'' {
                                    current_text.push('\\');
                                }
                                if let Some(letter) = self.peek() {
                                    current_text.push(letter);
                                    self.advance();
                                }
                            }
                        },
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
                        let hole_line = self.line;
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
                                        parts.push(InterpolationPart::Expression(expr, hole_line));
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
                    } else if let Some(named) = self.read_short_interpolation() {
                        // `#@name`, `#@@name`, and `#$name` interpolate that
                        // variable without braces around it.
                        if !current_text.is_empty() {
                            parts.push(InterpolationPart::Text(current_text.clone()));
                            current_text.clear();
                        }
                        parts.push(InterpolationPart::Expression(named, self.line));
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

impl<'a> Lexer<'a> {
    /// The variable a `#` interpolates without braces: `#@name`, `#@@name`,
    /// and `#$name`. The cursor sits just past the `#`. Nothing is consumed
    /// where what follows names no variable, so `"#@ "` stands as it reads.
    pub(super) fn read_short_interpolation(&mut self) -> Option<String> {
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;
        let mut named = String::new();
        match self.peek() {
            Some('@') => {
                named.push('@');
                self.advance();
                if self.peek() == Some('@') {
                    named.push('@');
                    self.advance();
                }
            }
            Some('$') => {
                named.push('$');
                self.advance();
            }
            _ => return None,
        }
        // A name reads the way Ruby lets one be written, so `#@ip[` ends at
        // the bracket and `#@ ` names nothing at all.
        let starts_a_name = |letter: char| letter.is_alphabetic() || letter == '_';
        if !self.peek().is_some_and(starts_a_name) {
            self.chars = saved_chars;
            self.prepend = saved_prepend;
            self.line = saved_line;
            self.column = saved_column;
            self.offset = saved_offset;
            return None;
        }
        while let Some(letter) = self.peek() {
            if letter.is_alphanumeric() || letter == '_' {
                named.push(letter);
                self.advance();
            } else {
                break;
            }
        }
        Some(named)
    }
}

impl<'a> Lexer<'a> {
    /// The byte a control or meta escape names. The cursor sits on the letter
    /// that opened it: `c` for `\cX`, `C` for `\C-X`, or `M` for `\M-X`.
    /// Nothing is consumed where what follows spells no such escape.
    pub(super) fn read_control_escape(&mut self) -> Option<u8> {
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;
        let rewind = |held: &mut Self| {
            held.chars = saved_chars.clone();
            held.prepend = saved_prepend.clone();
            held.line = saved_line;
            held.column = saved_column;
            held.offset = saved_offset;
        };
        let opener = self.peek()?;
        self.advance();
        // `\C-` and `\M-` are written with the hyphen, while `\c` is not.
        if opener != 'c' {
            if self.peek() != Some('-') {
                rewind(self);
                return None;
            }
            self.advance();
        }
        // The escapes stack, so `\M-\C-z` is meta over control.
        let held = match self.peek() {
            Some('\\') => {
                self.advance();
                match self.peek() {
                    Some('c') | Some('C') | Some('M') => self.read_control_escape()?,
                    Some(letter) if letter.is_ascii() => {
                        self.advance();
                        letter as u8
                    }
                    _ => {
                        rewind(self);
                        return None;
                    }
                }
            }
            Some(letter) if letter.is_ascii() => {
                self.advance();
                letter as u8
            }
            _ => {
                rewind(self);
                return None;
            }
        };
        Some(match opener {
            // Meta sets the high bit, and control keeps the low six.
            'M' => held | 0x80,
            _ => held & 0x9f,
        })
    }
}
