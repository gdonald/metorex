//! Call frame tracking for the Metorex virtual machine.
//!
//! This module provides call frame information used for debugging and stack traces.

use crate::scope::Scope;
use std::cell::RefCell;
use std::rc::Rc;

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
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Human-readable frame identifier (method/function name).
    name: String,
    /// Optional source location ("file:line") to aid debugging.
    location: Option<String>,
    /// The file the call site sits in, when known. `location` carries only a
    /// line and column, so this is what names the file for a backtrace.
    source_file: Option<String>,
    /// What the frame is running.
    kind: FrameKind,
    /// How many blocks deep the frame sits inside the scope it names. Zero
    /// where the frame runs the scope itself rather than a block written in
    /// it, which is what tells `foo` from `block in foo`.
    block_depth: u32,
    /// The scope the block this frame runs was written in, for a block frame.
    written_in: Option<String>,
    /// The class path of the module the running method was defined in, for
    /// a method frame or a block written in one. None for a singleton
    /// method.
    owner_path: Option<String>,
    /// The scope that was current when the frame was pushed.
    entering_scope: Option<Rc<RefCell<Scope>>>,
}

impl CallFrame {
    /// Create a new call frame description for a block body.
    pub fn new(name: impl Into<String>, location: Option<String>) -> Self {
        Self {
            name: name.into(),
            location,
            source_file: None,
            kind: FrameKind::Block,
            block_depth: 1,
            written_in: None,
            owner_path: None,
            entering_scope: None,
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
            source_file: None,
            kind: FrameKind::Method {
                callee: callee.into(),
                defined: defined.into(),
            },
            block_depth: 0,
            written_in: None,
            owner_path: None,
            entering_scope: None,
        }
    }

    /// Create a frame for a class body or a loaded file.
    pub fn boundary(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: None,
            source_file: None,
            kind: FrameKind::Boundary,
            block_depth: 0,
            written_in: None,
            owner_path: None,
            entering_scope: None,
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

    /// Record the file the call site sits in.
    /// Record where the frame was entered from, which is what pairs a
    /// backtrace entry with the frame that made the call.
    pub fn with_location(mut self, location: Option<String>) -> Self {
        self.location = location;
        self
    }

    pub fn with_source_file(mut self, source_file: Option<String>) -> Self {
        self.source_file = source_file;
        self
    }

    /// The file the call site sits in, when it was recorded.
    pub fn source_file(&self) -> Option<&str> {
        self.source_file.as_deref()
    }

    /// Say the frame runs a block written `depth` blocks inside the scope it
    /// names.
    pub fn nested_in_a_block(mut self, depth: u32) -> Self {
        self.block_depth = depth;
        self
    }

    /// How many blocks deep the frame sits inside the scope it names.
    pub fn block_depth(&self) -> u32 {
        self.block_depth
    }

    /// Say which scope the block this frame runs was written in.
    pub fn written_in_scope(mut self, scope: Option<String>) -> Self {
        self.written_in = scope;
        self
    }

    /// The scope the block this frame runs was written in.
    pub fn written_in(&self) -> Option<&str> {
        self.written_in.as_deref()
    }

    /// Say which module the running method was defined in, by class path.
    pub fn owned_by(mut self, owner_path: Option<String>) -> Self {
        self.owner_path = owner_path;
        self
    }

    /// The class path of the module the running method was defined in.
    pub fn owner_path(&self) -> Option<&str> {
        self.owner_path.as_deref()
    }

    /// Record the scope that was current when the frame was pushed.
    pub(crate) fn entered_from(&mut self, scope: Rc<RefCell<Scope>>) {
        self.entering_scope = Some(scope);
    }

    /// The scope that was current when the frame was pushed, which is the
    /// scope of the code that made the call.
    pub(crate) fn entering_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        self.entering_scope.clone()
    }
}
