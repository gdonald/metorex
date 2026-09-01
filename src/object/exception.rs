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
    /// Exit status (used by SystemExit)
    pub status: Option<i64>,
    /// Offending name (used by NameError / NoMethodError)
    pub name: Option<String>,
    /// The object the call was made on (used by NameError / NoMethodError).
    pub receiver: Option<Box<Object>>,
    /// The Array `#backtrace` hands out. Ruby answers the same object every
    /// time, so an update through it is visible on the next call.
    pub backtrace_array: Option<Object>,
    /// The file and line of each backtrace entry, which
    /// `#backtrace_locations` reports as Location objects.
    pub backtrace_sites: Option<Vec<(String, usize, String)>>,
    /// The Array `#backtrace_locations` hands out, kept for the same reason.
    pub backtrace_locations_array: Option<Object>,
    /// The class this exception was built from. An exception is identified by
    /// its type name elsewhere, which cannot name an anonymous class, so the
    /// class itself is kept when one is known.
    pub class: Option<std::rc::Rc<crate::class::Class>>,
    /// Instance variables a user-defined subclass set on itself.
    pub instance_vars: indexmap::IndexMap<String, Object>,
    /// Whether a message was supplied. An exception built without one reports
    /// its class name instead, which an explicitly empty message does not.
    pub message_given: bool,
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
            status: None,
            name: None,
            receiver: None,
            backtrace_array: None,
            backtrace_sites: None,
            backtrace_locations_array: None,
            class: None,
            instance_vars: indexmap::IndexMap::new(),
            message_given: true,
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
            status: None,
            name: None,
            receiver: None,
            backtrace_array: None,
            backtrace_sites: None,
            backtrace_locations_array: None,
            class: None,
            instance_vars: indexmap::IndexMap::new(),
            message_given: true,
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
            status: None,
            name: None,
            receiver: None,
            backtrace_array: None,
            backtrace_sites: None,
            backtrace_locations_array: None,
            class: None,
            instance_vars: indexmap::IndexMap::new(),
            message_given: true,
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
            status: None,
            name: None,
            receiver: None,
            backtrace_array: None,
            backtrace_sites: None,
            backtrace_locations_array: None,
            class: None,
            instance_vars: indexmap::IndexMap::new(),
            message_given: true,
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
            status: None,
            name: None,
            receiver: None,
            backtrace_array: None,
            backtrace_sites: None,
            backtrace_locations_array: None,
            class: None,
            instance_vars: indexmap::IndexMap::new(),
            message_given: true,
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
