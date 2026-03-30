// Bytecode Instruction Set for Metorex
//
// Each opcode is a single byte. Operands follow the opcode in the byte stream.
// Operand sizes are documented per-opcode.

/// Bytecode instruction opcodes.
///
/// Operand notation:
///   (u8)   — 1-byte operand (constant index, local slot, etc.)
///   (u16)  — 2-byte operand in big-endian (jump offset, wide constant index)
///   (none) — no operand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
    // ── Constants and literals ───────────────────────────────────────────
    /// Load a constant from the constant pool. Operand: (u8) index.
    Constant = 0,

    /// Load a constant using a wide index. Operand: (u16) index.
    ConstantLong = 1,

    /// Push `nil` onto the stack.
    Nil = 2,

    /// Push `true` onto the stack.
    True = 3,

    /// Push `false` onto the stack.
    False = 4,

    // ── Stack manipulation ──────────────────────────────────────────────
    /// Pop the top value from the stack.
    Pop = 10,

    // ── Local variables ─────────────────────────────────────────────────
    /// Read a local variable. Operand: (u8) stack slot.
    GetLocal = 20,

    /// Write to a local variable. Operand: (u8) stack slot.
    SetLocal = 21,

    // ── Global variables ────────────────────────────────────────────────
    /// Read a global variable. Operand: (u8) constant index (name).
    GetGlobal = 22,

    /// Write to a global variable. Operand: (u8) constant index (name).
    SetGlobal = 23,

    /// Define a new global variable. Operand: (u8) constant index (name).
    DefineGlobal = 24,

    // ── Instance variables ──────────────────────────────────────────────
    /// Read an instance variable. Operand: (u8) constant index (name).
    GetInstance = 25,

    /// Write to an instance variable. Operand: (u8) constant index (name).
    SetInstance = 26,

    // ── Upvalues (closures) ─────────────────────────────────────────────
    /// Read a captured upvalue. Operand: (u8) upvalue index.
    GetUpvalue = 27,

    /// Write to a captured upvalue. Operand: (u8) upvalue index.
    SetUpvalue = 28,

    /// Close an upvalue, moving it from stack to heap. (none)
    CloseUpvalue = 29,

    // ── Arithmetic ──────────────────────────────────────────────────────
    /// Pop two values, push their sum.
    Add = 30,

    /// Pop two values, push their difference.
    Subtract = 31,

    /// Pop two values, push their product.
    Multiply = 32,

    /// Pop two values, push their quotient.
    Divide = 33,

    /// Pop two values, push the remainder.
    Modulo = 34,

    /// Negate the top value on the stack (unary minus).
    Negate = 35,

    // ── Comparison ──────────────────────────────────────────────────────
    /// Pop two values, push `true` if equal.
    Equal = 40,

    /// Pop two values, push `true` if not equal.
    NotEqual = 41,

    /// Pop two values, push `true` if left > right.
    Greater = 42,

    /// Pop two values, push `true` if left < right.
    Less = 43,

    /// Pop two values, push `true` if left >= right.
    GreaterEqual = 44,

    /// Pop two values, push `true` if left <= right.
    LessEqual = 45,

    // ── Logical ─────────────────────────────────────────────────────────
    /// Logical NOT of the top value.
    Not = 50,

    // ── Control flow ────────────────────────────────────────────────────
    /// Unconditional forward jump. Operand: (u16) offset.
    Jump = 60,

    /// Jump if top of stack is falsy (nil or false). Operand: (u16) offset.
    JumpIfFalse = 61,

    /// Unconditional backward jump (loops). Operand: (u16) offset.
    Loop = 62,

    // ── Function / method calls ─────────────────────────────────────────
    /// Call a function/method. Operand: (u8) argument count.
    Call = 70,

    /// Optimized method invocation (receiver on stack). Operands: (u8) name index, (u8) arg count.
    Invoke = 71,

    /// Return from the current function. (none)
    Return = 72,

    // ── Closures ────────────────────────────────────────────────────────
    /// Create a closure. Operand: (u8) constant index (function).
    /// Followed by pairs of (u8 is_local, u8 index) for each upvalue.
    Closure = 80,

    // ── Classes and objects ─────────────────────────────────────────────
    /// Define a class. Operand: (u8) constant index (name).
    Class = 90,

    /// Add a method to the class on top of stack. Operand: (u8) constant index (name).
    Method = 91,

    // ── Collections ─────────────────────────────────────────────────────
    /// Create an array from the top N stack values. Operand: (u8) count.
    Array = 100,

    /// Create a hash from the top 2*N stack values. Operand: (u8) pair count.
    Hash = 101,

    /// Index read: pop index and collection, push result.
    IndexGet = 102,

    /// Index write: pop value, index, and collection; perform assignment.
    IndexSet = 103,

    // ── Exception handling ──────────────────────────────────────────────
    /// Begin a try block. Operand: (u16) catch jump offset.
    Try = 110,

    /// Catch an exception. (none)
    Catch = 111,

    /// Begin a finally block. (none)
    Finally = 112,

    /// End a try/catch/finally block. (none)
    EndTry = 113,

    /// Raise an exception (top of stack). (none)
    Raise = 114,

    // ── Pattern matching ────────────────────────────────────────────────
    /// Begin a match expression. (none)
    Match = 120,

    /// Test a pattern against the match subject. Operand: (u16) jump offset if no match.
    MatchPattern = 121,
}

