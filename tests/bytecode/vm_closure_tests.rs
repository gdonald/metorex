// Tests for closure and upvalue execution in the bytecode VM (13.8)

use metorex::bytecode::opcode::OpCode;
use metorex::bytecode::vm::{BytecodeVm, CallFrame, ClosureObj, UpvalueObj};
use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::object::{CompiledFunction, Object};
use metorex::parser::Parser;
use std::cell::RefCell;
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

fn run_ok(source: &str) -> Object {
    run(source).expect("execution failed")
}

// ── UpvalueObj tests ────────────────────────────────────────────────

#[test]
fn upvalue_new_open() {
    let uv = UpvalueObj::new_open(5, Object::Int(42));
    assert!(uv.is_open());
    assert_eq!(uv.stack_index, 5);
    assert_eq!(*uv.value.borrow(), Object::Int(42));
}

#[test]
fn upvalue_close() {
    let mut uv = UpvalueObj::new_open(3, Object::Int(10));
    assert!(uv.is_open());
    uv.close(Object::Int(99));
    assert!(!uv.is_open());
    assert_eq!(uv.stack_index, usize::MAX);
    assert_eq!(*uv.value.borrow(), Object::Int(99));
}

#[test]
fn upvalue_clone() {
    let uv = UpvalueObj::new_open(0, Object::String(Rc::new("hello".to_string())));
    let cloned = uv.clone();
    assert_eq!(cloned.stack_index, 0);
}

#[test]
fn upvalue_debug() {
    let uv = UpvalueObj::new_open(1, Object::Nil);
    let debug = format!("{:?}", uv);
    assert!(debug.contains("UpvalueObj"));
}

// ── ClosureObj tests ────────────────────────────────────────────────

#[test]
fn closure_obj_new() {
    let func = Rc::new(CompiledFunction::new("f".to_string(), 0));
    let closure = ClosureObj {
        function: func.clone(),
        upvalues: vec![],
    };
    assert_eq!(closure.function.name, "f");
    assert!(closure.upvalues.is_empty());
}

#[test]
fn closure_obj_with_upvalues() {
    let func = Rc::new(CompiledFunction::new("f".to_string(), 0));
    let uv = Rc::new(RefCell::new(UpvalueObj::new_open(0, Object::Int(42))));
    let closure = ClosureObj {
        function: func,
        upvalues: vec![uv],
    };
    assert_eq!(closure.upvalues.len(), 1);
}

#[test]
fn closure_obj_clone() {
    let func = Rc::new(CompiledFunction::new("f".to_string(), 0));
    let closure = ClosureObj {
        function: func,
        upvalues: vec![],
    };
    let cloned = closure.clone();
    assert_eq!(cloned.function.name, "f");
}

#[test]
fn closure_obj_debug() {
    let func = Rc::new(CompiledFunction::new("test".to_string(), 1));
    let closure = ClosureObj {
        function: func,
        upvalues: vec![],
    };
    let debug = format!("{:?}", closure);
    assert!(debug.contains("ClosureObj"));
}

// ── CallFrame with closure ──────────────────────────────────────────

#[test]
fn call_frame_new_closure() {
    let func = Rc::new(CompiledFunction::new("f".to_string(), 0));
    let uv = Rc::new(RefCell::new(UpvalueObj::new_open(0, Object::Int(1))));
    let closure = ClosureObj {
        function: func,
        upvalues: vec![uv],
    };
    let frame = CallFrame::new_closure(closure, 5);
    assert_eq!(frame.slot_offset, 5);
    assert_eq!(frame.upvalues.len(), 1);
    assert_eq!(frame.ip, 0);
}

// ── Closure execution via compile-and-run ───────────────────────────

#[test]
fn closure_captures_enclosing_local() {
    let result = run_ok(
        "def make_adder(x)\n  def adder(y)\n    return x + y\n  end\n  return adder\nend\nf = make_adder(10)\nreturn f(5)",
    );
    assert_eq!(result, Object::Int(15));
}

#[test]
fn closure_captures_and_modifies_variable() {
    let result = run_ok(
        "def counter\n  n = 0\n  def increment\n    n = n + 1\n    return n\n  end\n  return increment\nend\nf = counter()\nf()\nreturn f()",
    );
    assert_eq!(result, Object::Int(2));
}

#[test]
fn closure_captures_multiple_variables() {
    let result = run_ok(
        "def make_pair(a, b)\n  def sum\n    return a + b\n  end\n  return sum\nend\nf = make_pair(3, 7)\nreturn f()",
    );
    assert_eq!(result, Object::Int(10));
}

#[test]
fn closure_two_level_capture() {
    // Two-level nesting (direct capture) works; deeper nesting requires
    // upvalue forwarding which is not yet implemented in the compiler.
    let result = run_ok(
        "def outer\n  x = 100\n  def inner\n    return x\n  end\n  return inner\nend\nf = outer()\nreturn f()",
    );
    assert_eq!(result, Object::Int(100));
}

// ── VM upvalue capture deduplication ────────────────────────────────

#[test]
fn vm_capture_upvalue_deduplicates() {
    let mut vm = BytecodeVm::new();
    // Push a value on the stack to capture
    vm.push(Object::Int(42));
    let uv1 = vm.capture_upvalue_public(0);
    let uv2 = vm.capture_upvalue_public(0);
    // Should be the same Rc
    assert!(Rc::ptr_eq(&uv1, &uv2));
}

#[test]
fn vm_close_upvalues_closes_at_index() {
    let mut vm = BytecodeVm::new();
    vm.push(Object::Int(10));
    vm.push(Object::Int(20));
    let uv = vm.capture_upvalue_public(1);
    assert!(uv.borrow().is_open());

    vm.close_upvalues_public(1);
    assert!(!uv.borrow().is_open());
    assert_eq!(*uv.borrow().value.borrow(), Object::Int(20));
}

#[test]
fn vm_close_upvalues_preserves_lower() {
    let mut vm = BytecodeVm::new();
    vm.push(Object::Int(10));
    vm.push(Object::Int(20));
    let uv0 = vm.capture_upvalue_public(0);
    let _uv1 = vm.capture_upvalue_public(1);

    vm.close_upvalues_public(1); // only close slot 1 and above
    assert!(uv0.borrow().is_open()); // slot 0 should still be open
}
