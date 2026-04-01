// Tests for the Chunk bytecode container

use metorex::bytecode::chunk::Chunk;
use metorex::bytecode::opcode::OpCode;
use metorex::object::Object;
use std::rc::Rc;

// ── Construction ────────────────────────────────────────────────────────

#[test]
fn new_chunk_is_empty() {
    let chunk = Chunk::new();
    assert!(chunk.is_empty());
    assert_eq!(chunk.len(), 0);
    assert_eq!(chunk.constants_count(), 0);
}

#[test]
fn default_chunk_is_empty() {
    let chunk = Chunk::default();
    assert!(chunk.is_empty());
}

// ── write_byte / write_opcode ───────────────────────────────────────────

#[test]
fn write_byte_appends() {
    let mut chunk = Chunk::new();
    chunk.write_byte(42, 1);
    assert_eq!(chunk.len(), 1);
    assert_eq!(chunk.read_byte(0), 42);
    assert_eq!(chunk.get_line(0), 1);
}

#[test]
fn write_opcode_appends() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Return, 5);
    assert_eq!(chunk.len(), 1);
    assert_eq!(chunk.read_byte(0), OpCode::Return.to_byte());
    assert_eq!(chunk.get_line(0), 5);
}

#[test]
fn write_op_u8_appends_two_bytes() {
    let mut chunk = Chunk::new();
    chunk.write_op_u8(OpCode::Constant, 7, 10);
    assert_eq!(chunk.len(), 2);
    assert_eq!(
        OpCode::from_byte(chunk.read_byte(0)),
        Some(OpCode::Constant)
    );
    assert_eq!(chunk.read_byte(1), 7);
    assert_eq!(chunk.get_line(0), 10);
    assert_eq!(chunk.get_line(1), 10);
}

#[test]
fn write_op_u16_appends_three_bytes() {
    let mut chunk = Chunk::new();
    chunk.write_op_u16(OpCode::Jump, 0x0102, 20);
    assert_eq!(chunk.len(), 3);
    assert_eq!(OpCode::from_byte(chunk.read_byte(0)), Some(OpCode::Jump));
    assert_eq!(chunk.read_byte(1), 0x01);
    assert_eq!(chunk.read_byte(2), 0x02);
    assert_eq!(chunk.read_u16(1), 0x0102);
}

// ── read_u16 ────────────────────────────────────────────────────────────

#[test]
fn read_u16_big_endian() {
    let mut chunk = Chunk::new();
    chunk.write_byte(0xFF, 1);
    chunk.write_byte(0x00, 1);
    assert_eq!(chunk.read_u16(0), 0xFF00);

    let mut chunk2 = Chunk::new();
    chunk2.write_byte(0x00, 1);
    chunk2.write_byte(0xFF, 1);
    assert_eq!(chunk2.read_u16(0), 0x00FF);
}

// ── Constants ───────────────────────────────────────────────────────────

#[test]
fn add_constant_returns_index() {
    let mut chunk = Chunk::new();
    let idx0 = chunk.add_constant(Object::Int(42));
    let idx1 = chunk.add_constant(Object::Float(3.14));
    let idx2 = chunk.add_constant(Object::String(Rc::new("hello".to_string())));

    assert_eq!(idx0, Some(0));
    assert_eq!(idx1, Some(1));
    assert_eq!(idx2, Some(2));
    assert_eq!(chunk.constants_count(), 3);
}

#[test]
fn get_constant_retrieves_by_index() {
    let mut chunk = Chunk::new();
    chunk.add_constant(Object::Int(42));
    chunk.add_constant(Object::Bool(true));

    assert_eq!(*chunk.get_constant(0), Object::Int(42));
    assert_eq!(*chunk.get_constant(1), Object::Bool(true));
}

#[test]
fn write_constant_uses_short_form_for_small_index() {
    let mut chunk = Chunk::new();
    chunk.write_constant(5, 1);
    assert_eq!(chunk.len(), 2); // OP_CONSTANT + 1 byte
    assert_eq!(
        OpCode::from_byte(chunk.read_byte(0)),
        Some(OpCode::Constant)
    );
    assert_eq!(chunk.read_byte(1), 5);
}

#[test]
fn write_constant_uses_long_form_for_large_index() {
    let mut chunk = Chunk::new();
    chunk.write_constant(256, 1);
    assert_eq!(chunk.len(), 3); // OP_CONSTANT_LONG + 2 bytes
    assert_eq!(
        OpCode::from_byte(chunk.read_byte(0)),
        Some(OpCode::ConstantLong)
    );
    assert_eq!(chunk.read_u16(1), 256);
}

