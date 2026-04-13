// Tests for statement compilation: return, assignment, control flow, for, break.

use metorex::bytecode::opcode::OpCode;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::parser::Parser;

fn compile(source: &str) -> metorex::bytecode::chunk::Chunk {
    let tokens = Lexer::new(source).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let compiler = Compiler::new();
    compiler.compile(&stmts).expect("compile failed")
}

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

// ── Return ──────────────────────────────────────────────────────────────────

#[test]
fn compile_return_with_value() {
    let chunk = compile("return 42");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Constant));
    assert!(ops.contains(&OpCode::Return));
}

#[test]
fn compile_return_nil() {
    let chunk = compile("return");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Nil));
    assert!(ops.contains(&OpCode::Return));
}

// ── Assignment statements ────────────────────────────────────────────────────

#[test]
fn compile_assignment() {
    let chunk = compile("x = 42");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_instance_var_assignment() {
    let chunk = compile("@x = 42");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::SetInstance));
}

#[test]
fn compile_index_set() {
    let chunk = compile("a = [1, 2, 3]\na[0] = 42");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::IndexSet));
}

// ── If / unless / while fallthrough ─────────────────────────────────────────

#[test]
fn compile_if_statement_fallthrough() {
    let chunk = compile("if true\n  42\nend");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_while_statement_fallthrough() {
    let chunk = compile("while false\n  42\nend");
    assert!(!chunk.is_empty());
}

// ── Class / def / module fallthrough ────────────────────────────────────────

#[test]
fn compile_class_statement_fallthrough() {
    let chunk = compile("class Foo\nend");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_def_statement_fallthrough() {
    let chunk = compile("def foo\n  42\nend");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_module_falls_through() {
    let chunk = compile("module Foo\nend");
    assert!(!chunk.is_empty());
}

// ── begin/rescue / raise fallthrough ────────────────────────────────────────

#[test]
fn compile_begin_rescue_falls_through() {
    let chunk = compile("begin\n  42\nrescue\n  0\nend");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_raise_falls_through() {
    let chunk = compile("raise \"oops\"");
    assert!(!chunk.is_empty());
}

// ── For loop with break ──────────────────────────────────────────────────────

#[test]
fn compile_for_with_break_patches_jump() {
    let chunk = compile("for x in [1, 2, 3]\n  break\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Jump));
    assert!(ops.contains(&OpCode::Loop));
}

#[test]
fn compile_break_with_local_in_loop_body() {
    let chunk = compile("while true\n  x = 1\n  break\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Jump));
}

// ── Lambda ──────────────────────────────────────────────────────────────────

#[test]
fn compile_lambda_produces_compiled_function() {
    let chunk = compile("lambda do |x| x end");
    assert!(!chunk.is_empty());
}
