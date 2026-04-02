// Float error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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

// ══════════════════════════════════════════════════════════════════════════════
// Float methods
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn float_abs_method() {
    let result = run(r#"
(-3.14).abs
"#);
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn float_ceil_method() {
    let result = run(r#"
3.2.ceil
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn float_floor_method() {
    let result = run(r#"
3.8.floor
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn float_to_i_method() {
    let result = run(r#"
3.7.to_i
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn float_to_f_method() {
    let result = run(r#"
3.14.to_f
"#);
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn float_round_method() {
    let result = run(r#"
3.14159.round(2)
"#);
    assert_eq!(result, Some(Object::Float(3.14)));
}
