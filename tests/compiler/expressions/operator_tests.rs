// Tests for binary, unary, and logical operator compilation.

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

// ── Binary operations ───────────────────────────────────────────────────────

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

// ── Unary operations ────────────────────────────────────────────────────────

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
    assert!(!ops.contains(&OpCode::Negate));
    assert!(ops.contains(&OpCode::Constant));
}

// ── Logical short-circuit ───────────────────────────────────────────────────

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

// ── Assignment operators as errors in expression context ───────────────────

#[test]
fn compile_assign_op_in_expression_context_errors() {
    use metorex::ast::{BinaryOp, Expression};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let expr = Expression::BinaryOp {
        op: BinaryOp::Assign,
        left: Box::new(Expression::Identifier {
            name: "x".to_string(),
            position: pos,
        }),
        right: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos,
        }),
        position: pos,
    };
    let stmt = metorex::ast::Statement::Expression {
        expression: expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Assignment operators should be handled as statements"),
        "Error was: {}",
        err
    );
}

#[test]
fn compile_add_assign_op_in_expression_context_errors() {
    use metorex::ast::{BinaryOp, Expression};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let expr = Expression::BinaryOp {
        op: BinaryOp::AddAssign,
        left: Box::new(Expression::Identifier {
            name: "x".to_string(),
            position: pos,
        }),
        right: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos,
        }),
        position: pos,
    };
    let stmt = metorex::ast::Statement::Expression {
        expression: expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
}

#[test]
fn compile_subtract_assign_op_in_expression_context_errors() {
    use metorex::ast::{BinaryOp, Expression};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let expr = Expression::BinaryOp {
        op: BinaryOp::SubtractAssign,
        left: Box::new(Expression::Identifier {
            name: "x".to_string(),
            position: pos,
        }),
        right: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos,
        }),
        position: pos,
    };
    let stmt = metorex::ast::Statement::Expression {
        expression: expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
}

#[test]
fn compile_multiply_assign_op_in_expression_context_errors() {
    use metorex::ast::{BinaryOp, Expression};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let expr = Expression::BinaryOp {
        op: BinaryOp::MultiplyAssign,
        left: Box::new(Expression::Identifier {
            name: "x".to_string(),
            position: pos,
        }),
        right: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos,
        }),
        position: pos,
    };
    let stmt = metorex::ast::Statement::Expression {
        expression: expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
}

#[test]
fn compile_divide_assign_op_in_expression_context_errors() {
    use metorex::ast::{BinaryOp, Expression};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let expr = Expression::BinaryOp {
        op: BinaryOp::DivideAssign,
        left: Box::new(Expression::Identifier {
            name: "x".to_string(),
            position: pos,
        }),
        right: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos,
        }),
        position: pos,
    };
    let stmt = metorex::ast::Statement::Expression {
        expression: expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
}

// ── Binary ops not supported by the compiler ────────────────────────────────

#[test]
fn compile_power_op_not_supported() {
    let tokens = metorex::lexer::Lexer::new("2 ** 3").tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("**"), "Error was: {}", err);
}

#[test]
fn compile_spaceship_op_not_supported() {
    let tokens = metorex::lexer::Lexer::new("1 <=> 2").tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("<=>"), "Error was: {}", err);
}

#[test]
fn compile_bitwise_and_op_not_supported() {
    let tokens = metorex::lexer::Lexer::new("3 & 1").tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("&"), "Error was: {}", err);
}

#[test]
fn compile_bitwise_or_op_not_supported() {
    let tokens = metorex::lexer::Lexer::new("3 | 1").tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("|"), "Error was: {}", err);
}

#[test]
fn compile_xor_op_not_supported() {
    let tokens = metorex::lexer::Lexer::new("3 ^ 1").tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("^"), "Error was: {}", err);
}
