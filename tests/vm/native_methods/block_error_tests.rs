// Block error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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
// AST methods - block statements access (line 180 of mod.rs)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn block_object_statements_access() {
    let result = run(r#"
b = lambda do |x|
  x + 1
end
b.statements
"#);
    assert!(result.is_some());
}

#[test]
fn block_object_arity_access() {
    let result = run(r#"
b = lambda do |x, y|
  x + y
end
b.arity
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Block call method
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn block_call_method() {
    let result = run(r#"
b = lambda do |x|
  x * 2
end
b.call(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}
