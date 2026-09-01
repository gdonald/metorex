// Tests for the Bytecode Disassembler

use metorex::bytecode::chunk::Chunk;
use metorex::bytecode::disassembler::{disassemble, disassemble_instruction};
use metorex::bytecode::opcode::OpCode;
use metorex::object::Object;
use std::rc::Rc;

// ── disassemble: full chunk ─────────────────────────────────────────────

#[test]
fn disassemble_empty_chunk() {
    let chunk = Chunk::new();
    let output = disassemble(&chunk, "empty");
    assert_eq!(output, "== empty ==\n");
}

#[test]
fn disassemble_simple_return() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Return, 1);
    let output = disassemble(&chunk, "test");
    assert!(output.contains("== test =="));
    assert!(output.contains("OP_RETURN"));
    assert!(output.contains("0000"));
    assert!(output.contains("1")); // line number
}

#[test]
fn disassemble_constant_instruction() {
    let mut chunk = Chunk::new();
    let idx = chunk.add_constant(Object::Int(42)).unwrap();
    chunk.write_constant(idx, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let output = disassemble(&chunk, "const");
    assert!(output.contains("OP_CONSTANT"));
    assert!(output.contains("42"));
    assert!(output.contains("OP_RETURN"));
}

#[test]
fn disassemble_constant_long_instruction() {
    let mut chunk = Chunk::new();
    // Fill constants to force a long constant
    for i in 0..256 {
        chunk.add_constant(Object::Int(i));
    }
    let idx = chunk.add_constant(Object::Int(999)).unwrap();
    chunk.write_constant(idx, 5);

    let output = disassemble(&chunk, "long");
    assert!(output.contains("OP_CONSTANT_LONG"));
    assert!(output.contains("999"));
}

#[test]
fn disassemble_string_constant() {
    let mut chunk = Chunk::new();
    let idx = chunk
        .add_constant(Object::String(Rc::new("hello".to_string())))
        .unwrap();
    chunk.write_op_u8(OpCode::Constant, idx as u8, 1);

    let output = disassemble(&chunk, "str");
    assert!(output.contains("OP_CONSTANT"));
    assert!(output.contains("\"hello\""));
}

// ── Line number display ─────────────────────────────────────────────────

#[test]
fn same_line_shows_pipe() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Add, 2);

    let output = disassemble(&chunk, "lines");
    let lines: Vec<&str> = output.lines().collect();
    // First instruction shows line 1
    assert!(lines[1].contains("   1"));
    // Second instruction same line shows "|"
    assert!(lines[2].contains("   |"));
    // Third instruction shows line 2
    assert!(lines[3].contains("   2"));
}

// ── Offset tracking ─────────────────────────────────────────────────────

#[test]
fn offsets_are_correct() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Nil, 1); // offset 0, 1 byte
    chunk.write_op_u8(OpCode::GetLocal, 3, 1); // offset 1, 2 bytes
    chunk.write_opcode(OpCode::Return, 1); // offset 3, 1 byte

    let output = disassemble(&chunk, "offsets");
    assert!(output.contains("0000"));
    assert!(output.contains("0001"));
    assert!(output.contains("0003"));
}

// ── Operand display ─────────────────────────────────────────────────────

#[test]
fn byte_operand_shows_slot() {
    let mut chunk = Chunk::new();
    chunk.write_op_u8(OpCode::GetLocal, 5, 1);
    chunk.write_op_u8(OpCode::SetLocal, 7, 1);

    let output = disassemble(&chunk, "locals");
    assert!(output.contains("OP_GET_LOCAL"));
    assert!(output.contains("5"));
    assert!(output.contains("OP_SET_LOCAL"));
    assert!(output.contains("7"));
}

#[test]
fn jump_operand_shows_offset() {
    let mut chunk = Chunk::new();
    chunk.write_op_u16(OpCode::Jump, 42, 1);
    chunk.write_op_u16(OpCode::JumpIfFalse, 100, 2);
    chunk.write_op_u16(OpCode::Loop, 10, 3);

    let output = disassemble(&chunk, "jumps");
    assert!(output.contains("OP_JUMP"));
    assert!(output.contains("42"));
    assert!(output.contains("OP_JUMP_IF_FALSE"));
    assert!(output.contains("100"));
    assert!(output.contains("OP_LOOP"));
    assert!(output.contains("10"));
}

#[test]
fn global_operand_shows_name() {
    let mut chunk = Chunk::new();
    let idx = chunk
        .add_constant(Object::String(Rc::new("my_var".to_string())))
        .unwrap();
    chunk.write_op_u8(OpCode::DefineGlobal, idx as u8, 1);
    chunk.write_op_u8(OpCode::GetGlobal, idx as u8, 2);

    let output = disassemble(&chunk, "globals");
    assert!(output.contains("OP_DEFINE_GLOBAL"));
    assert!(output.contains("\"my_var\""));
    assert!(output.contains("OP_GET_GLOBAL"));
}

