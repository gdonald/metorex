// Tests for statement compilation (12.3)
//
// Exercises expression statements, variable declaration/assignment,
// block scoping, and return statements.

use metorex::ast::{Expression, Statement};
use metorex::bytecode::disassembler::disassemble;
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

/// Helper: compile AST statements directly (bypasses parser).
fn compile_stmts(stmts: &[Statement]) -> metorex::bytecode::chunk::Chunk {
    let compiler = Compiler::new();
    compiler.compile(stmts).expect("compile failed")
}

fn pos() -> Position {
    Position {
        line: 1,
        column: 0,
        offset: 0,
    }
}

// ── Expression statement compilation (12.3.1) ─────────────────────────

#[test]
fn expression_statement_emits_pop() {
    let chunk = compile("42");
    let ops = opcodes(&chunk);
    // Expression statement: push constant, pop it (result unused), then return
    assert_eq!(ops, vec![OpCode::Constant, OpCode::Pop, OpCode::Return]);
}

#[test]
fn multiple_expression_statements_each_pop() {
    let chunk = compile("1\n2\n3");
    let ops = opcodes(&chunk);
    // Each expression statement pushes a constant then pops
    assert_eq!(
        ops,
        vec![
            OpCode::Constant,
            OpCode::Pop,
            OpCode::Constant,
            OpCode::Pop,
            OpCode::Constant,
            OpCode::Pop,
            OpCode::Return,
        ]
    );
}

#[test]
fn expression_statement_with_method_call() {
    let chunk = compile("x.length");
    let ops = opcodes(&chunk);
    // GetGlobal(x), Invoke(length), Pop, Return
    assert!(ops.contains(&OpCode::Invoke));
    assert!(ops.contains(&OpCode::Pop));
}

// ── Variable declaration compilation (12.3.2) ─────────────────────────

#[test]
fn global_variable_declaration() {
    let chunk = compile("x = 10");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Constant));
    assert!(ops.contains(&OpCode::DefineGlobal));
}

#[test]
fn global_variable_read_after_declaration() {
    let chunk = compile("x = 10\nx");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::DefineGlobal));
    assert!(ops.contains(&OpCode::GetGlobal));
}

#[test]
fn multiple_global_declarations() {
    let chunk = compile("x = 1\ny = 2\nz = 3");
    let ops = opcodes(&chunk);
    let define_count = ops.iter().filter(|&&op| op == OpCode::DefineGlobal).count();
    assert_eq!(define_count, 3);
}

// ── Assignment compilation (12.3.3) ───────────────────────────────────

#[test]
fn assignment_to_instance_variable() {
    let chunk = compile("@name = \"Alice\"");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::SetInstance));
}

#[test]
fn assignment_to_index() {
    let chunk = compile("arr = [1, 2, 3]\narr[0] = 99");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::IndexSet));
}

#[test]
fn assignment_invalid_target_errors() {
    // 42 = x is not a valid assignment target;
    // The parser won't produce this, so construct AST directly.
    let stmts = vec![Statement::Assignment {
        target: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        value: Expression::IntLiteral {
            value: 1,
            position: pos(),
        },
        position: pos(),
    }];
    let compiler = Compiler::new();
    let result = compiler.compile(&stmts);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Invalid assignment target"),
        "Error was: {err}"
    );
}

// ── Block compilation (12.3.4) ────────────────────────────────────────

#[test]
fn block_compiles_inner_statements() {
    // A block with a single expression statement inside
    let stmts = vec![Statement::Block {
        statements: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(),
            },
            position: pos(),
        }],
        position: pos(),
    }];
    let chunk = compile_stmts(&stmts);
    let ops = opcodes(&chunk);
    // Should contain the inner constant + pop, plus final return
    assert!(ops.contains(&OpCode::Constant));
    assert!(ops.contains(&OpCode::Pop));
    assert!(ops.contains(&OpCode::Return));
}

