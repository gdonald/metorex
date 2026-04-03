// Tests for function and method compilation (12.5)
//
// Exercises function definition compilation, nested compiler for function body,
// parameter compilation, OP_RETURN at function end, constant pool storage,
// and method compilation.

use std::rc::Rc;

use metorex::ast::{Parameter, Statement};
use metorex::bytecode::opcode::OpCode;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::lexer::token::Position;
use metorex::object::{CompiledFunction, Object};
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

/// Helper: find the first CompiledFunction in the constant pool.
fn find_compiled_function(
    chunk: &metorex::bytecode::chunk::Chunk,
) -> Option<&metorex::object::CompiledFunction> {
    for i in 0..256 {
        // Scan the constant pool
        let constant =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| chunk.get_constant(i)));
        match constant {
            Ok(Object::CompiledFunction(func)) => return Some(func),
            Ok(_) => continue,
            Err(_) => break, // out of range
        }
    }
    None
}

// ── Function definition compilation (12.5.1) ─────────────────────────

#[test]
fn compile_simple_function_def() {
    let chunk = compile("def greet\n  42\nend");
    let ops = opcodes(&chunk);
    // Function stored as constant, then DefineGlobal
    assert!(ops.contains(&OpCode::Constant) || ops.contains(&OpCode::ConstantLong));
    assert!(ops.contains(&OpCode::DefineGlobal));
}

#[test]
fn compile_function_stored_in_constant_pool() {
    let chunk = compile("def add(a, b)\n  a + b\nend");
    let func = find_compiled_function(&chunk);
    assert!(
        func.is_some(),
        "Expected a CompiledFunction in constant pool"
    );
    let func = func.unwrap();
    assert_eq!(func.name, "add");
    assert_eq!(func.arity, 2);
}

// ── Nested compiler for function body (12.5.2) ──────────────────────

#[test]
fn function_body_compiled_into_separate_chunk() {
    let chunk = compile("def square(x)\n  x * x\nend");
    let func = find_compiled_function(&chunk).unwrap();
    // The function's chunk should contain its own bytecode
    let body_ops = opcodes(&func.chunk);
    assert!(
        body_ops.contains(&OpCode::Multiply),
        "Expected Multiply in function body, got: {:?}",
        body_ops
    );
}

#[test]
fn function_body_has_return() {
    let chunk = compile("def noop\nend");
    let func = find_compiled_function(&chunk).unwrap();
    let body_ops = opcodes(&func.chunk);
    assert!(
        body_ops.contains(&OpCode::Return),
        "Expected Return in function body"
    );
}

// ── Function parameters (12.5.3) ─────────────────────────────────────

#[test]
fn function_parameters_become_locals() {
    let chunk = compile("def add(a, b)\n  a\nend");
    let func = find_compiled_function(&chunk).unwrap();
    let body_ops = opcodes(&func.chunk);
    // Parameters are locals, so accessing them uses GetLocal
    assert!(
        body_ops.contains(&OpCode::GetLocal),
        "Expected GetLocal for parameter access, got: {:?}",
        body_ops
    );
}

#[test]
fn function_arity_zero() {
    let chunk = compile("def greet\n  42\nend");
    let func = find_compiled_function(&chunk).unwrap();
    assert_eq!(func.arity, 0);
}

#[test]
fn function_arity_three() {
    let chunk = compile("def f(a, b, c)\n  a\nend");
    let func = find_compiled_function(&chunk).unwrap();
    assert_eq!(func.arity, 3);
}

#[test]
fn function_with_variadic_param_not_counted_in_arity() {
    let chunk = compile("def f(a, *rest)\n  a\nend");
    let func = find_compiled_function(&chunk).unwrap();
    // *rest is variadic, not counted in arity
    assert_eq!(func.arity, 1);
}

// ── OP_RETURN at function end (12.5.4) ───────────────────────────────

