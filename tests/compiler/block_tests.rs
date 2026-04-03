// Tests for block/lambda compilation (12.8)
//
// Exercises compiling blocks as closures, capturing block environment,
// OP_CLOSURE for blocks, and storing block objects in constant pool.

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

/// Helper: find all compiled functions recursively.
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
                find_compiled_functions_recursive(&func.chunk, funcs);
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }
}

// ── Compile blocks as closures (12.8.1) ─────────────────────────────

#[test]
fn compile_simple_lambda() {
    let chunk = compile("lambda do 42 end");
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    assert_eq!(block.arity, 0);
}

#[test]
fn compile_lambda_with_parameter() {
    let chunk = compile("lambda do |x| x end");
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    assert_eq!(block.arity, 1);
}

#[test]
fn compile_lambda_with_multiple_parameters() {
    let chunk = compile("lambda do |a, b| a end");
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    assert_eq!(block.arity, 2);
}

#[test]
fn block_body_has_return() {
    let chunk = compile("lambda do 42 end");
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    let body_ops = opcodes(&block.chunk);
    assert!(body_ops.contains(&OpCode::Return));
}

// ── Capture block environment (12.8.2) ──────────────────────────────

#[test]
fn block_captures_enclosing_variable() {
    // x is defined in outer function, block captures it
    let source = "def outer\n  x = 10\n  lambda do x end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    let body_ops = opcodes(&block.chunk);
    assert!(
        body_ops.contains(&OpCode::GetUpvalue),
        "Block should capture x via GetUpvalue, got: {:?}",
        body_ops
    );
}

#[test]
fn block_captures_function_parameter() {
    let source = "def outer(val)\n  lambda do val end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    let body_ops = opcodes(&block.chunk);
    assert!(body_ops.contains(&OpCode::GetUpvalue));
}

#[test]
fn block_without_captures_no_closure_op() {
    let chunk = compile("lambda do 42 end");
    let ops = opcodes(&chunk);
    assert!(
        !ops.contains(&OpCode::Closure),
        "No OP_CLOSURE for block without captures"
    );
}

// ── OP_CLOSURE for blocks (12.8.3) ──────────────────────────────────

#[test]
fn block_with_captures_emits_closure() {
    let source = "def outer\n  x = 1\n  lambda do x end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let outer = funcs.iter().find(|f| f.name == "outer").unwrap();
    let outer_ops = opcodes(&outer.chunk);
    assert!(
        outer_ops.contains(&OpCode::Closure),
        "Expected OP_CLOSURE for block capturing variables, got: {:?}",
        outer_ops
    );
}

// ── Store block in constant pool (12.8.4) ───────────────────────────

#[test]
fn block_stored_as_compiled_function() {
    let chunk = compile("lambda do |x| x + 1 end");
    let funcs = find_compiled_functions(&chunk);
    assert!(
        funcs.iter().any(|f| f.name == "<block>"),
        "Expected a <block> CompiledFunction in constant pool"
    );
}

#[test]
fn block_body_contains_correct_bytecode() {
    let chunk = compile("lambda do |a, b| a + b end");
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    let body_ops = opcodes(&block.chunk);
    assert!(
        body_ops.contains(&OpCode::Add),
        "Block body should contain Add"
    );
    assert!(
        body_ops.contains(&OpCode::GetLocal),
        "Block body should access parameters via GetLocal"
    );
}

// ── Block with control flow in body ─────────────────────────────────

#[test]
fn block_with_if_in_body() {
    let source = "lambda do |x| if x\n    1\n  else\n    2\n  end end";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    let body_ops = opcodes(&block.chunk);
    assert!(body_ops.contains(&OpCode::JumpIfFalse));
}

// ── Block as argument to method ─────────────────────────────────────

#[test]
fn block_as_expression_statement() {
    // Block as a standalone expression (gets popped)
    let chunk = compile("lambda do 42 end");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Pop));
}

// ── Multiple blocks ─────────────────────────────────────────────────

#[test]
fn multiple_blocks_in_same_scope() {
    let chunk = compile("lambda do 1 end\nlambda do 2 end");
    let funcs = find_compiled_functions(&chunk);
    let block_count = funcs.iter().filter(|f| f.name == "<block>").count();
    assert_eq!(block_count, 2, "Expected 2 blocks");
}

// ── Block with assignment ───────────────────────────────────────────

#[test]
fn block_with_local_assignment() {
    let chunk = compile("lambda do\n  x = 42\n  x\nend");
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    let body_ops = opcodes(&block.chunk);
    assert!(body_ops.contains(&OpCode::GetLocal));
}

// ── Block captures multiple variables ───────────────────────────────

#[test]
fn block_captures_multiple_variables() {
    let source = "def f\n  a = 1\n  b = 2\n  lambda do a + b end\nend";
    let chunk = compile(source);
    let funcs = find_compiled_functions(&chunk);
    let block = funcs.iter().find(|f| f.name == "<block>").unwrap();
    let body_ops = opcodes(&block.chunk);
    let upvalue_count = body_ops
        .iter()
        .filter(|&&op| op == OpCode::GetUpvalue)
        .count();
    assert_eq!(upvalue_count, 2, "Expected 2 upvalue reads for a and b");
}
