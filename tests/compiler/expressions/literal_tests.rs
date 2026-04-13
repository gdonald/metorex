// Tests for literal expression compilation.

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
fn compile_symbol_literal() {
    let chunk = compile(":hello");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_interpolated_string() {
    let chunk = compile("x = 42\n\"value: #{x}\"");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_empty_interpolated_string() {
    let chunk = compile("\"\"");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_instance_variable() {
    let chunk = compile("@x");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::GetInstance));
}

#[test]
fn compile_global_variable() {
    let chunk = compile("$x");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::GetGlobal));
}

#[test]
fn compile_class_variable() {
    let chunk = compile("@@x");
    assert!(!chunk.is_empty());
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

#[test]
fn compile_empty_string_literal() {
    let chunk = compile("\"\"");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_interpolated_string_with_text_and_expr() {
    let chunk = compile("\"hello #{1} world\"");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Constant));
    assert!(ops.contains(&OpCode::Add));
}

#[test]
fn compile_interpolated_string_empty_parts_via_ast() {
    use metorex::ast::{Expression, Statement};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let expr = Expression::InterpolatedString {
        parts: vec![],
        position: pos,
    };
    let stmt = Statement::Expression {
        expression: expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let chunk = compiler.compile(&[stmt]).expect("compile failed");
    assert!(!chunk.is_empty());
    let mut found_empty_string = false;
    for i in 0..chunk.constants_count() {
        if let Object::String(s) = chunk.get_constant(i)
            && s.is_empty()
        {
            found_empty_string = true;
            break;
        }
    }
    assert!(
        found_empty_string,
        "Empty InterpolatedString should produce empty string constant"
    );
}