impl OpCode {
    /// Convert a raw byte to an OpCode, if valid.
    pub fn from_byte(byte: u8) -> Option<Self> {
        // Safety: We check every valid discriminant
        match byte {
            0 => Some(Self::Constant),
            1 => Some(Self::ConstantLong),
            2 => Some(Self::Nil),
            3 => Some(Self::True),
            4 => Some(Self::False),
            10 => Some(Self::Pop),
            20 => Some(Self::GetLocal),
            21 => Some(Self::SetLocal),
            22 => Some(Self::GetGlobal),
            23 => Some(Self::SetGlobal),
            24 => Some(Self::DefineGlobal),
            25 => Some(Self::GetInstance),
            26 => Some(Self::SetInstance),
            27 => Some(Self::GetUpvalue),
            28 => Some(Self::SetUpvalue),
            29 => Some(Self::CloseUpvalue),
            30 => Some(Self::Add),
            31 => Some(Self::Subtract),
            32 => Some(Self::Multiply),
            33 => Some(Self::Divide),
            34 => Some(Self::Modulo),
            35 => Some(Self::Negate),
            40 => Some(Self::Equal),
            41 => Some(Self::NotEqual),
            42 => Some(Self::Greater),
            43 => Some(Self::Less),
            44 => Some(Self::GreaterEqual),
            45 => Some(Self::LessEqual),
            50 => Some(Self::Not),
            60 => Some(Self::Jump),
            61 => Some(Self::JumpIfFalse),
            62 => Some(Self::Loop),
            70 => Some(Self::Call),
            71 => Some(Self::Invoke),
            72 => Some(Self::Return),
            80 => Some(Self::Closure),
            90 => Some(Self::Class),
            91 => Some(Self::Method),
            100 => Some(Self::Array),
            101 => Some(Self::Hash),
            102 => Some(Self::IndexGet),
            103 => Some(Self::IndexSet),
            110 => Some(Self::Try),
            111 => Some(Self::Catch),
            112 => Some(Self::Finally),
            113 => Some(Self::EndTry),
            114 => Some(Self::Raise),
            120 => Some(Self::Match),
            121 => Some(Self::MatchPattern),
            _ => None,
        }
    }

