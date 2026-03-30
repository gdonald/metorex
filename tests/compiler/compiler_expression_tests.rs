// Tests for expression compilation (12.2)
//
// We parse source code into AST, then compile it and inspect the resulting
// bytecode to verify correct instruction sequences.

use metorex::bytecode::disassembler::disassemble;
use metorex::bytecode::opcode::OpCode;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;

/// Helper: parse source, compile, return the chunk.
fn compile(source: &str) -> metorex::bytecode::chunk::Chunk {
    let tokens = Lexer::new(source).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let compiler = Compiler::new();
    compiler.compile(&stmts).expect("compile failed")
}

/// Helper: get opcodes from a chunk (just the opcode bytes, skipping operands).
fn opcodes(chunk: &metorex::bytecode::chunk::Chunk) -> Vec<OpCode> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < chunk.len() {
        let byte = chunk.read_byte(offset);
        if let Some(op) = OpCode::from_byte(byte) {
            result.push(op);
            offset += 1 + op.operand_size();
        } else {
            offset += 1;
        }
    }
    result
}

// ── Literal compilation ─────────────────────────────────────────────────

#[test]
fn compile_int_literal() {
    let chunk = compile("42");
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Constant, OpCode::Pop, OpCode::Return]);
    assert_eq!(*chunk.get_constant(0), Object::Int(42));
}

#[test]
fn compile_float_literal() {
    let chunk = compile("3.14");
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Constant, OpCode::Pop, OpCode::Return]);
    assert_eq!(*chunk.get_constant(0), Object::Float(3.14));
}

#[test]
fn compile_string_literal() {
    let chunk = compile("\"hello\"");
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Constant, OpCode::Pop, OpCode::Return]);
}

#[test]
fn compile_true() {
    let chunk = compile("true");
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::True, OpCode::Pop, OpCode::Return]);
}

#[test]
fn compile_false() {
    let chunk = compile("false");
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::False, OpCode::Pop, OpCode::Return]);
}

#[test]
fn compile_nil() {
    let chunk = compile("nil");
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Nil, OpCode::Pop, OpCode::Return]);
}

// ── Binary operations ───────────────────────────────────────────────────

#[test]
fn compile_addition() {
    let chunk = compile("1 + 2");
    let ops = opcodes(&chunk);
    assert_eq!(
        ops,
        vec![
            OpCode::Constant,
            OpCode::Constant,
            OpCode::Add,
            OpCode::Pop,
            OpCode::Return
        ]
    );
}

#[test]
fn compile_subtraction() {
    let chunk = compile("5 - 3");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Subtract));
}

#[test]
fn compile_multiplication() {
    let chunk = compile("4 * 7");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Multiply));
}

#[test]
fn compile_division() {
    let chunk = compile("10 / 2");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Divide));
}

#[test]
fn compile_modulo() {
    let chunk = compile("10 % 3");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Modulo));
}

#[test]
fn compile_comparison_equal() {
    let chunk = compile("1 == 2");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Equal));
}

#[test]
fn compile_comparison_not_equal() {
    let chunk = compile("1 != 2");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::NotEqual));
}

#[test]
fn compile_comparison_less() {
    let chunk = compile("1 < 2");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Less));
}

#[test]
fn compile_comparison_greater() {
    let chunk = compile("1 > 2");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Greater));
}

#[test]
fn compile_comparison_less_equal() {
    let chunk = compile("1 <= 2");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::LessEqual));
}

#[test]
fn compile_comparison_greater_equal() {
    let chunk = compile("1 >= 2");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::GreaterEqual));
}

// ── Unary operations ────────────────────────────────────────────────────

#[test]
fn compile_negate() {
    let chunk = compile("-5");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Negate));
}

#[test]
fn compile_not() {
    let chunk = compile("!true");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Not));
}

#[test]
fn compile_unary_plus_is_noop() {
    let chunk = compile("+5");
    let ops = opcodes(&chunk);
    // Unary plus should not emit Negate
    assert!(!ops.contains(&OpCode::Negate));
    assert!(ops.contains(&OpCode::Constant));
}

// ── Logical short-circuit ───────────────────────────────────────────────

#[test]
fn compile_and_uses_jump() {
    let chunk = compile("true && false");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::JumpIfFalse));
}

#[test]
fn compile_or_uses_jumps() {
    let chunk = compile("true || false");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::JumpIfFalse));
    assert!(ops.contains(&OpCode::Jump));
}

// ── Variable access ─────────────────────────────────────────────────────

#[test]
fn compile_global_variable_access() {
    let chunk = compile("x");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::GetGlobal));
}

// ── Assignment ──────────────────────────────────────────────────────────

#[test]
fn compile_global_assignment() {
    let chunk = compile("x = 42");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Constant));
    assert!(ops.contains(&OpCode::DefineGlobal));
}

// ── Array literal ───────────────────────────────────────────────────────

#[test]
fn compile_array_literal() {
    let chunk = compile("[1, 2, 3]");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Array));
    // Should have 3 constants + Array(3) + Pop + Return
}

#[test]
fn compile_empty_array() {
    let chunk = compile("[]");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Array));
}

// ── Hash literal ────────────────────────────────────────────────────────

#[test]
fn compile_hash_literal() {
    let chunk = compile("{\"a\" => 1, \"b\" => 2}");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Hash));
}

// ── Index access ────────────────────────────────────────────────────────

#[test]
fn compile_index_access() {
    let chunk = compile("x[0]");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::IndexGet));
}

// ── Method call ─────────────────────────────────────────────────────────

#[test]
fn compile_method_call() {
    let chunk = compile("x.length");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Invoke));
}

#[test]
fn compile_method_call_with_args() {
    let chunk = compile("x.push(1)");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Invoke));
}

// ── Bare function call ──────────────────────────────────────────────────

#[test]
fn compile_function_call() {
    let chunk = compile("puts(42)");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Call));
}

// ── Grouped expression ──────────────────────────────────────────────────

#[test]
fn compile_grouped_expression() {
    let chunk = compile("(1 + 2) * 3");
    let ops = opcodes(&chunk);
    // Should produce: Const(1), Const(2), Add, Const(3), Multiply
    assert!(ops.contains(&OpCode::Add));
    assert!(ops.contains(&OpCode::Multiply));
}

// ── Complex expressions ─────────────────────────────────────────────────

#[test]
fn compile_nested_arithmetic() {
    let chunk = compile("(1 + 2) * (3 - 4)");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Add));
    assert!(ops.contains(&OpCode::Subtract));
    assert!(ops.contains(&OpCode::Multiply));
}

// ── Disassembly sanity check ────────────────────────────────────────────

#[test]
fn compiled_chunk_disassembles() {
    let chunk = compile("x = 1 + 2");
    let output = disassemble(&chunk, "x = 1 + 2");
    assert!(output.contains("OP_CONSTANT"));
    assert!(output.contains("OP_ADD"));
    assert!(output.contains("OP_DEFINE_GLOBAL"));
    assert!(output.contains("OP_RETURN"));
}

// ── Return statement ────────────────────────────────────────────────────

#[test]
fn compile_return_with_value() {
    let chunk = compile("return 42");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Constant));
    assert!(ops.contains(&OpCode::Return));
}
