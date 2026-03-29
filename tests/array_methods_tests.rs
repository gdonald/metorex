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

#[test]
fn array_length() {
    let result = run("[1, 2, 3].length");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_size_alias() {
    let result = run("[1, 2, 3].size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_push_returns_array() {
    let result = run(r#"
arr = [1, 2]
arr.push(3)
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

#[test]
fn array_pop_returns_last_element() {
    let result = run(r#"
arr = [1, 2, 3]
arr.pop
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_pop_mutates_array() {
    let result = run(r#"
arr = [1, 2, 3]
arr.pop
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![Object::Int(1), Object::Int(2)]))
    );
}

#[test]
fn array_pop_empty_returns_nil() {
    let result = run(r#"
arr = []
arr.pop
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_shift_returns_first_element() {
    let result = run(r#"
arr = [1, 2, 3]
arr.shift
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn array_shift_mutates_array() {
    let result = run(r#"
arr = [1, 2, 3]
arr.shift
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![Object::Int(2), Object::Int(3)]))
    );
}

#[test]
fn array_shift_empty_returns_nil() {
    let result = run(r#"
arr = []
arr.shift
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_unshift_prepends_element() {
    let result = run(r#"
arr = [2, 3]
arr.unshift(1)
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

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

#[test]
fn array_reverse_integers() {
    let result = run(r#"
arr = [1, 2, 3, 4, 5]
arr.reverse
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(5),
            Object::Int(4),
            Object::Int(3),
            Object::Int(2),
            Object::Int(1),
        ]))
    );
}

#[test]
fn array_reverse_does_not_mutate_original() {
    let result = run(r#"
arr = [1, 2, 3]
arr.reverse
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

#[test]
fn array_map_doubles() {
    let result = run("[1, 2, 3].map { |n| n * 2 }");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6),
        ]))
    );
}

#[test]
fn array_select_filters_evens() {
    let result = run("[1, 2, 3, 4, 5, 6].select { |n| n % 2 == 0 }");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6),
        ]))
    );
}

#[test]
fn array_filter_alias() {
    let result = run("[1, 2, 3, 4].filter { |n| n > 2 }");
    assert_eq!(
        result,
        Some(Object::array(vec![Object::Int(3), Object::Int(4)]))
    );
}

#[test]
fn array_reduce_with_initial() {
    let result = run("[1, 2, 3, 4, 5].reduce(0) { |acc, n| acc + n }");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn array_reduce_without_initial() {
    let result = run("[1, 2, 3, 4, 5].reduce { |acc, n| acc + n }");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn array_each_iterates() {
    let result = run(r#"
sum = 0
[1, 2, 3].each { |n| sum = sum + n }
sum
"#);
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn array_join_with_separator() {
    let result = run(r#"
arr = ["a", "b", "c"]
arr.join(", ")
"#);
    assert_eq!(result, Some(Object::string("a, b, c")));
}

#[test]
fn array_join_no_separator() {
    let result = run(r#"
arr = ["a", "b", "c"]
arr.join
"#);
    assert_eq!(result, Some(Object::string("abc")));
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

// ── each control flow ────────────────────────────────────────────────────────

#[test]
fn array_each_with_break() {
    let result = run(r#"
sum = 0
[1, 2, 3, 4, 5].each do |n|
  if n == 3
    break
  end
  sum = sum + n
end
sum
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_each_with_continue() {
    let result = run(r#"
sum = 0
[1, 2, 3, 4, 5].each do |n|
  if n == 3
    continue
  end
  sum = sum + n
end
sum
"#);
    assert_eq!(result, Some(Object::Int(12)));
}

#[test]
fn array_each_error_no_block() {
    let err = run_err("[1, 2, 3].each");
    assert!(err.contains("block"));
}

#[test]
fn array_each_error_with_args() {
    let err = run_err("[1, 2, 3].each(1) { |n| n }");
    assert!(err.contains("argument"));
}

// ── map error paths ──────────────────────────────────────────────────────────

#[test]
fn array_map_error_no_block() {
    let err = run_err("[1, 2, 3].map");
    assert!(err.contains("block"));
}

#[test]
fn array_map_error_with_args() {
    let err = run_err("[1, 2, 3].map(1) { |n| n }");
    assert!(err.contains("argument"));
}

// ── select/filter error paths ─────────────────────────────────────────────────

#[test]
fn array_select_error_no_block() {
    let err = run_err("[1, 2, 3].select");
    assert!(err.contains("block"));
}

#[test]
fn array_select_error_with_args() {
    let err = run_err("[1, 2, 3].select(1) { |n| n }");
    assert!(err.contains("argument"));
}

#[test]
fn array_filter_error_no_block() {
    let err = run_err("[1, 2, 3].filter");
    assert!(err.contains("block"));
}

// ── reduce error paths ───────────────────────────────────────────────────────

#[test]
fn array_reduce_error_no_block() {
    let err = run_err("[1, 2, 3].reduce");
    assert!(err.contains("block"));
}

#[test]
fn array_reduce_error_too_many_args() {
    let err = run_err("[1, 2, 3].reduce(0, 1) { |acc, n| acc + n }");
    assert!(err.contains("argument"));
}

#[test]
fn array_reduce_empty_returns_nil() {
    let result = run("[].reduce { |acc, n| acc + n }");
    assert_eq!(result, Some(Object::Nil));
}

// ── error paths for basic methods ────────────────────────────────────────────

#[test]
fn array_length_error_with_args() {
    let err = run_err("[1, 2, 3].length(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_push_error_wrong_args() {
    let err = run_err("[1, 2, 3].push");
    assert!(err.contains("argument"));
}

#[test]
fn array_pop_error_with_args() {
    let err = run_err("[1, 2, 3].pop(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_size_error_with_args() {
    let err = run_err("[1, 2, 3].size(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_shift_error_with_args() {
    let err = run_err("[1, 2, 3].shift(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_unshift_error_wrong_args() {
    let err = run_err("[1, 2, 3].unshift");
    assert!(err.contains("argument"));
}

#[test]
fn array_sort_error_with_args() {
    let err = run_err("[1, 2, 3].sort(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_reverse_error_with_args() {
    let err = run_err("[1, 2, 3].reverse(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_join_error_too_many_args() {
    let err = run_err(r#"["a", "b"].join(",", "extra")"#);
    assert!(err.contains("argument"));
}

#[test]
fn array_join_error_non_string_sep() {
    let err = run_err(r#"["a", "b"].join(42)"#);
    assert!(err.contains("String"));
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

// ── append alias ─────────────────────────────────────────────────────────────

#[test]
fn array_append_alias() {
    let result = run(r#"
arr = [1, 2]
arr.append(3)
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}
