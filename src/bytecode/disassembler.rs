// Bytecode Disassembler
//
// Produces human-readable output from a Chunk, showing each instruction
// with its offset, line number, opcode name, and operand values.

use super::chunk::Chunk;
use super::opcode::OpCode;

/// Disassemble an entire chunk, returning the formatted output.
pub fn disassemble(chunk: &Chunk, name: &str) -> String {
    let mut output = String::new();
    output.push_str(&format!("== {} ==\n", name));

    let mut offset = 0;
    while offset < chunk.len() {
        let (text, next) = disassemble_instruction(chunk, offset);
        output.push_str(&text);
        output.push('\n');
        offset = next;
    }

    output
}

/// Disassemble a single instruction at the given offset.
///
/// Returns `(formatted_line, next_offset)`.
pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> (String, usize) {
    let byte = chunk.read_byte(offset);
    let line = chunk.get_line(offset);

    // Line number column: show the line or "|" if same as previous
    let line_str = if offset > 0 && line == chunk.get_line(offset - 1) {
        "   |".to_string()
    } else {
        format!("{:4}", line)
    };

    let Some(op) = OpCode::from_byte(byte) else {
        return (
            format!("{:04} {} Unknown opcode {}", offset, line_str, byte),
            offset + 1,
        );
    };

    match op.operand_size() {
        0 => simple_instruction(op, offset, &line_str),
        1 => byte_instruction(chunk, op, offset, &line_str),
        2 => word_instruction(chunk, op, offset, &line_str),
        _ => (
            format!("{:04} {} {}", offset, line_str, op.name()),
            offset + 1,
        ),
    }
}

/// Format a simple instruction (no operands).
fn simple_instruction(op: OpCode, offset: usize, line_str: &str) -> (String, usize) {
    (
        format!("{:04} {} {}", offset, line_str, op.name()),
        offset + 1,
    )
}

/// Format an instruction with a single u8 operand.
fn byte_instruction(chunk: &Chunk, op: OpCode, offset: usize, line_str: &str) -> (String, usize) {
    let operand = chunk.read_byte(offset + 1);

    let detail = match op {
        // For constant-referencing instructions, show the constant value
        OpCode::Constant
        | OpCode::DefineGlobal
        | OpCode::GetGlobal
        | OpCode::SetGlobal
        | OpCode::GetInstance
        | OpCode::SetInstance
        | OpCode::Class
        | OpCode::Method
        | OpCode::Closure => {
            let constant = chunk.get_constant(operand as usize);
            format!("{:4} '{}'", operand, format_constant(constant))
        }
        // For local/upvalue slots and counts, just show the number
        _ => format!("{:4}", operand),
    };

    (
        format!("{:04} {} {:20} {}", offset, line_str, op.name(), detail),
        offset + 2,
    )
}

/// Format an instruction with a u16 operand.
fn word_instruction(chunk: &Chunk, op: OpCode, offset: usize, line_str: &str) -> (String, usize) {
    let operand = chunk.read_u16(offset + 1);

    let detail = match op {
        OpCode::ConstantLong => {
            let constant = chunk.get_constant(operand as usize);
            format!("{:4} '{}'", operand, format_constant(constant))
        }
        OpCode::Invoke => {
            // Invoke has name_index (first byte) and arg_count (second byte)
            let name_idx = chunk.read_byte(offset + 1);
            let arg_count = chunk.read_byte(offset + 2);
            let constant = chunk.get_constant(name_idx as usize);
            format!("({} args) '{}'", arg_count, format_constant(constant))
        }
        // Jump offsets
        _ => format!("{:4}", operand),
    };

    (
        format!("{:04} {} {:20} {}", offset, line_str, op.name(), detail),
        offset + 3,
    )
}

/// Format a constant value for display in disassembly.
fn format_constant(obj: &crate::object::Object) -> String {
    use crate::object::Object;
    match obj {
        Object::Int(i) => i.to_string(),
        Object::Float(f) => f.to_string(),
        Object::String(s) => format!("\"{}\"", s),
        Object::Symbol(s) => format!(":{}", s),
        Object::Bool(b) => b.to_string(),
        Object::Nil => "nil".to_string(),
        _ => format!("{:?}", obj),
    }
}
