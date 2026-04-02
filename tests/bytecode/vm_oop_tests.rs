// Tests for class and object execution in the bytecode VM (13.9)

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

// ── OP_CLASS (13.9.1-2) ─────────────────────────────────────────────

#[test]
fn class_definition_creates_class() {
    let result = run_ok("class Foo\nend\nreturn Foo");
    match result {
        Object::Class(c) => assert_eq!(c.name(), "Foo"),
        _ => panic!("Expected Class, got {:?}", result),
    }
}

#[test]
fn class_stored_as_global() {
    let result = run_ok("class Bar\nend\nreturn Bar");
    assert!(matches!(result, Object::Class(_)));
}

#[test]
fn multiple_classes() {
    let result = run_ok("class A\nend\nclass B\nend\nreturn A");
    match result {
        Object::Class(c) => assert_eq!(c.name(), "A"),
        _ => panic!("Expected Class"),
    }
}

// ── OP_METHOD (13.9.3-4) ───────────────────────────────────────────

#[test]
fn class_with_method_defined() {
    let result = run_ok("class Foo\n  def greet\n    return 42\n  end\nend\nreturn Foo");
    match result {
        Object::Class(c) => {
            assert!(c.find_method("greet").is_some());
        }
        _ => panic!("Expected Class with method"),
    }
}

#[test]
fn class_with_multiple_methods() {
    let result = run_ok(
        "class Calc\n  def add(a, b)\n    return a + b\n  end\n  def sub(a, b)\n    return a - b\n  end\nend\nreturn Calc",
    );
    match result {
        Object::Class(c) => {
            assert!(c.find_method("add").is_some());
            assert!(c.find_method("sub").is_some());
        }
        _ => panic!("Expected Class"),
    }
}

// ── OP_INVOKE (13.9.5) ─────────────────────────────────────────────

#[test]
fn invoke_method_on_instance() {
    // This requires a compiled method to be stored and invokable.
    // The current implementation stores CompiledFunction as "ClassName#method"
    // in globals. For Invoke to work, we need an instance.
    // For now, test that class definition works without error.
    let result = run("class Greeter\n  def hello\n    return 99\n  end\nend");
    assert!(result.is_ok());
}

// ── OP_GET_INSTANCE / OP_SET_INSTANCE (13.9.6-7) ───────────────────

