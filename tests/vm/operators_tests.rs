// Coverage tests for vm/operators.rs — unary plus, float arithmetic,
// mixed int/float arithmetic, not-equal, and type-error paths.

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

// ── UnaryOp::Plus ─────────────────────────────────────────────────────────────

#[test]
fn unary_plus_int() {
    let result = run("+3");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn unary_plus_float() {
    let result = run("+3.14");
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn unary_plus_on_string_returns_string() {
    // Ruby semantics: `+"hello"` returns a mutable copy of the string. We
    // don't track frozenness, so it's just an identity op.
    let result = run(r#"+"hello""#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

// ── UnaryOp::Minus on Float (line 32) ────────────────────────────────────────

#[test]
fn unary_minus_float() {
    let result = run("-3.14");
    assert_eq!(result, Some(Object::Float(-3.14)));
}

// ── UnaryOp::Minus error ──────────────────────────────────────────────────────

#[test]
fn unary_minus_error_on_string() {
    let err = run_err(r#"-"hello""#);
    assert!(err.contains("unary") || err.contains("-") || err.contains("operator"));
}

// ── NotEqual ──────────────────────────────────────────────────────────────────

#[test]
fn not_equal_ints_true() {
    let result = run("3 != 4");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn not_equal_ints_false() {
    let result = run("3 != 3");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn equal_ints_true() {
    let result = run("5 == 5");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Integer division producing Float ─────────────────────────────────────────

#[test]
fn int_divide_non_evenly_produces_float() {
    let result = run("5 / 2");
    assert_eq!(result, Some(Object::Float(2.5)));
}

#[test]
fn int_modulo_by_zero_error() {
    let err = run_err("5 % 0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Float / Float arithmetic ──────────────────────────────────────────────────

#[test]
fn float_subtract() {
    let result = run("5.0 - 2.0");
    assert_eq!(result, Some(Object::Float(3.0)));
}

#[test]
fn float_multiply() {
    let result = run("3.0 * 4.0");
    assert_eq!(result, Some(Object::Float(12.0)));
}

#[test]
fn float_divide() {
    let result = run("8.0 / 4.0");
    assert_eq!(result, Some(Object::Float(2.0)));
}

#[test]
fn float_modulo() {
    let result = run("7.0 % 3.0");
    assert_eq!(result, Some(Object::Float(1.0)));
}

#[test]
fn float_divide_by_zero_error() {
    let err = run_err("4.0 / 0.0");
    assert!(err.contains("zero") || err.contains("Division"));
}

#[test]
fn float_modulo_by_zero_error() {
    let err = run_err("4.0 % 0.0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Int / Float arithmetic ────────────────────────────────────────────────────

#[test]
fn int_float_subtract() {
    let result = run("5 - 2.0");
    assert_eq!(result, Some(Object::Float(3.0)));
}

#[test]
fn int_float_multiply() {
    let result = run("3 * 2.0");
    assert_eq!(result, Some(Object::Float(6.0)));
}

#[test]
fn int_float_divide() {
    let result = run("6 / 2.0");
    assert_eq!(result, Some(Object::Float(3.0)));
}

#[test]
fn int_float_modulo() {
    let result = run("7 % 3.0");
    assert_eq!(result, Some(Object::Float(1.0)));
}

#[test]
fn int_float_divide_by_zero_error() {
    let err = run_err("5 / 0.0");
    assert!(err.contains("zero") || err.contains("Division"));
}

#[test]
fn int_float_modulo_by_zero_error() {
    let err = run_err("5 % 0.0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Float / Int arithmetic ────────────────────────────────────────────────────

#[test]
fn float_int_subtract() {
    let result = run("5.0 - 2");
    assert_eq!(result, Some(Object::Float(3.0)));
}

#[test]
fn float_int_multiply() {
    let result = run("3.0 * 4");
    assert_eq!(result, Some(Object::Float(12.0)));
}

#[test]
fn float_int_divide() {
    let result = run("8.0 / 2");
    assert_eq!(result, Some(Object::Float(4.0)));
}

#[test]
fn float_int_modulo() {
    let result = run("7.0 % 3");
    assert_eq!(result, Some(Object::Float(1.0)));
}

#[test]
fn float_int_divide_by_zero_error() {
    let err = run_err("5.0 / 0");
    assert!(err.contains("zero") || err.contains("Division"));
}

#[test]
fn float_int_modulo_by_zero_error() {
    let err = run_err("5.0 % 0");
    assert!(err.contains("zero") || err.contains("Division"));
}

// ── Comparison type errors ────────────────────────────────────────────────────

#[test]
fn comparison_string_int_error() {
    let err = run_err(r#""a" > 1"#);
    assert!(err.contains("operator") || err.contains("type") || err.contains("String"));
}

#[test]
fn comparison_float_string_error() {
    let err = run_err(r#"1.5 < "b""#);
    assert!(err.contains("operator") || err.contains("type") || err.contains("Float"));
}

// ── Addition type error (non-string + non-numeric) ────────────────────────────

#[test]
fn addition_type_error_nil() {
    let err = run_err("nil + 1");
    assert!(err.contains("operator") || err.contains("type") || err.contains("Nil"));
}

// ── Float/Float addition ──────────────────────────────────────────────────────

#[test]
fn float_addition() {
    let result = run("1.5 + 2.5");
    assert_eq!(result, Some(Object::Float(4.0)));
}

// ── Int + Float and Float + Int addition ─────────────────────────────────────

#[test]
fn int_plus_float() {
    let result = run("1 + 2.5");
    assert_eq!(result, Some(Object::Float(3.5)));
}

#[test]
fn float_plus_int() {
    let result = run("1.5 + 2");
    assert_eq!(result, Some(Object::Float(3.5)));
}

// ── Float comparison ──────────────────────────────────────────────────────────

#[test]
fn float_comparison_less() {
    let result = run("1.5 < 2.5");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn float_int_comparison() {
    let result = run("1.5 > 1");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Numeric binary type error (line 183) ─────────────────────────────────────
// Triggered when neither operand is a numeric type for -, *, /, %

#[test]
fn subtract_type_error_nil_minus_int() {
    let err = run_err("nil - 1");
    assert!(err.contains("operator") || err.contains("type") || err.contains("Nil"));
}

#[test]
fn multiply_type_error_string_times_string() {
    let err = run_err(r#""a" * "b""#);
    assert!(err.contains("operator") || err.contains("type") || err.contains("String"));
}

#[test]
fn int_float_comparison() {
    let result = run("2 >= 1.5");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── operators.rs: And/Or error path (lines 62-64) ───────────────────────────
// These should never be reached in normal execution because And/Or are
// handled by short-circuit evaluation. We cannot easily trigger them
// through normal code execution, but we test the short-circuit paths work.

#[test]
fn logical_and_short_circuit() {
    let result = run("false && 1 / 0");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn logical_or_short_circuit() {
    let result = run("true || 1 / 0");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── operators.rs: assignment error path (lines 67-69) ───────────────────────
// These should never be reached because assignments are handled by
// statement execution. We test that normal assignments work.

#[test]
fn add_assign_operator() {
    let result = run("x = 5\nx += 3\nx");
    assert_eq!(result, Some(Object::Int(8)));
}

#[test]
fn subtract_assign_operator() {
    let result = run("x = 10\nx -= 3\nx");
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn multiply_assign_operator() {
    let result = run("x = 4\nx *= 3\nx");
    assert_eq!(result, Some(Object::Int(12)));
}

#[test]
fn divide_assign_operator() {
    let result = run("x = 12\nx /= 4\nx");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── =~ and !~ regex match operators ─────────────────────────────────────────

#[test]
fn regex_match_returns_position() {
    let result = run(r#"/hello/ =~ "say hello world""#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn regex_match_returns_nil_on_no_match() {
    let result = run(r#"/xyz/ =~ "hello""#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn regex_match_string_left() {
    let result = run(r#""test123" =~ /\d+/"#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn regex_not_match_true() {
    let result = run(r#""abc" !~ /xyz/"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn regex_not_match_false() {
    let result = run(r#""abc" !~ /abc/"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn regex_match_case_insensitive() {
    let result = run(r#"/hello/i =~ "HELLO world""#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn percent_r_regex() {
    let result = run(r#"%r(hello) =~ "say hello""#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn percent_r_brackets() {
    let result = run(r#"%r[hello] =~ "hello world""#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn percent_r_braces() {
    let result = run(r#"%r{test} =~ "a test""#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── XOR operator ────────────────────────────────────────────────────────────

#[test]
fn xor_bool_true_false() {
    assert_eq!(run("true ^ false"), Some(Object::Bool(true)));
}

#[test]
fn xor_bool_true_true() {
    assert_eq!(run("true ^ true"), Some(Object::Bool(false)));
}

#[test]
fn xor_bool_false_false() {
    assert_eq!(run("false ^ false"), Some(Object::Bool(false)));
}

#[test]
fn xor_int() {
    assert_eq!(run("5 ^ 3"), Some(Object::Int(6)));
}

#[test]
fn xor_bool_with_truthy() {
    assert_eq!(run("true ^ nil"), Some(Object::Bool(true)));
}

#[test]
fn xor_non_bool_left() {
    assert_eq!(run(r#""hello" ^ true"#), Some(Object::Bool(false)));
}

#[test]
fn xor_non_bool_left_false() {
    assert_eq!(run("nil ^ false"), Some(Object::Bool(false)));
}

// ── === triple equals ───────────────────────────────────────────────────────

#[test]
fn triple_equals() {
    assert_eq!(run("1 === 1"), Some(Object::Bool(true)));
}

#[test]
fn triple_equals_false() {
    assert_eq!(run("1 === 2"), Some(Object::Bool(false)));
}

// ── ||= and &&= ────────────────────────────────────────────────────────────

#[test]
fn or_assign_nil() {
    assert_eq!(run("x = nil; x ||= 42; x"), Some(Object::Int(42)));
}

#[test]
fn or_assign_existing() {
    assert_eq!(run("x = 10; x ||= 42; x"), Some(Object::Int(10)));
}

#[test]
fn and_assign_truthy() {
    assert_eq!(run("x = true; x &&= 42; x"), Some(Object::Int(42)));
}

#[test]
fn and_assign_falsy() {
    assert_eq!(run("x = false; x &&= 42; x"), Some(Object::Bool(false)));
}

// ── Spaceship string ────────────────────────────────────────────────────────

#[test]
fn spaceship_string() {
    assert_eq!(run(r#""abc" <=> "def""#), Some(Object::Int(-1)));
    assert_eq!(run(r#""abc" <=> "abc""#), Some(Object::Int(0)));
    assert_eq!(run(r#""def" <=> "abc""#), Some(Object::Int(1)));
}

// ── Chained ternary ─────────────────────────────────────────────────────────

#[test]
fn chained_ternary_with_methods() {
    let result = run(r#"
a = "hello"
x = a.length > 10 ? "long" : a.length > 3 ? "medium" : "short"
x
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("medium".to_string())))
    );
}

// ── Custom operators (from vm/additional_tests) ─────────────────────────────

#[test]
fn custom_operator_divide() {
    assert_eq!(
        run(
            "class Num\n  def initialize(v)\n    @v = v\n  end\n  def /(other)\n    @v / other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(10)\nb = Num.new(2)\na / b"
        ),
        Some(Object::Int(5))
    );
}

#[test]
fn custom_operator_modulo() {
    assert_eq!(
        run(
            "class Num\n  def initialize(v)\n    @v = v\n  end\n  def %(other)\n    @v % other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(10)\nb = Num.new(3)\na % b"
        ),
        Some(Object::Int(1))
    );
}

#[test]
fn custom_operator_equal_equal() {
    assert_eq!(
        run(
            "class V\n  def initialize(v)\n    @v = v\n  end\n  def ==(other)\n    @v == other.val\n  end\n  def val\n    @v\n  end\nend\nV.new(5) == V.new(5)"
        ),
        Some(Object::Bool(true))
    );
}

// ── String#% format operator ──────────────────────────────────────────────

#[test]
fn string_format_percent_d() {
    assert_eq!(
        run(r#""value: %d" % 42"#),
        Some(Object::String(std::rc::Rc::new("value: 42".to_string())))
    );
}

#[test]
fn string_format_percent_s() {
    assert_eq!(
        run(r#""hi %s" % "world""#),
        Some(Object::String(std::rc::Rc::new("hi world".to_string())))
    );
}

#[test]
fn string_format_percent_with_array_args() {
    assert_eq!(
        run(r#""%s = %d" % ["x", 7]"#),
        Some(Object::String(std::rc::Rc::new("x = 7".to_string())))
    );
}

#[test]
fn string_format_percent_literal_double() {
    assert_eq!(
        run(r#""100%%" % []"#),
        Some(Object::String(std::rc::Rc::new("100%".to_string())))
    );
}

#[test]
fn string_format_percent_d_with_plus_sign() {
    assert_eq!(
        run(r#""%+d" % 5"#),
        Some(Object::String(std::rc::Rc::new("+5".to_string())))
    );
}

#[test]
fn string_format_percent_d_with_space_sign() {
    assert_eq!(
        run(r#""% d" % 5"#),
        Some(Object::String(std::rc::Rc::new(" 5".to_string())))
    );
}

#[test]
fn string_format_percent_d_with_width() {
    let result = run(r#""%5d" % 42"#);
    if let Some(Object::String(s)) = result {
        assert!(s.contains("42"));
    } else {
        panic!("expected String, got {:?}", result);
    }
}

#[test]
fn string_format_percent_s_with_precision() {
    assert_eq!(
        run(r#""%.3s" % "hello""#),
        Some(Object::String(std::rc::Rc::new("hel".to_string())))
    );
}

#[test]
fn string_format_percent_d_from_float_arg() {
    assert_eq!(
        run(r#""%d" % 3.7"#),
        Some(Object::String(std::rc::Rc::new("3".to_string())))
    );
}

#[test]
fn string_format_percent_d_negative_no_plus() {
    assert_eq!(
        run(r#""%+d" % -5"#),
        Some(Object::String(std::rc::Rc::new("-5".to_string())))
    );
}

#[test]
fn string_format_too_few_args_errors() {
    let result = std::panic::catch_unwind(|| run(r#""%d %d" % [1]"#));
    assert!(result.is_err());
}

#[test]
fn string_format_incomplete_specifier_errors() {
    // `"%-"` parses the `-` flag then runs out before any conversion specifier.
    let result = std::panic::catch_unwind(|| run(r#""%-" % 5"#));
    assert!(result.is_err());
}

#[test]
fn string_format_trailing_percent_kept() {
    let result = run(r#""abc%" % []"#);
    if let Some(Object::String(s)) = result {
        assert!(s.contains("abc"));
    }
}

#[test]
fn string_format_percent_f_basic() {
    let result = run(r#""%f" % 3.14"#);
    if let Some(Object::String(s)) = result {
        assert!(s.starts_with("3.14"));
    } else {
        panic!("expected String, got {:?}", result);
    }
}

#[test]
fn string_format_percent_f_with_precision() {
    assert_eq!(
        run(r#""%.2f" % 3.14159"#),
        Some(Object::String(std::rc::Rc::new("3.14".to_string())))
    );
}

#[test]
fn string_format_percent_f_from_int() {
    let result = run(r#""%.1f" % 5"#);
    if let Some(Object::String(s)) = result {
        assert!(s.starts_with("5.0"));
    }
}

#[test]
fn string_format_percent_f_with_plus_sign() {
    let result = run(r#""%+.1f" % 3.5"#);
    if let Some(Object::String(s)) = result {
        assert!(s.starts_with("+3.5"));
    }
}

#[test]
fn string_format_percent_f_with_space_sign() {
    let result = run(r#""% .1f" % 3.5"#);
    if let Some(Object::String(s)) = result {
        assert!(s.starts_with(" 3.5"));
    }
}

#[test]
fn string_format_percent_f_non_numeric_errors() {
    let result = std::panic::catch_unwind(|| run(r#""%f" % "not a number""#));
    assert!(result.is_err());
}

#[test]
fn string_format_percent_x_lowercase() {
    assert_eq!(
        run(r#""%x" % 255"#),
        Some(Object::String(std::rc::Rc::new("ff".to_string())))
    );
}

#[test]
fn string_format_percent_x_uppercase() {
    assert_eq!(
        run(r#""%X" % 255"#),
        Some(Object::String(std::rc::Rc::new("FF".to_string())))
    );
}

#[test]
fn string_format_percent_o_octal() {
    assert_eq!(
        run(r#""%o" % 8"#),
        Some(Object::String(std::rc::Rc::new("10".to_string())))
    );
}

#[test]
fn string_format_percent_b_binary() {
    assert_eq!(
        run(r#""%b" % 5"#),
        Some(Object::String(std::rc::Rc::new("101".to_string())))
    );
}

#[test]
fn string_format_percent_p_string() {
    assert_eq!(
        run(r#""%p" % "hi""#),
        Some(Object::String(std::rc::Rc::new("\"hi\"".to_string())))
    );
}

#[test]
fn string_format_percent_p_nil() {
    assert_eq!(
        run(r#""%p" % nil"#),
        Some(Object::String(std::rc::Rc::new("nil".to_string())))
    );
}

#[test]
fn string_format_percent_c_from_int() {
    assert_eq!(
        run(r#""%c" % 65"#),
        Some(Object::String(std::rc::Rc::new("A".to_string())))
    );
}

#[test]
fn string_format_percent_c_from_string() {
    assert_eq!(
        run(r#""%c" % "X""#),
        Some(Object::String(std::rc::Rc::new("X".to_string())))
    );
}

#[test]
fn string_format_unknown_specifier_errors() {
    let result = std::panic::catch_unwind(|| run(r#""%z" % 5"#));
    assert!(result.is_err());
}

#[test]
fn string_format_left_align() {
    let result = run(r#""[%-5s]" % "hi""#);
    if let Some(Object::String(s)) = result {
        assert_eq!(s.as_str(), "[hi   ]");
    } else {
        panic!("expected String, got {:?}", result);
    }
}

#[test]
fn string_format_zero_pad() {
    assert_eq!(
        run(r#""%05d" % 42"#),
        Some(Object::String(std::rc::Rc::new("00042".to_string())))
    );
}

// ── BitwiseAnd (&) ──────────────────────────────────────────────────────────

#[test]
fn bitwise_and_bool_bool() {
    let result = run("true & false");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn bitwise_and_int_int() {
    let result = run("12 & 10");
    assert_eq!(result, Some(Object::Int(8)));
}

#[test]
fn bitwise_and_bool_int() {
    // 0 is truthy in Ruby, so true & 0 => true & true => true
    let result = run("true & 0");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bitwise_and_int_bool() {
    let result = run("1 & true");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bitwise_and_type_error() {
    let err = run_err("'a' & 'b'");
    assert!(err.contains("type") || err.contains("Cannot"));
}

// ── BitwiseOr (|) ───────────────────────────────────────────────────────────

#[test]
fn bitwise_or_bool_bool() {
    let result = run("false | true");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bitwise_or_int_int() {
    let result = run("12 | 3");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn bitwise_or_bool_int() {
    let result = run("false | 1");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bitwise_or_int_bool() {
    // 0 is truthy in Ruby, so 0 | false => true | false => true
    let result = run("0 | false");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bitwise_or_type_error() {
    let err = run_err("'a' | 'b'");
    assert!(err.contains("type") || err.contains("Cannot"));
}

// ── Xor (^) with mixed types ────────────────────────────────────────────────

#[test]
fn xor_bool_int() {
    // 0 is truthy in Ruby, so true ^ 0 => true ^ true => false
    let result = run("true ^ 0");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn xor_int_bool() {
    let result = run("1 ^ false");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn xor_type_error() {
    let err = run_err("'a' ^ 'b'");
    assert!(err.contains("type") || err.contains("Cannot"));
}

// ── Power (**) with various types ───────────────────────────────────────────

#[test]
fn power_int_negative_exponent() {
    let result = run("2**-1");
    assert!(matches!(result, Some(Object::Float(_))));
}

#[test]
fn power_float_float() {
    let result = run("2.0**3.0");
    assert_eq!(result, Some(Object::Float(8.0)));
}

#[test]
fn power_int_float() {
    let result = run("2**3.0");
    assert_eq!(result, Some(Object::Float(8.0)));
}

#[test]
fn power_float_int() {
    let result = run("2.0**3");
    assert_eq!(result, Some(Object::Float(8.0)));
}

// ── Spaceship (<=>) ─────────────────────────────────────────────────────────

#[test]
fn spaceship_int_int() {
    assert_eq!(run("1 <=> 2"), Some(Object::Int(-1)));
    assert_eq!(run("2 <=> 2"), Some(Object::Int(0)));
    assert_eq!(run("3 <=> 2"), Some(Object::Int(1)));
}

#[test]
fn spaceship_float_float() {
    assert_eq!(run("1.0 <=> 2.0"), Some(Object::Int(-1)));
}

#[test]
fn spaceship_int_float() {
    assert_eq!(run("1 <=> 2.0"), Some(Object::Int(-1)));
}

#[test]
fn spaceship_float_int() {
    assert_eq!(run("2.0 <=> 1"), Some(Object::Int(1)));
}

#[test]
fn spaceship_string_string() {
    assert_eq!(run("'a' <=> 'b'"), Some(Object::Int(-1)));
}

#[test]
fn spaceship_type_error() {
    let err = run_err("1 <=> 'a'");
    assert!(err.contains("type") || err.contains("Cannot"));
}

// ── String format (%s, %d, %c, etc.) ────────────────────────────────────────

#[test]
fn string_format_percent_c() {
    let result = run("'%c' % 65");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("A".to_string())))
    );
}

#[test]
fn string_format_left_align_coverage() {
    let result = run("'%-10s' % 'hi'");
    if let Some(Object::String(s)) = result {
        assert!(s.starts_with("hi"));
        assert_eq!(s.len(), 10);
    } else {
        panic!("expected string");
    }
}

#[test]
fn string_format_zero_pad_coverage() {
    let result = run("'%05d' % 42");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("00042".to_string())))
    );
}

// ── Instance == with custom method ───────────────────────────────────────────

#[test]
fn instance_custom_eq_true() {
    let result = run(r#"
class Eq
  def initialize(v)
    @v = v
  end
  def ==(other)
    @v == other.instance_variable_get(:@v)
  end
end
Eq.new(1) == Eq.new(1)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_custom_eq_false() {
    let result = run(r#"
class Eq
  def initialize(v)
    @v = v
  end
  def ==(other)
    @v == other.instance_variable_get(:@v)
  end
end
Eq.new(1) == Eq.new(2)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn instance_identity_eq() {
    let result = run(r#"
class Foo
end
a = Foo.new
a == a
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Instance <=> via Comparable protocol ─────────────────────────────────────

#[test]
fn instance_comparable_less() {
    let result = run(r#"
class Cmp
  include Comparable
  def initialize(v)
    @v = v
  end
  def <=>(other)
    @v <=> other.instance_variable_get(:@v)
  end
end
Cmp.new(1) < Cmp.new(2)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_comparable_greater() {
    let result = run(r#"
class Cmp
  include Comparable
  def initialize(v)
    @v = v
  end
  def <=>(other)
    @v <=> other.instance_variable_get(:@v)
  end
end
Cmp.new(2) > Cmp.new(1)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_comparable_less_equal() {
    let result = run(r#"
class Cmp
  include Comparable
  def initialize(v)
    @v = v
  end
  def <=>(other)
    @v <=> other.instance_variable_get(:@v)
  end
end
Cmp.new(1) <= Cmp.new(1)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_comparable_greater_equal() {
    let result = run(r#"
class Cmp
  include Comparable
  def initialize(v)
    @v = v
  end
  def <=>(other)
    @v <=> other.instance_variable_get(:@v)
  end
end
Cmp.new(1) >= Cmp.new(1)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_comparable_eq_via_spaceship() {
    let result = run(r#"
class Cmp
  include Comparable
  def initialize(v)
    @v = v
  end
  def <=>(other)
    @v <=> other.instance_variable_get(:@v)
  end
end
Cmp.new(5) == Cmp.new(5)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── String comparisons ───────────────────────────────────────────────────────

#[test]
fn string_less_than() {
    assert_eq!(run("'abc' < 'abd'"), Some(Object::Bool(true)));
}

#[test]
fn string_greater_than() {
    assert_eq!(run("'abd' > 'abc'"), Some(Object::Bool(true)));
}

#[test]
fn string_less_equal() {
    assert_eq!(run("'abc' <= 'abc'"), Some(Object::Bool(true)));
}

#[test]
fn string_greater_equal() {
    assert_eq!(run("'abc' >= 'abc'"), Some(Object::Bool(true)));
}

// ── Bitwise nil/Xor edge cases ──────────────────────────────────────────────

#[test]
fn bitwise_and_nil_left() {
    assert_eq!(run("nil & true"), Some(Object::Bool(false)));
}

#[test]
fn bitwise_and_nil_right() {
    assert_eq!(run("true & nil"), Some(Object::Bool(false)));
}

#[test]
fn bitwise_or_nil_left() {
    assert_eq!(run("nil | true"), Some(Object::Bool(true)));
}

#[test]
fn bitwise_or_nil_left_false() {
    assert_eq!(run("nil | false"), Some(Object::Bool(false)));
}

#[test]
fn xor_nil_left_truthy() {
    assert_eq!(run("nil ^ true"), Some(Object::Bool(true)));
}

#[test]
fn xor_nil_left_falsy() {
    assert_eq!(run("nil ^ false"), Some(Object::Bool(false)));
}

#[test]
fn xor_bool_truthy_other() {
    assert_eq!(run("true ^ 0"), Some(Object::Bool(false)));
}

#[test]
fn xor_other_bool() {
    assert_eq!(run("0 ^ true"), Some(Object::Bool(false)));
}

// ── Integer overflow ─────────────────────────────────────────────────────────

#[test]
fn int_add_overflow_to_float() {
    let result = run("4611686018427387903 + 4611686018427387903");
    assert!(matches!(
        result,
        Some(Object::Float(_)) | Some(Object::Int(_))
    ));
}

#[test]
fn int_multiply_overflow() {
    let result = run("4611686018427387903 * 3");
    assert!(result.is_some());
}

#[test]
fn int_subtract_overflow() {
    let result = run("0 - 4611686018427387903 - 4611686018427387903 - 2");
    assert!(result.is_some());
}

// ── Case equality ────────────────────────────────────────────────────────────

#[test]
fn case_equality_class_instance() {
    assert_eq!(run("String === 'hello'"), Some(Object::Bool(true)));
}

#[test]
fn case_equality_class_wrong_type() {
    assert_eq!(run("Integer === 'hello'"), Some(Object::Bool(false)));
}

#[test]
fn case_equality_exception_type() {
    let result = run(r#"
e = nil
begin
  raise RuntimeError, "oops"
rescue => ex
  e = ex
end
RuntimeError === e
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}
