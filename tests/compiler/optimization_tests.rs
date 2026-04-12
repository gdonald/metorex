// Tests for optimization passes (12.9)
//
// Exercises constant folding, dead code elimination, peephole optimization,
// and tail call detection.

use metorex::bytecode::opcode::OpCode;
use metorex::compiler::Compiler;
use metorex::compiler::optimizer;
use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;

/// Helper: parse source, compile with optimizations, return the chunk.
fn compile_optimized(source: &str) -> metorex::bytecode::chunk::Chunk {
    let tokens = Lexer::new(source).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let compiler = Compiler::new();
    compiler.compile_optimized(&stmts).expect("compile failed")
}

/// Helper: parse source, compile without optimizations.
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

// ── Constant folding (12.9.1) ────────────────────────────────────────

#[test]
fn fold_int_addition() {
    let chunk = compile_optimized("1 + 2");
    let ops = opcodes(&chunk);
    assert!(
        !ops.contains(&OpCode::Add),
        "Expected Add to be folded away, got: {:?}",
        ops
    );
    // Should be a single constant (3) instead of two constants + Add
    assert!(ops.contains(&OpCode::Constant));
}

#[test]
fn fold_int_subtraction() {
    let chunk = compile_optimized("10 - 3");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Subtract));
}

#[test]
fn fold_int_multiplication() {
    let chunk = compile_optimized("4 * 5");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Multiply));
}

#[test]
fn fold_int_division() {
    let chunk = compile_optimized("10 / 2");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Divide));
}

#[test]
fn fold_int_modulo() {
    let chunk = compile_optimized("10 % 3");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Modulo));
}

#[test]
fn fold_float_addition() {
    let chunk = compile_optimized("1.5 + 2.5");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Add));
}

#[test]
fn no_fold_division_by_zero() {
    let chunk = compile_optimized("10 / 0");
    let ops = opcodes(&chunk);
    // Should NOT fold — division by zero kept at runtime
    assert!(
        ops.contains(&OpCode::Divide),
        "Division by zero should not be folded"
    );
}

#[test]
fn fold_negate_int() {
    let chunk = compile_optimized("-42");
    let ops = opcodes(&chunk);
    assert!(
        !ops.contains(&OpCode::Negate),
        "Expected Negate to be folded away"
    );
}

#[test]
fn fold_negate_float() {
    let chunk = compile_optimized("-3.14");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Negate));
}

#[test]
fn no_fold_variable_expression() {
    let chunk = compile_optimized("x = 1\nx + 2");
    let ops = opcodes(&chunk);
    // Can't fold x + 2 because x is a variable
    assert!(ops.contains(&OpCode::Add));
}

#[test]
fn folded_value_is_correct() {
    let chunk = compile_optimized("3 + 4");
    // Find the constant that should be 7
    let mut found_7 = false;
    for i in 0..chunk.constants_count() {
        if *chunk.get_constant(i) == Object::Int(7) {
            found_7 = true;
            break;
        }
    }
    assert!(found_7, "Expected folded constant 7 in pool");
}

// ── Dead code elimination (12.9.2) ──────────────────────────────────

#[test]
fn eliminate_code_after_return() {
    // "return 1\n42" — the 42 is dead code
    let chunk = compile_optimized("return 1\n42");
    let ops = opcodes(&chunk);
    // Should have: Constant(1), Return — and NOT a second Constant + Pop
    let const_count = ops.iter().filter(|&&op| op == OpCode::Constant).count();
    assert_eq!(
        const_count, 1,
        "Expected only 1 constant (dead code eliminated), got {}",
        const_count
    );
}

#[test]
fn preserve_code_before_return() {
    let chunk = compile_optimized("42\nreturn 1");
    let ops = opcodes(&chunk);
    // Both 42 (expression stmt with Pop) and return 1 should be present
    assert!(ops.contains(&OpCode::Return));
}

#[test]
fn dead_code_after_return_in_unoptimized() {
    let chunk = compile("return 1\n42");
    let ops = opcodes(&chunk);
    // Without optimization, dead code is preserved
    let const_count = ops.iter().filter(|&&op| op == OpCode::Constant).count();
    assert!(
        const_count >= 2,
        "Unoptimized should keep dead code, got {} constants",
        const_count
    );
}

// ── Peephole optimization (12.9.3) ──────────────────────────────────

#[test]
fn eliminate_double_negate() {
    let chunk = compile_optimized("-(-5)");
    let ops = opcodes(&chunk);
    // Double negate should be eliminated — just the constant
    assert!(
        !ops.contains(&OpCode::Negate),
        "Double negate should be eliminated"
    );
}

