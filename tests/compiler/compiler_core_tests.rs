// Tests for Compiler core structure (12.1)

use metorex::bytecode::opcode::OpCode;
use metorex::compiler::Compiler;

// ── Construction ────────────────────────────────────────────────────────

#[test]
fn new_compiler_creates_empty_chunk() {
    let compiler = Compiler::new();
    assert!(compiler.current_chunk().is_empty());
    assert_eq!(compiler.current_chunk().constants_count(), 0);
}

#[test]
fn default_compiler() {
    let compiler = Compiler::default();
    assert!(compiler.current_chunk().is_empty());
}

// ── compile() entry point ───────────────────────────────────────────────

#[test]
fn compile_empty_program() {
    let compiler = Compiler::new();
    let chunk = compiler.compile(&[]).expect("compile failed");
    // Should emit OP_RETURN at the end
    assert_eq!(chunk.len(), 1);
    assert_eq!(OpCode::from_byte(chunk.read_byte(0)), Some(OpCode::Return));
}
