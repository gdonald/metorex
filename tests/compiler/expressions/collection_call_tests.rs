// Tests for collection, index, call, variable, and grouped expression compilation.

use metorex::bytecode::disassembler::disassemble;
use metorex::bytecode::opcode::OpCode;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::object::Object;
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

// ── Variable access ─────────────────────────────────────────────────────────

#[test]
fn compile_global_variable_access() {
    let chunk = compile("x");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::GetGlobal));
}

#[test]
fn compile_global_identifier_resolves_globally() {
    let chunk = compile("x = 1\nx");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::GetGlobal));
}

// ── Assignment ──────────────────────────────────────────────────────────────

#[test]
fn compile_global_assignment() {
    let chunk = compile("x = 42");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Constant));
    assert!(ops.contains(&OpCode::DefineGlobal));
}

// ── Array literal ───────────────────────────────────────────────────────────

#[test]
fn compile_array_literal() {
    let chunk = compile("[1, 2, 3]");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Array));
}

#[test]
fn compile_empty_array() {
    let chunk = compile("[]");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Array));
}

// ── Hash literal ────────────────────────────────────────────────────────────

#[test]
fn compile_hash_literal() {
    let chunk = compile("{\"a\" => 1, \"b\" => 2}");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Hash));
}

// ── Index access ────────────────────────────────────────────────────────────

#[test]
fn compile_index_access() {
    let chunk = compile("x[0]");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::IndexGet));
}

// ── Method call ─────────────────────────────────────────────────────────────

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

// ── Bare function call ──────────────────────────────────────────────────────

#[test]
fn compile_function_call() {
    let chunk = compile("puts(42)");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Call));
}

#[test]
fn compile_function_call_multi_args() {
    let chunk = compile("puts(1, 2, 3)");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Call));
}

// ── Grouped / complex expressions ───────────────────────────────────────────

#[test]
fn compile_grouped_expression() {
    let chunk = compile("(1 + 2) * 3");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Add));
    assert!(ops.contains(&OpCode::Multiply));
}

#[test]
fn compile_nested_arithmetic() {
    let chunk = compile("(1 + 2) * (3 - 4)");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Add));
    assert!(ops.contains(&OpCode::Subtract));
    assert!(ops.contains(&OpCode::Multiply));
}

// ── Disassembly sanity check ────────────────────────────────────────────────

#[test]
fn compiled_chunk_disassembles() {
    let chunk = compile("x = 1 + 2");
    let output = disassemble(&chunk, "x = 1 + 2");
    assert!(output.contains("OP_CONSTANT"));
    assert!(output.contains("OP_ADD"));
    assert!(output.contains("OP_DEFINE_GLOBAL"));
    assert!(output.contains("OP_RETURN"));
}

// ── Range ──────────────────────────────────────────────────────────────────

#[test]
fn compile_range_inclusive() {
    let chunk = compile("1..10");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_range_exclusive() {
    let chunk = compile("1...10");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_range_inclusive_produces_call() {
    let chunk = compile("1..10");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Call));
}

#[test]
fn compile_range_exclusive_produces_call() {
    let chunk = compile("1...10");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Call));
}

#[test]
fn compile_range_inclusive_has_false_exclusive() {
    let chunk = compile("1..5");
    let mut found_false = false;
    for i in 0..chunk.constants_count() {
        if *chunk.get_constant(i) == Object::Bool(false) {
            found_false = true;
            break;
        }
    }
    assert!(
        found_false,
        "Inclusive range should have Bool(false) constant"
    );
}

#[test]
fn compile_range_exclusive_has_true_exclusive() {
    let chunk = compile("1...5");
    let mut found_true = false;
    for i in 0..chunk.constants_count() {
        if *chunk.get_constant(i) == Object::Bool(true) {
            found_true = true;
            break;
        }
    }
    assert!(
        found_true,
        "Exclusive range should have Bool(true) constant"
    );
}

// ── Self expression ─────────────────────────────────────────────────────────

#[test]
fn compile_self_expression() {
    let chunk = compile("self");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_self_in_method_body() {
    use metorex::ast::{Expression, Statement};
    use metorex::lexer::token::Position;
    let pos = Position {
        line: 1,
        column: 0,
        offset: 0,
    };
    let stmts = vec![Statement::MethodDef {
        is_class_method: false,
        name: "me".to_string(),
        parameters: vec![],
        body: vec![Statement::Expression {
            expression: Expression::SelfExpr { position: pos },
            position: pos,
        }],
        position: pos,
    }];
    let compiler = Compiler::new();
    let chunk = compiler.compile(&stmts).expect("compile failed");
    for i in 0..256 {
        let constant =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| chunk.get_constant(i)));
        match constant {
            Ok(Object::CompiledFunction(f)) if f.name == "me" => {
                let body_ops = opcodes(&f.chunk);
                assert!(
                    body_ops.contains(&OpCode::GetLocal),
                    "Expected GetLocal for self, got: {:?}",
                    body_ops
                );
                return;
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }
    panic!("Expected compiled method 'me'");
}

// ── Disassembly of constant types ───────────────────────────────────────────

#[test]
fn disassemble_nil_constant() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.add_constant(Object::Nil);
    chunk.write_op_u8(OpCode::Constant, 0, 1);
    chunk.write_opcode(OpCode::Return, 1);
    let output = disassemble(&chunk, "nil_test");
    assert!(
        output.contains("nil"),
        "Expected nil in disassembly: {}",
        output
    );
}

#[test]
fn disassemble_non_standard_constant() {
    use metorex::object::CompiledFunction;
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    let func = CompiledFunction::new("test_fn".to_string(), 0);
    chunk.add_constant(Object::CompiledFunction(std::rc::Rc::new(func)));
    chunk.write_op_u8(OpCode::Constant, 0, 1);
    chunk.write_opcode(OpCode::Return, 1);
    let output = disassemble(&chunk, "func_test");
    assert!(!output.is_empty());
}
