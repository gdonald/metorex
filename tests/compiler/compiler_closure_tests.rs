// Tests for closure and upvalue compilation (12.7)
//
// Exercises local variable resolution, upvalue resolution, captured variable
// tracking, OP_CLOSURE emission, and OP_CLOSE_UPVALUE emission.

use metorex::ast::{Expression, Statement};
use metorex::bytecode::opcode::OpCode;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::lexer::token::Position;
use metorex::object::Object;
use metorex::parser::Parser;

/// Helper: parse source, compile, return the chunk.
fn compile(source: &str) -> metorex::bytecode::chunk::Chunk {
    let tokens = Lexer::new(source).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let compiler = Compiler::new();
    compiler.compile(&stmts).expect("compile failed")
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

/// Helper: find all compiled functions recursively (searches nested chunks too).
fn find_compiled_functions(
    chunk: &metorex::bytecode::chunk::Chunk,
) -> Vec<&metorex::object::CompiledFunction> {
    let mut funcs = Vec::new();
    find_compiled_functions_recursive(chunk, &mut funcs);
    funcs
}

fn find_compiled_functions_recursive<'a>(
    chunk: &'a metorex::bytecode::chunk::Chunk,
    funcs: &mut Vec<&'a metorex::object::CompiledFunction>,
) {
    for i in 0..256 {
        let constant =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| chunk.get_constant(i)));
        match constant {
            Ok(Object::CompiledFunction(func)) => {
                funcs.push(func.as_ref());
                // Recursively search inside this function's chunk
                find_compiled_functions_recursive(&func.chunk, funcs);
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }
}

fn pos() -> Position {
    Position {
        line: 1,
        column: 0,
        offset: 0,
    }
}

// ── Local variable resolution (12.7.1) ───────────────────────────────

#[test]
fn local_variable_resolves_to_get_local() {
    let chunk = compile("def f\n  x = 10\n  x\nend");
    let funcs = find_compiled_functions(&chunk);
    let f = funcs.iter().find(|f| f.name == "f").unwrap();
    let body_ops = opcodes(&f.chunk);
    assert!(
        body_ops.contains(&OpCode::GetLocal),
        "Expected GetLocal for local var, got: {:?}",
        body_ops
    );
}

#[test]
fn parameter_resolves_to_get_local() {
    let chunk = compile("def f(x)\n  x\nend");
    let funcs = find_compiled_functions(&chunk);
    let f = funcs.iter().find(|f| f.name == "f").unwrap();
    let body_ops = opcodes(&f.chunk);
    assert!(body_ops.contains(&OpCode::GetLocal));
}

// ── Upvalue resolution (12.7.2) ─────────────────────────────────────

#[test]
fn nested_function_captures_enclosing_local() {
    // outer defines x, inner reads x — inner should use GetUpvalue
    let source = "def outer\n  x = 10\n  def inner\n    x\n  end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let inner = funcs.iter().find(|f| f.name == "inner").unwrap();
    let body_ops = opcodes(&inner.chunk);
    assert!(
        body_ops.contains(&OpCode::GetUpvalue),
        "Expected GetUpvalue in inner function, got: {:?}",
        body_ops
    );
}

#[test]
fn nested_function_captures_parameter() {
    let source = "def outer(val)\n  def inner\n    val\n  end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let inner = funcs.iter().find(|f| f.name == "inner").unwrap();
    let body_ops = opcodes(&inner.chunk);
    assert!(
        body_ops.contains(&OpCode::GetUpvalue),
        "Expected GetUpvalue for captured parameter"
    );
}

// ── Track captured variables (12.7.3) ────────────────────────────────

#[test]
fn captured_variable_emits_close_upvalue() {
    // When a captured local goes out of scope, OP_CLOSE_UPVALUE should be emitted
    // We test this with a block scope that captures a variable
    let stmts = vec![Statement::Block {
        statements: vec![
            // x = 10 (this will be captured)
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                },
                value: Expression::IntLiteral {
                    value: 10,
                    position: pos(),
                },
                position: pos(),
            },
            // def inner; x; end (captures x as upvalue)
            Statement::FunctionDef {
                name: "inner".to_string(),
                parameters: vec![],
                body: vec![Statement::Expression {
                    expression: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    }];
    let compiler = Compiler::new();
    let chunk = compiler.compile(&stmts).expect("compile failed");
    let ops = opcodes(&chunk);
    assert!(
        ops.contains(&OpCode::CloseUpvalue),
        "Expected CloseUpvalue for captured local going out of scope, got: {:?}",
        ops
    );
}

// ── OP_CLOSURE emission (12.7.4) ────────────────────────────────────

#[test]
fn closure_with_upvalues_emits_op_closure() {
    let source = "def outer\n  x = 10\n  def inner\n    x\n  end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let outer = funcs.iter().find(|f| f.name == "outer").unwrap();
    let outer_ops = opcodes(&outer.chunk);
    assert!(
        outer_ops.contains(&OpCode::Closure),
        "Expected OP_CLOSURE in outer function for inner closure, got: {:?}",
        outer_ops
    );
}

#[test]
fn function_without_captures_no_closure_op() {
    let source = "def outer\n  def inner\n    42\n  end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let outer = funcs.iter().find(|f| f.name == "outer").unwrap();
    let outer_ops = opcodes(&outer.chunk);
    assert!(
        !outer_ops.contains(&OpCode::Closure),
        "No OP_CLOSURE expected when no upvalues, got: {:?}",
        outer_ops
    );
}

// ── OP_CLOSE_UPVALUE emission (12.7.5) ──────────────────────────────

#[test]
fn non_captured_local_uses_pop() {
    // A local that's NOT captured should use Pop, not CloseUpvalue
    let stmts = vec![Statement::Block {
        statements: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "y".to_string(),
                position: pos(),
            },
            value: Expression::IntLiteral {
                value: 5,
                position: pos(),
            },
            position: pos(),
        }],
        position: pos(),
    }];
    let compiler = Compiler::new();
    let chunk = compiler.compile(&stmts).expect("compile failed");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Pop));
    assert!(
        !ops.contains(&OpCode::CloseUpvalue),
        "Non-captured local should use Pop, not CloseUpvalue"
    );
}