#[test]
fn class_definition_with_initialize() {
    // Test that initialize with instance vars compiles and runs
    let result =
        run("class Person\n  def initialize(name)\n    @name = name\n  end\nend\nreturn Person");
    match result {
        Ok(Object::Class(c)) => {
            assert_eq!(c.name(), "Person");
            assert!(c.find_method("initialize").is_some());
        }
        Ok(other) => panic!("Expected Class, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

// ── Error paths ─────────────────────────────────────────────────────

#[test]
fn invoke_on_non_instance_errors() {
    // Invoking a method on an integer should error
    let result = run("x = 42\nreturn x.length");
    assert!(result.is_err());
}

// ── Class as value ──────────────────────────────────────────────────

#[test]
fn class_is_truthy() {
    let result = run_ok("class Foo\nend\nif Foo\n  return 1\nend\nreturn 2");
    assert_eq!(result, Object::Int(1));
}

#[test]
fn class_name_accessible() {
    let result = run_ok("class MyClass\nend\nreturn MyClass");
    match result {
        Object::Class(c) => assert_eq!(c.name(), "MyClass"),
        _ => panic!("Expected Class"),
    }
}

// ── Method lookup stored correctly ──────────────────────────────────

#[test]
fn method_function_stored_in_globals() {
    // After class compilation, the method's compiled function should be
    // stored as "ClassName#methodName" in globals
    let tokens = Lexer::new("class Foo\n  def bar\n    return 1\n  end\nend").tokenize();
    let stmts = Parser::new(tokens).parse().unwrap();
    let chunk = Compiler::new().compile(&stmts).unwrap();
    let mut vm = BytecodeVm::new();
    vm.execute(&chunk).unwrap();

    let method_func = vm.get_global("Foo#bar");
    assert!(method_func.is_some(), "Expected Foo#bar in globals");
    assert!(matches!(method_func.unwrap(), Object::CompiledFunction(_)));
}

// ── Empty class ─────────────────────────────────────────────────────

#[test]
fn empty_class_has_no_methods() {
    let result = run_ok("class Empty\nend\nreturn Empty");
    match result {
        Object::Class(c) => {
            assert!(c.find_method("anything").is_none());
        }
        _ => panic!("Expected Class"),
    }
}

// ── Class followed by other code ────────────────────────────────────

#[test]
fn code_after_class_runs() {
    let result = run_ok("class Foo\nend\nx = 42\nreturn x");
    assert_eq!(result, Object::Int(42));
}

#[test]
fn multiple_classes_and_globals() {
    let result = run_ok("class A\nend\nclass B\nend\nx = 10\ny = 20\nreturn x + y");
    assert_eq!(result, Object::Int(30));
}

// ── OP_INVOKE on Instance ──────────────────────────────────────────

#[test]
fn invoke_method_returns_value() {
    let src = r#"
class Adder
  def add(a, b)
    return a + b
  end
end
obj = Adder.new
return obj.add(3, 4)
"#;
    let result = run_ok(src);
    assert_eq!(result, Object::Int(7));
}

#[test]
fn invoke_method_wrong_arg_count_errors() {
    let src = r#"
class Foo
  def bar(x)
    return x
  end
end
obj = Foo.new
return obj.bar(1, 2)
"#;
    let result = run(src);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("Expected 1 arguments but got 2"),
        "Error was: {}",
        err
    );
}

#[test]
fn invoke_undefined_method_errors() {
    let src = r#"
class Foo
end
obj = Foo.new
return obj.missing
"#;
    let result = run(src);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Undefined method"), "Error was: {}", err);
}

// ── OP_INVOKE on Class receiver (instantiation) ────────────────────

#[test]
fn invoke_new_creates_instance() {
    let src = r#"
class Dog
end
d = Dog.new
return d
"#;
    let result = run_ok(src);
    assert!(
        matches!(result, Object::Instance(_)),
        "Expected Instance, got {:?}",
        result
    );
}

#[test]
fn invoke_undefined_class_method_errors() {
    let src = r#"
class Foo
end
return Foo.something
"#;
    let result = run(src);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Undefined class method"), "Error was: {}", err);
}

// ── OP_INVOKE on non-instance (error path) ─────────────────────────

#[test]
fn invoke_method_on_string_errors() {
    let src = "x = \"hello\"\nreturn x.foo";
    let result = run(src);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Cannot call method"), "Error was: {}", err);
}

// ── OP_GET_INSTANCE / OP_SET_INSTANCE ──────────────────────────────

#[test]
fn get_and_set_instance_variable() {
    // Test OP_SET_INSTANCE and OP_GET_INSTANCE using a hand-built chunk
    // that places an Instance at slot 0 (slot_offset of the execute frame).
    use metorex::bytecode::chunk::Chunk;
    use metorex::bytecode::opcode::OpCode;
    use metorex::object::{Class, Instance};
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut chunk = Chunk::new();

    // Constant 0: the Instance object
    let class = Rc::new(Class::new("TestClass".to_string(), None));
    let instance = Instance::new(Rc::clone(&class));
    let inst_obj = Object::Instance(Rc::new(RefCell::new(instance)));
    let inst_idx = chunk.add_constant(inst_obj).unwrap();

    // Constant 1: the ivar name "x"
    let name_idx = chunk
        .add_constant(Object::String(Rc::new("x".to_string())))
        .unwrap();

    // Constant 2: value 42
    let val_idx = chunk.add_constant(Object::Int(42)).unwrap();

    // Push instance to slot 0 (this is where slot_offset points)
    chunk.write_constant(inst_idx, 1);

    // Push 42
    chunk.write_constant(val_idx, 1);

    // OP_SET_INSTANCE with name_idx — reads stack[slot_offset]=instance, sets @x=42
    chunk.write_opcode(OpCode::SetInstance, 1);
    chunk.write_byte(name_idx as u8, 1);

    // Pop the value left by SetInstance
    chunk.write_opcode(OpCode::Pop, 1);

    // OP_GET_INSTANCE with name_idx — reads stack[slot_offset]=instance, pushes @x
    chunk.write_opcode(OpCode::GetInstance, 1);
    chunk.write_byte(name_idx as u8, 1);

    // Return @x
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Int(42));
}

#[test]
fn get_instance_variable_returns_nil_when_unset() {
    // Test OP_GET_INSTANCE for an unset variable returns nil.
    // Uses hand-built chunk to ensure the Instance is at slot_offset.
    use metorex::bytecode::chunk::Chunk;
    use metorex::bytecode::opcode::OpCode;
    use metorex::object::{Class, Instance};
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut chunk = Chunk::new();
    let class = Rc::new(Class::new("T".to_string(), None));
    let instance = Instance::new(Rc::clone(&class));
    let inst_obj = Object::Instance(Rc::new(RefCell::new(instance)));
    let inst_idx = chunk.add_constant(inst_obj).unwrap();
    let name_idx = chunk
        .add_constant(Object::String(Rc::new("missing".to_string())))
        .unwrap();

    // Push instance to slot 0
    chunk.write_constant(inst_idx, 1);
    // GetInstance @missing (not set, so returns nil)
    chunk.write_opcode(OpCode::GetInstance, 1);
    chunk.write_byte(name_idx as u8, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk).unwrap();
    assert_eq!(result, Object::Nil);
}

#[test]
fn get_instance_on_non_instance_errors() {
    // Test OP_GET_INSTANCE error path when receiver is not an Instance.
    use metorex::bytecode::chunk::Chunk;
    use metorex::bytecode::opcode::OpCode;
    use std::rc::Rc;

    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Object::String(Rc::new("x".to_string())))
        .unwrap();

    // Push a non-instance value to slot 0
    chunk.write_opcode(OpCode::True, 1);
    // Try GetInstance — should error
    chunk.write_opcode(OpCode::GetInstance, 1);
    chunk.write_byte(name_idx as u8, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Cannot access instance variable outside of instance"),
        "Error was: {}",
        err
    );
}

#[test]
fn set_instance_on_non_instance_errors() {
    // Test OP_SET_INSTANCE error path when receiver is not an Instance.
    use metorex::bytecode::chunk::Chunk;
    use metorex::bytecode::opcode::OpCode;
    use std::rc::Rc;

    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Object::String(Rc::new("x".to_string())))
        .unwrap();

    // Push a non-instance value to slot 0
    chunk.write_opcode(OpCode::True, 1);
    // Push value to set
    let val_idx = chunk.add_constant(Object::Int(1)).unwrap();
    chunk.write_constant(val_idx, 1);
    // Try SetInstance — should error
    chunk.write_opcode(OpCode::SetInstance, 1);
    chunk.write_byte(name_idx as u8, 1);
    chunk.write_opcode(OpCode::Return, 1);

    let mut vm = BytecodeVm::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Cannot set instance variable outside of instance"),
        "Error was: {}",
        err
    );
}
