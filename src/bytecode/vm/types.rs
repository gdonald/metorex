// Bytecode VM types: UpvalueObj, ClosureObj, CallFrame

use std::cell::RefCell;
use std::rc::Rc;

use crate::bytecode::chunk::Chunk;
use crate::bytecode::opcode::OpCode;
use crate::object::{CompiledFunction, Object};

// ── Upvalue (13.8) ─────────────────────────────────────────────────────

/// A runtime upvalue — a captured variable that may still be on the stack
/// (open) or has been moved to the heap (closed).
#[derive(Debug, Clone)]
pub struct UpvalueObj {
    /// The captured value. While the local is still alive on the stack,
    /// this is kept in sync via `stack_index`. Once the local goes out
    /// of scope, the value is "closed" — moved here permanently.
    pub value: Rc<RefCell<Object>>,
    /// Stack index this upvalue points to while it is open.
    /// Set to `usize::MAX` once closed.
    pub stack_index: usize,
}

impl UpvalueObj {
    pub fn new_open(stack_index: usize, value: Object) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            stack_index,
        }
    }

    pub fn is_open(&self) -> bool {
        self.stack_index != usize::MAX
    }

    pub fn close(&mut self, value: Object) {
        *self.value.borrow_mut() = value;
        self.stack_index = usize::MAX;
    }
}

/// A runtime closure — a compiled function bundled with its captured upvalues.
#[derive(Debug, Clone)]
pub struct ClosureObj {
    pub function: Rc<CompiledFunction>,
    pub upvalues: Vec<Rc<RefCell<UpvalueObj>>>,
}

// ── Call Frame (13.2) ──────────────────────────────────────────────────

/// A single call frame on the VM's call stack.
///
/// Each function/closure invocation pushes a new frame. The frame tracks
/// the function being executed, the instruction pointer within its chunk,
/// and the base offset into the value stack where the frame's locals begin.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The compiled function being executed.
    pub function: Rc<CompiledFunction>,
    /// Instruction pointer — offset into `function.chunk.code`.
    pub ip: usize,
    /// Base slot in the value stack where this frame's locals start.
    pub slot_offset: usize,
    /// Upvalues captured by this frame's closure (empty for non-closures).
    pub upvalues: Vec<Rc<RefCell<UpvalueObj>>>,
}

impl CallFrame {
    /// Create a new call frame.
    pub fn new(function: Rc<CompiledFunction>, slot_offset: usize) -> Self {
        Self {
            function,
            ip: 0,
            slot_offset,
            upvalues: Vec::new(),
        }
    }

    /// Create a call frame for a closure with upvalues.
    pub fn new_closure(closure: ClosureObj, slot_offset: usize) -> Self {
        Self {
            function: closure.function,
            ip: 0,
            slot_offset,
            upvalues: closure.upvalues,
        }
    }

    /// Get a reference to this frame's bytecode chunk.
    pub fn chunk(&self) -> &Chunk {
        &self.function.chunk
    }

    /// Read the byte at the current IP and advance IP by 1.
    pub fn read_byte(&mut self) -> u8 {
        let byte = self.function.chunk.read_byte(self.ip);
        self.ip += 1;
        byte
    }

    /// Read a u16 (big-endian) at the current IP and advance IP by 2.
    pub fn read_u16(&mut self) -> u16 {
        let val = self.function.chunk.read_u16(self.ip);
        self.ip += 2;
        val
    }

    /// Read the current opcode and advance IP.
    pub fn read_opcode(&mut self) -> Option<OpCode> {
        let byte = self.read_byte();
        OpCode::from_byte(byte)
    }

    /// Get the source line for the current IP (for error reporting).
    pub fn current_line(&self) -> usize {
        if self.ip > 0 {
            self.function.chunk.get_line(self.ip - 1)
        } else {
            0
        }
    }
}
