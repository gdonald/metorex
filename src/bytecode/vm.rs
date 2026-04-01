// Bytecode Virtual Machine
//
// A stack-based VM that executes compiled bytecode chunks.

use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::chunk::Chunk;
use crate::bytecode::opcode::OpCode;
use crate::error::{MetorexError, SourceLocation};
use crate::object::{CompiledFunction, Object};

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
}

impl CallFrame {
    /// Create a new call frame.
    pub fn new(function: Rc<CompiledFunction>, slot_offset: usize) -> Self {
        Self {
            function,
            ip: 0,
            slot_offset,
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

// ── Bytecode VM (13.1) ────────────────────────────────────────────────

/// The bytecode virtual machine.
///
/// Executes compiled bytecode using a value stack and call frame stack.
pub struct BytecodeVm {
    /// Value stack — operands and temporaries.
    stack: Vec<Object>,
    /// Call frame stack — one per active function invocation.
    frames: Vec<CallFrame>,
    /// Global variables.
    globals: HashMap<String, Object>,
    /// Open upvalues (heap-allocated captured locals).
    open_upvalues: Vec<Object>,
}

impl BytecodeVm {
    /// Create a new VM.
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            open_upvalues: Vec::new(),
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
        let func = CompiledFunction {
            name: String::new(),
            arity: 0,
            chunk: chunk.clone(),
        };
        let frame = CallFrame::new(Rc::new(func), 0);
        self.push_frame(frame);
        self.run()
    }

    /// Main execution loop — fetch, decode, execute.
    fn run(&mut self) -> Result<Object, MetorexError> {
        loop {
            let frame = self.current_frame_mut()?;
            let chunk_len = frame.chunk().len();

            if frame.ip >= chunk_len {
                return Ok(Object::Nil);
            }

            let op = frame.read_opcode();
            let Some(op) = op else {
                let line = self.current_frame()?.current_line();
                return Err(MetorexError::runtime_error(
                    "Invalid opcode",
                    SourceLocation::new(line, 0, 0),
                ));
            };

            match op {
                OpCode::Return => {
                    let result = self.pop().unwrap_or(Object::Nil);
                    let finished_frame = self.pop_frame()?;

                    if self.frames.is_empty() {
                        return Ok(result);
                    }

                    // Discard the finished frame's stack slots
                    self.stack.truncate(finished_frame.slot_offset);
                    self.push(result);
                }

                OpCode::Constant => {
                    let idx = self.current_frame_mut()?.read_byte() as usize;
                    let val = self.current_frame()?.chunk().get_constant(idx).clone();
                    self.push(val);
                }

                OpCode::ConstantLong => {
                    let idx = self.current_frame_mut()?.read_u16() as usize;
                    let val = self.current_frame()?.chunk().get_constant(idx).clone();
                    self.push(val);
                }

                OpCode::Nil => self.push(Object::Nil),
                OpCode::True => self.push(Object::Bool(true)),
                OpCode::False => self.push(Object::Bool(false)),
                OpCode::Pop => {
                    self.pop()?;
                }

                // ── Arithmetic ─────────────────────────────────────────
                OpCode::Add => self.binary_op(binary_add)?,
                OpCode::Subtract => self.binary_op(binary_sub)?,
                OpCode::Multiply => self.binary_op(binary_mul)?,
                OpCode::Divide => self.binary_op(binary_div)?,
                OpCode::Modulo => self.binary_op(binary_mod)?,

                OpCode::Negate => {
                    let val = self.pop()?;
                    let result = match val {
                        Object::Int(n) => Object::Int(-n),
                        Object::Float(f) => Object::Float(-f),
                        _ => {
                            return Err(self.runtime_err("Cannot negate non-numeric value"));
                        }
                    };
                    self.push(result);
                }

                OpCode::Not => {
                    let val = self.pop()?;
                    self.push(Object::Bool(val.is_falsy()));
                }

                // ── Comparison ─────────────────────────────────────────
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Object::Bool(a.equals(&b)));
                }
                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Object::Bool(!a.equals(&b)));
                }
                OpCode::Less => self.binary_op(compare_less)?,
                OpCode::Greater => self.binary_op(compare_greater)?,
                OpCode::LessEqual => self.binary_op(compare_less_equal)?,
                OpCode::GreaterEqual => self.binary_op(compare_greater_equal)?,

                // ── Variables ──────────────────────────────────────────
                OpCode::GetLocal => {
                    let slot = self.current_frame_mut()?.read_byte() as usize;
                    let offset = self.current_frame()?.slot_offset;
                    let val = self.stack[offset + slot].clone();
                    self.push(val);
                }

                OpCode::SetLocal => {
                    let slot = self.current_frame_mut()?.read_byte() as usize;
                    let offset = self.current_frame()?.slot_offset;
                    let val = self.peek(0)?.clone();
                    self.stack[offset + slot] = val;
                }

                OpCode::GetGlobal => {
                    let idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(idx)?;
                    match self.globals.get(&name) {
                        Some(val) => {
                            let val = val.clone();
                            self.push(val);
                        }
                        None => {
                            return Err(self.runtime_err(&format!("Undefined variable '{}'", name)));
                        }
                    }
                }

                OpCode::SetGlobal => {
                    let idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(idx)?;
                    let val = self.peek(0)?.clone();
                    if !self.set_global(&name, val) {
                        return Err(self.runtime_err(&format!("Undefined variable '{}'", name)));
                    }
                }

                OpCode::DefineGlobal => {
                    let idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(idx)?;
                    let val = self.pop()?;
                    self.define_global(name, val);
                }

                // ── Control flow ───────────────────────────────────────
                OpCode::Jump => {
                    let offset = self.current_frame_mut()?.read_u16() as usize;
                    self.current_frame_mut()?.ip += offset;
                }

                OpCode::JumpIfFalse => {
                    let offset = self.current_frame_mut()?.read_u16() as usize;
                    if self.peek(0)?.is_falsy() {
                        self.current_frame_mut()?.ip += offset;
                    }
                }

                OpCode::Loop => {
                    let offset = self.current_frame_mut()?.read_u16() as usize;
                    self.current_frame_mut()?.ip -= offset;
                }

                // ── Function calls ─────────────────────────────────────
                OpCode::Call => {
                    let arg_count = self.current_frame_mut()?.read_byte() as usize;
                    let callee_idx = self.stack.len() - 1 - arg_count;
                    let callee = self.stack[callee_idx].clone();
                    match callee {
                        Object::CompiledFunction(func) => {
                            if func.arity as usize != arg_count {
                                return Err(self.runtime_err(&format!(
                                    "Expected {} arguments but got {}",
                                    func.arity, arg_count
                                )));
                            }
                            // slot_offset points past the callee to the first argument
                            let frame = CallFrame::new(func, callee_idx + 1);
                            self.push_frame(frame);
                        }
                        _ => {
                            return Err(self.runtime_err("Can only call functions"));
                        }
                    }
                }

                // ── Collections ────────────────────────────────────────
                OpCode::Array => {
                    let count = self.current_frame_mut()?.read_byte() as usize;
                    let start = self.stack.len() - count;
                    let elements: Vec<Object> = self.stack.drain(start..).collect();
                    self.push(Object::Array(Rc::new(std::cell::RefCell::new(elements))));
                }

                OpCode::Hash => {
                    let count = self.current_frame_mut()?.read_byte() as usize;
                    let pair_count = count * 2;
                    let start = self.stack.len() - pair_count;
                    let items: Vec<Object> = self.stack.drain(start..).collect();
                    let mut map = HashMap::new();
                    for pair in items.chunks(2) {
                        let key = match &pair[0] {
                            Object::String(s) => s.to_string(),
                            other => other.to_string(),
                        };
                        map.insert(key, pair[1].clone());
                    }
                    self.push(Object::Dict(Rc::new(std::cell::RefCell::new(map))));
                }

                OpCode::IndexGet => {
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    let result = index_get(&collection, &index)?;
                    self.push(result);
                }

                OpCode::IndexSet => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    index_set(&collection, &index, &value)?;
                    self.push(value);
                }