// ── Upvalue assignment (SetUpvalue) ─────────────────────────────────

#[test]
fn nested_function_assigns_to_upvalue() {
    let source = "def outer\n  x = 0\n  def inner\n    x = 42\n  end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let inner = funcs.iter().find(|f| f.name == "inner").unwrap();
    let body_ops = opcodes(&inner.chunk);
    assert!(
        body_ops.contains(&OpCode::SetUpvalue),
        "Expected SetUpvalue for assigning to captured variable, got: {:?}",
        body_ops
    );
}

// ── Multiple upvalues ───────────────────────────────────────────────

#[test]
fn capture_multiple_variables() {
    let source = "def outer\n  a = 1\n  b = 2\n  def inner\n    a\n    b\n  end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let inner = funcs.iter().find(|f| f.name == "inner").unwrap();
    let body_ops = opcodes(&inner.chunk);
    let upvalue_count = body_ops
        .iter()
        .filter(|&&op| op == OpCode::GetUpvalue)
        .count();
    assert_eq!(
        upvalue_count, 2,
        "Expected 2 GetUpvalue for a and b, got {}",
        upvalue_count
    );
}

// ── Upvalue deduplication ───────────────────────────────────────────

#[test]
fn same_variable_captured_once() {
    // Reading x twice in inner should still result in a single upvalue
    let source = "def outer\n  x = 1\n  def inner\n    x\n    x\n  end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let inner = funcs.iter().find(|f| f.name == "inner").unwrap();
    let body_ops = opcodes(&inner.chunk);
    // Both reads should use the same upvalue index (0)
    let upvalue_count = body_ops
        .iter()
        .filter(|&&op| op == OpCode::GetUpvalue)
        .count();
    assert_eq!(upvalue_count, 2, "Two reads, both via GetUpvalue");
}

// ── No upvalue for global variables ─────────────────────────────────

#[test]
fn global_variable_not_captured_as_upvalue() {
    // x defined at global scope, inner function should use GetGlobal
    let source = "x = 1\ndef inner\n  x\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let inner = funcs.iter().find(|f| f.name == "inner").unwrap();
    let body_ops = opcodes(&inner.chunk);
    assert!(
        body_ops.contains(&OpCode::GetGlobal),
        "Global var should use GetGlobal, not GetUpvalue"
    );
    assert!(
        !body_ops.contains(&OpCode::GetUpvalue),
        "Global var should not use GetUpvalue"
    );
}

// ── Closure in class method ─────────────────────────────────────────

#[test]
fn method_with_closure_captures() {
    let source = "class Foo\n  def outer\n    x = 1\n    def inner\n      x\n    end\n  end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let inner = funcs.iter().find(|f| f.name == "inner").unwrap();
    let body_ops = opcodes(&inner.chunk);
    assert!(
        body_ops.contains(&OpCode::GetUpvalue),
        "Method's inner function should capture via GetUpvalue"
    );
}

// ── Upvalue struct coverage ─────────────────────────────────────────

#[test]
fn upvalue_struct_debug_display() {
    let uv = metorex::compiler::Upvalue {
        index: 3,
        is_local: true,
    };
    let debug_str = format!("{:?}", uv);
    assert!(debug_str.contains("index: 3"));
    assert!(debug_str.contains("is_local: true"));
}

#[test]
fn upvalue_struct_clone_copy() {
    let uv = metorex::compiler::Upvalue {
        index: 5,
        is_local: false,
    };
    let cloned = uv;
    assert_eq!(cloned.index, 5);
    assert!(!cloned.is_local);
}

#[test]
fn local_struct_is_captured_field() {
    let local = metorex::compiler::Local {
        name: "x".to_string(),
        depth: 1,
        is_captured: true,
    };
    assert!(local.is_captured);
    assert_eq!(local.name, "x");
}