#[test]
fn invoke_shows_args_and_name() {
    let mut chunk = Chunk::new();
    let idx = chunk
        .add_constant(Object::String(Rc::new("greet".to_string())))
        .unwrap();
    // Invoke: first byte = name index, second byte = arg count
    chunk.write_op_u16(OpCode::Invoke, (idx << 8) | 2, 1);

    let output = disassemble(&chunk, "invoke");
    assert!(output.contains("OP_INVOKE"));
    assert!(output.contains("\"greet\""));
    assert!(output.contains("2 args"));
}

// ── Unknown opcode ──────────────────────────────────────────────────────

#[test]
fn unknown_opcode_shows_byte_value() {
    let mut chunk = Chunk::new();
    chunk.write_byte(255, 1);

    let output = disassemble(&chunk, "unknown");
    assert!(output.contains("Unknown opcode 255"));
}

// ── disassemble_instruction: single instruction ─────────────────────────

#[test]
fn disassemble_instruction_returns_next_offset() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_op_u8(OpCode::GetLocal, 0, 1);
    chunk.write_op_u16(OpCode::Jump, 10, 2);

    let (_, next) = disassemble_instruction(&chunk, 0);
    assert_eq!(next, 1); // Nil is 1 byte

    let (_, next) = disassemble_instruction(&chunk, 1);
    assert_eq!(next, 3); // GetLocal is 2 bytes

    let (_, next) = disassemble_instruction(&chunk, 3);
    assert_eq!(next, 6); // Jump is 3 bytes
}

// ── Realistic program ───────────────────────────────────────────────────

#[test]
fn disassemble_realistic_program() {
    // Simulate: x = 1 + 2
    let mut chunk = Chunk::new();
    let c1 = chunk.add_constant(Object::Int(1)).unwrap();
    let c2 = chunk.add_constant(Object::Int(2)).unwrap();
    let name = chunk
        .add_constant(Object::String(Rc::new("x".to_string())))
        .unwrap();

    chunk.write_constant(c1, 1);
    chunk.write_constant(c2, 1);
    chunk.write_opcode(OpCode::Add, 1);
    chunk.write_op_u8(OpCode::DefineGlobal, name as u8, 1);
    chunk.write_opcode(OpCode::Return, 2);

    let output = disassemble(&chunk, "x = 1 + 2");
    assert!(output.contains("== x = 1 + 2 =="));
    assert!(output.contains("OP_CONSTANT"));
    assert!(output.contains("'1'"));
    assert!(output.contains("'2'"));
    assert!(output.contains("OP_ADD"));
    assert!(output.contains("OP_DEFINE_GLOBAL"));
    assert!(output.contains("\"x\""));
    assert!(output.contains("OP_RETURN"));
}

// ── All simple opcodes disassemble ──────────────────────────────────────

#[test]
fn all_simple_opcodes_disassemble() {
    let simple_ops = [
        OpCode::Nil,
        OpCode::True,
        OpCode::False,
        OpCode::Pop,
        OpCode::CloseUpvalue,
        OpCode::Add,
        OpCode::Subtract,
        OpCode::Multiply,
        OpCode::Divide,
        OpCode::Modulo,
        OpCode::Negate,
        OpCode::Equal,
        OpCode::NotEqual,
        OpCode::Greater,
        OpCode::Less,
        OpCode::GreaterEqual,
        OpCode::LessEqual,
        OpCode::Not,
        OpCode::Return,
        OpCode::IndexGet,
        OpCode::IndexSet,
        OpCode::Catch,
        OpCode::Finally,
        OpCode::EndTry,
        OpCode::Raise,
        OpCode::Match,
    ];

    for op in simple_ops {
        let mut chunk = Chunk::new();
        chunk.write_opcode(op, 1);
        let output = disassemble(&chunk, "test");
        assert!(
            output.contains(op.name()),
            "Missing {} in output: {}",
            op.name(),
            output
        );
    }
}

// ── format_constant for various types (disassembler.rs lines 124-129) ──────

#[test]
fn disassemble_symbol_constant() {
    let mut chunk = Chunk::new();
    let idx = chunk.add_constant(metorex::object::Object::Symbol(std::rc::Rc::new(
        "hello".to_string(),
    )));
    chunk.write_constant(idx.unwrap(), 1);
    let output = disassemble(&chunk, "symbol_test");
    assert!(
        output.contains(":hello") || output.contains("hello"),
        "Output: {}",
        output
    );
}

#[test]
fn disassemble_bool_constant() {
    let mut chunk = Chunk::new();
    let idx = chunk.add_constant(metorex::object::Object::Bool(true));
    chunk.write_constant(idx.unwrap(), 1);
    let output = disassemble(&chunk, "bool_test");
    assert!(output.contains("true"), "Output: {}", output);
}

#[test]
fn disassemble_nil_constant() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Nil, 1);
    let output = disassemble(&chunk, "nil_test");
    assert!(output.contains("OP_NIL"), "Output: {}", output);
}

#[test]
fn disassemble_float_constant() {
    let mut chunk = Chunk::new();
    let idx = chunk.add_constant(metorex::object::Object::Float(3.14));
    chunk.write_constant(idx.unwrap(), 1);
    let output = disassemble(&chunk, "float_test");
    assert!(output.contains("3.14"), "Output: {}", output);
}