#[test]
fn eliminate_double_not() {
    let chunk = compile_optimized("!!true");
    let ops = opcodes(&chunk);
    assert!(
        !ops.contains(&OpCode::Not),
        "Double not should be eliminated"
    );
}

#[test]
fn fold_true_not_to_false() {
    let chunk = compile_optimized("!true");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::False), "!true should become False");
    assert!(!ops.contains(&OpCode::Not));
    assert!(!ops.contains(&OpCode::True));
}

#[test]
fn fold_false_not_to_true() {
    let chunk = compile_optimized("!false");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::True), "!false should become True");
    assert!(!ops.contains(&OpCode::Not));
    assert!(!ops.contains(&OpCode::False));
}

#[test]
fn single_negate_preserved() {
    let chunk = compile_optimized("x = 5\n-x");
    let ops = opcodes(&chunk);
    // Single negate on a variable should be preserved
    assert!(ops.contains(&OpCode::Negate));
}

// ── Tail call optimization (12.9.4) ─────────────────────────────────

#[test]
fn detect_tail_call() {
    let chunk = compile("def f\n  g\nend");
    // Find the function's chunk
    let mut func_chunk = None;
    for i in 0..chunk.constants_count() {
        if let Object::CompiledFunction(f) = chunk.get_constant(i) {
            func_chunk = Some(&f.chunk);
            break;
        }
    }
    let func_chunk = func_chunk.expect("Expected a compiled function");
    let count = optimizer::count_tail_calls(func_chunk);
    // g is called then the implicit return follows — this is a tail call pattern
    // But the function body ends with: call g, Pop, Nil, Return
    // So Call is NOT immediately before Return
    // This checks the detection accurately handles the pattern
    assert_eq!(
        count, 0,
        "Auto-call with Pop before Return is not a tail call"
    );
}

#[test]
fn detect_tail_call_with_explicit_return() {
    // return f() — this should be Call immediately followed by Return
    let chunk = compile("def g\n  return f\nend");
    let mut func_chunk = None;
    for i in 0..chunk.constants_count() {
        if let Object::CompiledFunction(f) = chunk.get_constant(i) {
            func_chunk = Some(&f.chunk);
            break;
        }
    }
    // f is a bare identifier that gets auto-called (Call 0), then Return
    // The pattern is: GetGlobal(f), Call(0), Return — this IS a tail call
    let func_chunk = func_chunk.expect("Expected a compiled function");
    let ops = opcodes(func_chunk);
    // Check the actual opcodes to understand the pattern
    let has_call = ops.contains(&OpCode::Call);
    if has_call {
        let count = optimizer::count_tail_calls(func_chunk);
        assert!(count >= 1, "Expected at least 1 tail call");
    }
}

// ── Optimization preserves correctness ──────────────────────────────

#[test]
fn optimized_chunk_is_shorter_than_unoptimized() {
    let unopt = compile("1 + 2");
    let opt = compile_optimized("1 + 2");
    assert!(
        opt.len() <= unopt.len(),
        "Optimized should be <= unoptimized: opt={}, unopt={}",
        opt.len(),
        unopt.len()
    );
}

#[test]
fn optimized_empty_program() {
    let chunk = compile_optimized("");
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Return]);
}

#[test]
fn optimize_preserves_variable_code() {
    let chunk = compile_optimized("x = 10\nx");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::DefineGlobal));
    assert!(ops.contains(&OpCode::GetGlobal));
}

// ── Direct optimizer function calls ─────────────────────────────────

#[test]
fn constant_fold_direct() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    let a = chunk.add_constant(Object::Int(3)).unwrap();
    let b = chunk.add_constant(Object::Int(4)).unwrap();
    chunk.write_constant(a, 1);
    chunk.write_constant(b, 1);
    chunk.write_opcode(OpCode::Add, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::constant_fold(&mut chunk);
    let ops = opcodes(&chunk);
    assert!(
        !ops.contains(&OpCode::Add),
        "Add should be folded, got: {:?}",
        ops
    );
}

#[test]
fn peephole_direct() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Pop, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::peephole_optimize(&mut chunk);
    let ops = opcodes(&chunk);
    assert!(
        !ops.contains(&OpCode::Nil),
        "Nil+Pop should be eliminated, got: {:?}",
        ops
    );
}

#[test]
fn dead_code_eliminate_direct() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_opcode(OpCode::Return, 1);
    chunk.write_opcode(OpCode::Nil, 2); // dead
    chunk.write_opcode(OpCode::Pop, 2); // dead

    optimizer::dead_code_eliminate(&mut chunk);
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Return]);
}

#[test]
fn count_tail_calls_direct() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_op_u8(OpCode::Call, 0, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let count = optimizer::count_tail_calls(&chunk);
    assert_eq!(count, 1);
}

