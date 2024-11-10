// Parser module for Metorex
// Converts a stream of tokens into an Abstract Syntax Tree (AST)

mod error;
mod expressions;
mod statements;
mod token_stream;

use crate::ast::Statement;
use crate::error::MetorexError;
use crate::lexer::{Token, TokenKind};

use error::ErrorHandler;
use token_stream::TokenStream;

/// The parser converts a token stream into an AST
pub struct Parser {
    /// Token stream for navigation
    stream: TokenStream,
    /// Error handler for reporting and recovery
    error_handler: ErrorHandler,
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            stream: TokenStream::new(tokens),
            error_handler: ErrorHandler::new(),
        }
    }

    /// Get the current token without consuming it
    fn peek(&self) -> &Token {
        self.stream.peek()
    }

    /// Peek ahead by offset tokens
    fn peek_ahead(&self, offset: usize) -> &Token {
        self.stream.peek_ahead(offset)
    }

    /// Get the previous token
    fn previous(&self) -> &Token {
        self.stream.previous()
    }

    /// Check if we're at the end of the token stream
    fn is_at_end(&self) -> bool {
        self.stream.is_at_end()
    }

    /// Advance to the next token and return the previous one
    fn advance(&mut self) -> Token {
        self.stream.advance()
    }

    /// Check if the current token matches any of the given kinds
    fn check(&self, kinds: &[TokenKind]) -> bool {
        self.stream.check(kinds)
    }

    /// Check if the current token matches a specific kind (handles complex matching)
    fn match_kind(&self, kind: &TokenKind) -> bool {
        self.stream.match_kind(kind)
    }

    /// Consume the current token if it matches any of the given kinds
    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        self.stream.match_token(kinds)
    }

    /// Expect a specific token kind and consume it, or report an error
    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<Token, MetorexError> {
        if self.match_kind(&kind) {
            Ok(self.advance())
        } else {
            let _token = self.peek();
            Err(self.error_at_current(message))
        }
    }

    /// Skip newlines and comments
    fn skip_whitespace(&mut self) {
        self.stream.skip_whitespace()
    }

    /// Get a reference to the token stream for advanced operations
    pub(crate) fn stream(&self) -> &TokenStream {
        &self.stream
    }

    /// Create an error at the current token
    fn error_at_current(&self, message: &str) -> MetorexError {
        self.error_handler.error_at_current(message, self.peek())
    }

    /// Create an error at the previous token
    fn error_at_previous(&self, message: &str) -> MetorexError {
        self.error_handler
            .error_at_previous(message, self.previous())
    }

    /// Report an error and enter panic mode
    fn report_error(&mut self, error: MetorexError) {
        self.error_handler.report_error(error);
    }

    /// Synchronize after an error (panic mode recovery)
    /// Skip tokens until we find a statement boundary
    fn synchronize(&mut self) {
        self.error_handler.start_synchronize();

        while !self.is_at_end() {
            // If we just passed a newline or semicolon, we're at a statement boundary
            if matches!(
                self.previous().kind,
                TokenKind::Newline | TokenKind::Semicolon
            ) {
                return;
            }

            // Also synchronize at the start of a new statement
            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Def
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Do
                | TokenKind::End => return,
                _ => {}
            }

            self.advance();
        }
    }

    /// Parse a complete program (list of statements)
    pub fn parse(&mut self) -> Result<Vec<Statement>, Vec<MetorexError>> {
        let mut statements = Vec::new();

        // Skip leading whitespace
        self.skip_whitespace();

        while !self.is_at_end() {
            // Skip any whitespace between statements
            self.skip_whitespace();

            if self.is_at_end() {
                break;
            }

            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.report_error(err);
                    self.synchronize();
                }
            }

            // Skip trailing whitespace after statement
            self.skip_whitespace();
        }

        if self.error_handler.has_errors() {
            Err(self.error_handler.errors().to_vec())
        } else {
            Ok(statements)
        }
    }
}
