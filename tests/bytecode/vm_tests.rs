// Tests for the bytecode VM (13.1 - 13.6)
//
// Tests VM structure, call frames, basic instruction execution,
// variable access, and control flow execution.

use metorex::bytecode::opcode::OpCode;
use metorex::bytecode::vm::{BytecodeVm, CallFrame};
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::object::{CompiledFunction, Object};
use metorex::parser::Parser;
use std::rc::Rc;

/// Helper: compile source and execute on the bytecode VM.
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

/// Helper: compile and run, expect success.
fn run_ok(source: &str) -> Object {
    run(source).expect("execution failed")
}

// ── 13.1 Stack-Based VM Structure ────────────────────────────────────

#[test]
fn vm_new_creates_empty_vm() {
    let vm = BytecodeVm::new();
    assert_eq!(vm.stack_size(), 0);
    assert_eq!(vm.frame_count(), 0);
}

#[test]
fn vm_default_creates_empty_vm() {
    let vm = BytecodeVm::default();
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn vm_push_pop() {
    let mut vm = BytecodeVm::new();
    vm.push(Object::Int(42));
    assert_eq!(vm.stack_size(), 1);
    let val = vm.pop().unwrap();
    assert_eq!(val, Object::Int(42));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn vm_pop_empty_stack_errors() {
    let mut vm = BytecodeVm::new();
    assert!(vm.pop().is_err());
}

#[test]
fn vm_peek() {
    let mut vm = BytecodeVm::new();
    vm.push(Object::Int(1));
    vm.push(Object::Int(2));
    assert_eq!(*vm.peek(0).unwrap(), Object::Int(2));
    assert_eq!(*vm.peek(1).unwrap(), Object::Int(1));
    assert_eq!(vm.stack_size(), 2); // peek doesn't remove
}

#[test]
fn vm_peek_out_of_range_errors() {
    let vm = BytecodeVm::new();
    assert!(vm.peek(0).is_err());
}

#[test]
fn vm_globals() {
    let mut vm = BytecodeVm::new();
    vm.define_global("x".to_string(), Object::Int(10));
    assert_eq!(*vm.get_global("x").unwrap(), Object::Int(10));
    assert!(vm.get_global("y").is_none());
    assert!(vm.set_global("x", Object::Int(20)));
    assert_eq!(*vm.get_global("x").unwrap(), Object::Int(20));
    assert!(!vm.set_global("y", Object::Int(30))); // doesn't exist
}

// ── 13.2 Call Frame Implementation ──────────────────────────────────

#[test]
fn call_frame_new() {
    let func = Rc::new(CompiledFunction::new("test".to_string(), 2));
    let frame = CallFrame::new(func.clone(), 5);
    assert_eq!(frame.ip, 0);
    assert_eq!(frame.slot_offset, 5);
    assert_eq!(frame.function.name, "test");
    assert_eq!(frame.function.arity, 2);
}

#[test]
fn call_frame_read_byte() {
    let mut func = CompiledFunction::new("f".to_string(), 0);
    func.chunk.write_byte(42, 1);
    func.chunk.write_byte(99, 1);
    let mut frame = CallFrame::new(Rc::new(func), 0);
    assert_eq!(frame.read_byte(), 42);
    assert_eq!(frame.read_byte(), 99);
    assert_eq!(frame.ip, 2);
}

#[test]
fn call_frame_read_u16() {
    let mut func = CompiledFunction::new("f".to_string(), 0);
    func.chunk.write_byte(0x01, 1);
    func.chunk.write_byte(0x02, 1);
    let mut frame = CallFrame::new(Rc::new(func), 0);
    assert_eq!(frame.read_u16(), 0x0102);
    assert_eq!(frame.ip, 2);
}

#[test]
fn call_frame_read_opcode() {
    let mut func = CompiledFunction::new("f".to_string(), 0);
    func.chunk.write_opcode(OpCode::Return, 1);
    let mut frame = CallFrame::new(Rc::new(func), 0);
    assert_eq!(frame.read_opcode(), Some(OpCode::Return));
}

#[test]
fn call_frame_current_line() {
    let mut func = CompiledFunction::new("f".to_string(), 0);
    func.chunk.write_opcode(OpCode::Nil, 42);
    let mut frame = CallFrame::new(Rc::new(func), 0);
    frame.read_byte(); // advance ip
    assert_eq!(frame.current_line(), 42);
}

#[test]
fn call_frame_current_line_at_start() {
    let func = CompiledFunction::new("f".to_string(), 0);
    let frame = CallFrame::new(Rc::new(func), 0);
    assert_eq!(frame.current_line(), 0); // ip=0, returns 0
}

#[test]
fn vm_frame_push_pop() {
    let mut vm = BytecodeVm::new();
    let func = Rc::new(CompiledFunction::new("f".to_string(), 0));
    let frame = CallFrame::new(func, 0);
    vm.push_frame(frame);
    assert_eq!(vm.frame_count(), 1);
    assert!(vm.current_frame().is_ok());
    assert!(vm.current_frame_mut().is_ok());
    vm.pop_frame().unwrap();
    assert_eq!(vm.frame_count(), 0);
}

#[test]
fn vm_pop_frame_empty_errors() {
    let mut vm = BytecodeVm::new();
    assert!(vm.pop_frame().is_err());
}

#[test]
fn vm_current_frame_empty_errors() {
    let vm = BytecodeVm::new();
    assert!(vm.current_frame().is_err());
}

#[test]
fn vm_current_frame_mut_empty_errors() {
    let mut vm = BytecodeVm::new();
    assert!(vm.current_frame_mut().is_err());
}

// ── 13.3-13.4 Basic Instruction Execution ───────────────────────────

#[test]
fn execute_empty_program() {
    let result = run_ok("");
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_int_literal() {
    // "42" is an expression statement: Constant(42), Pop, Return
    // The return value is Nil (from the implicit return)
    let result = run_ok("42");
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_return_int() {
    let result = run_ok("return 42");
    assert_eq!(result, Object::Int(42));
}

#[test]
fn execute_return_float() {
    let result = run_ok("return 3.14");
    assert_eq!(result, Object::Float(3.14));
}

#[test]
fn execute_return_string() {
    let result = run_ok("return \"hello\"");
    assert_eq!(result, Object::String(Rc::new("hello".to_string())));
}

#[test]
fn execute_return_true() {
    let result = run_ok("return true");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_return_false() {
    let result = run_ok("return false");
    assert_eq!(result, Object::Bool(false));
}

#[test]
fn execute_return_nil() {
    let result = run_ok("return nil");
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_addition() {
    let result = run_ok("return 1 + 2");
    assert_eq!(result, Object::Int(3));
}

#[test]
fn execute_subtraction() {
    let result = run_ok("return 10 - 3");
    assert_eq!(result, Object::Int(7));
}

#[test]
fn execute_multiplication() {
    let result = run_ok("return 4 * 5");
    assert_eq!(result, Object::Int(20));
}

#[test]
fn execute_division() {
    let result = run_ok("return 10 / 3");
    assert_eq!(result, Object::Int(3));
}

#[test]
fn execute_modulo() {
    let result = run_ok("return 10 % 3");
    assert_eq!(result, Object::Int(1));
}

#[test]
fn execute_float_arithmetic() {
    let result = run_ok("return 1.5 + 2.5");
    assert_eq!(result, Object::Float(4.0));
}

#[test]
fn execute_string_concatenation() {
    let result = run_ok("return \"hello\" + \" world\"");
    assert_eq!(result, Object::String(Rc::new("hello world".to_string())));
}

#[test]
fn execute_negate() {
    let result = run_ok("return -42");
    assert_eq!(result, Object::Int(-42));
}

#[test]
fn execute_not() {
    let result = run_ok("return !true");
    assert_eq!(result, Object::Bool(false));
}

#[test]
fn execute_comparison_equal() {
    let result = run_ok("return 1 == 1");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_not_equal() {
    let result = run_ok("return 1 != 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_less() {
    let result = run_ok("return 1 < 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_greater() {
    let result = run_ok("return 2 > 1");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_less_equal() {
    let result = run_ok("return 1 <= 1");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_comparison_greater_equal() {
    let result = run_ok("return 2 >= 2");
    assert_eq!(result, Object::Bool(true));
}

#[test]
fn execute_division_by_zero_error() {
    let result = run("return 1 / 0");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Division by zero"));
}

#[test]
fn execute_negate_string_error() {
    let result = run("return -\"hello\"");
    assert!(result.is_err());
}

// ── 13.5 Variable Access Execution ──────────────────────────────────

#[test]
fn execute_global_variable() {
    let result = run_ok("x = 42\nreturn x");
    assert_eq!(result, Object::Int(42));
}

#[test]
fn execute_global_variable_reassignment() {
    let result = run_ok("x = 1\nx = 2\nreturn x");
    assert_eq!(result, Object::Int(2));
}

#[test]
fn execute_multiple_globals() {
    let result = run_ok("x = 10\ny = 20\nreturn x + y");
    assert_eq!(result, Object::Int(30));
}

#[test]
fn execute_undefined_variable_error() {
    let result = run("return undefined_var");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Undefined variable"));
}

// ── 13.6 Control Flow Execution ─────────────────────────────────────

#[test]
fn execute_if_true_branch() {
    let result = run_ok("if true\n  return 1\nend");
    assert_eq!(result, Object::Int(1));
}

#[test]
fn execute_if_false_skips() {
    let result = run_ok("if false\n  return 1\nend\nreturn 2");
    assert_eq!(result, Object::Int(2));
}

#[test]
fn execute_if_else() {
    let result = run_ok("if false\n  return 1\nelse\n  return 2\nend");
    assert_eq!(result, Object::Int(2));
}

#[test]
fn execute_while_loop() {
    // Use a function to test while loop: function locals work correctly
    // with SetLocal inside the while body scope.
    let result = run_ok(
        "def count\n  x = 0\n  while x < 5\n    x = x + 1\n  end\n  return x\nend\nreturn count()",
    );
    assert_eq!(result, Object::Int(5));
}

#[test]
fn execute_while_false_skips() {
    let result = run_ok("while false\n  return 1\nend\nreturn 2");
    assert_eq!(result, Object::Int(2));
}

// ── 13.7 Function Call Execution ────────────────────────────────────

#[test]
fn execute_function_definition_and_call() {
    let result = run_ok("def add(a, b)\n  return a + b\nend\nreturn add(3, 4)");
    assert_eq!(result, Object::Int(7));
}

#[test]
fn execute_function_wrong_arg_count_error() {
    let result = run("def f(a)\n  return a\nend\nreturn f(1, 2)");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Expected 1 arguments but got 2")
    );
}

#[test]
fn execute_call_non_function_error() {
    // "42()" — calling an integer
    let result = run("x = 42\nreturn x(1)");
    // This might fail at compile time or runtime depending on how the compiler handles it
    // The compiler emits Call for any call expression
    assert!(result.is_err());
}

// ── 13.10 Collection Execution ──────────────────────────────────────

#[test]
fn execute_array_literal() {
    let result = run_ok("return [1, 2, 3]");
    match result {
        Object::Array(arr) => {
            let arr = arr.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Object::Int(1));
            assert_eq!(arr[2], Object::Int(3));
        }
        _ => panic!("Expected Array, got {:?}", result),
    }
}

#[test]
fn execute_hash_literal() {
    let result = run_ok("return {\"a\" => 1, \"b\" => 2}");
    match result {
        Object::Dict(dict) => {
            let dict = dict.borrow();
            assert_eq!(dict.len(), 2);
            assert_eq!(*dict.get("a").unwrap(), Object::Int(1));
        }
        _ => panic!("Expected Dict, got {:?}", result),
    }
}

#[test]
fn execute_array_index_get() {
    let result = run_ok("a = [10, 20, 30]\nreturn a[1]");
    assert_eq!(result, Object::Int(20));
}

#[test]
fn execute_nested_arithmetic() {
    let result = run_ok("return (2 + 3) * (4 - 1)");
    assert_eq!(result, Object::Int(15));
}

// ── Mixed int/float arithmetic ──────────────────────────────────────

#[test]
fn execute_int_float_addition() {
    let result = run_ok("return 1 + 2.5");
    assert_eq!(result, Object::Float(3.5));
}

#[test]
fn execute_int_float_comparison() {
    let result = run_ok("return 1 < 2.5");
    assert_eq!(result, Object::Bool(true));
}

// ── VM: Invalid opcode handling (vm.rs lines 213-218) ──────────────────

#[test]
fn execute_invalid_opcode_errors() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    // Write an invalid opcode byte
    chunk.write_byte(0xFF, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid opcode"), "Error was: {}", err);
}

// ── VM: Return from empty frame (vm.rs lines 209, 214-217) ────────────

#[test]
fn execute_return_with_empty_stack_returns_nil() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    // Just a Return with nothing on stack: pop returns Nil via unwrap_or
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Nil);
}

// ── VM: ConstantLong execution (vm.rs lines 242-244) ───────────────────

#[test]
fn execute_constant_long() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    // Add more than 255 constants to force ConstantLong
    for i in 0..256 {
        chunk.add_constant(Object::Int(i));
    }
    let idx = chunk.add_constant(Object::Int(9999)).unwrap();
    assert!(idx > 255);
    chunk.write_constant(idx, 1); // emits ConstantLong
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Int(9999));
}

// ── VM: Negate error path (vm.rs line 265) ─────────────────────────────

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

// ── VM: SetGlobal for undefined variable (vm.rs lines 324-328) ─────────

#[test]
fn execute_set_global_undefined_variable_errors() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    // Push a value
    chunk.write_opcode(OpCode::Nil, 1);
    // Try to SetGlobal for a variable that was never defined
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

// ── VM: Jump instruction (vm.rs lines 341-342) ─────────────────────────

#[test]
fn execute_jump_skips_code() {
    use metorex::bytecode::chunk::Chunk;

    let mut chunk = Chunk::new();
    // Push 42 first
    let idx = chunk.add_constant(Object::Int(42)).unwrap();
    chunk.write_constant(idx, 1);
    // Jump forward 2 bytes (skip over the Pop instruction)
    chunk.write_op_u16(OpCode::Jump, 1, 1);
    // This Pop should be skipped
    chunk.write_opcode(OpCode::Pop, 1);
    // Return (should return 42 since Pop was skipped)
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Int(42));
}

// ── VM: Float negate ───────────────────────────────────────────────────

#[test]
fn execute_negate_float() {
    let result = run_ok("return -2.5");
    assert_eq!(result, Object::Float(-2.5));
}

// ── VM: Mixed type arithmetic errors ───────────────────────────────────

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

// ── Float-Int mixed arithmetic (covers reverse operand paths) ───────

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

// ── Additional type error paths ─────────────────────────────────────

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

// ── IndexSet and Hash with non-string key ───────────────────────────

#[test]
fn execute_array_index_set() {
    let result = run_ok("a = [1, 2, 3]\na[0] = 99\nreturn a[0]");
    assert_eq!(result, Object::Int(99));
}

#[test]
fn execute_hash_with_non_string_key() {
    // Hash keys that aren't strings get to_string'd
    let chunk = {
        let tokens = Lexer::new("return {1 => \"one\"}").tokenize();
        let stmts = Parser::new(tokens)
            .parse()
            .map_err(|errs| {
                errs.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap();
        Compiler::new().compile(&stmts).unwrap()
    };
    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
}

// ── Index type error ────────────────────────────────────────────────

#[test]
fn execute_index_get_type_error() {
    let result = run("return 42[0]");
    assert!(result.is_err());
}

#[test]
fn execute_index_set_out_of_bounds() {
    let result = run("a = [1]\na[5] = 99");
    assert!(result.is_err());
}

#[test]
fn execute_index_set_type_error() {
    let result = run("a = 42\na[0] = 1");
    assert!(result.is_err());
}

// ── Dict index operations ───────────────────────────────────────────

#[test]
fn execute_dict_index_get() {
    let result = run_ok("h = {\"a\" => 1}\nreturn h[\"a\"]");
    assert_eq!(result, Object::Int(1));
}

#[test]
fn execute_dict_index_get_missing() {
    let result = run_ok("h = {\"a\" => 1}\nreturn h[\"z\"]");
    assert_eq!(result, Object::Nil);
}

#[test]
fn execute_dict_index_set() {
    let result = run_ok("h = {\"a\" => 1}\nh[\"b\"] = 2\nreturn h[\"b\"]");
    assert_eq!(result, Object::Int(2));
}

// ── Array negative index ────────────────────────────────────────────

#[test]
fn execute_array_negative_index() {
    let result = run_ok("a = [10, 20, 30]\nreturn a[-1]");
    assert_eq!(result, Object::Int(30));
}