                // ── Upvalues (placeholder) ─────────────────────────────
                OpCode::GetUpvalue
                | OpCode::SetUpvalue
                | OpCode::CloseUpvalue
                | OpCode::Closure => {
                    // Skip operand bytes for now
                    let _operand = self.current_frame_mut()?.read_byte();
                    if op == OpCode::Closure {
                        // Closure has additional upvalue descriptors
                        let uv_count = _operand as usize;
                        for _ in 0..uv_count {
                            self.current_frame_mut()?.read_byte(); // is_local
                            self.current_frame_mut()?.read_byte(); // index
                        }
                    }
                }

                // ── Class/method (placeholder) ─────────────────────────
                OpCode::Class => {
                    let _name_idx = self.current_frame_mut()?.read_byte();
                }
                OpCode::Method => {
                    let _name_idx = self.current_frame_mut()?.read_byte();
                }
                OpCode::Invoke => {
                    let _operand = self.current_frame_mut()?.read_u16();
                }
                OpCode::GetInstance | OpCode::SetInstance => {
                    let _idx = self.current_frame_mut()?.read_byte();
                }

                // ── Exception handling (placeholder) ──────────────────
                OpCode::Try => {
                    let _operand = self.current_frame_mut()?.read_u16();
                }
                OpCode::Catch | OpCode::Finally | OpCode::EndTry | OpCode::Raise => {}
                OpCode::Match => {}
                OpCode::MatchPattern => {
                    let _operand = self.current_frame_mut()?.read_u16();
                }
            }
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────

    fn binary_op<F>(&mut self, op: F) -> Result<(), MetorexError>
    where
        F: FnOnce(&Object, &Object) -> Result<Object, String>,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = op(&a, &b).map_err(|msg| self.runtime_err(&msg))?;
        self.push(result);
        Ok(())
    }

    fn read_string_constant(&self, idx: usize) -> Result<String, MetorexError> {
        let constant = self.current_frame()?.chunk().get_constant(idx);
        match constant {
            Object::String(s) => Ok(s.to_string()),
            _ => Err(self.runtime_err("Expected string constant")),
        }
    }

    fn runtime_err(&self, msg: &str) -> MetorexError {
        let line = self.frames.last().map_or(0, |f| f.current_line());
        MetorexError::runtime_error(msg, SourceLocation::new(line, 0, 0))
    }
}