    /// Convert this opcode to its byte representation.
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Human-readable name for this opcode (for disassembly / debugging).
    pub fn name(self) -> &'static str {
        match self {
            Self::Constant => "OP_CONSTANT",
            Self::ConstantLong => "OP_CONSTANT_LONG",
            Self::Nil => "OP_NIL",
            Self::True => "OP_TRUE",
            Self::False => "OP_FALSE",
            Self::Pop => "OP_POP",
            Self::GetLocal => "OP_GET_LOCAL",
            Self::SetLocal => "OP_SET_LOCAL",
            Self::GetGlobal => "OP_GET_GLOBAL",
            Self::SetGlobal => "OP_SET_GLOBAL",
            Self::DefineGlobal => "OP_DEFINE_GLOBAL",
            Self::GetInstance => "OP_GET_INSTANCE",
            Self::SetInstance => "OP_SET_INSTANCE",
            Self::GetUpvalue => "OP_GET_UPVALUE",
            Self::SetUpvalue => "OP_SET_UPVALUE",
            Self::CloseUpvalue => "OP_CLOSE_UPVALUE",
            Self::Add => "OP_ADD",
            Self::Subtract => "OP_SUBTRACT",
            Self::Multiply => "OP_MULTIPLY",
            Self::Divide => "OP_DIVIDE",
            Self::Modulo => "OP_MODULO",
            Self::Negate => "OP_NEGATE",
            Self::Equal => "OP_EQUAL",
            Self::NotEqual => "OP_NOT_EQUAL",
            Self::Greater => "OP_GREATER",
            Self::Less => "OP_LESS",
            Self::GreaterEqual => "OP_GREATER_EQUAL",
            Self::LessEqual => "OP_LESS_EQUAL",
            Self::Not => "OP_NOT",
            Self::Jump => "OP_JUMP",
            Self::JumpIfFalse => "OP_JUMP_IF_FALSE",
            Self::Loop => "OP_LOOP",
            Self::Call => "OP_CALL",
            Self::Invoke => "OP_INVOKE",
            Self::Return => "OP_RETURN",
            Self::Closure => "OP_CLOSURE",
            Self::Class => "OP_CLASS",
            Self::Method => "OP_METHOD",
            Self::Array => "OP_ARRAY",
            Self::Hash => "OP_HASH",
            Self::IndexGet => "OP_INDEX_GET",
            Self::IndexSet => "OP_INDEX_SET",
            Self::Try => "OP_TRY",
            Self::Catch => "OP_CATCH",
            Self::Finally => "OP_FINALLY",
            Self::EndTry => "OP_END_TRY",
            Self::Raise => "OP_RAISE",
            Self::Match => "OP_MATCH",
            Self::MatchPattern => "OP_MATCH_PATTERN",
        }
    }

    /// Number of operand bytes this instruction consumes after the opcode byte.
    pub fn operand_size(self) -> usize {
        match self {
            // No operands
            Self::Nil
            | Self::True
            | Self::False
            | Self::Pop
            | Self::CloseUpvalue
            | Self::Add
            | Self::Subtract
            | Self::Multiply
            | Self::Divide
            | Self::Modulo
            | Self::Negate
            | Self::Equal
            | Self::NotEqual
            | Self::Greater
            | Self::Less
            | Self::GreaterEqual
            | Self::LessEqual
            | Self::Not
            | Self::Return
            | Self::IndexGet
            | Self::IndexSet
            | Self::Catch
            | Self::Finally
            | Self::EndTry
            | Self::Raise
            | Self::Match => 0,

            // 1-byte operand
            Self::Constant
            | Self::GetLocal
            | Self::SetLocal
            | Self::GetGlobal
            | Self::SetGlobal
            | Self::DefineGlobal
            | Self::GetInstance
            | Self::SetInstance
            | Self::GetUpvalue
            | Self::SetUpvalue
            | Self::Call
            | Self::Closure
            | Self::Class
            | Self::Method
            | Self::Array
            | Self::Hash => 1,

            // 2-byte operand
            Self::ConstantLong
            | Self::Jump
            | Self::JumpIfFalse
            | Self::Loop
            | Self::Invoke
            | Self::Try
            | Self::MatchPattern => 2,
        }
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
