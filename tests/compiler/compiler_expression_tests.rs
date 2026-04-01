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

// ── Empty interpolated string (compiler/expressions.rs line 55) ─────────────

#[test]
fn compile_empty_string_literal() {
    let chunk = compile("\"\"");
    assert!(!chunk.is_empty());
}

// ── Identifier resolves globally (compiler/expressions.rs line 82) ──────────

#[test]
fn compile_global_identifier_resolves_globally() {
    let chunk = compile("x = 1\nx");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::GetGlobal));
}

// ── Self expression (compiler/expressions.rs lines 268-272) ─────────────────

#[test]
fn compile_self_expression() {
    let chunk = compile("self");
    // self compiles to some opcode — just verify it produces output
    assert!(!chunk.is_empty());
}

// ── Unimplemented expression types (compiler/expressions.rs lines 298-302) ──

#[test]
fn compile_super_not_implemented() {
    let tokens = metorex::lexer::Lexer::new("super").tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let compiler = metorex::compiler::Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(
        err.contains("not yet implemented") || err.contains("Compilation"),
        "Error was: {}",
        err
    );
}

#[test]
fn compile_scope_resolution_not_implemented() {
    let tokens = metorex::lexer::Lexer::new("Foo::Bar").tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let compiler = metorex::compiler::Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(
        err.contains("not yet implemented") || err.contains("Compilation"),
        "Error was: {}",
        err
    );
}

#[test]
fn compile_lambda_produces_compiled_function() {
    let chunk = compile("lambda do |x| x end");
    assert!(!chunk.is_empty());
}

#[test]
fn compile_case_not_yet_implemented() {
    let tokens = metorex::lexer::Lexer::new("x = case 1\nwhen 1\n  42\nend").tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let compiler = metorex::compiler::Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(
        err.contains("not yet implemented") || err.contains("Compilation"),
        "Error was: {}",
        err
    );
}

// ── Statement compilation ──────────────────────────────────────────────

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

#[test]
fn compile_return_nil() {
    let chunk = compile("return");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Nil));
    assert!(ops.contains(&OpCode::Return));
}

// ── Range ──────────────────────────────────────────────────────────────

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

// ── Fallthrough statement types ────────────────────────────────────────

#[test]
fn compile_if_statement_fallthrough() {
    let chunk = compile("if true\n  42\nend");
    assert!(!chunk.is_empty());
}

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
fn compile_while_statement_fallthrough() {
    let chunk = compile("while false\n  42\nend");
    assert!(!chunk.is_empty());
}

// ── Function call with multiple args ───────────────────────────────────

#[test]
fn compile_function_call_multi_args() {
    let chunk = compile("puts(1, 2, 3)");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Call));
}

// ── Empty interpolation parts (expressions.rs line 55) ────────────────

#[test]
fn compile_interpolated_string_with_text_and_expr() {
    let chunk = compile("\"hello #{1} world\"");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Constant));
    assert!(ops.contains(&OpCode::Add));
}

#[test]
fn compile_interpolated_string_empty_parts_via_ast() {
    // Directly construct an InterpolatedString with zero parts to hit line 55
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
    // Should emit a constant empty string
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

// ── Assignment operator error in expression context (expressions.rs lines 167-169) ──

#[test]
fn compile_assign_op_in_expression_context_errors() {
    // Directly construct AST with BinaryOp::Assign in expression position
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

// ── Range compilation (expressions.rs lines 270-274) ────────────────────

#[test]
fn compile_range_inclusive_produces_call() {
    let chunk = compile("1..10");
    let ops = opcodes(&chunk);
    // Range compiles to: start, end, bool(exclusive), Call 3
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
    // The exclusive flag for inclusive range should be false
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
    // The exclusive flag for exclusive range should be true
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

// ── If expression not implemented (expressions.rs line 359) ─────────────

#[test]
fn compile_if_expression_not_implemented() {
    use metorex::ast::{Expression, Statement};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let if_expr = Expression::If {
        condition: Box::new(Expression::BoolLiteral {
            value: true,
            position: pos,
        }),
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos,
            },
            position: pos,
        }],
        elsif_branches: vec![],
        else_branch: None,
        position: pos,
    };
    let stmt = Statement::Expression {
        expression: if_expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not yet implemented"), "Error was: {}", err);
}

// ── Unless expression not implemented (expressions.rs line 360) ─────────

#[test]
fn compile_unless_expression_not_implemented() {
    use metorex::ast::{Expression, Statement};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let unless_expr = Expression::Unless {
        condition: Box::new(Expression::BoolLiteral {
            value: true,
            position: pos,
        }),
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos,
            },
            position: pos,
        }],
        else_branch: None,
        position: pos,
    };
    let stmt = Statement::Expression {
        expression: unless_expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not yet implemented"), "Error was: {}", err);
}
