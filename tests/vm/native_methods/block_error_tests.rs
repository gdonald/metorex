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

// ── Block with variadic *args parameter (via method yield) ──────────────────

#[test]
fn block_with_variadic_via_method() {
    let result = run(r#"
def test_var
  yield 1, 2, 3, 4
end
test_var { |a, *rest| rest.length }
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn block_with_variadic_no_extras_via_method() {
    let result = run(r#"
def test_var
  yield 1
end
test_var { |a, *rest| rest.length }
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn block_with_variadic_and_trailing_via_method() {
    let result = run(r#"
def test_var
  yield 1, 2, 3, 4, 5
end
result = test_var { |a, *mid, z| mid.length }
result
"#);
    assert!(result.is_some());
}

// ── Block with &block parameter ─────────────────────────────────────────────

#[test]
fn block_with_block_param_via_method() {
    let result = run(r#"
def test_blk
  yield 5
end
test_blk { |x, &blk| x * 2 }
"#);
    assert_eq!(result, Some(Object::Int(10)));
}