#[test]
fn function_body_ends_with_implicit_return() {
    let chunk = compile("def f\n  42\nend");
    let func = find_compiled_function(&chunk).unwrap();
    let body_ops = opcodes(&func.chunk);
    // Last two ops should be Nil, Return (implicit return after body)
    let len = body_ops.len();
    assert!(len >= 2);
    assert_eq!(body_ops[len - 2], OpCode::Nil);
    assert_eq!(body_ops[len - 1], OpCode::Return);
}

#[test]
fn function_with_explicit_return() {
    let chunk = compile("def f\n  return 42\nend");
    let func = find_compiled_function(&chunk).unwrap();
    let body_ops = opcodes(&func.chunk);
    // Should have both explicit and implicit returns
    let return_count = body_ops.iter().filter(|&&op| op == OpCode::Return).count();
    assert!(
        return_count >= 2,
        "Expected at least 2 Returns (explicit + implicit)"
    );
}

// ── Constant pool storage (12.5.5) ──────────────────────────────────

#[test]
fn multiple_functions_stored_in_constant_pool() {
    let chunk = compile("def foo\n  1\nend\ndef bar\n  2\nend");
    let mut func_count = 0;
    for i in 0..256 {
        let constant =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| chunk.get_constant(i)));
        match constant {
            Ok(Object::CompiledFunction(_)) => func_count += 1,
            Ok(_) => continue,
            Err(_) => break,
        }
    }
    assert_eq!(
        func_count, 2,
        "Expected 2 compiled functions in constant pool"
    );
}

#[test]
fn function_name_stored_correctly() {
    let chunk = compile("def my_func(x)\n  x\nend");
    let func = find_compiled_function(&chunk).unwrap();
    assert_eq!(func.name, "my_func");
}

// ── Method compilation (12.5.6) ──────────────────────────────────────

#[test]
fn compile_class_with_method() {
    // Methods are compiled inside class bodies. Since class compilation is
    // a stub (12.6), we test method compilation via the MethodDef statement
    // directly in a class context — but the class body falls through.
    // Instead, test the method def as a standalone statement:
    // (The parser only produces MethodDef inside ClassDef, so we test
    // the function compilation path which is structurally identical.)
    let chunk = compile("def greet(name)\n  name\nend");
    let func = find_compiled_function(&chunk).unwrap();
    assert_eq!(func.name, "greet");
    assert_eq!(func.arity, 1);
    let body_ops = opcodes(&func.chunk);
    assert!(body_ops.contains(&OpCode::GetLocal));
}

// ── Function body with control flow ──────────────────────────────────

#[test]
fn function_with_if_in_body() {
    let chunk = compile("def check(x)\n  if x\n    1\n  else\n    2\n  end\nend");
    let func = find_compiled_function(&chunk).unwrap();
    let body_ops = opcodes(&func.chunk);
    assert!(body_ops.contains(&OpCode::JumpIfFalse));
}

#[test]
fn function_with_while_in_body() {
    let chunk = compile("def loop_fn\n  while false\n    42\n  end\nend");
    let func = find_compiled_function(&chunk).unwrap();
    let body_ops = opcodes(&func.chunk);
    assert!(body_ops.contains(&OpCode::Loop));
}

#[test]
fn function_with_local_variables() {
    let chunk = compile("def f\n  x = 10\n  x\nend");
    let func = find_compiled_function(&chunk).unwrap();
    let body_ops = opcodes(&func.chunk);
    assert!(
        body_ops.contains(&OpCode::GetLocal),
        "Expected GetLocal for local variable in function"
    );
}

// ── Nested function definitions ──────────────────────────────────────

#[test]
fn nested_function_definition() {
    let chunk = compile("def outer\n  def inner\n    42\n  end\nend");
    // The outer function should have a CompiledFunction constant (inner)
    let outer = find_compiled_function(&chunk).unwrap();
    let inner_ops = opcodes(&outer.chunk);
    // inner is defined as a global inside outer's body
    assert!(
        inner_ops.contains(&OpCode::DefineGlobal),
        "Expected inner function DefineGlobal in outer body"
    );
}

