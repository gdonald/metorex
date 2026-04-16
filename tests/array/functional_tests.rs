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

// ── Array#select ─────────────────────────────────────────────────────────────

#[test]
fn array_select_filters() {
    let result = run("[1, 2, 3, 4, 5].select { |x| x > 3 }");
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    } else {
        panic!("expected array");
    }
}

// ── Array#partition ──────────────────────────────────────────────────────────

#[test]
fn array_partition_splits() {
    let result = run(r#"
result = [1, 2, 3, 4].partition { |x| x > 2 }
result.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn array_partition_empty() {
    let result = run("[].partition { |x| x }");
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    } else {
        panic!("expected array");
    }
}

// ── Array#reduce / inject ────────────────────────────────────────────────────

#[test]
fn array_reduce_sum() {
    let result = run("[1, 2, 3].reduce { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn array_reduce_with_initial_value() {
    let result = run("[1, 2, 3].reduce(10) { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(16)));
}

#[test]
fn array_reduce_empty() {
    let result = run("[].reduce { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_inject_alias() {
    let result = run("[1, 2, 3, 4].inject { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(10)));
}

// ── Array#any? / all? / none? ────────────────────────────────────────────────

#[test]
fn array_any_with_block() {
    let result = run("[1, 2, 3].any? { |x| x > 2 }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn array_any_without_block() {
    let result = run("[nil, false, 1].any?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn array_any_empty() {
    let result = run("[].any?");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn array_all_with_block() {
    let result = run("[2, 4, 6].all? { |x| x > 0 }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn array_all_false() {
    let result = run("[2, 4, 6].all? { |x| x > 3 }");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn array_all_empty() {
    let result = run("[].all?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn array_none_with_block() {
    let result = run("[1, 2, 3].none? { |x| x > 5 }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn array_none_empty() {
    let result = run("[].none?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn array_none_false() {
    let result = run("[1, 2, 3].none? { |x| x > 2 }");
    assert_eq!(result, Some(Object::Bool(false)));
}