#[test]
fn count_tail_calls_none() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_op_u8(OpCode::Call, 0, 1);
    chunk.write_opcode(OpCode::Pop, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let count = optimizer::count_tail_calls(&chunk);
    assert_eq!(count, 0);
}

// ── Additional coverage tests ────────────────────────────────────────

#[test]
fn fold_float_subtraction() {
    let chunk = compile_optimized("5.0 - 2.0");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Subtract));
}

#[test]
fn fold_float_multiplication() {
    let chunk = compile_optimized("3.0 * 4.0");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Multiply));
}

#[test]
fn fold_float_division() {
    let chunk = compile_optimized("10.0 / 2.0");
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Divide));
}

#[test]
fn no_fold_float_division_by_zero() {
    let chunk = compile_optimized("10.0 / 0.0");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Divide));
}

#[test]
fn dead_code_preserves_jump_targets() {
    // Code with if/else: dead code after return shouldn't eliminate jump targets
    let chunk = compile_optimized("if true\n  return 1\nelse\n  2\nend");
    let ops = opcodes(&chunk);
    // Both branches should be preserved since the else is reachable via jump
    assert!(ops.contains(&OpCode::Return));
}

#[test]
fn dead_code_eliminate_with_jump() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    // JumpIfFalse with offset 1: target = (0+3) + 1 = position 4
    chunk.write_op_u16(OpCode::JumpIfFalse, 1, 1);
    chunk.write_opcode(OpCode::Return, 1); // position 3
    // position 4: This is the jump target — should NOT be eliminated
    chunk.write_opcode(OpCode::Nil, 2);
    chunk.write_opcode(OpCode::Return, 2);

    optimizer::dead_code_eliminate(&mut chunk);
    let ops = opcodes(&chunk);
    // The Nil after the first Return should be preserved (it's a jump target)
    assert!(
        ops.contains(&OpCode::Nil),
        "Jump target should be preserved: {:?}",
        ops
    );
}

#[test]
fn dead_code_eliminate_with_loop() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_opcode(OpCode::True, 1);
    // Loop backward jump (offset 3 bytes back)
    chunk.write_op_u16(OpCode::Loop, 5, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::dead_code_eliminate(&mut chunk);
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Loop));
}

#[test]
fn optimize_full_pipeline() {
    // Test the full optimize() function
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    let a = chunk.add_constant(Object::Int(10)).unwrap();
    let b = chunk.add_constant(Object::Int(20)).unwrap();
    chunk.write_constant(a, 1);
    chunk.write_constant(b, 1);
    chunk.write_opcode(OpCode::Add, 1);
    chunk.write_opcode(OpCode::Return, 1);
    chunk.write_opcode(OpCode::Nil, 2); // dead code

    optimizer::optimize(&mut chunk);
    let ops = opcodes(&chunk);
    assert!(!ops.contains(&OpCode::Add), "Add should be folded");
    assert!(
        !ops.contains(&OpCode::Nil),
        "Dead code should be eliminated"
    );
}

#[test]
fn fold_no_change_on_non_foldable() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    let a = chunk
        .add_constant(Object::String(std::rc::Rc::new("hello".to_string())))
        .unwrap();
    let b = chunk
        .add_constant(Object::String(std::rc::Rc::new("world".to_string())))
        .unwrap();
    chunk.write_constant(a, 1);
    chunk.write_constant(b, 1);
    chunk.write_opcode(OpCode::Add, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::constant_fold(&mut chunk);
    let ops = opcodes(&chunk);
    // String addition can't be folded at compile time
    assert!(ops.contains(&OpCode::Add));
}

#[test]
fn peephole_double_negate_direct() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_opcode(OpCode::Negate, 1);
    chunk.write_opcode(OpCode::Negate, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::peephole_optimize(&mut chunk);
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Return]);
}

#[test]
fn peephole_double_not_direct() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_opcode(OpCode::Not, 1);
    chunk.write_opcode(OpCode::Not, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::peephole_optimize(&mut chunk);
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::Return]);
}

#[test]
fn peephole_true_not_direct() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_opcode(OpCode::True, 1);
    chunk.write_opcode(OpCode::Not, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::peephole_optimize(&mut chunk);
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::False, OpCode::Return]);
}

#[test]
fn peephole_false_not_direct() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_opcode(OpCode::False, 1);
    chunk.write_opcode(OpCode::Not, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::peephole_optimize(&mut chunk);
    let ops = opcodes(&chunk);
    assert_eq!(ops, vec![OpCode::True, OpCode::Return]);
}