// ── Function definition produces output in main chunk ────────────────

#[test]
fn function_def_emits_define_global_in_main_chunk() {
    let chunk = compile("def foo\n  1\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::DefineGlobal));
}

#[test]
fn function_def_followed_by_call() {
    let chunk = compile("def foo\n  1\nend\nfoo");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::DefineGlobal));
    assert!(ops.contains(&OpCode::GetGlobal));
}

// ── Method compilation via direct AST (12.5.6) ──────────────────────

fn pos() -> Position {
    Position {
        line: 1,
        column: 0,
        offset: 0,
    }
}

fn compile_stmts(stmts: &[Statement]) -> metorex::bytecode::chunk::Chunk {
    let compiler = Compiler::new();
    compiler.compile(stmts).expect("compile failed")
}

#[test]
fn method_def_emits_method_opcode() {
    let stmts = vec![Statement::MethodDef {
        name: "greet".to_string(),
        parameters: vec![Parameter::simple("name".to_string(), pos())],
        body: vec![Statement::Expression {
            expression: metorex::ast::Expression::Identifier {
                name: "name".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        position: pos(),
    }];
    let chunk = compile_stmts(&stmts);
    let ops = opcodes(&chunk);
    assert!(
        ops.contains(&OpCode::Method),
        "Expected OP_METHOD for method definition, got: {:?}",
        ops
    );
}

#[test]
fn method_def_stores_compiled_function() {
    let stmts = vec![Statement::MethodDef {
        name: "calculate".to_string(),
        parameters: vec![
            Parameter::simple("a".to_string(), pos()),
            Parameter::simple("b".to_string(), pos()),
        ],
        body: vec![],
        position: pos(),
    }];
    let chunk = compile_stmts(&stmts);
    let func = find_compiled_function(&chunk).unwrap();
    assert_eq!(func.name, "calculate");
    assert_eq!(func.arity, 2);
}

// ── CompiledFunction struct coverage ─────────────────────────────────

#[test]
fn compiled_function_new() {
    let func = CompiledFunction::new("test".to_string(), 3);
    assert_eq!(func.name, "test");
    assert_eq!(func.arity, 3);
    assert!(func.chunk.is_empty());
}

#[test]
fn compiled_function_display_named() {
    let func = CompiledFunction::new("greet".to_string(), 1);
    assert_eq!(format!("{}", func), "<fn greet>");
}

#[test]
fn compiled_function_display_anonymous() {
    let func = CompiledFunction::new(String::new(), 0);
    assert_eq!(format!("{}", func), "<script>");
}

#[test]
fn compiled_function_debug_named() {
    let func = CompiledFunction::new("calc".to_string(), 2);
    assert_eq!(format!("{:?}", func), "<fn calc>");
}

#[test]
fn compiled_function_debug_anonymous() {
    let func = CompiledFunction::new(String::new(), 0);
    assert_eq!(format!("{:?}", func), "<script>");
}

#[test]
fn compiled_function_equality() {
    let a = CompiledFunction::new("f".to_string(), 1);
    let b = CompiledFunction::new("f".to_string(), 1);
    let c = CompiledFunction::new("g".to_string(), 1);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn compiled_function_clone() {
    let func = CompiledFunction::new("test".to_string(), 2);
    let cloned = func.clone();
    assert_eq!(func, cloned);
}

#[test]
fn object_compiled_function_type_name() {
    let func = CompiledFunction::new("f".to_string(), 0);
    let obj = Object::CompiledFunction(Rc::new(func));
    assert_eq!(obj.type_name(), "CompiledFunction");
}

#[test]
fn object_compiled_function_display() {
    let func = CompiledFunction::new("hello".to_string(), 0);
    let obj = Object::CompiledFunction(Rc::new(func));
    assert_eq!(format!("{}", obj), "<fn hello>");
}
