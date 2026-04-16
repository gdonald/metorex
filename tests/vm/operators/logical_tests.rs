use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

// ── Short-circuit ────────────────────────────────────────────────────────────

#[test]
fn logical_and_short_circuit() {
    assert_eq!(run("false && 1 / 0"), Some(Object::Bool(false)));
}

#[test]
fn logical_or_short_circuit() {
    assert_eq!(run("true || 1 / 0"), Some(Object::Bool(true)));
}

// ── Assignment operators ────────────────────────────────────────────────────

#[test]
fn add_assign_operator() {
    assert_eq!(run("x = 5\nx += 3\nx"), Some(Object::Int(8)));
}

#[test]
fn subtract_assign_operator() {
    assert_eq!(run("x = 10\nx -= 3\nx"), Some(Object::Int(7)));
}

#[test]
fn multiply_assign_operator() {
    assert_eq!(run("x = 4\nx *= 3\nx"), Some(Object::Int(12)));
}

#[test]
fn divide_assign_operator() {
    assert_eq!(run("x = 12\nx /= 4\nx"), Some(Object::Int(3)));
}

// ── ||= and &&= ────────────────────────────────────────────────────────────

#[test]
fn or_assign_nil() {
    assert_eq!(run("x = nil; x ||= 42; x"), Some(Object::Int(42)));
}

#[test]
fn or_assign_existing() {
    assert_eq!(run("x = 10; x ||= 42; x"), Some(Object::Int(10)));
}

#[test]
fn and_assign_truthy() {
    assert_eq!(run("x = true; x &&= 42; x"), Some(Object::Int(42)));
}

#[test]
fn and_assign_falsy() {
    assert_eq!(run("x = false; x &&= 42; x"), Some(Object::Bool(false)));
}
