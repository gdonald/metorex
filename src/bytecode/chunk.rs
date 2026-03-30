// Chunk — bytecode container
//
// A Chunk holds a sequence of bytecode instructions, a constant pool,
// and line-number information for debugging and error reporting.

use crate::object::Object;

use super::opcode::OpCode;

/// A compiled chunk of bytecode.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Raw bytecode bytes (opcodes + operands).
    code: Vec<u8>,

    /// Constant pool: literals and names referenced by index.
    constants: Vec<Object>,

    /// Line number for each byte in `code` (parallel array).
    lines: Vec<usize>,
}

impl Chunk {
    /// Create a new empty chunk.
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    // ── Writing ─────────────────────────────────────────────────────────

    /// Append a single byte to the bytecode stream.
    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// Append an opcode (convenience wrapper around `write_byte`).
    pub fn write_opcode(&mut self, op: OpCode, line: usize) {
        self.write_byte(op.to_byte(), line);
    }

    /// Append an opcode followed by a single u8 operand.
    pub fn write_op_u8(&mut self, op: OpCode, operand: u8, line: usize) {
        self.write_opcode(op, line);
        self.write_byte(operand, line);
    }

    /// Append an opcode followed by a u16 operand (big-endian).
    pub fn write_op_u16(&mut self, op: OpCode, operand: u16, line: usize) {
        self.write_opcode(op, line);
        self.write_byte((operand >> 8) as u8, line);
        self.write_byte(operand as u8, line);
    }

    // ── Constants ───────────────────────────────────────────────────────

    /// Add a constant to the pool and return its index.
    ///
    /// Returns `None` if the constant pool would exceed `u16::MAX` entries.
    pub fn add_constant(&mut self, value: Object) -> Option<u16> {
        let index = self.constants.len();
        if index > u16::MAX as usize {
            return None;
        }
        self.constants.push(value);
        Some(index as u16)
    }

    /// Write an `OP_CONSTANT` or `OP_CONSTANT_LONG` instruction for the given
    /// constant index.
    pub fn write_constant(&mut self, index: u16, line: usize) {
        if index <= u8::MAX as u16 {
            self.write_op_u8(OpCode::Constant, index as u8, line);
        } else {
            self.write_op_u16(OpCode::ConstantLong, index, line);
        }
    }

    // ── Reading ─────────────────────────────────────────────────────────

    /// Read the byte at the given offset.
    pub fn read_byte(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    /// Read a u16 value (big-endian) starting at `offset`.
    pub fn read_u16(&self, offset: usize) -> u16 {
        let hi = self.code[offset] as u16;
        let lo = self.code[offset + 1] as u16;
        (hi << 8) | lo
    }

    /// Retrieve a constant by index.
    pub fn get_constant(&self, index: usize) -> &Object {
        &self.constants[index]
    }

    /// Get the source line number for the byte at `offset`.
    pub fn get_line(&self, offset: usize) -> usize {
        self.lines[offset]
    }

    // ── Accessors ───────────────────────────────────────────────────────

    /// Raw bytecode slice.
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// Number of bytes in the bytecode stream.
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// Whether the chunk contains no bytecode.
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    /// Number of constants in the pool.
    pub fn constants_count(&self) -> usize {
        self.constants.len()
    }

    // ── Patching (for back-patching jump offsets) ───────────────────────

    /// Overwrite the byte at `offset` with `byte`.
    pub fn patch_byte(&mut self, offset: usize, byte: u8) {
        self.code[offset] = byte;
    }

    /// Overwrite the u16 at `offset` (big-endian).
    pub fn patch_u16(&mut self, offset: usize, value: u16) {
        self.code[offset] = (value >> 8) as u8;
        self.code[offset + 1] = value as u8;
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
