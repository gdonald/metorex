// Exception handling types - Exception and SourceLocation

use super::Object;

/// Source location for exceptions
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    /// File name or path
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self { file, line, column }
    }
}

/// Exception object for error handling
#[derive(Debug, Clone, PartialEq)]
pub struct Exception {
    /// Exception type/class name
    pub exception_type: String,
    /// Error message
    pub message: String,
    /// Optional backtrace
    pub backtrace: Option<Vec<String>>,
    /// Source location where the exception occurred
    pub location: Option<SourceLocation>,
    /// Cause chain (wrapped exception)
    pub cause: Option<Box<Object>>,
}

impl Exception {
    /// Create a new exception
    pub fn new(exception_type: String, message: String) -> Self {
        Self {
            exception_type,
            message,
            backtrace: None,
            location: None,
            cause: None,
        }
    }

    /// Create an exception with backtrace
    pub fn with_backtrace(exception_type: String, message: String, backtrace: Vec<String>) -> Self {
        Self {
            exception_type,
            message,
            backtrace: Some(backtrace),
            location: None,
            cause: None,
        }
    }

    /// Create an exception with source location
    pub fn with_location(
        exception_type: String,
        message: String,
        location: SourceLocation,
    ) -> Self {
        Self {
            exception_type,
            message,
            backtrace: None,
            location: Some(location),
            cause: None,
        }
    }

    /// Create an exception with a cause
    pub fn with_cause(exception_type: String, message: String, cause: Object) -> Self {
        Self {
            exception_type,
            message,
            backtrace: None,
            location: None,
            cause: Some(Box::new(cause)),
        }
    }

    /// Create an exception with all fields
    pub fn with_all(
        exception_type: String,
        message: String,
        backtrace: Option<Vec<String>>,
        location: Option<SourceLocation>,
        cause: Option<Object>,
    ) -> Self {
        Self {
            exception_type,
            message,
            backtrace,
            location,
            cause: cause.map(Box::new),
        }
    }

    /// Get the full exception chain
    pub fn exception_chain(&self) -> Vec<String> {
        let mut chain = vec![format!("{}: {}", self.exception_type, self.message)];

        if let Some(ref cause_obj) = self.cause
            && let Object::Exception(cause_exc) = cause_obj.as_ref()
        {
            let cause = cause_exc.borrow();
            chain.extend(cause.exception_chain());
        }

        chain
    }
}
