// Compiler: AST to Bytecode
//
// Walks the AST and emits bytecode instructions into a Chunk.

mod expressions;
mod statements;

use crate::ast::Statement;
use crate::bytecode::chunk::Chunk;
use crate::bytecode::opcode::OpCode;
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::token::Position;
use crate::object::Object;
use std::rc::Rc;

/// A local variable tracked during compilation.
#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: i32, // -1 means "declared but not yet defined"
}

/// Tracks a loop being compiled so break/continue know where to jump.
#[derive(Debug, Clone)]
struct LoopContext {
    /// Bytecode offset of the loop start (for continue / Loop instruction).
    start: usize,
    /// Scope depth when the loop began (so break/continue can pop locals).
    scope_depth: i32,
    /// Placeholder offsets for break jumps that need to be patched at loop end.
    break_jumps: Vec<usize>,
}

/// Compiler state for producing bytecode from an AST.
pub struct Compiler {
    /// The chunk being compiled into.
    chunk: Chunk,

    /// Local variable stack (compile-time only).
    locals: Vec<Local>,

    /// Current scope depth (0 = global).
    scope_depth: i32,

    /// Stack of active loops (for break/continue).
    loop_stack: Vec<LoopContext>,
}

impl Compiler {
    /// Create a new compiler.
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loop_stack: Vec::new(),
        }
    }

    /// Compile a list of statements (a program) into a Chunk.
    pub fn compile(mut self, program: &[Statement]) -> Result<Chunk, MetorexError> {
        for stmt in program {
            self.compile_statement(stmt)?;
        }
        self.emit_op(OpCode::Return, 0);
        Ok(self.chunk)
    }

    // ── Scope management ────────────────────────────────────────────────

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, line: usize) {
        self.scope_depth -= 1;
        // Pop locals that went out of scope
        while let Some(local) = self.locals.last() {
            if local.depth <= self.scope_depth {
                break;
            }
            self.emit_op(OpCode::Pop, line);
            self.locals.pop();
        }
    }

    // ── Local variable management ───────────────────────────────────────

    fn add_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: -1, // not yet defined
        });
    }

    fn mark_initialized(&mut self) {
        if let Some(local) = self.locals.last_mut() {
            local.depth = self.scope_depth;
        }
    }

    /// Resolve a local variable by name. Returns its stack slot index.
    fn resolve_local(&self, name: &str) -> Option<u8> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i as u8);
            }
        }
        None
    }

    // ── Loop management ─────────────────────────────────────────────────

    fn begin_loop(&mut self, start: usize) {
        self.loop_stack.push(LoopContext {
            start,
            scope_depth: self.scope_depth,
            break_jumps: Vec::new(),
        });
    }

    fn end_loop(&mut self) -> LoopContext {
        self.loop_stack.pop().expect("end_loop without begin_loop")
    }

    /// Emit Pop instructions for locals between the current scope and the
    /// loop's scope depth (used by break/continue to clean up the stack).
    fn emit_pops_to_loop_scope(&mut self, line: usize) {
        let loop_depth = self
            .loop_stack
            .last()
            .expect("emit_pops_to_loop_scope outside loop")
            .scope_depth;
        let pop_count = self
            .locals
            .iter()
            .rev()
            .take_while(|local| local.depth > loop_depth)
            .count();
        for _ in 0..pop_count {
            self.emit_op(OpCode::Pop, line);
        }
    }

    // ── Emitters ────────────────────────────────────────────────────────

    fn emit_op(&mut self, op: OpCode, line: usize) {
        self.chunk.write_opcode(op, line);
    }

    fn emit_op_u8(&mut self, op: OpCode, operand: u8, line: usize) {
        self.chunk.write_op_u8(op, operand, line);
    }

    fn emit_op_u16(&mut self, op: OpCode, operand: u16, line: usize) {
        self.chunk.write_op_u16(op, operand, line);
    }

    fn emit_constant(&mut self, value: Object, line: usize) -> Result<(), MetorexError> {
        let index = self.chunk.add_constant(value).ok_or_else(|| {
            MetorexError::runtime_error(
                "Too many constants in one chunk (max 65535)",
                SourceLocation::new(line, 0, 0),
            )
        })?;
        self.chunk.write_constant(index, line);
        Ok(())
    }

    /// Emit a jump instruction with a placeholder offset. Returns the offset
    /// of the placeholder so it can be patched later.
    fn emit_jump(&mut self, op: OpCode, line: usize) -> usize {
        self.emit_op_u16(op, 0xFFFF, line);
        self.chunk.len() - 2 // offset of the u16 placeholder
    }

    /// Patch a previously emitted jump to point to the current position.
    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.len() - (offset + 2);
        self.chunk.patch_u16(offset, jump as u16);
    }

    /// Emit a loop instruction that jumps backward.
    fn emit_loop(&mut self, loop_start: usize, line: usize) {
        let offset = self.chunk.len() - loop_start + 3; // +3 for the Loop opcode + u16
        self.emit_op_u16(OpCode::Loop, offset as u16, line);
    }

    /// Make a constant for a name string and return its index.
    fn identifier_constant(&mut self, name: &str) -> Result<u8, MetorexError> {
        let index = self
            .chunk
            .add_constant(Object::String(Rc::new(name.to_string())))
            .ok_or_else(|| {
                MetorexError::runtime_error(
                    "Too many constants in one chunk",
                    SourceLocation::new(0, 0, 0),
                )
            })?;
        if index > u8::MAX as u16 {
            return Err(MetorexError::runtime_error(
                "Too many global variable names in one chunk (max 256 for short form)",
                SourceLocation::new(0, 0, 0),
            ));
        }
        Ok(index as u8)
    }

    fn pos_line(pos: &Position) -> usize {
        pos.line
    }

    /// Access the compiled chunk (for testing).
    pub fn current_chunk(&self) -> &Chunk {
        &self.chunk
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
