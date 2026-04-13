// Tests for bytecode VM error paths, invalid opcodes, and mixed-type arithmetic.

use metorex::bytecode::opcode::OpCode;
use metorex::bytecode::vm::BytecodeVm;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use std::rc::Rc;

fn run(source: &str) -> Result<Object, String> {
    let tokens = Lexer::new(source).tokenize();
    let stmts = Parser::new(tokens).parse().map_err(|errs| {
        errs.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    })?;
    let compiler = Compiler::new();
    let chunk = compiler.compile(&stmts).map_err(|e| e.to_string())?;
    let mut vm = BytecodeVm::new();
    vm.execute(&chunk).map_err(|e| e.to_string())
}

fn run_ok(source: &str) -> Object {
    run(source).expect("execution failed")
}

// ── VM: Invalid opcode handling ─────────────────────────────────────

#[test]
fn execute_invalid_opcode_errors() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    chunk.write_byte(0xFF, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid opcode"), "Error was: {}", err);
}

// ── VM: Return from empty frame ─────────────────────────────────────

#[test]
fn execute_return_with_empty_stack_returns_nil() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Nil);
}

// ── VM: ConstantLong execution ──────────────────────────────────────

#[test]
fn execute_constant_long() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    for i in 0..256 {
        chunk.add_constant(Object::Int(i));
    }
    let idx = chunk.add_constant(Object::Int(9999)).unwrap();
    assert!(idx > 255);
    chunk.write_constant(idx, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Int(9999));
}

// ── VM: Negate errors ────────────────────────────────────────────────

#[test]
fn execute_negate_non_numeric_errors() {
    let result = run("return -\"hello\"");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Cannot negate"), "Error was: {}", err);
}

#[test]
fn execute_negate_bool_errors() {
    let result = run("return -true");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Cannot negate"), "Error was: {}", err);
}

#[test]
fn execute_negate_float() {
    let result = run_ok("return -2.5");
    assert_eq!(result, Object::Float(-2.5));
}

// ── VM: SetGlobal for undefined variable ───────────────────────────

#[test]
fn execute_set_global_undefined_variable_errors() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Nil, 1);
    let idx = chunk
        .add_constant(Object::String(Rc::new("undefined_var".to_string())))
        .unwrap();
    chunk.write_op_u8(OpCode::SetGlobal, idx as u8, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Undefined variable"), "Error was: {}", err);
}

// ── VM: Jump instruction ────────────────────────────────────────────

#[test]
fn execute_jump_skips_code() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    let idx = chunk.add_constant(Object::Int(42)).unwrap();
    chunk.write_constant(idx, 1);
    chunk.write_op_u16(OpCode::Jump, 1, 1);
    chunk.write_opcode(OpCode::Pop, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Int(42));
}

// ── VM: Empty chunk returns nil ─────────────────────────────────────

#[test]
fn execute_empty_chunk_past_end_returns_nil() {
    use metorex::bytecode::chunk::Chunk;

    let chunk = Chunk::new();
    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Nil);
}

// ── VM: Exception placeholder opcodes ──────────────────────────────

#[test]
fn execute_try_opcode_is_noop() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    chunk.write_op_u16(OpCode::Try, 0, 1);
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Return, 1);
    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_catch_opcode_is_noop() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Catch, 1);
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Return, 1);
    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_match_opcode_is_noop() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::Match, 1);
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Return, 1);
    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_match_pattern_opcode_is_noop() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    chunk.write_op_u16(OpCode::MatchPattern, 0, 1);
    chunk.write_opcode(OpCode::Nil, 1);
    chunk.write_opcode(OpCode::Return, 1);
    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Nil);
}

// ── Mixed type arithmetic errors ────────────────────────────────────

#[test]
fn execute_add_type_error() {
    let result = run("return true + 1");
    assert!(result.is_err());
}

#[test]
fn execute_subtract_type_error() {
    let result = run("return \"hello\" - 1");
    assert!(result.is_err());
}

#[test]
fn execute_multiply_type_error() {
    let result = run("return nil * 1");
    assert!(result.is_err());
}

