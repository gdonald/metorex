// Bytecode Virtual Machine
//
// A stack-based VM that executes compiled bytecode chunks.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::chunk::Chunk;
use crate::bytecode::opcode::OpCode;
use crate::class::Class;
use crate::error::{MetorexError, SourceLocation};
use crate::object::{CompiledFunction, Instance, Method, Object};

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
    /// Open upvalues — tracked so we can close them when locals go out of scope.
    open_upvalues: Vec<Rc<RefCell<UpvalueObj>>>,
    /// Maps function pointer addresses to closure upvalues. When a function
    /// is wrapped by OP_CLOSURE, its upvalues are stored here and looked up
    /// when the function is called.
    closure_upvalues: HashMap<usize, Vec<Rc<RefCell<UpvalueObj>>>>,
    /// Pending instance from a class constructor call — stored so OP_RETURN
    /// from initialize can return the instance instead of nil.
    pending_instance: Option<Rc<RefCell<Instance>>>,
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

                    // If returning from an initializer, push the instance
                    if let Some(inst) = self.pending_instance.take() {
                        self.push(Object::Instance(inst));
                    } else {
                        self.push(result);
                    }
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
                            // Look up closure upvalues by function pointer address
                            let key = Rc::as_ptr(&func) as usize;
                            let mut frame = CallFrame::new(func, callee_idx + 1);
                            if let Some(upvalues) = self.closure_upvalues.get(&key) {
                                frame.upvalues = upvalues.clone();
                            }
                            self.push_frame(frame);
                        }
                        Object::NativeFunction(name) => {
                            let args: Vec<Object> = self.stack.drain(callee_idx + 1..).collect();
                            let result = self.call_native(&name, &args)?;
                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Object::Class(class) => {
                            // Calling a class as a function creates an instance
                            let args: Vec<Object> = self.stack.drain(callee_idx + 1..).collect();
                            let instance = Instance::new(Rc::clone(&class));
                            let inst_rc = Rc::new(RefCell::new(instance));

                            // Call initialize if it exists
                            let init_key = format!("{}#initialize", class.name());
                            if let Some(Object::CompiledFunction(init_func)) =
                                self.globals.get(&init_key).cloned()
                            {
                                // Push instance as receiver, then args
                                self.stack.truncate(callee_idx);
                                self.push(Object::Instance(Rc::clone(&inst_rc)));
                                for arg in &args {
                                    self.push(arg.clone());
                                }
                                let slot = callee_idx;
                                let frame = CallFrame::new(init_func, slot);
                                self.push_frame(frame);
                                // After init returns, the instance will be on the stack
                                // We store it so Return can put it back
                                self.pending_instance = Some(inst_rc);
                            } else {
                                self.stack.truncate(callee_idx);
                                self.push(Object::Instance(inst_rc));
                            }
                        }
                        _ => {
                            return Err(self.runtime_err("Can only call functions and classes"));
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

                // ── Closures and upvalues (13.8) ─────────────────────
                OpCode::Closure => {
                    let uv_count = self.current_frame_mut()?.read_byte() as usize;
                    // The function is on top of the stack
                    let func_val = self.peek(0)?.clone();
                    let func = match func_val {
                        Object::CompiledFunction(f) => f,
                        _ => return Err(self.runtime_err("Expected function for closure")),
                    };

                    let mut upvalues = Vec::with_capacity(uv_count);
                    for _ in 0..uv_count {
                        let is_local = self.current_frame_mut()?.read_byte() != 0;
                        let index = self.current_frame_mut()?.read_byte() as usize;

                        if is_local {
                            // Capture a local from the enclosing frame
                            let slot = self.current_frame()?.slot_offset + index;
                            let uv = self.capture_upvalue(slot);
                            upvalues.push(uv);
                        } else {
                            // Forward an upvalue from the enclosing closure
                            let enclosing_uv = self.current_frame()?.upvalues.get(index).cloned();
                            match enclosing_uv {
                                Some(uv) => upvalues.push(uv),
                                None => {
                                    return Err(self.runtime_err("Invalid upvalue index"));
                                }
                            }
                        }
                    }

                    // Store the upvalues keyed by the Rc pointer address of the function.
                    // When this function is later called (even after being stored in a
                    // global and retrieved), the upvalues are looked up by this key.
                    let key = Rc::as_ptr(&func) as usize;
                    self.closure_upvalues.insert(key, upvalues);
                }

                OpCode::GetUpvalue => {
                    let index = self.current_frame_mut()?.read_byte() as usize;
                    let val = {
                        let frame = self.current_frame()?;
                        match frame.upvalues.get(index) {
                            Some(uv) => uv.borrow().value.borrow().clone(),
                            None => {
                                return Err(self.runtime_err("Invalid upvalue index"));
                            }
                        }
                    };
                    self.push(val);
                }

                OpCode::SetUpvalue => {
                    let index = self.current_frame_mut()?.read_byte() as usize;
                    let val = self.peek(0)?.clone();
                    let frame = self.current_frame()?;
                    match frame.upvalues.get(index) {
                        Some(uv) => {
                            *uv.borrow().value.borrow_mut() = val;
                        }
                        None => {
                            return Err(self.runtime_err("Invalid upvalue index"));
                        }
                    }
                }

                OpCode::CloseUpvalue => {
                    let stack_top = self.stack.len() - 1;
                    self.close_upvalues(stack_top);
                    self.pop()?;
                }

                // ── Classes and objects (13.9) ─────────────────────────
                OpCode::Class => {
                    let name_idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(name_idx)?;
                    let class = Class::new(name, None);
                    self.push(Object::Class(Rc::new(class)));
                }

                OpCode::Method => {
                    let name_idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(name_idx)?;
                    // Stack: [..., class, compiled_function]
                    let func_val = self.pop()?;
                    let class_val = self.peek(0)?;
                    match (class_val, func_val) {
                        (Object::Class(class), Object::CompiledFunction(func)) => {
                            let method = Method::new(name.clone(), vec![], vec![]);
                            // Store the compiled function in the method's closure vars
                            // so the VM can look it up when calling
                            let mut m = method;
                            m.owner = Some(class.name().to_string());
                            let method_rc = Rc::new(m);
                            class.define_method(&name, method_rc);
                            // Also store the compiled function for bytecode execution
                            let key = format!("{}#{}", class.name(), name);
                            self.define_global(key, Object::CompiledFunction(func));
                        }
                        _ => {
                            return Err(
                                self.runtime_err("OP_METHOD requires class and function on stack")
                            );
                        }
                    }
                }

                OpCode::Invoke => {
                    let operand = self.current_frame_mut()?.read_u16();
                    let name_idx = (operand >> 8) as usize;
                    let arg_count = (operand & 0xFF) as usize;
                    let method_name = self.read_string_constant(name_idx)?;

                    // The receiver is below the arguments on the stack
                    let receiver = self.peek(arg_count)?.clone();

                    match &receiver {
                        Object::Instance(inst) => {
                            let class_name = inst.borrow().class_name().to_string();
                            let key = format!("{}#{}", class_name, method_name);
                            if let Some(Object::CompiledFunction(func)) =
                                self.globals.get(&key).cloned()
                            {
                                if func.arity as usize != arg_count {
                                    return Err(self.runtime_err(&format!(
                                        "Expected {} arguments but got {}",
                                        func.arity, arg_count
                                    )));
                                }
                                let callee_idx = self.stack.len() - 1 - arg_count;
                                let frame = CallFrame::new(func, callee_idx + 1);
                                self.push_frame(frame);
                            } else {
                                return Err(self.runtime_err(&format!(
                                    "Undefined method '{}' for {}",
                                    method_name, class_name
                                )));
                            }
                        }
                        Object::Class(class) => {
                            // Calling ClassName() — instantiation
                            if method_name == "new" || method_name == class.name() {
                                let instance = Instance::new(Rc::clone(class));
                                // Pop receiver + args, push instance
                                for _ in 0..arg_count {
                                    self.pop()?;
                                }
                                // Pop the class (receiver)
                                let receiver_idx = self.stack.len() - 1;
                                self.stack[receiver_idx] =
                                    Object::Instance(Rc::new(RefCell::new(instance)));
                            } else {
                                return Err(self.runtime_err(&format!(
                                    "Undefined class method '{}'",
                                    method_name
                                )));
                            }
                        }
                        _ => {
                            // For non-instance receivers, try native methods
                            // (length, push, etc.) — return an error for now
                            return Err(self.runtime_err(&format!(
                                "Cannot call method '{}' on {}",
                                method_name,
                                receiver.type_name()
                            )));
                        }
                    }
                }

                OpCode::GetInstance => {
                    let name_idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(name_idx)?;
                    // 'self' is the first local in the current frame (slot 0)
                    let offset = self.current_frame()?.slot_offset;
                    if offset < self.stack.len() {
                        let receiver = self.stack[offset].clone();
                        match receiver {
                            Object::Instance(inst) => {
                                let val =
                                    inst.borrow().get_var(&name).cloned().unwrap_or(Object::Nil);
                                self.push(val);
                            }
                            _ => {
                                return Err(self.runtime_err(
                                    "Cannot access instance variable outside of instance",
                                ));
                            }
                        }
                    } else {
                        self.push(Object::Nil);
                    }
                }

                OpCode::SetInstance => {
                    let name_idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(name_idx)?;
                    let value = self.peek(0)?.clone();
                    let offset = self.current_frame()?.slot_offset;
                    if offset < self.stack.len() {
                        let receiver = self.stack[offset].clone();
                        match receiver {
                            Object::Instance(inst) => {
                                inst.borrow_mut().set_var(name, value);
                            }
                            _ => {
                                return Err(self.runtime_err(
                                    "Cannot set instance variable outside of instance",
                                ));
                            }
                        }
                    }
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

    // ── Native function dispatch (14.1) ────────────────────────────────

    fn call_native(&mut self, name: &str, args: &[Object]) -> Result<Object, MetorexError> {
        match name {
            "puts" => {
                for arg in args {
                    println!("{}", arg);
                }
                Ok(Object::Nil)
            }
            "print" => {
                for arg in args {
                    print!("{}", arg);
                }
                Ok(Object::Nil)
            }
            "p" => {
                for arg in args {
                    println!("{:?}", arg);
                }
                Ok(Object::Nil)
            }
            "define_method" => {
                // define_method(:name, compiled_function)
                // or define_method(:name) { block }
                // In bytecode context: args[0] = name (String/Symbol),
                // args[1] = CompiledFunction (the block body)
                if args.is_empty() {
                    return Err(self.runtime_err("define_method requires at least one argument"));
                }
                let method_name = match &args[0] {
                    Object::String(s) => s.to_string(),
                    Object::Symbol(s) => s.to_string(),
                    _ => {
                        return Err(self.runtime_err(
                            "define_method: first argument must be a String or Symbol",
                        ));
                    }
                };

                // The function/block should be the second argument
                if args.len() < 2 {
                    return Err(
                        self.runtime_err("define_method requires a function as second argument")
                    );
                }
                let func = match &args[1] {
                    Object::CompiledFunction(f) => Rc::clone(f),
                    _ => {
                        return Err(
                            self.runtime_err("define_method: second argument must be a function")
                        );
                    }
                };

                // Find the class on the stack (the most recent Class in globals
                // or on the stack). For now, store as a global method.
                let method = Method::new(method_name.clone(), vec![], vec![]);
                let method_rc = Rc::new(method);

                // If there's a class context, attach to it
                // Look for a Class on the stack
                let mut attached = false;
                for val in self.stack.iter().rev() {
                    if let Object::Class(class) = val {
                        class.define_method(&method_name, method_rc.clone());
                        let key = format!("{}#{}", class.name(), method_name);
                        self.globals
                            .insert(key, Object::CompiledFunction(func.clone()));
                        attached = true;
                        break;
                    }
                }

                if !attached {
                    // Store as a global function
                    self.globals
                        .insert(method_name, Object::CompiledFunction(func));
                }

                Ok(Object::Nil)
            }
            _ => Err(self.runtime_err(&format!("Unknown native function '{}'", name))),
        }
    }

    // ── Upvalue helpers (13.8) ────────────────────────────────────────

    /// Capture a local variable as an upvalue (public for testing).
    pub fn capture_upvalue_public(&mut self, stack_index: usize) -> Rc<RefCell<UpvalueObj>> {
        self.capture_upvalue(stack_index)
    }

    /// Close upvalues at or above `last` (public for testing).
    pub fn close_upvalues_public(&mut self, last: usize) {
        self.close_upvalues(last);
    }

    /// Capture a local variable as an upvalue. If an open upvalue for this
    /// stack slot already exists, reuse it (so multiple closures share the
    /// same upvalue cell).
    fn capture_upvalue(&mut self, stack_index: usize) -> Rc<RefCell<UpvalueObj>> {
        // Check if we already have an open upvalue for this slot
        for uv in &self.open_upvalues {
            if uv.borrow().stack_index == stack_index {
                return Rc::clone(uv);
            }
        }
        // Create a new open upvalue
        let value = self.stack[stack_index].clone();
        let uv = Rc::new(RefCell::new(UpvalueObj::new_open(stack_index, value)));
        self.open_upvalues.push(Rc::clone(&uv));
        uv
    }

    /// Close all upvalues that point to stack slots at or above `last`.
    /// This moves the value from the stack into the upvalue cell.
    fn close_upvalues(&mut self, last: usize) {
        self.open_upvalues.retain(|uv| {
            let mut uv_ref = uv.borrow_mut();
            if uv_ref.stack_index >= last {
                // Close this upvalue: copy the current stack value into it
                let value = self.stack[uv_ref.stack_index].clone();
                uv_ref.close(value);
                false // remove from open list
            } else {
                true // keep in open list
            }
        });
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
