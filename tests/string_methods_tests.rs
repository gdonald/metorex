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
fn string_length() {
    let result = run(r#""hello".length"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn string_size_alias() {
    let result = run(r#""hello".size"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn string_upcase() {
    let result = run(r#""hello".upcase"#);
    assert_eq!(result, Some(Object::string("HELLO")));
}

#[test]
fn string_downcase() {
    let result = run(r#""HELLO".downcase"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_reverse() {
    let result = run(r#""hello".reverse"#);
    assert_eq!(result, Some(Object::string("olleh")));
}

#[test]
fn string_trim() {
    let result = run(r#""  hello  ".trim"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_strip_alias() {
    let result = run(r#""  hello  ".strip"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_split_with_separator() {
    let result = run(r#""one,two,three".split(",")"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("one"),
            Object::string("two"),
            Object::string("three"),
        ]))
    );
}

#[test]
fn string_split_whitespace() {
    let result = run(r#""hello world".split"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("hello"),
            Object::string("world"),
        ]))
    );
}

#[test]
fn string_slice() {
    let result = run(r#""hello world".slice(0, 5)"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_slice_negative_index() {
    let result = run(r#""hello world".slice(-5, 5)"#);
    assert_eq!(result, Some(Object::string("world")));
}

#[test]
fn string_include_true() {
    let result = run(r#""hello world".include?("hello")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_include_false() {
    let result = run(r#""hello world".include?("xyz")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_contains_alias() {
    let result = run(r#""hello world".contains?("world")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_starts_with_true() {
    let result = run(r#""hello world".starts_with?("hello")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_starts_with_false() {
    let result = run(r#""hello world".starts_with?("world")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_ends_with_true() {
    let result = run(r#""hello world".ends_with?("world")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_ends_with_false() {
    let result = run(r#""hello world".ends_with?("hello")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_start_with_true() {
    let result = run(r#""hello world".start_with?("hello")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_start_with_multiple_one_match() {
    let result = run(r#""hello world".start_with?("nope", "hel")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_end_with_true() {
    let result = run(r#""hello world".end_with?("world")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_ljust_pads_with_spaces() {
    let result = run(r#""hi".ljust(5)"#);
    assert_eq!(result, Some(Object::string("hi   ")));
}

#[test]
fn string_ljust_with_pad_string() {
    let result = run(r#""hi".ljust(6, "-")"#);
    assert_eq!(result, Some(Object::string("hi----")));
}

#[test]
fn string_ljust_no_change_when_wider() {
    let result = run(r#""hello".ljust(3)"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_rjust_pads_with_spaces() {
    let result = run(r#""hi".rjust(5)"#);
    assert_eq!(result, Some(Object::string("   hi")));
}

#[test]
fn string_rjust_with_pad_string() {
    let result = run(r#""7".rjust(4, "0")"#);
    assert_eq!(result, Some(Object::string("0007")));
}

#[test]
fn string_tty_returns_false() {
    let result = run(r#""hello".tty?"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_isatty_returns_false() {
    let result = run(r#""hello".isatty"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_flush_returns_nil() {
    let result = run(r#""hello".flush"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn string_sync_returns_nil() {
    let result = run(r#""hello".sync"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn string_fsync_returns_nil() {
    let result = run(r#""hello".fsync"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn string_ljust_no_args_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hi".ljust"#));
    assert!(result.is_err());
}

#[test]
fn string_ljust_non_integer_width_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hi".ljust("3")"#));
    assert!(result.is_err());
}

#[test]
fn string_ljust_empty_pad_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hi".ljust(5, "")"#));
    assert!(result.is_err());
}

#[test]
fn string_ljust_non_string_pad_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hi".ljust(5, 42)"#));
    assert!(result.is_err());
}

#[test]
fn string_ljust_too_many_args_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hi".ljust(5, "-", "extra")"#));
    assert!(result.is_err());
}

#[test]
fn string_end_with_false() {
    let result = run(r#""hello world".end_with?("hello")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_start_with_false_all() {
    let result = run(r#""hello".start_with?("x", "y")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_start_with_no_args_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hello".start_with?"#));
    assert!(result.is_err());
}

#[test]
fn string_start_with_non_string_arg_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hello".start_with?(42)"#));
    assert!(result.is_err());
}

// ── String#+ method (line 57-78) — exercised via .send(:+, …) ─────────────

#[test]
fn string_plus_method_via_send_concatenates() {
    let result = run(r#""hello".send(:+, ", world")"#);
    assert_eq!(result, Some(Object::string("hello, world")));
}

#[test]
fn string_plus_method_via_send_no_args_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hello".send(:+)"#));
    assert!(result.is_err());
}

#[test]
fn string_plus_method_via_send_non_string_arg_errors() {
    let result = std::panic::catch_unwind(|| run(r#""hello".send(:+, 42)"#));
    assert!(result.is_err());
}

#[test]
fn array_join_with_separator() {
    let result = run(r#"["one", "two", "three"].join(", ")"#);
    assert_eq!(result, Some(Object::string("one, two, three")));
}

#[test]
fn array_join_no_separator() {
    let result = run(r#"["one", "two", "three"].join"#);
    assert_eq!(result, Some(Object::string("onetwothree")));
}

// ── chars ────────────────────────────────────────────────────────────────────

#[test]
fn string_chars_returns_array_of_chars() {
    let result = run(r#""abc".chars"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("a"),
            Object::string("b"),
            Object::string("c"),
        ]))
    );
}

#[test]
fn string_chars_error_with_args() {
    let err = run_err(r#""hello".chars(1)"#);
    assert!(err.contains("argument"));
}

// ── bytes ────────────────────────────────────────────────────────────────────

#[test]
fn string_bytes_returns_array_of_ints() {
    let result = run(r#""abc".bytes"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(97),
            Object::Int(98),
            Object::Int(99),
        ]))
    );
}

#[test]
fn string_bytes_error_with_args() {
    let err = run_err(r#""hello".bytes(1)"#);
    assert!(err.contains("argument"));
}

// ── each_char ────────────────────────────────────────────────────────────────

#[test]
fn string_each_char_iterates() {
    let result = run(r#"
result = []
"abc".each_char { |c| result.push(c) }
result
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("a"),
            Object::string("b"),
            Object::string("c"),
        ]))
    );
}

#[test]
fn string_each_char_error_with_args() {
    let err = run_err(r#""hello".each_char(1) { |c| c }"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_each_char_error_no_block() {
    let err = run_err(r#""hello".each_char"#);
    assert!(err.contains("block"));
}

// ── slice ────────────────────────────────────────────────────────────────────

#[test]
fn string_slice_past_end_returns_empty() {
    let result = run(r#""hi".slice(10, 5)"#);
    assert_eq!(result, Some(Object::string("")));
}

#[test]
fn string_slice_error_wrong_arg_count() {
    let err = run_err(r#""hello".slice(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_slice_error_wrong_type() {
    let err = run_err(r#""hello".slice("x", 2)"#);
    assert!(err.contains("Integer"));
}

// ── error paths for basic methods ────────────────────────────────────────────

#[test]
fn string_length_error_with_args() {
    let err = run_err(r#""hello".length(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_upcase_error_with_args() {
    let err = run_err(r#""hello".upcase(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_downcase_error_with_args() {
    let err = run_err(r#""hello".downcase(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_trim_error_with_args() {
    let err = run_err(r#""hello".trim(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_reverse_error_with_args() {
    let err = run_err(r#""hello".reverse(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_size_error_with_args() {
    let err = run_err(r#""hello".size(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_strip_error_with_args() {
    let err = run_err(r#""hello".strip(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_split_error_too_many_args() {
    let err = run_err(r#""hello".split(",", "extra")"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_split_error_non_string_sep() {
    let err = run_err(r#""hello".split(42)"#);
    assert!(err.contains("String"));
}

#[test]
fn string_include_error_wrong_count() {
    let err = run_err(r#""hello".include?"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_include_error_non_string_arg() {
    let err = run_err(r#""hello".include?(42)"#);
    assert!(err.contains("String"));
}

#[test]
fn string_starts_with_error_wrong_count() {
    let err = run_err(r#""hello".starts_with?"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_starts_with_error_non_string_arg() {
    let err = run_err(r#""hello".starts_with?(42)"#);
    assert!(err.contains("String"));
}

#[test]
fn string_ends_with_error_wrong_count() {
    let err = run_err(r#""hello".ends_with?"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_ends_with_error_non_string_arg() {
    let err = run_err(r#""hello".ends_with?(42)"#);
    assert!(err.contains("String"));
}

// ── + operator via method dispatch ──────────────────────────────────────────

#[test]
fn string_concat_operator() {
    let result = run(r#""hello" + " world""#);
    assert_eq!(result, Some(Object::string("hello world")));
}

// ── to_i ────────────────────────────────────────────────────────────────────

#[test]
fn string_to_i_basic() {
    assert_eq!(run(r#""42".to_i"#), Some(Object::Int(42)));
}

#[test]
fn string_to_i_with_spaces() {
    assert_eq!(run(r#""  99  ".to_i"#), Some(Object::Int(99)));
}

#[test]
fn string_to_i_non_numeric() {
    assert_eq!(run(r#""hello".to_i"#), Some(Object::Int(0)));
}

#[test]
fn string_to_i_negative() {
    assert_eq!(run(r#""-7".to_i"#), Some(Object::Int(-7)));
}

#[test]
fn string_to_i_leading_digits() {
    assert_eq!(run(r#""3abc".to_i"#), Some(Object::Int(3)));
}

#[test]
fn string_to_i_error_with_args() {
    let err = run_err(r#""42".to_i(10)"#);
    assert!(err.contains("argument"));
}

// ── to_f ────────────────────────────────────────────────────────────────────

#[test]
fn string_to_f_basic() {
    assert_eq!(run(r#""3.14".to_f"#), Some(Object::Float(3.14)));
}

#[test]
fn string_to_f_non_numeric() {
    assert_eq!(run(r#""hello".to_f"#), Some(Object::Float(0.0)));
}

#[test]
fn string_to_f_error_with_args() {
    let err = run_err(r#""3.14".to_f(1)"#);
    assert!(err.contains("argument"));
}

// ── dup ─────────────────────────────────────────────────────────────────────

#[test]
fn string_dup() {
    assert_eq!(run(r#""hello".dup"#), Some(Object::string("hello")));
}

#[test]
fn string_dup_error_with_args() {
    let err = run_err(r#""hello".dup(1)"#);
    assert!(err.contains("argument"));
}

// ── gsub ────────────────────────────────────────────────────────────────────

#[test]
fn string_gsub_basic() {
    assert_eq!(
        run(r#""hello world".gsub("o", "0")"#),
        Some(Object::string("hell0 w0rld"))
    );
}

#[test]
fn string_gsub_error_wrong_count() {
    let err = run_err(r#""hello".gsub("o")"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_gsub_error_non_string_pattern() {
    let err = run_err(r#""hello".gsub(42, "x")"#);
    assert!(err.contains("String"));
}

#[test]
fn string_gsub_error_non_string_replacement() {
    let err = run_err(r#""hello".gsub("o", 42)"#);
    assert!(err.contains("String"));
}

// ── sub ─────────────────────────────────────────────────────────────────────

#[test]
fn string_sub_basic() {
    assert_eq!(
        run(r#""hello hello".sub("hello", "hi")"#),
        Some(Object::string("hi hello"))
    );
}

#[test]
fn string_sub_error_wrong_count() {
    let err = run_err(r#""hello".sub("o")"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_sub_error_non_string_pattern() {
    let err = run_err(r#""hello".sub(42, "x")"#);
    assert!(err.contains("String"));
}

#[test]
fn string_sub_error_non_string_replacement() {
    let err = run_err(r#""hello".sub("o", 42)"#);
    assert!(err.contains("String"));
}

// ── empty? ──────────────────────────────────────────────────────────────────

#[test]
fn string_empty_true() {
    assert_eq!(run(r#""".empty?"#), Some(Object::Bool(true)));
}

#[test]
fn string_empty_false() {
    assert_eq!(run(r#""hi".empty?"#), Some(Object::Bool(false)));
}

#[test]
fn string_empty_error_with_args() {
    let err = run_err(r#""hi".empty?(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn string_dup_independent_coverage() {
    let result = run("a = \"hello\"\nb = a.dup\nb");
    assert_eq!(result, Some(Object::string("hello")));
}

// ── match? (string_methods.rs lines 36-63) ──────────────────────────────

#[test]
fn string_match_with_regex_true() {
    let result = run(r#""hello".match?(/ell/)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_match_with_regex_false() {
    let result = run(r#""hello".match?(/xyz/)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_match_with_regex_case_insensitive_flag() {
    let result = run(r#""Hello".match?(/HELLO/i)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_match_with_string_pattern() {
    let result = run(r#""hello".match?("ell")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_match_with_invalid_pattern_returns_false() {
    // Regex compile error (line 63): invalid regex → returns false.
    let result = run(r#""hello".match?("[invalid")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_match_with_non_regex_non_string_errors() {
    let err = run_err(r#""hello".match?(42)"#);
    assert!(err.contains("Regexp") || err.contains("String") || err.contains("match?"));
}

#[test]
fn string_match_with_wrong_arg_count_errors() {
    let err = run_err(r#""hello".match?()"#);
    assert!(err.contains("argument") || err.contains("match?"));
}

// ── String#last (lines 135-139) ─────────────────────────────────────────

#[test]
fn string_last_on_nonempty() {
    assert_eq!(run(r#""hello".last"#), Some(Object::string("o")));
}

#[test]
fn string_last_on_empty_returns_nil() {
    assert_eq!(run(r#""".last"#), Some(Object::Nil));
}