#[test]
fn execute_modulo_by_zero_error() {
    let result = run("return 10 % 0");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Modulo by zero"), "Error was: {}", err);
}

#[test]
fn execute_modulo_type_error() {
    let result = run("return \"hello\" % 1");
    assert!(result.is_err());
}

#[test]
fn execute_compare_type_error() {
    let result = run("return \"hello\" < 1");
    assert!(result.is_err());
}

#[test]
fn execute_compare_greater_type_error() {
    let result = run("return \"hello\" > 1");
    assert!(result.is_err());
}

#[test]
fn execute_compare_less_equal_type_error() {
    let result = run("return \"hello\" <= 1");
    assert!(result.is_err());
}

#[test]
fn execute_compare_greater_equal_type_error() {
    let result = run("return \"hello\" >= 1");
    assert!(result.is_err());
}

#[test]
fn execute_bool_add_error() {
    let result = run("return true + true");
    assert!(result.is_err());
}

#[test]
fn execute_nil_subtract_error() {
    let result = run("return nil - 1");
    assert!(result.is_err());
}

#[test]
fn execute_divide_type_error() {
    let result = run("return \"hello\" / 1");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Cannot divide"), "Error was: {}", err);
}

// ── Float-Int mixed arithmetic ──────────────────────────────────────

#[test]
fn execute_float_plus_int() {
    let result = run_ok("return 2.5 + 1");
    assert_eq!(result, Object::Float(3.5));
}

#[test]
fn execute_float_minus_int() {
    let result = run_ok("return 5.5 - 2");
    assert_eq!(result, Object::Float(3.5));
}

#[test]
fn execute_float_times_int() {
    let result = run_ok("return 2.5 * 4");
    assert_eq!(result, Object::Float(10.0));
}

#[test]
fn execute_float_div_int() {
    let result = run_ok("return 10.0 / 4");
    assert_eq!(result, Object::Float(2.5));
}

#[test]
fn execute_float_div_int_zero() {
    let result = run("return 10.0 / 0");
    assert!(result.is_err());
}

#[test]
fn execute_int_div_float_zero() {
    let result = run("return 10 / 0.0");
    assert!(result.is_err());
}

#[test]
fn execute_float_div_float_zero() {
    let result = run("return 10.0 / 0.0");
    assert!(result.is_err());
}

#[test]
fn execute_int_float_division() {
    let result = run_ok("return 10 / 2.5");
    assert_eq!(result, Object::Float(4.0));
}

// ── Float comparisons ──────────────────────────────────────────────

#[test]
fn execute_float_less_int() {
    let result = run_ok("return 1.5 < 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_float_greater_int() {
    let result = run_ok("return 2.5 > 1");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_float_less_equal_int() {
    let result = run_ok("return 2.0 <= 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_float_greater_equal_int() {
    let result = run_ok("return 2.0 >= 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_float_float_comparison() {
    let result = run_ok("return 1.5 < 2.5");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_float_greater_float() {
    let result = run_ok("return 3.5 > 2.5");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_float_less_equal_float() {
    let result = run_ok("return 2.5 <= 2.5");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_float_greater_equal_float() {
    let result = run_ok("return 3.5 >= 2.5");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_float_float_subtraction() {
    let result = run_ok("return 5.0 - 2.0");
    assert_eq!(result, Object::Float(3.0));
}

#[test]
fn execute_float_float_multiplication() {
    let result = run_ok("return 2.0 * 3.0");
    assert_eq!(result, Object::Float(6.0));
}

#[test]
fn execute_float_float_division() {
    let result = run_ok("return 10.0 / 2.0");
    assert_eq!(result, Object::Float(5.0));
}

#[test]
fn execute_int_less_float() {
    let result = run_ok("return 1 < 2.5");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_int_greater_float() {
    let result = run_ok("return 3 > 2.5");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_int_less_equal_float() {
    let result = run_ok("return 2 <= 2.0");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_int_greater_equal_float() {
    let result = run_ok("return 2 >= 2.0");
    assert_eq!(result, Object::Bool(true));
}
