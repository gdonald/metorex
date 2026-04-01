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

// ── compile_optimized() entry point (mod.rs line 226) ──────────────────

#[test]
fn compile_optimized_empty_program() {
    let compiler = Compiler::new();
    let chunk = compiler
        .compile_optimized(&[])
        .expect("compile_optimized failed");
    assert_eq!(chunk.len(), 1);
    assert_eq!(OpCode::from_byte(chunk.read_byte(0)), Some(OpCode::Return));
}

#[test]
fn compile_optimized_folds_constants() {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;

    let tokens = Lexer::new("1 + 2").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let compiler = Compiler::new();
    let chunk = compiler
        .compile_optimized(&stmts)
        .expect("compile_optimized failed");
    // The Add should be folded away
    let mut has_add = false;
    let mut offset = 0;
    while offset < chunk.len() {
        let byte = chunk.read_byte(offset);
        if let Some(op) = OpCode::from_byte(byte) {
            if op == OpCode::Add {
                has_add = true;
            }
            offset += 1 + op.operand_size();
        } else {
            offset += 1;
        }
    }
    assert!(!has_add, "Add should be folded by compile_optimized");
}
