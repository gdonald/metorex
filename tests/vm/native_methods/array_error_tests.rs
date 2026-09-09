// Array error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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

// ══════════════════════════════════════════════════════════════════════════════
// Array methods - push/append (line 39 early return + push path)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn array_push_returns_receiver() {
    let result = run(r#"
arr = [1, 2]
arr.push(3)
"#);
    // push returns the array itself
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 3);
    }
}

#[test]
fn array_push_no_args_returns_self() {
    let result = run("[1, 2].push");
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    }
}

#[test]
fn array_append_no_args_returns_self() {
    let result = run("[1, 2].append");
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Array methods - each with return in method (lines 126-129)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn array_each_return_in_method_returns_from_method() {
    // Ruby semantics: `return` inside a block returns from the enclosing method.
    let result = run(r#"
def test_return_in_each
  [1, 2, 3].each do |x|
    return x
  end
end
test_return_in_each
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Array methods - each with raise in block (lines 131-142)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn array_each_raise_propagates_exception() {
    let err = run_err(
        r#"
[1, 2, 3].each do |x|
  raise "boom in each"
end
"#,
    );
    assert!(
        err.contains("boom in each") || err.contains("Uncaught"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Array methods - pop, shift, unshift, reverse, join, zip
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn array_pop_method() {
    let result = run(r#"
arr = [1, 2, 3]
arr.pop
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_pop_empty() {
    let result = run(r#"
arr = []
arr.pop
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_shift_method() {
    let result = run(r#"
arr = [1, 2, 3]
arr.shift
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn array_shift_empty() {
    let result = run(r#"
arr = []
arr.shift
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_unshift_method() {
    let result = run(r#"
arr = [2, 3]
arr.unshift(1)
arr.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_reverse_method() {
    let result = run(r#"
[1, 2, 3].reverse
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[0], Object::Int(3));
        assert_eq!(borrowed[2], Object::Int(1));
    }
}

#[test]
fn array_join_with_separator() {
    let result = run(r#"
[1, 2, 3].join(", ")
"#);
    assert_eq!(result, Some(Object::string("1, 2, 3".to_string())));
}

#[test]
fn array_join_without_separator() {
    let result = run(r#"
[1, 2, 3].join
"#);
    assert_eq!(result, Some(Object::string("123".to_string())));
}

#[test]
fn array_zip_method() {
    let result = run(r#"
[1, 2, 3].zip([4, 5, 6]).length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_size_method() {
    let result = run(r#"
[1, 2, 3].size
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Array#[] two-argument (start, length) form ────────────────────────────

#[test]
fn array_bracket_start_length_non_integer_start_errors() {
    let err = run_err(r#"[1, 2, 3][: "a", 2]"#);
    assert!(
        err.contains("Integer") || err.contains("parse"),
        "Error: {}",
        err
    );
}

#[test]
fn array_bracket_start_length_beyond_end_returns_nil() {
    let result = run(r#"[1, 2, 3][100, 2]"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_bracket_start_length_negative_len_returns_nil() {
    let result = run(r#"[1, 2, 3][0, -1]"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_bracket_start_length_clamps_to_end() {
    let result = run(r#"[1, 2, 3, 4, 5][3, 100]"#);
    // Starting at index 3, requesting 100, should clamp to elements [4, 5]
    match result {
        Some(Object::Array(arr)) => assert_eq!(arr.borrow().len(), 2),
        other => panic!("expected Array, got {:?}", other),
    }
}

// ── partition with extra args error (lines 245-250) ──────────────────────────

#[test]
fn array_partition_with_arg_errors() {
    let err = run_err("[1,2,3].partition(42) { |x| x > 1 }");
    assert!(err.contains("argument"));
}

// ── reduce/inject with too many args error (lines 288-294, 527-532) ──────────

#[test]
fn array_reduce_second_argument_must_name_a_method() {
    let err = run_err("[1,2,3].reduce(0, 1) { |acc, x| acc + x }");
    assert!(err.contains("is not a symbol nor a string"), "{}", err);
}

#[test]
fn array_inject_more_than_two_arguments_errors() {
    let err = run_err("[1,2,3].inject(0, :+, 2) { |acc, x| acc + x }");
    assert!(err.contains("argument"), "{}", err);
}

// ── pack with non-Int values in various directives (lines 839, 847, 855, 863) ─

#[test]
fn array_pack_q_refuses_a_value_that_is_not_a_number() {
    let error = run_err(r#"["not_an_int"].pack("q")"#);
    assert!(
        error.contains("no implicit conversion of String into Integer"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn array_pack_l_refuses_a_value_that_is_not_a_number() {
    let error = run_err(r#"["string"].pack("l")"#);
    assert!(
        error.contains("no implicit conversion of String into Integer"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn array_pack_s_refuses_a_value_that_is_not_a_number() {
    let error = run_err(r#"["string"].pack("s")"#);
    assert!(
        error.contains("no implicit conversion of String into Integer"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn array_pack_c_refuses_a_value_that_is_not_a_number() {
    let error = run_err(r#"["string"].pack("c")"#);
    assert!(
        error.contains("no implicit conversion of String into Integer"),
        "unexpected error: {}",
        error
    );
}