impl Default for BytecodeVm {
    fn default() -> Self {
        Self::new()
    }
}

// ── Arithmetic helpers ────────────────────────────────────────────────

fn binary_add(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a + b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a + b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Float(*a as f64 + b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a + *b as f64)),
        (Object::String(a), Object::String(b)) => {
            Ok(Object::String(Rc::new(format!("{}{}", a, b))))
        }
        _ => Err(format!(
            "Cannot add {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

fn binary_sub(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a - b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a - b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Float(*a as f64 - b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a - *b as f64)),
        _ => Err(format!(
            "Cannot subtract {} from {}",
            b.type_name(),
            a.type_name()
        )),
    }
}

fn binary_mul(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a * b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a * b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Float(*a as f64 * b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a * *b as f64)),
        _ => Err(format!(
            "Cannot multiply {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

fn binary_div(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(_), Object::Int(0)) => Err("Division by zero".to_string()),
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a / b)),
        (Object::Float(_), Object::Float(b)) if *b == 0.0 => Err("Division by zero".to_string()),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Float(a / b)),
        (Object::Int(a), Object::Float(b)) if *b == 0.0 => Err("Division by zero".to_string()),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Float(*a as f64 / b)),
        (Object::Float(_), Object::Int(0)) => Err("Division by zero".to_string()),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Float(a / *b as f64)),
        _ => Err(format!(
            "Cannot divide {} by {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

fn binary_mod(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(_), Object::Int(0)) => Err("Modulo by zero".to_string()),
        (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a % b)),
        _ => Err(format!(
            "Cannot modulo {} by {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

// ── Comparison helpers ────────────────────────────────────────────────

fn compare_less(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Bool(a < b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Bool(a < b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Bool((*a as f64) < *b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Bool(*a < *b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

fn compare_greater(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Bool(a > b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Bool(a > b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Bool(*a as f64 > *b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Bool(*a > *b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

fn compare_less_equal(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Bool(a <= b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Bool(a <= b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Bool(*a as f64 <= *b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Bool(*a <= *b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

fn compare_greater_equal(a: &Object, b: &Object) -> Result<Object, String> {
    match (a, b) {
        (Object::Int(a), Object::Int(b)) => Ok(Object::Bool(a >= b)),
        (Object::Float(a), Object::Float(b)) => Ok(Object::Bool(a >= b)),
        (Object::Int(a), Object::Float(b)) => Ok(Object::Bool(*a as f64 >= *b)),
        (Object::Float(a), Object::Int(b)) => Ok(Object::Bool(*a >= *b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

// ── Collection helpers ────────────────────────────────────────────────

fn index_get(collection: &Object, index: &Object) -> Result<Object, MetorexError> {
    match (collection, index) {
        (Object::Array(arr), Object::Int(i)) => {
            let arr = arr.borrow();
            let idx = if *i < 0 {
                (arr.len() as i64 + i) as usize
            } else {
                *i as usize
            };
            Ok(arr.get(idx).cloned().unwrap_or(Object::Nil))
        }
        (Object::Dict(dict), Object::String(key)) => {
            let dict = dict.borrow();
            Ok(dict.get(key.as_str()).cloned().unwrap_or(Object::Nil))
        }
        _ => Err(MetorexError::runtime_error(
            format!(
                "Cannot index {} with {}",
                collection.type_name(),
                index.type_name()
            ),
            SourceLocation::new(0, 0, 0),
        )),
    }
}

fn index_set(collection: &Object, index: &Object, value: &Object) -> Result<(), MetorexError> {
    match (collection, index) {
        (Object::Array(arr), Object::Int(i)) => {
            let mut arr = arr.borrow_mut();
            let idx = if *i < 0 {
                (arr.len() as i64 + i) as usize
            } else {
                *i as usize
            };
            if idx < arr.len() {
                arr[idx] = value.clone();
                Ok(())
            } else {
                Err(MetorexError::runtime_error(
                    "Array index out of bounds",
                    SourceLocation::new(0, 0, 0),
                ))
            }
        }
        (Object::Dict(dict), Object::String(key)) => {
            let mut dict = dict.borrow_mut();
            dict.insert(key.to_string(), value.clone());
            Ok(())
        }
        _ => Err(MetorexError::runtime_error(
            format!(
                "Cannot index {} with {}",
                collection.type_name(),
                index.type_name()
            ),
            SourceLocation::new(0, 0, 0),
        )),
    }
}
