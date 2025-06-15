// Metorex Error Handling Foundation
//
// This module defines the comprehensive error type system for the Metorex language.
// It provides structured error reporting with source location information and
// formatting utilities for beautiful error messages.

use std::fmt;
use thiserror::Error;

/// Source location information for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset in the source file
    pub offset: usize,
    /// Optional filename
    pub filename: Option<String>,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
            filename: None,
        }
    }

    /// Create a source location with a filename
    pub fn with_filename(line: usize, column: usize, offset: usize, filename: String) -> Self {
        Self {
            line,
            column,
            offset,
            filename: Some(filename),
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(filename) = &self.filename {
            write!(f, "{}:{}:{}", filename, self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}

/// Stack frame information for runtime error traces
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    /// Function or method name
    pub function_name: String,
    /// Source location where the call occurred
    pub location: SourceLocation,
}

impl StackFrame {
    /// Create a new stack frame
    pub fn new(function_name: String, location: SourceLocation) -> Self {
        Self {
            function_name,
            location,
        }
    }
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  at {} ({})", self.function_name, self.location)
    }
}

/// Main error type for Metorex
#[derive(Error, Debug, Clone)]
pub enum MetorexError {
    /// Syntax errors encountered during parsing
    #[error("Syntax error at {location}: {message}")]
    SyntaxError {
        message: String,
        location: SourceLocation,
    },

    /// Runtime errors encountered during execution
    #[error("Runtime error at {location}: {message}")]
    RuntimeError {
        message: String,
        location: SourceLocation,
        stack_trace: Vec<StackFrame>,
    },

    /// Type mismatch or type-related errors
    #[error("Type error at {location}: {message}")]
    TypeError {
        message: String,
        location: SourceLocation,
        expected: Option<String>,
        found: Option<String>,
    },

    /// IO errors (file operations, etc.)
    #[error("IO error: {0}")]
    IoError(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Uncaught exception that needs to be propagated as ControlFlow
    #[error("Runtime error at {location}: Uncaught exception: {message}")]
    UncaughtException {
        exception: crate::object::Object,
        location: SourceLocation,
        message: String,
    },
}

// Custom From implementation for std::io::Error
impl From<std::io::Error> for MetorexError {
    fn from(err: std::io::Error) -> Self {
        MetorexError::IoError(err.to_string())
    }
}

impl MetorexError {
    /// Create a new syntax error
    pub fn syntax_error(message: impl Into<String>, location: SourceLocation) -> Self {
        Self::SyntaxError {
            message: message.into(),
            location,
        }
    }

    /// Create a new runtime error with an empty stack trace
    pub fn runtime_error(message: impl Into<String>, location: SourceLocation) -> Self {
        Self::RuntimeError {
            message: message.into(),
            location,
            stack_trace: Vec::new(),
        }
    }

    /// Create a new runtime error with a stack trace
    pub fn runtime_error_with_trace(
        message: impl Into<String>,
        location: SourceLocation,
        stack_trace: Vec<StackFrame>,
    ) -> Self {
        Self::RuntimeError {
            message: message.into(),
            location,
            stack_trace,
        }
    }

    /// Create a new type error
    pub fn type_error(message: impl Into<String>, location: SourceLocation) -> Self {
        Self::TypeError {
            message: message.into(),
            location,
            expected: None,
            found: None,
        }
    }

    /// Create a new type error with expected and found types
    pub fn type_error_with_types(
        message: impl Into<String>,
        location: SourceLocation,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::TypeError {
            message: message.into(),
            location,
            expected: Some(expected.into()),
            found: Some(found.into()),
        }
    }

    /// Create a new internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Add a stack frame to a runtime error
    pub fn with_stack_frame(self, frame: StackFrame) -> Self {
        match self {
            Self::RuntimeError {
                message,
                location,
                mut stack_trace,
            } => {
                stack_trace.push(frame);
                Self::RuntimeError {
                    message,
                    location,
                    stack_trace,
                }
            }
            other => other,
        }
    }

    /// Get the source location associated with this error, if any
    pub fn location(&self) -> Option<&SourceLocation> {
        match self {
            Self::SyntaxError { location, .. }
            | Self::RuntimeError { location, .. }
            | Self::TypeError { location, .. } => Some(location),
            _ => None,
        }
    }
}

/// Result type alias for Metorex operations
pub type Result<T> = std::result::Result<T, MetorexError>;

/// Error reporting utilities
pub mod reporting {
    use super::*;

    /// Format an error with source code context
    pub fn format_error_with_source(error: &MetorexError, source: &str) -> String {
        let mut output = String::new();

        // Add the main error message
        output.push_str(&format!("Error: {}\n", error));

        // Add source code snippet if location is available
        if let Some(location) = error.location()
            && let Some(snippet) = extract_source_snippet(source, location)
        {
            output.push('\n');
            output.push_str(&snippet);
        }

        // Add stack trace for runtime errors
        if let MetorexError::RuntimeError { stack_trace, .. } = error
            && !stack_trace.is_empty()
        {
            output.push_str("\n\nStack trace:\n");
            for frame in stack_trace {
                output.push_str(&format!("{}\n", frame));
            }
        }

        output
    }

    /// Extract a snippet of source code around the error location
    fn extract_source_snippet(source: &str, location: &SourceLocation) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();

        if location.line == 0 || location.line > lines.len() {
            return None;
        }

        let line_idx = location.line - 1;
        let line = lines[line_idx];

        let mut snippet = String::new();

        // Line number width for alignment
        let line_num_width = location.line.to_string().len();

        // Show the line with the error
        snippet.push_str(&format!(
            "{:width$} | {}\n",
            location.line,
            line,
            width = line_num_width
        ));

        // Show the error indicator (^)
        snippet.push_str(&format!(
            "{:width$} | {}^\n",
            "",
            " ".repeat(location.column.saturating_sub(1)),
            width = line_num_width
        ));

        Some(snippet)
    }

    /// Format an error for display in REPL or CLI
    pub fn format_error_compact(error: &MetorexError) -> String {
        format!("{}", error)
    }
}
