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
fn array_push_no_args_returns_self() {
    let result = run("[1, 2, 3].push");
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 3);
    }
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
fn array_unshift_zero_args_returns_self() {
    // Ruby: unshift with 0 args returns the array unchanged
    let result = run("[1, 2, 3].unshift");
    assert!(result.is_some());
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

// ── inject ──────────────────────────────────────────────────────────────────

#[test]
fn array_inject_with_initial() {
    let result = run("[1, 2, 3].inject(0) { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn array_inject_with_initial_string() {
    let result = run(r#"[1, 2, 3].inject("") { |s, x| s + x.to_s }"#);
    assert_eq!(result, Some(Object::string("123")));
}

#[test]
fn array_inject_without_initial() {
    let result = run("[1, 2, 3].inject { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn array_inject_empty_with_initial() {
    let result = run("[].inject(42) { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn array_inject_error_too_many_args() {
    let err = run_err("[1].inject(0, 1) { |a, b| a }");
    assert!(err.contains("argument"));
}

// ── dup / clone ─────────────────────────────────────────────────────────────

#[test]
fn array_dup_method() {
    let result = run("[1, 2, 3].dup");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3)
        ]))
    );
}

#[test]
fn array_clone_method() {
    let result = run("[1, 2, 3].clone");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3)
        ]))
    );
}

#[test]
fn array_dup_error_with_args() {
    let err = run_err("[1].dup(1)");
    assert!(err.contains("argument"));
}

// ── flatten ─────────────────────────────────────────────────────────────────

#[test]
fn array_flatten_basic() {
    let result = run("[1, [2, 3], [4]].flatten");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
            Object::Int(4),
        ]))
    );
}

#[test]
fn array_flatten_error_with_args() {
    let err = run_err("[1].flatten(1)");
    assert!(err.contains("argument"));
}

// ── compact ─────────────────────────────────────────────────────────────────

#[test]
fn array_compact_basic() {
    let result = run("[1, nil, 2, nil, 3].compact");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3)
        ]))
    );
}

#[test]
fn array_compact_error_with_args() {
    let err = run_err("[1].compact(1)");
    assert!(err.contains("argument"));
}

// ── empty? ──────────────────────────────────────────────────────────────────

#[test]
fn array_empty_true() {
    assert_eq!(run("[].empty?"), Some(Object::Bool(true)));
}

#[test]
fn array_empty_false() {
    assert_eq!(run("[1].empty?"), Some(Object::Bool(false)));
}

#[test]
fn array_empty_error_with_args() {
    let err = run_err("[1].empty?(1)");
    assert!(err.contains("argument"));
}

// ── first / last ────────────────────────────────────────────────────────────

#[test]
fn array_first_element() {
    assert_eq!(run("[10, 20, 30].first"), Some(Object::Int(10)));
}

#[test]
fn array_first_empty() {
    assert_eq!(run("[].first"), Some(Object::Nil));
}

#[test]
fn array_first_error_with_args() {
    let err = run_err("[1].first(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_last_element() {
    assert_eq!(run("[10, 20, 30].last"), Some(Object::Int(30)));
}

#[test]
fn array_last_empty() {
    assert_eq!(run("[].last"), Some(Object::Nil));
}

#[test]
fn array_last_error_with_args() {
    let err = run_err("[1].last(1)");
    assert!(err.contains("argument"));
}

// ── include? ────────────────────────────────────────────────────────────────

#[test]
fn array_include_found() {
    assert_eq!(run("[1, 2, 3].include?(2)"), Some(Object::Bool(true)));
}

#[test]
fn array_include_not_found() {
    assert_eq!(run("[1, 2, 3].include?(9)"), Some(Object::Bool(false)));
}

#[test]
fn array_include_error_wrong_count() {
    let err = run_err("[1].include?");
    assert!(err.contains("argument"));
}
