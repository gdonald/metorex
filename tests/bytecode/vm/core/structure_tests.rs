// Tests for BytecodeVm structure and CallFrame (13.1-13.2).

use metorex::bytecode::opcode::OpCode;
use metorex::bytecode::vm::{BytecodeVm, CallFrame};
use metorex::object::{CompiledFunction, Object};
use std::rc::Rc;

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
    assert_eq!(vm.stack_size(), 2);
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
    assert!(!vm.set_global("y", Object::Int(30)));
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
    frame.read_byte();
    assert_eq!(frame.current_line(), 42);
}

#[test]
fn call_frame_current_line_at_start() {
    let func = CompiledFunction::new("f".to_string(), 0);
    let frame = CallFrame::new(Rc::new(func), 0);
    assert_eq!(frame.current_line(), 0);
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
