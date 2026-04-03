// Tests for class compilation (12.6)
//
// Exercises class definition, OP_CLASS emission, method compilation within
// classes, OP_METHOD for each method, and inheritance compilation.

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

/// Helper: find compiled functions in the constant pool.
fn find_compiled_functions(
    chunk: &metorex::bytecode::chunk::Chunk,
) -> Vec<&metorex::object::CompiledFunction> {
    let mut funcs = Vec::new();
    for i in 0..256 {
        let constant =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| chunk.get_constant(i)));
        match constant {
            Ok(Object::CompiledFunction(func)) => funcs.push(func.as_ref()),
            Ok(_) => continue,
            Err(_) => break,
        }
    }
    funcs
}

// ── Class definition compilation (12.6.1) ─────────────────────────────

#[test]
fn compile_empty_class() {
    let chunk = compile("class Foo\nend");
    let ops = opcodes(&chunk);
    assert!(
        ops.contains(&OpCode::Class),
        "Expected OP_CLASS, got: {:?}",
        ops
    );
}

#[test]
fn compile_class_defines_global() {
    let chunk = compile("class Foo\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::DefineGlobal));
}

// ── OP_CLASS instruction (12.6.2) ────────────────────────────────────

#[test]
fn class_name_in_constant_pool() {
    let chunk = compile("class MyClass\nend");
    // The class name should be stored as a string constant
    let mut found = false;
    for i in 0..256 {
        let constant =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| chunk.get_constant(i)));
        match constant {
            Ok(Object::String(s)) if s.as_str() == "MyClass" => {
                found = true;
                break;
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }
    assert!(found, "Expected 'MyClass' string in constant pool");
}

// ── Compile class methods (12.6.3) ──────────────────────────────────

#[test]
fn compile_class_with_one_method() {
    let chunk = compile("class Foo\n  def bar\n    42\n  end\nend");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Class));
    assert!(ops.contains(&OpCode::Method));
    let funcs = find_compiled_functions(&chunk);
    assert!(
        funcs.iter().any(|f| f.name == "bar"),
        "Expected compiled method 'bar'"
    );
}

#[test]
fn compile_class_with_multiple_methods() {
    let chunk = compile("class Foo\n  def bar\n    1\n  end\n  def baz\n    2\n  end\nend");
    let ops = opcodes(&chunk);
    let method_count = ops.iter().filter(|&&op| op == OpCode::Method).count();
    assert_eq!(method_count, 2, "Expected 2 OP_METHOD instructions");
    let funcs = find_compiled_functions(&chunk);
    let names: Vec<&str> = funcs.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"bar"), "Expected method 'bar'");
    assert!(names.contains(&"baz"), "Expected method 'baz'");
}

// ── OP_METHOD for each method (12.6.4) ──────────────────────────────

#[test]
fn method_with_parameters_compiled() {
    let chunk = compile("class Calc\n  def add(a, b)\n    a + b\n  end\nend");
    let funcs = find_compiled_functions(&chunk);
    let add = funcs.iter().find(|f| f.name == "add").unwrap();
    assert_eq!(add.arity, 2);
}

#[test]
fn method_body_has_correct_bytecode() {
    let chunk = compile("class Calc\n  def double(x)\n    x + x\n  end\nend");
    let funcs = find_compiled_functions(&chunk);
    let double = funcs.iter().find(|f| f.name == "double").unwrap();
    let body_ops = opcodes(&double.chunk);
    assert!(
        body_ops.contains(&OpCode::Add),
        "Expected Add in method body, got: {:?}",
        body_ops
    );
}

// ── Inheritance compilation (12.6.5) ────────────────────────────────

#[test]
fn compile_class_with_inheritance() {
    let chunk = compile("class Animal\nend\nclass Dog < Animal\nend");
    let ops = opcodes(&chunk);
    let class_count = ops.iter().filter(|&&op| op == OpCode::Class).count();
    assert_eq!(class_count, 2, "Expected 2 OP_CLASS for Animal and Dog");
    // Dog inherits from Animal, so GetGlobal should be used to look up Animal
    assert!(
        ops.contains(&OpCode::GetGlobal),
        "Expected GetGlobal for superclass lookup"
    );
}

#[test]
fn compile_class_without_inheritance_no_extra_get_global() {
    let chunk = compile("class Standalone\nend");
    let ops = opcodes(&chunk);
    // Class emits: OP_CLASS, DefineGlobal, GetGlobal (to put class on stack for methods), Pop
    // No extra GetGlobal for a superclass
    let get_global_count = ops.iter().filter(|&&op| op == OpCode::GetGlobal).count();
    // One GetGlobal is for putting the class back on the stack for method attachment
    assert_eq!(
        get_global_count, 1,
        "Expected exactly 1 GetGlobal (class on stack for methods), got {}",
        get_global_count
    );
}

#[test]
fn compile_inheritance_emits_superclass_get_global() {
    let chunk = compile("class Base\nend\nclass Child < Base\nend");
    let ops = opcodes(&chunk);
    // Child should have an extra GetGlobal for Base
    let get_global_count = ops.iter().filter(|&&op| op == OpCode::GetGlobal).count();
    // Base: 1 GetGlobal (class on stack) + Child: 1 GetGlobal (superclass) + 1 GetGlobal (class on stack)
    assert!(
        get_global_count >= 3,
        "Expected at least 3 GetGlobal ops, got {}",
        get_global_count
    );
}

// ── Class with initialize method ────────────────────────────────────

#[test]
fn compile_class_with_initialize() {
    let chunk = compile("class Person\n  def initialize(name)\n    @name = name\n  end\nend");
    let funcs = find_compiled_functions(&chunk);
    let init = funcs.iter().find(|f| f.name == "initialize").unwrap();
    assert_eq!(init.arity, 1);
    let body_ops = opcodes(&init.chunk);
    assert!(
        body_ops.contains(&OpCode::SetInstance),
        "Expected SetInstance in initialize body"
    );
}

// ── Complex class scenarios ─────────────────────────────────────────

#[test]
fn compile_class_with_method_using_control_flow() {
    let chunk = compile(
        "class Checker\n  def check(x)\n    if x\n      1\n    else\n      2\n    end\n  end\nend",
    );
    let funcs = find_compiled_functions(&chunk);
    let check = funcs.iter().find(|f| f.name == "check").unwrap();
    let body_ops = opcodes(&check.chunk);
    assert!(body_ops.contains(&OpCode::JumpIfFalse));
}

#[test]
fn compile_class_pops_after_body() {
    let chunk = compile("class Foo\nend");
    let ops = opcodes(&chunk);
    // After the class body, the class is popped from the stack
    assert!(ops.contains(&OpCode::Pop));
}

#[test]
fn compile_multiple_classes() {
    let chunk = compile("class A\nend\nclass B\nend\nclass C\nend");
    let ops = opcodes(&chunk);
    let class_count = ops.iter().filter(|&&op| op == OpCode::Class).count();
    assert_eq!(class_count, 3);
}

#[test]
fn compile_class_then_instantiate() {
    let chunk = compile("class Foo\nend\nFoo");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::Class));
    assert!(ops.contains(&OpCode::GetGlobal));
}
