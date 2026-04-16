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

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── each/map/select: return and exception inside block ───────────────────────

#[test]
fn array_each_return_inside_block_error() {
    let err = run_err(
        r#"
[1, 2, 3].each do |n|
  return n
end
"#,
    );
    assert!(err.contains("return") || err.contains("loop"));
}

#[test]
fn array_each_exception_inside_block_error() {
    let err = run_err(r#"[1, 2, 3].each { |n| raise "block error" }"#);
    assert!(err.contains("block error") || err.contains("exception") || err.contains("Uncaught"));
}

#[test]
fn array_map_exception_inside_block_error() {
    let err = run_err(r#"[1, 2, 3].map { |n| raise "map error" }"#);
    assert!(err.contains("map error") || err.contains("exception") || err.contains("Uncaught"));
}

#[test]
fn array_select_exception_inside_block_error() {
    let err = run_err(r#"[1, 2, 3].select { |n| raise "select error" }"#);
    assert!(err.contains("select error") || err.contains("exception") || err.contains("Uncaught"));
}

// ── Array#[] wrong arg count ─────────────────────────────────────────────────

#[test]
fn array_index_wrong_arg_count() {
    let err = run_err("[1, 2, 3].[](1, 2, 3)");
    assert!(err.contains("argument"));
}

// ── Array#each without block ─────────────────────────────────────────────────

#[test]
fn array_each_without_block() {
    let err = run_err("[1, 2, 3].each");
    assert!(err.contains("block") || err.contains("requires"));
}

// ── Array#map without block ──────────────────────────────────────────────────

#[test]
fn array_map_without_block() {
    let err = run_err("[1, 2, 3].map");
    assert!(err.contains("block") || err.contains("requires"));
}

// ── Array#select without block ───────────────────────────────────────────────

#[test]
fn array_select_without_block() {
    let err = run_err("[1, 2, 3].select");
    assert!(err.contains("block") || err.contains("requires"));
}

// ── Array#partition without block ────────────────────────────────────────────

#[test]
fn array_partition_without_block() {
    let err = run_err("[1, 2, 3].partition");
    assert!(err.contains("block") || err.contains("requires"));
}

// ── Array#reduce without block ───────────────────────────────────────────────

#[test]
fn array_reduce_without_block() {
    let err = run_err("[1, 2, 3].reduce");
    assert!(err.contains("block") || err.contains("requires"));
}

// ── Array#inject without block ───────────────────────────────────────────────

#[test]
fn array_inject_without_block() {
    let err = run_err("[1, 2, 3].inject");
    assert!(err.contains("block") || err.contains("requires"));
}

// ── Array#each error cases ───────────────────────────────────────────────────

#[test]
fn array_each_with_args_error() {
    let err = run_err("[1].each(1) { |x| x }");
    assert!(err.contains("argument"));
}

// ── Array#map error cases ────────────────────────────────────────────────────

#[test]
fn array_map_with_args_error() {
    let err = run_err("[1].map(1) { |x| x }");
    assert!(err.contains("argument"));
}

// ── Array#each/map/select: return value from block ───────────────────────────

#[test]
fn array_map_returns_transformed_array() {
    let result = run("[1, 2, 3].map { |x| x * 10 }");
    if let Some(Object::Array(arr)) = result {
        let arr = arr.borrow();
        assert_eq!(arr[0], Object::Int(10));
        assert_eq!(arr[1], Object::Int(20));
        assert_eq!(arr[2], Object::Int(30));
    } else {
        panic!("expected array");
    }
}
