// Token stream management for the parser
// Handles token navigation, matching, and whitespace handling

use crate::lexer::{Token, TokenKind};

/// Encapsulates token navigation state and operations
pub struct TokenStream {
    /// The tokens to parse
    tokens: Vec<Token>,
    /// Current position in the token stream
    current: usize,
}

impl TokenStream {
    /// Create a new token stream from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Get the current token without consuming it
    pub fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            // If we're past the end, return the last token (should be EOF)
            self.tokens.last().unwrap()
        })
    }

    /// Get the token at an offset from the current position
    #[allow(dead_code)]
    pub fn peek_ahead(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.current + offset)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    /// Get the previous token
    pub fn previous(&self) -> &Token {
        if self.current > 0 {
            &self.tokens[self.current - 1]
        } else {
            &self.tokens[0]
        }
    }

    /// Check if we're at the end of the token stream
    pub fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::EOF)
    }

    /// Advance to the next token and return the previous one
    pub fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    /// Check if the current token matches any of the given kinds
    pub fn check(&self, kinds: &[TokenKind]) -> bool {
        if self.is_at_end() {
            return false;
        }
        kinds.iter().any(|kind| self.match_kind(kind))
    }

    /// Check if the current token matches a specific kind (handles complex matching)
    pub fn match_kind(&self, kind: &TokenKind) -> bool {
        let current = &self.peek().kind;
        match (kind, current) {
            (TokenKind::Ident(_), TokenKind::Ident(_)) => true,
            (TokenKind::Int(_), TokenKind::Int(_)) => true,
            (TokenKind::Float(_), TokenKind::Float(_)) => true,
            (TokenKind::String(_), TokenKind::String(_)) => true,
            (TokenKind::InterpolatedString(_), TokenKind::InterpolatedString(_)) => true,
            (TokenKind::InstanceVar(_), TokenKind::InstanceVar(_)) => true,
            (TokenKind::ClassVar(_), TokenKind::ClassVar(_)) => true,
            (TokenKind::Comment(_), TokenKind::Comment(_)) => true,
            _ => kind == current,
        }
    }

    /// Consume the current token if it matches any of the given kinds
    pub fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        if self.check(kinds) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Skip any newline tokens
    #[allow(dead_code)]
    pub fn skip_newlines(&mut self) {
        while self.match_token(&[TokenKind::Newline]) {
            // Keep consuming newlines
        }
    }

    /// Skip any comment tokens
    #[allow(dead_code)]
    pub fn skip_comments(&mut self) {
        while matches!(self.peek().kind, TokenKind::Comment(_)) {
            self.advance();
        }
    }

    /// Skip newlines and comments
    pub fn skip_whitespace(&mut self) {
        while let TokenKind::Newline | TokenKind::Comment(_) = &self.peek().kind {
            self.advance();
        }
    }

    /// Get direct access to tokens for advanced operations (e.g., lookahead)
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Get the current position in the token stream
    pub fn current_position(&self) -> usize {
        self.current
    }
}
