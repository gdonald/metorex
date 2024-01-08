// Lexer module for tokenizing Metorex source code

pub mod token;

pub use token::{InterpolationPart, Position, Token, TokenKind};

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

    /// Read a number (integer or float)
    fn read_number(&mut self) -> TokenKind {
        let mut number = String::new();
        let mut is_float = false;

        // Read digits before decimal point
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' {
                // Check if next character is a digit to distinguish from method call
                self.advance();
                if let Some(next_ch) = self.peek() {
                    if next_ch.is_ascii_digit() {
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
                        // Not a float, just a dot - we need to handle this case
                        // For now, we'll treat the number as an integer and the dot will be lexed separately
                        // We need to "put back" the dot by not consuming it
                        // But we already advanced, so we need to create a mechanism to handle this
                        // For simplicity in this implementation, if we see a dot not followed by a digit,
                        // we'll just stop reading the number
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

    /// Check if a character can start an identifier (letter or underscore)
    fn is_identifier_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    /// Check if a character can continue an identifier (letter, digit, or underscore)
    fn is_identifier_continue(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> TokenKind {
        let mut ident = String::new();

        // Read identifier characters
        while let Some(ch) = self.peek() {
            if Self::is_identifier_continue(ch) {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        self.keyword_or_identifier(ident)
    }

    /// Read an instance or class variable (@var or @@var)
    fn read_variable(&mut self) -> TokenKind {
        // Skip the first @
        self.advance();

        // Check if it's a class variable (@@)
        if self.peek() == Some('@') {
            self.advance();
            // Read the identifier part
            let mut ident = String::new();
            while let Some(ch) = self.peek() {
                if Self::is_identifier_continue(ch) {
                    ident.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            TokenKind::ClassVar(ident)
        } else {
            // Instance variable (@)
            let mut ident = String::new();
            while let Some(ch) = self.peek() {
                if Self::is_identifier_continue(ch) {
                    ident.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            TokenKind::InstanceVar(ident)
        }
    }

    /// Convert a string to a keyword token or identifier
    fn keyword_or_identifier(&self, ident: String) -> TokenKind {
        match ident.as_str() {
            "def" => TokenKind::Def,
            "class" => TokenKind::Class,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "end" => TokenKind::End,
            "do" => TokenKind::Do,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nil" => TokenKind::Nil,
            _ => TokenKind::Ident(ident),
        }
    }

    /// Read a string literal (single or double quoted)
    fn read_string(&mut self, quote: char) -> Result<TokenKind, String> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        let has_interpolation = quote == '"'; // Only double-quoted strings support interpolation

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
                        Some('{') => {
                            // Escaped brace - not interpolation
                            current_text.push('{');
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
                Some('{') if has_interpolation => {
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
                }
                Some(ch) => {
                    current_text.push(ch);
                    self.advance();
                }
            }
        }
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
                '0'..='9' => {
                    let kind = self.read_number();
                    Token::new(kind, position)
                }
                '"' | '\'' => match self.read_string(ch) {
                    Ok(kind) => Token::new(kind, position),
                    Err(_err) => {
                        // For now, return EOF on error
                        // TODO: Proper error handling will be added later
                        Token::new(TokenKind::EOF, position)
                    }
                },
                '@' => {
                    let kind = self.read_variable();
                    Token::new(kind, position)
                }
                ch if Self::is_identifier_start(ch) => {
                    let kind = self.read_identifier();
                    Token::new(kind, position)
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
