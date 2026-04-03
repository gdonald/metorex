// Bytecode Virtual Machine
//
// A stack-based VM that executes compiled bytecode chunks.

mod arithmetic;
mod collections;
mod comparison;
mod execution;
mod natives;
pub mod types;
mod upvalues;

pub use types::{CallFrame, ClosureObj, UpvalueObj};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::chunk::Chunk;
use crate::error::{MetorexError, SourceLocation};
use crate::object::{CompiledFunction, Instance, Object};

// ── Bytecode VM (13.1) ────────────────────────────────────────────────

/// The bytecode virtual machine.
///
/// Executes compiled bytecode using a value stack and call frame stack.
pub struct BytecodeVm {
    /// Value stack — operands and temporaries.
    pub(super) stack: Vec<Object>,
    /// Call frame stack — one per active function invocation.
    pub(super) frames: Vec<CallFrame>,
    /// Global variables.
    pub(super) globals: HashMap<String, Object>,
    /// Open upvalues — tracked so we can close them when locals go out of scope.
    pub(super) open_upvalues: Vec<Rc<RefCell<UpvalueObj>>>,
    /// Maps function pointer addresses to closure upvalues. When a function
    /// is wrapped by OP_CLOSURE, its upvalues are stored here and looked up
    /// when the function is called.
    pub(super) closure_upvalues: HashMap<usize, Vec<Rc<RefCell<UpvalueObj>>>>,
    /// Pending instance from a class constructor call — stored so OP_RETURN
    /// from initialize can return the instance instead of nil.
    pub(super) pending_instance: Option<Rc<RefCell<Instance>>>,
}

impl BytecodeVm {
    /// Create a new VM.
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            open_upvalues: Vec::new(),
            closure_upvalues: HashMap::new(),
            pending_instance: None,
        }
    }

    /// Register built-in native functions in globals.
    pub fn register_natives(&mut self) {
        for name in &["puts", "print", "p", "define_method"] {
            self.globals
                .insert(name.to_string(), Object::NativeFunction(name.to_string()));
        }
    }

    // ── Stack operations (13.1.8) ──────────────────────────────────────

    /// Push a value onto the stack.
    pub fn push(&mut self, value: Object) {
        self.stack.push(value);
    }

    /// Pop a value from the stack.
    pub fn pop(&mut self) -> Result<Object, MetorexError> {
        self.stack.pop().ok_or_else(|| {
            MetorexError::runtime_error("Stack underflow", SourceLocation::new(0, 0, 0))
        })
    }

    /// Peek at the top of the stack without removing.
    pub fn peek(&self, distance: usize) -> Result<&Object, MetorexError> {
        let len = self.stack.len();
        if distance >= len {
            return Err(MetorexError::runtime_error(
                "Stack underflow on peek",
                SourceLocation::new(0, 0, 0),
            ));
        }
        Ok(&self.stack[len - 1 - distance])
    }

    /// Current stack size.
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    // ── Frame operations (13.2.5) ──────────────────────────────────────

    /// Push a new call frame.
    pub fn push_frame(&mut self, frame: CallFrame) {
        self.frames.push(frame);
    }

    /// Pop the top call frame.
    pub fn pop_frame(&mut self) -> Result<CallFrame, MetorexError> {
        self.frames.pop().ok_or_else(|| {
            MetorexError::runtime_error("Call frame underflow", SourceLocation::new(0, 0, 0))
        })
    }

    /// Get a mutable reference to the current (top) call frame.
    pub fn current_frame_mut(&mut self) -> Result<&mut CallFrame, MetorexError> {
        self.frames.last_mut().ok_or_else(|| {
            MetorexError::runtime_error("No active call frame", SourceLocation::new(0, 0, 0))
        })
    }

    /// Get a reference to the current (top) call frame.
    pub fn current_frame(&self) -> Result<&CallFrame, MetorexError> {
        self.frames.last().ok_or_else(|| {
            MetorexError::runtime_error("No active call frame", SourceLocation::new(0, 0, 0))
        })
    }

    /// Number of active call frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    // ── Globals ────────────────────────────────────────────────────────

    /// Define a global variable.
    pub fn define_global(&mut self, name: String, value: Object) {
        self.globals.insert(name, value);
    }

    /// Get a global variable.
    pub fn get_global(&self, name: &str) -> Option<&Object> {
        self.globals.get(name)
    }

    /// Set an existing global variable. Returns false if it doesn't exist.
    pub fn set_global(&mut self, name: &str, value: Object) -> bool {
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }

    // ── Execution entry point ──────────────────────────────────────────

    /// Execute a compiled chunk as the top-level script.
    pub fn execute(&mut self, chunk: &Chunk) -> Result<Object, MetorexError> {
        self.register_natives();
        let func = CompiledFunction {
            name: String::new(),
            arity: 0,
            chunk: chunk.clone(),
        };
        let frame = CallFrame::new(Rc::new(func), 0);
        self.push_frame(frame);
        self.run()
    }

    // ── Helpers ────────────────────────────────────────────────────────

    pub(super) fn binary_op<F>(&mut self, op: F) -> Result<(), MetorexError>
    where
        F: FnOnce(&Object, &Object) -> Result<Object, String>,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = op(&a, &b).map_err(|msg| self.runtime_err(&msg))?;
        self.push(result);
        Ok(())
    }

    pub(super) fn read_string_constant(&self, idx: usize) -> Result<String, MetorexError> {
        let constant = self.current_frame()?.chunk().get_constant(idx);
        match constant {
            Object::String(s) => Ok(s.to_string()),
            _ => Err(self.runtime_err("Expected string constant")),
        }
    }

    pub(super) fn runtime_err(&self, msg: &str) -> MetorexError {
        let line = self.frames.last().map_or(0, |f| f.current_line());
        MetorexError::runtime_error(msg, SourceLocation::new(line, 0, 0))
    }
}

impl Default for BytecodeVm {
    fn default() -> Self {
        Self::new()
    }
}
