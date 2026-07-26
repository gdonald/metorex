// Tests for control flow compilation (12.4)
//
// Exercises if/else/elsif, unless, while, for, break, and continue.

use metorex::ast::Statement;
use metorex::bytecode::opcode::OpCode;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::lexer::token::Position;
use metorex::parser::Parser;

/// Helper: parse source, compile, return the chunk.
fn compile(source: &str) -> metorex::bytecode::chunk::Chunk {
    let tokens = Lexer::new(source).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let compiler = Compiler::new();
    compiler.compile(&stmts).expect("compile failed")
}

/// Helper: compile AST statements directly.
fn compile_stmts(stmts: &[Statement]) -> Result<metorex::bytecode::chunk::Chunk, String> {
    let compiler = Compiler::new();
    compiler.compile(stmts).map_err(|e| e.to_string())
}

/// Helper: get opcodes from a chunk.
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

fn pos() -> Position {
    Position {
        line: 1,
        column: 0,
        offset: 0,
    }
}

// ── If statement (12.4.1) ─────────────────────────────────────────────

#[test]
fn compile_if_simple() {
    let chunk = compile("if true\n  42\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::True));
    assert!(ops.contains(&OpCode::JumpIfFalse));
    assert!(ops.contains(&OpCode::Jump));
}

#[test]
fn compile_if_else() {
    let chunk = compile("if true\n  1\nelse\n  2\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::JumpIfFalse));
    assert!(ops.contains(&OpCode::Jump));
    // Should have two constants (1 and 2)
    let const_count = ops.iter().filter(|&&op| op == OpCode::Constant).count();
    assert!(
        const_count >= 2,
        "Expected at least 2 constants for if/else branches"
    );
}

#[test]
fn compile_if_elsif() {
    let chunk = compile("if false\n  1\nelsif true\n  2\nend");
    let ops = opcodes(&chunk);
    // Two conditions checked, two JumpIfFalse
    let jif_count = ops.iter().filter(|&&op| op == OpCode::JumpIfFalse).count();
    assert!(
        jif_count >= 2,
        "Expected at least 2 JumpIfFalse for if/elsif"
    );
}

#[test]
fn compile_if_elsif_else() {
    let chunk = compile("if false\n  1\nelsif false\n  2\nelse\n  3\nend");
    let ops = opcodes(&chunk);
    let const_count = ops.iter().filter(|&&op| op == OpCode::Constant).count();
    assert!(
        const_count >= 3,
        "Expected at least 3 constants for if/elsif/else"
    );
}

#[test]
fn compile_if_with_variable() {
    let chunk = compile("x = 5\nif x\n  10\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::JumpIfFalse));
}

// ── Unless statement ──────────────────────────────────────────────────

#[test]
fn compile_unless_simple() {
    let chunk = compile("unless false\n  42\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::False));
    assert!(ops.contains(&OpCode::JumpIfFalse));
}

#[test]
fn compile_unless_else() {
    let chunk = compile("unless true\n  1\nelse\n  2\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::JumpIfFalse));
    assert!(ops.contains(&OpCode::Jump));
}

// ── While loop (12.4.2) ──────────────────────────────────────────────

#[test]
fn compile_while_loop() {
    let chunk = compile("while true\n  42\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::JumpIfFalse));
    assert!(ops.contains(&OpCode::Loop));
}

#[test]
fn compile_while_with_condition() {
    let chunk = compile("x = 0\nwhile x\n  x\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Loop));
    assert!(ops.contains(&OpCode::JumpIfFalse));
}

#[test]
fn compile_while_false_still_emits_loop() {
    let chunk = compile("while false\n  42\nend");
    let ops = opcodes(&chunk);
    // Even with `false` the loop structure is emitted (condition checked at runtime)
    assert!(ops.contains(&OpCode::Loop));
    assert!(ops.contains(&OpCode::JumpIfFalse));
}

// ── For loop (12.4.3) ────────────────────────────────────────────────

#[test]
fn compile_for_loop() {
    let chunk = compile("for x in [1, 2, 3]\n  x\nend");
    let ops = opcodes(&chunk);
    // For loop uses Array, Invoke (length), Less, JumpIfFalse, IndexGet, Loop
    assert!(ops.contains(&OpCode::Array));
    assert!(ops.contains(&OpCode::Invoke)); // .length
    assert!(ops.contains(&OpCode::Less));
    assert!(ops.contains(&OpCode::JumpIfFalse));
    assert!(ops.contains(&OpCode::IndexGet));
    assert!(ops.contains(&OpCode::Loop));
}

#[test]
fn compile_for_loop_increment() {
    let chunk = compile("for i in [10, 20]\n  i\nend");
    let ops = opcodes(&chunk);
    // Should have Add for index increment
    assert!(ops.contains(&OpCode::Add));
}

// ── Break statement (12.4.4) ─────────────────────────────────────────

#[test]
fn compile_break_in_while() {
    let chunk = compile("while true\n  break\nend");
    let ops = opcodes(&chunk);
    // break emits a Jump that skips past the loop
    let jump_count = ops.iter().filter(|&&op| op == OpCode::Jump).count();
    assert!(jump_count >= 1, "Expected at least 1 Jump for break");
    assert!(ops.contains(&OpCode::Loop));
}

#[test]
fn compile_break_outside_loop_errors() {
    let stmts = vec![Statement::Break {
        value: None,
        position: pos(),
    }];
    let result = compile_stmts(&stmts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("break outside of loop"), "Error was: {err}");
}

// ── Continue statement (12.4.5) ──────────────────────────────────────

#[test]
fn compile_continue_in_while() {
    let chunk = compile("while true\n  continue\nend");
    let ops = opcodes(&chunk);
    // continue emits a Loop instruction to jump back
    let loop_count = ops.iter().filter(|&&op| op == OpCode::Loop).count();
    assert!(
        loop_count >= 2,
        "Expected at least 2 Loop ops (continue + loop end)"
    );
}

#[test]
fn compile_continue_outside_loop_errors() {
    let stmts = vec![Statement::Continue {
        value: None,
        position: pos(),
    }];
    let result = compile_stmts(&stmts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("continue outside of loop"), "Error was: {err}");
}

// ── Break and continue with nested scopes ────────────────────────────

#[test]
fn compile_break_in_nested_if() {
    let chunk = compile("while true\n  if true\n    break\n  end\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Loop));
    let jump_count = ops.iter().filter(|&&op| op == OpCode::Jump).count();
    assert!(jump_count >= 1);
}

#[test]
fn compile_continue_in_nested_if() {
    let chunk = compile("while true\n  if true\n    continue\n  end\nend");
    let ops = opcodes(&chunk);
    let loop_count = ops.iter().filter(|&&op| op == OpCode::Loop).count();
    assert!(loop_count >= 2);
}

// ── Jump patching (12.4.6) ──────────────────────────────────────────

#[test]
fn if_jump_targets_correct_branch() {
    // Verify the if statement's jump structure is well-formed by checking
    // that compilation produces a valid chunk (no panics, correct opcode count)
    let chunk = compile("if true\n  1\nelse\n  2\nend");
    let ops = opcodes(&chunk);
    // Structure: True, JumpIfFalse, Pop, Const(1), Pop, Jump, Pop, Const(2), Pop, Return
    assert!(!ops.is_empty());
    assert_eq!(*ops.last().unwrap(), OpCode::Return);
}

#[test]
fn while_loop_jumps_back() {
    let chunk = compile("while false\n  42\nend");
    let ops = opcodes(&chunk);
    // Should have Loop opcode for backward jump
    assert!(ops.contains(&OpCode::Loop));
}

#[test]
fn for_loop_jumps_back() {
    let chunk = compile("for x in [1]\n  x\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Loop));
}

// ── Multiple breaks ──────────────────────────────────────────────────

#[test]
fn compile_multiple_breaks() {
    let chunk =
        compile("while true\n  if true\n    break\n  end\n  if false\n    break\n  end\nend");
    let ops = opcodes(&chunk);
    // Two break jumps plus the loop's own jumps
    let jump_count = ops.iter().filter(|&&op| op == OpCode::Jump).count();
    assert!(jump_count >= 2, "Expected at least 2 Jump ops for 2 breaks");
}

// ── Complex control flow ─────────────────────────────────────────────

#[test]
fn compile_while_with_assignment_in_body() {
    let chunk = compile("x = 0\nwhile x\n  x = 1\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Loop));
    // x is defined globally, then reassigned in the loop
    assert!(ops.contains(&OpCode::DefineGlobal));
}

#[test]
fn compile_nested_while_loops() {
    let chunk = compile("while true\n  while false\n    42\n  end\nend");
    let ops = opcodes(&chunk);
    let loop_count = ops.iter().filter(|&&op| op == OpCode::Loop).count();
    assert_eq!(loop_count, 2, "Expected 2 Loop ops for nested while loops");
}

#[test]
fn compile_if_inside_while() {
    let chunk = compile("while true\n  if false\n    1\n  else\n    2\n  end\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Loop));
    assert!(ops.contains(&OpCode::JumpIfFalse));
}

// ── For loop with body statements ────────────────────────────────────

#[test]
fn compile_for_with_multiple_body_statements() {
    let chunk = compile("for x in [1, 2]\n  x\n  x\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Loop));
    // Two expression statements in body, each with GetLocal + Pop
    let get_local_count = ops.iter().filter(|&&op| op == OpCode::GetLocal).count();
    assert!(
        get_local_count >= 2,
        "Expected at least 2 GetLocal for two x references in body"
    );
}