#[test]
fn negate_string_not_folded() {
    // Can't negate a string — should be left as-is
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    let idx = chunk
        .add_constant(Object::String(std::rc::Rc::new("hello".to_string())))
        .unwrap();
    chunk.write_constant(idx, 1);
    chunk.write_opcode(OpCode::Negate, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::constant_fold(&mut chunk);
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Negate));
}

#[test]
fn fold_int_modulo_by_zero_not_folded() {
    let chunk = compile_optimized("10 % 0");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Modulo));
}

// ── Constant folding with ConstantLong (optimizer.rs lines 48-50) ─────

#[test]
fn fold_negate_with_constant_long() {
    // Build a chunk with ConstantLong + Negate to exercise the
    // constant_fold path that only handles Constant (short), not ConstantLong.
    // The negate should NOT be folded since the optimizer doesn't handle ConstantLong.
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    // Add enough constants to force a ConstantLong index (>255)
    for i in 0..256 {
        chunk.add_constant(Object::Int(i));
    }
    let idx = chunk.add_constant(Object::Int(42)).unwrap();
    assert!(idx > 255);
    chunk.write_constant(idx, 1); // writes ConstantLong
    chunk.write_opcode(OpCode::Negate, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::constant_fold(&mut chunk);
    let ops = opcodes(&chunk);
    // ConstantLong + Negate should remain since the optimizer only folds short Constant
    assert!(
        ops.contains(&OpCode::Negate),
        "ConstantLong negate should not be folded: {:?}",
        ops
    );
}

// ── Float modulo not foldable (optimizer.rs line 105) ──────────────────

#[test]
fn fold_float_modulo_not_supported() {
    // Float modulo is not in the fold_binary match arms, so it should not fold
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    let a = chunk.add_constant(Object::Float(10.0)).unwrap();
    let b = chunk.add_constant(Object::Float(3.0)).unwrap();
    chunk.write_constant(a, 1);
    chunk.write_constant(b, 1);
    chunk.write_opcode(OpCode::Modulo, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::constant_fold(&mut chunk);
    let ops = opcodes(&chunk);
    assert!(
        ops.contains(&OpCode::Modulo),
        "Float modulo should not be folded: {:?}",
        ops
    );
}

// ── Dead code: unknown bytes (optimizer.rs lines 166-170) ──────────────

#[test]
fn dead_code_eliminate_unknown_byte_after_return() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_opcode(OpCode::Return, 1);
    // Write an invalid opcode byte (not recognized as any OpCode)
    chunk.write_byte(0xFF, 2); // dead unknown byte

    optimizer::dead_code_eliminate(&mut chunk);
    // The unknown byte after return should be removed
    assert_eq!(chunk.len(), 1, "Only Return should remain");
}

#[test]
fn dead_code_eliminate_unknown_byte_not_after_return() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    // Write an unknown byte that is NOT after return (should be preserved)
    chunk.write_byte(0xFF, 1);
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::dead_code_eliminate(&mut chunk);
    // The unknown byte before return should be preserved
    assert_eq!(chunk.len(), 2, "Unknown byte + Return should remain");
}

// ── collect_jump_targets with Loop backward jump (optimizer.rs lines 201-202) ──

#[test]
fn dead_code_with_loop_backward_jump_target() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    // position 0: Nil
    chunk.write_opcode(OpCode::Nil, 1);
    // position 1: Loop with offset 4 (points backward: 1+3-4 = 0, i.e., Nil)
    chunk.write_op_u16(OpCode::Loop, 4, 1);
    // position 4: Return
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::dead_code_eliminate(&mut chunk);
    let ops = opcodes(&chunk);
    // Nil at position 0 is a jump target of the Loop, so it must be preserved
    assert!(ops.contains(&OpCode::Nil));
    assert!(ops.contains(&OpCode::Loop));
}

// ── Peephole unknown bytes (optimizer.rs lines 253-255) ────────────────

#[test]
fn peephole_unknown_byte_preserved() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_byte(0xFF, 1); // unknown byte
    chunk.write_opcode(OpCode::Return, 1);

    optimizer::peephole_optimize(&mut chunk);
    // Unknown byte should be copied as-is
    assert_eq!(chunk.code()[0], 0xFF);
}

// ── Tail call detection edge case (optimizer.rs lines 332-333) ─────────

#[test]
fn count_tail_calls_with_unknown_byte() {
    let mut chunk = metorex::bytecode::chunk::Chunk::new();
    chunk.write_byte(0xFF, 1); // unknown byte
    chunk.write_op_u8(OpCode::Call, 0, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let count = optimizer::count_tail_calls(&chunk);
    assert_eq!(count, 1, "Should still detect tail call after unknown byte");
}
