// Tests for runtime method definition in the bytecode VM (14.1)

use metorex::bytecode::vm::BytecodeVm;
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;

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

fn run_ok(source: &str) -> Object {
    run(source).expect("execution failed")
}

// ── Native function registration ────────────────────────────────────

#[test]
fn native_puts_is_registered() {
    let mut vm = BytecodeVm::new();
    vm.register_natives();
    assert!(vm.get_global("puts").is_some());
    assert!(vm.get_global("print").is_some());
    assert!(vm.get_global("p").is_some());
    assert!(vm.get_global("define_method").is_some());
}

// ── Calling native puts ─────────────────────────────────────────────

#[test]
fn native_puts_callable() {
    // puts(42) should not error
    let result = run("puts(42)");
    assert!(result.is_ok());
}

#[test]
fn native_print_callable() {
    let result = run("print(42)");
    assert!(result.is_ok());
}

#[test]
fn native_p_callable() {
    let result = run("p(42)");
    assert!(result.is_ok());
}

// ── Class instantiation via Call ────────────────────────────────────

#[test]
fn class_callable_creates_instance() {
    let result = run_ok("class Foo\nend\nreturn Foo()");
    assert!(
        matches!(result, Object::Instance(_)),
        "Expected Instance, got {:?}",
        result
    );
}

#[test]
fn class_callable_with_initialize() {
    // Initialize without instance vars (those need frame slot adjustment)
    let result =
        run_ok("class Counter\n  def initialize(n)\n    return n\n  end\nend\nreturn Counter(42)");
    // Counter(42) calls initialize, then returns the instance
    assert!(matches!(result, Object::Instance(_)));
}

// ── define_method error paths ───────────────────────────────────────

#[test]
fn define_method_no_args_error() {
    let result = run("define_method()");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("requires at least one argument")
    );
}

#[test]
fn define_method_non_string_name_error() {
    let result = run("define_method(42, 1)");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("first argument must be a String or Symbol")
    );
}

#[test]
fn define_method_missing_function_error() {
    let result = run("define_method(\"foo\")");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("requires a function as second argument")
    );
}

#[test]
fn define_method_non_function_body_error() {
    let result = run("define_method(\"foo\", 42)");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("second argument must be a function")
    );
}

// ── define_method with Symbol name (exercises args[0] = Symbol path) ─────

#[test]
fn define_method_symbol_name_requires_function_arg() {
    // Symbol path at line 45 of natives.rs — passing :name but no function.
    let result = run("define_method(:foo)");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("requires a function"));
}

// ── define_method happy path through natives dispatch ──────────────────

#[test]
fn define_method_with_compiled_function_succeeds_at_top_level() {
    use metorex::bytecode::chunk::Chunk;
    use metorex::bytecode::opcode::OpCode;
    use metorex::object::CompiledFunction;
    use std::rc::Rc;

    // Hand-craft a chunk that:
    //   push NativeFunction("define_method")
    //   push Symbol("m")
    //   push CompiledFunction (empty body returning nil)
    //   Call(2)
    //   Return
    let mut chunk = Chunk::new();
    let idx_native = chunk
        .add_constant(Object::NativeFunction("define_method".to_string()))
        .unwrap();
    let idx_sym = chunk
        .add_constant(Object::Symbol(Rc::new("m".to_string())))
        .unwrap();
    // An empty inner chunk for the function body.
    let mut inner = Chunk::new();
    let nil_idx = inner.add_constant(Object::Nil).unwrap();
    inner.write_constant(nil_idx, 1);
    inner.write_opcode(OpCode::Return, 1);
    let func = Rc::new(CompiledFunction {
        name: "m".to_string(),
        arity: 0,
        chunk: inner,
    });
    let idx_fn = chunk.add_constant(Object::CompiledFunction(func)).unwrap();

    chunk.write_constant(idx_native, 1);
    chunk.write_constant(idx_sym, 1);
    chunk.write_constant(idx_fn, 1);
    chunk.write_op_u8(OpCode::Call, 2, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    // The natives.rs define_method has no class on stack so it stores as
    // a global function and returns Nil.
    assert!(result.is_ok(), "define_method failed: {:?}", result);
}

// ── Unknown native function ─────────────────────────────────────────

#[test]
fn unknown_native_function_error() {
    // Create a NativeFunction manually and try to call it
    use metorex::bytecode::chunk::Chunk;
    use metorex::bytecode::opcode::OpCode;

    let mut chunk = Chunk::new();
    // Push NativeFunction("unknown") then call it
    let idx = chunk
        .add_constant(Object::NativeFunction("unknown_fn".to_string()))
        .unwrap();
    chunk.write_constant(idx, 1);
    chunk.write_op_u8(OpCode::Call, 0, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unknown native function")
    );
}
