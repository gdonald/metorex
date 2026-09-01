// 10.3.2 — Parser + VM Integration Tests
//
// These tests verify that parsed AST executes correctly in the VM,
// testing the handoff between parser output and VM execution.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_env(code: &str) -> VirtualMachine {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed");
    vm
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── Variable assignment and retrieval ──

#[test]
fn variable_assignment_and_access() {
    let vm = run_env("x = 42");
    assert_eq!(vm.environment().get("x"), Some(Object::Int(42)));
}

#[test]
fn variable_reassignment() {
    let vm = run_env("x = 1\nx = 2");
    assert_eq!(vm.environment().get("x"), Some(Object::Int(2)));
}

#[test]
fn multiple_variables() {
    let vm = run_env("a = 10\nb = 20\nc = a + b");
    assert_eq!(vm.environment().get("c"), Some(Object::Int(30)));
}

// ── Arithmetic operations execute correctly ──

#[test]
fn arithmetic_operations() {
    assert_eq!(run("2 + 3"), Some(Object::Int(5)));
    assert_eq!(run("10 - 4"), Some(Object::Int(6)));
    assert_eq!(run("3 * 7"), Some(Object::Int(21)));
    assert_eq!(run("15 % 4"), Some(Object::Int(3)));
}

#[test]
fn float_arithmetic() {
    assert_eq!(run("1.5 + 2.5"), Some(Object::Float(4.0)));
    assert_eq!(run("10.0 / 3.0"), Some(Object::Float(10.0 / 3.0)));
}

#[test]
fn mixed_int_float_arithmetic() {
    assert_eq!(run("1 + 2.5"), Some(Object::Float(3.5)));
}

// ── String operations ──

#[test]
fn string_concatenation() {
    assert_eq!(
        run("\"hello\" + \" world\""),
        Some(Object::String(Rc::new("hello world".to_string())))
    );
}

#[test]
fn string_interpolation_executes() {
    let vm = run_env("name = \"world\"\nresult = \"hello #{name}\"");
    assert_eq!(
        vm.environment().get("result"),
        Some(Object::String(Rc::new("hello world".to_string())))
    );
}

// ── Control flow ──

#[test]
fn if_true_branch() {
    let vm = run_env("x = if true\n  1\nelse\n  2\nend");
    assert_eq!(vm.environment().get("x"), Some(Object::Int(1)));
}

#[test]
fn if_false_branch() {
    let vm = run_env("x = if false\n  1\nelse\n  2\nend");
    assert_eq!(vm.environment().get("x"), Some(Object::Int(2)));
}

#[test]
fn while_loop_executes() {
    let vm = run_env("i = 0\nwhile i < 5\n  i += 1\nend");
    assert_eq!(vm.environment().get("i"), Some(Object::Int(5)));
}

#[test]
fn for_loop_executes() {
    let vm = run_env("sum = 0\nfor i in 1..5\n  sum += i\nend");
    assert_eq!(vm.environment().get("sum"), Some(Object::Int(15)));
}

// ── Function definitions and calls ──

#[test]
fn function_definition_and_call() {
    let vm = run_env("def add(a, b)\n  a + b\nend\nresult = add(3, 4)");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(7)));
}

#[test]
fn function_with_default_params() {
    // Call with the default value explicitly provided
    let vm = run_env("def greet(name = \"world\")\n  name\nend\nresult = greet(\"hello\")");
    assert_eq!(
        vm.environment().get("result"),
        Some(Object::String(Rc::new("hello".to_string())))
    );
}

#[test]
fn function_with_multiple_params_and_defaults() {
    let vm = run_env("def add(a, b = 10)\n  a + b\nend\nresult = add(5, 20)");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(25)));
}

#[test]
fn recursive_function() {
    let vm = run_env(
        "def fact(n)\n  if n <= 1\n    return 1\n  end\n  n * fact(n - 1)\nend\nresult = fact(5)",
    );
    assert_eq!(vm.environment().get("result"), Some(Object::Int(120)));
}

// ── Class instantiation and method dispatch ──

#[test]
fn class_instantiation_and_method() {
    let code = r#"class Dog
  def initialize(name)
    @name = name
  end
  def name
    @name
  end
end
d = Dog.new("Rex")
result = d.name"#;
    let vm = run_env(code);
    assert_eq!(
        vm.environment().get("result"),
        Some(Object::String(Rc::new("Rex".to_string())))
    );
}

#[test]
fn class_inheritance() {
    let code = r#"class Animal
  def speak
    "..."
  end
end
class Cat < Animal
  def speak
    "meow"
  end
end
result = Cat.new.speak"#;
    let vm = run_env(code);
    assert_eq!(
        vm.environment().get("result"),
        Some(Object::String(Rc::new("meow".to_string())))
    );
}

// ── Blocks and closures ──

#[test]
fn lambda_creation_and_call() {
    let vm = run_env("f = lambda do |x| x * 2 end\nresult = f.call(5)");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(10)));
}

#[test]
fn closure_captures_variable() {
    let vm = run_env("x = 10\nf = lambda do || x + 5 end\nresult = f.call");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(15)));
}

// ── Collections ──

#[test]
fn array_operations() {
    let vm = run_env("arr = [1, 2, 3]\narr.push(4)\nresult = arr.length");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(4)));
}

#[test]
fn hash_operations() {
    let vm = run_env("h = {\"a\" => 1, \"b\" => 2}\nresult = h.size");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(2)));
}

// ── Error propagation from VM ──

#[test]
fn undefined_variable_error() {
    let err = run_err("puts undefined_xyz");
    assert!(err.contains("Undefined variable"));
}

#[test]
fn division_by_zero_error() {
    let err = run_err("1 / 0");
    assert!(
        err.contains("divided by 0"),
        "Expected division by zero error, got: {}",
        err
    );
}

#[test]
fn wrong_argument_count_error() {
    let err = run_err("def foo(a)\n  a\nend\nfoo(1, 2)");
    assert!(err.contains("argument"));
}