#[test]
fn block_creates_scope_and_pops_locals() {
    // A block that declares a local variable — on scope exit the local is popped
    let stmts = vec![Statement::Block {
        statements: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(),
            },
            value: Expression::IntLiteral {
                value: 10,
                position: pos(),
            },
            position: pos(),
        }],
        position: pos(),
    }];
    let chunk = compile_stmts(&stmts);
    let ops = opcodes(&chunk);
    // The local variable is pushed onto the stack inside the block,
    // then popped when the scope ends.
    // Expect: Constant(10), Pop(scope cleanup), Return
    assert!(ops.contains(&OpCode::Constant));
    // At least one Pop for scope cleanup
    let pop_count = ops.iter().filter(|&&op| op == OpCode::Pop).count();
    assert!(pop_count >= 1, "Expected at least 1 Pop for scope cleanup");
}

#[test]
fn nested_blocks_create_nested_scopes() {
    // Outer block declares x, inner block declares y
    let stmts = vec![Statement::Block {
        statements: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                },
                value: Expression::IntLiteral {
                    value: 1,
                    position: pos(),
                },
                position: pos(),
            },
            Statement::Block {
                statements: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "y".to_string(),
                        position: pos(),
                    },
                    value: Expression::IntLiteral {
                        value: 2,
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    }];
    let chunk = compile_stmts(&stmts);
    let ops = opcodes(&chunk);
    // Both locals should be cleaned up
    let pop_count = ops.iter().filter(|&&op| op == OpCode::Pop).count();
    // Inner block pops y, outer block pops x
    assert!(
        pop_count >= 2,
        "Expected at least 2 Pops for nested scope cleanup, got {pop_count}"
    );
}

#[test]
fn block_local_resolved_within_scope() {
    // Inside a block, assigning x then reading x should use SetLocal/GetLocal
    let stmts = vec![Statement::Block {
        statements: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                },
                value: Expression::IntLiteral {
                    value: 5,
                    position: pos(),
                },
                position: pos(),
            },
            Statement::Expression {
                expression: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
        ],
        position: pos(),
    }];
    let chunk = compile_stmts(&stmts);
    let ops = opcodes(&chunk);
    // x should be resolved as a local (GetLocal), not global
    assert!(
        ops.contains(&OpCode::GetLocal),
        "Expected GetLocal for block-scoped variable, ops: {ops:?}"
    );
    assert!(
        !ops.contains(&OpCode::GetGlobal),
        "Block-scoped variable should not use GetGlobal"
    );
}

#[test]
fn empty_block_compiles() {
    let stmts = vec![Statement::Block {
        statements: vec![],
        position: pos(),
    }];
    let chunk = compile_stmts(&stmts);
    let ops = opcodes(&chunk);
    // Just the trailing Return from compile()
    assert_eq!(ops, vec![OpCode::Return]);
}

// ── Return statement compilation (12.3.5) ─────────────────────────────

#[test]
fn return_with_value() {
    let chunk = compile("return 42");
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Constant, OpCode::Return, OpCode::Return]);
    // First Return is from the return statement; second is the implicit one
    // appended by compile()
}

#[test]
fn return_without_value_pushes_nil() {
    let chunk = compile("return");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Nil));
    assert!(ops.contains(&OpCode::Return));
}

#[test]
fn return_with_expression() {
    let chunk = compile("return 1 + 2");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Add));
    assert!(ops.contains(&OpCode::Return));
}

// ── Disassembly verification ──────────────────────────────────────────

#[test]
fn statement_compilation_disassembles_correctly() {
    let chunk = compile("x = 1 + 2\nreturn x");
    let output = disassemble(&chunk, "statements");
    assert!(output.contains("OP_CONSTANT"));
    assert!(output.contains("OP_ADD"));
    assert!(output.contains("OP_RETURN"));
}

#[test]
fn block_compilation_disassembles() {
    let stmts = vec![Statement::Block {
        statements: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 99,
                position: pos(),
            },
            position: pos(),
        }],
        position: pos(),
    }];
    let chunk = compile_stmts(&stmts);
    let output = disassemble(&chunk, "block");
    assert!(output.contains("OP_CONSTANT"));
    assert!(output.contains("OP_POP"));
}
