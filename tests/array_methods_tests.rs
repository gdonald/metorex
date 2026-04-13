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

// ── Array#pack ───────────────────────────────────────────────────────────────

#[test]
fn array_pack_char() {
    let result = run("[65].pack('c')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 1);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_short() {
    let result = run("[1].pack('s')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 2);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_int() {
    let result = run("[1].pack('l')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_long() {
    let result = run("[1].pack('q')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_native_long() {
    let result = run("[0].pack('l!')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_unsupported_directive_error() {
    let err = run_err("[1].pack('Z')");
    assert!(err.contains("unsupported"));
}

#[test]
fn array_pack_wrong_arg_type_error() {
    let err = run_err("[1].pack(42)");
    assert!(err.contains("String"));
}

// ── Array#min / max ──────────────────────────────────────────────────────────

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

// ── Array#uniq ───────────────────────────────────────────────────────────────

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

// ── Array#pack various directives ────────────────────────────────────────────

#[test]
fn array_pack_v_directive() {
    let result = run("[1].pack('V')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_q_directive() {
    let result = run("[1].pack('Q')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_c_directive() {
    let result = run("[65].pack('C')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 1);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_s_directive() {
    let result = run("[1].pack('S')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 2);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_v_lowercase_directive() {
    let result = run("[1].pack('v')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 2);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_i_directive() {
    let result = run("[1].pack('I')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_l_directive() {
    let result = run("[1].pack('L')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_big_j_directive() {
    let result = run("[1].pack('J')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}

#[test]
fn array_pack_native_i_directive() {
    let result = run("[0].pack('i!')");
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    } else {
        panic!("expected string");
    }
}