#[test]
fn write_constant_boundary_at_255() {
    let mut chunk = Chunk::new();
    // 255 fits in u8 — should use short form
    chunk.write_constant(255, 1);
    assert_eq!(chunk.len(), 2);
    assert_eq!(
        OpCode::from_byte(chunk.read_byte(0)),
        Some(OpCode::Constant)
    );
    assert_eq!(chunk.read_byte(1), 255);
}

// ── Patching ────────────────────────────────────────────────────────────

#[test]
fn patch_byte_overwrites() {
    let mut chunk = Chunk::new();
    chunk.write_byte(0x00, 1);
    chunk.write_byte(0x00, 1);
    chunk.patch_byte(0, 0xFF);
    assert_eq!(chunk.read_byte(0), 0xFF);
    assert_eq!(chunk.read_byte(1), 0x00);
}

#[test]
fn patch_u16_overwrites_big_endian() {
    let mut chunk = Chunk::new();
    chunk.write_byte(0x00, 1);
    chunk.write_byte(0x00, 1);
    chunk.patch_u16(0, 0xABCD);
    assert_eq!(chunk.read_byte(0), 0xAB);
    assert_eq!(chunk.read_byte(1), 0xCD);
    assert_eq!(chunk.read_u16(0), 0xABCD);
}

// ── Line tracking ───────────────────────────────────────────────────────

#[test]
fn lines_tracked_per_byte() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Add, 2);
    chunk.write_opcode(OpCode::Return, 3);

    assert_eq!(chunk.get_line(0), 1);
    assert_eq!(chunk.get_line(1), 1);
    assert_eq!(chunk.get_line(2), 2);
    assert_eq!(chunk.get_line(3), 3);
}

// ── code() accessor ─────────────────────────────────────────────────────

#[test]
fn code_returns_raw_bytes() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Return, 1);
    let code = chunk.code();
    assert_eq!(code.len(), 2);
    assert_eq!(code[0], OpCode::Nil.to_byte());
    assert_eq!(code[1], OpCode::Return.to_byte());
}

// ── Realistic instruction sequence ──────────────────────────────────────

#[test]
fn realistic_instruction_sequence() {
    // Simulate: x = 1 + 2
    let mut chunk = Chunk::new();

    let c1 = chunk.add_constant(Object::Int(1)).unwrap();
    let c2 = chunk.add_constant(Object::Int(2)).unwrap();
    let name = chunk
        .add_constant(Object::String(Rc::new("x".to_string())))
        .unwrap();

    // Load 1
    chunk.write_constant(c1, 1);
    // Load 2
    chunk.write_constant(c2, 1);
    // Add
    chunk.write_opcode(OpCode::Add, 1);
    // Define global "x"
    chunk.write_op_u8(OpCode::DefineGlobal, name as u8, 1);
    // Return
    chunk.write_opcode(OpCode::Return, 1);

    assert_eq!(chunk.constants_count(), 3);
    assert_eq!(chunk.len(), 8); // 2 + 2 + 1 + 2 + 1
    assert_eq!(*chunk.get_constant(0), Object::Int(1));
    assert_eq!(*chunk.get_constant(1), Object::Int(2));
}

#[test]
fn jump_patch_workflow() {
    // Simulate: if condition then ... end
    let mut chunk = Chunk::new();

    // Emit condition (placeholder)
    chunk.write_opcode(OpCode::True, 1);

    // Emit jump_if_false with placeholder offset
    chunk.write_op_u16(OpCode::JumpIfFalse, 0xFFFF, 1);
    let jump_offset = chunk.len() - 2; // offset of the u16

    // Emit "then" body
    chunk.write_opcode(OpCode::Nil, 2);
    chunk.write_opcode(OpCode::Pop, 2);

    // Patch the jump to point here
    let target = chunk.len() - (jump_offset + 2);
    chunk.patch_u16(jump_offset, target as u16);

    // Verify the patched offset
    assert_eq!(chunk.read_u16(jump_offset), 2); // skip 2 bytes (Nil + Pop)
}

// ── add_constant many constants (chunk.rs line 67) ─────────────────────────

#[test]
fn chunk_add_many_constants() {
    let mut chunk = Chunk::new();
    for i in 0..100 {
        let idx = chunk.add_constant(metorex::object::Object::Int(i));
        assert!(idx.is_some());
        assert_eq!(idx.unwrap(), i as u16);
    }
}
