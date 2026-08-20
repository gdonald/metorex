//! Call frame tracking for the Metorex virtual machine.
//!
//! This module provides call frame information used for debugging and stack traces.

/// What kind of code a frame is running, which is what `__callee__` and
/// `__method__` walk the stack to find.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameKind {
    /// A method activation. `callee` is the name the method was called by,
    /// which differs from `defined` when the method was reached by an alias.
    Method { callee: String, defined: String },
    /// A block body, which reports the method that encloses it.
    Block,
    /// A class body or a loaded file, where no method is running at all.
    Boundary,
}

/// Call frame information stored on the VM call stack for debugging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrame {
    /// Human-readable frame identifier (method/function name).
    name: String,
    /// Optional source location ("file:line") to aid debugging.
    location: Option<String>,
    /// What the frame is running.
    kind: FrameKind,
}

impl CallFrame {
    /// Create a new call frame description for a block body.
    pub fn new(name: impl Into<String>, location: Option<String>) -> Self {
        Self {
            name: name.into(),
            location,
            kind: FrameKind::Block,
        }
    }

    /// Create a frame for a method activation.
    pub fn method(
        name: impl Into<String>,
        location: Option<String>,
        callee: impl Into<String>,
        defined: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            location,
            kind: FrameKind::Method {
                callee: callee.into(),
                defined: defined.into(),
            },
        }
    }

    /// Create a frame for a class body or a loaded file.
    pub fn boundary(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: None,
            kind: FrameKind::Boundary,
        }
    }

    /// What this frame is running.
    pub fn kind(&self) -> &FrameKind {
        &self.kind
    }

    /// Return the frame name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the optional source location.
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}
