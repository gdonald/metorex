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

// ── sort ─────────────────────────────────────────────────────────────────────

#[test]
fn array_sort_integers() {
    let result = run(r#"
arr = [3, 1, 4, 1, 5, 9, 2, 6]
arr.sort
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
            Object::Int(4),
            Object::Int(5),
            Object::Int(6),
            Object::Int(9),
        ]))
    );
}

#[test]
fn array_sort_strings() {
    let result = run(r#"
arr = ["banana", "apple", "cherry"]
arr.sort
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("apple"),
            Object::string("banana"),
            Object::string("cherry"),
        ]))
    );
}

#[test]
fn array_sort_does_not_mutate_original() {
    let result = run(r#"
arr = [3, 1, 2]
arr.sort
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(3),
            Object::Int(1),
            Object::Int(2),
        ]))
    );
}

// ── sort with floats (compare_for_sort float branches) ───────────────────────

#[test]
fn array_sort_floats() {
    let result = run("[3.1, 1.5, 2.7].sort");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Float(1.5),
            Object::Float(2.7),
            Object::Float(3.1),
        ]))
    );
}

#[test]
fn array_sort_mixed_int_float() {
    let result = run("[3, 1.5, 2].sort");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Float(1.5),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

#[test]
fn array_sort_float_int_mixed() {
    let result = run("[2.5, 1, 3.0].sort");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Float(2.5),
            Object::Float(3.0),
        ]))
    );
}

// ── zip ──────────────────────────────────────────────────────────────────────

#[test]
fn array_zip_basic() {
    let result = run("[1, 2, 3].zip([4, 5, 6])");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::array(vec![Object::Int(1), Object::Int(4)]),
            Object::array(vec![Object::Int(2), Object::Int(5)]),
            Object::array(vec![Object::Int(3), Object::Int(6)]),
        ]))
    );
}

#[test]
fn array_zip_shorter_arg() {
    let result = run("[1, 2, 3].zip([4, 5])");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::array(vec![Object::Int(1), Object::Int(4)]),
            Object::array(vec![Object::Int(2), Object::Int(5)]),
            Object::array(vec![Object::Int(3), Object::Nil]),
        ]))
    );
}

#[test]
fn array_zip_error_no_args() {
    let err = run_err("[1, 2].zip");
    assert!(err.contains("argument"));
}

#[test]
fn array_zip_error_non_array_arg() {
    let err = run_err("[1, 2].zip(42)");
    assert!(err.contains("Array"));
}

// ── transpose ────────────────────────────────────────────────────────────────

#[test]
fn array_transpose_basic() {
    let result = run("[[1, 2], [3, 4], [5, 6]].transpose");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::array(vec![Object::Int(1), Object::Int(3), Object::Int(5)]),
            Object::array(vec![Object::Int(2), Object::Int(4), Object::Int(6)]),
        ]))
    );
}

#[test]
fn array_transpose_empty() {
    let result = run("[].transpose");
    assert_eq!(result, Some(Object::array(vec![])));
}

#[test]
fn array_transpose_error_non_array_element() {
    let err = run_err("[1, 2, 3].transpose");
    assert!(err.contains("transpose"));
}

#[test]
fn array_transpose_error_with_args() {
    let err = run_err("[[1, 2]].transpose(1)");
    assert!(err.contains("argument"));
}

// ── min / max ───────────────────────────────────────────────────────────────

#[test]
fn array_min_int() {
    assert_eq!(run("[5, 2, 8, 1].min"), Some(Object::Int(1)));
}

#[test]
fn array_max_int() {
    assert_eq!(run("[5, 2, 8, 1].max"), Some(Object::Int(8)));
}

#[test]
fn array_min_float() {
    assert_eq!(run("[3.5, 1.2, 2.8].min"), Some(Object::Float(1.2)));
}

#[test]
fn array_max_float() {
    assert_eq!(run("[3.5, 1.2, 2.8].max"), Some(Object::Float(3.5)));
}

#[test]
fn array_min_empty() {
    assert_eq!(run("[].min"), Some(Object::Nil));
}

#[test]
fn array_max_empty() {
    assert_eq!(run("[].max"), Some(Object::Nil));
}

#[test]
fn array_min_error_with_args() {
    let err = run_err("[1].min(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_max_error_with_args() {
    let err = run_err("[1].max(1)");
    assert!(err.contains("argument"));
}

// ── uniq ────────────────────────────────────────────────────────────────────

#[test]
fn array_uniq_basic() {
    let result = run("[3, 1, 2, 1, 3].uniq");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(3),
            Object::Int(1),
            Object::Int(2)
        ]))
    );
}

#[test]
fn array_uniq_error_with_args() {
    let err = run_err("[1].uniq(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_dup_independent_coverage() {
    let result = run("a = [1, 2, 3]\nb = a.dup\nb << 4\na.length");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Array#[] with start,length ──────────────────────────────────────────────

#[test]
fn array_slice_with_start_and_length() {
    let result = run("[10, 20, 30, 40, 50][1, 3]");
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 3);
    } else {
        panic!("expected array");
    }
}

// ── Array#min / max coverage ─────────────────────────────────────────────────

#[test]
fn array_max_integers_coverage() {
    let result = run("[3, 1, 4, 1, 5].max");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn array_max_floats_coverage() {
    let result = run("[1.5, 2.7, 0.3].max");
    assert_eq!(result, Some(Object::Float(2.7)));
}

#[test]
fn array_max_empty_coverage() {
    let result = run("[].max");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_min_integers_coverage() {
    let result = run("[3, 1, 4, 1, 5].min");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn array_min_empty_coverage() {
    let result = run("[].min");
    assert_eq!(result, Some(Object::Nil));
}

// ── Array#uniq coverage ──────────────────────────────────────────────────────

#[test]
fn array_uniq_removes_duplicates_coverage() {
    let result = run("[1, 2, 2, 3, 3, 3].uniq.length");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_uniq_empty_coverage() {
    let result = run("[].uniq.length");
    assert_eq!(result, Some(Object::Int(0)));
}
