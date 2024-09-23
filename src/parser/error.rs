// Error handling for the parser
// Manages error creation, reporting, and recovery

use crate::error::{MetorexError, SourceLocation};
use crate::lexer::{Position, Token};

/// Error handling state for the parser
pub struct ErrorHandler {
    /// Errors encountered during parsing
    errors: Vec<MetorexError>,
    /// Flag indicating if we're in panic mode (error recovery)
    panic_mode: bool,
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            panic_mode: false,
        }
    }

    /// Get all collected errors
    pub fn errors(&self) -> &[MetorexError] {
        &self.errors
    }

    /// Check if any errors have been collected
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Convert a Position to a SourceLocation
    pub fn position_to_location(&self, position: Position) -> SourceLocation {
        SourceLocation::new(position.line, position.column, position.offset)
    }

    /// Create an error at the current token
    pub fn error_at_current(&self, message: &str, current_token: &Token) -> MetorexError {
        let location = self.position_to_location(current_token.position);
        MetorexError::syntax_error(message, location)
    }

    /// Create an error at the previous token
    pub fn error_at_previous(&self, message: &str, previous_token: &Token) -> MetorexError {
        let location = self.position_to_location(previous_token.position);
        MetorexError::syntax_error(message, location)
    }

    /// Report an error and enter panic mode
    pub fn report_error(&mut self, error: MetorexError) {
        if !self.panic_mode {
            self.panic_mode = true;
            self.errors.push(error);
        }
    }

    /// Exit panic mode (called at the start of synchronization)
    pub fn start_synchronize(&mut self) {
        self.panic_mode = false;
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}
