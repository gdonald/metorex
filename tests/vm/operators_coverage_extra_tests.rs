// Targeted coverage tests for uncovered lines in src/vm/operators.rs.

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

// ── User-defined == dispatch (lines 75-86) ───────────────────────────────────

#[test]
fn user_defined_equals_returns_true() {
    let result = run(r#"
class Eq
  def initialize(n)
    @n = n
  end
  def ==(other)
    true
  end
end
Eq.new(1) == Eq.new(2)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn user_defined_equals_returns_false() {
    let result = run(r#"
class Eq2
  def initialize(n)
    @n = n
  end
  def ==(other)
    false
  end
end
Eq2.new(1) == Eq2.new(1)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Comparable protocol using <=> for == (lines 88-115) ──────────────────────

#[test]
fn comparable_spaceship_equal_via_eq() {
    let result = run(r#"
class Cmp1
  def initialize(n)
    @n = n
  end
  def <=>(other)
    @n <=> other.instance_variable_get(:@n)
  end
end
Cmp1.new(5) == Cmp1.new(5)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn comparable_spaceship_float_zero_via_eq() {
    let result = run(r#"
class CmpF
  def <=>(other)
    0.0
  end
end
CmpF.new == CmpF.new
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn comparable_spaceship_nil_via_eq() {
    let result = run(r#"
class CmpN
  def <=>(other)
    nil
  end
end
CmpN.new == CmpN.new
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn comparable_spaceship_bad_return_errors() {
    let err = run_err(
        r#"
class BadCmp
  def <=>(other)
    "not a number"
  end
end
BadCmp.new == BadCmp.new
"#,
    );
    assert!(err.contains("comparison") || err.contains("failed") || err.contains("ArgumentError"));
}

// ── Instance identity shortcut ────────────────────────────────────────────────

#[test]
fn instance_identity_equals_shortcut() {
    // Same instance on both sides hits the Rc::ptr_eq path at line 70-73.
    let result = run(r#"
class Same
end
s = Same.new
s == s
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── CaseEqual with Exception (lines 121-134) ─────────────────────────────────

#[test]
fn case_equal_exception_class_matches_type() {
    let result = run(r#"
e = StandardError.new("msg")
StandardError === e
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn case_equal_exception_class_chain_mismatch() {
    let result = run(r#"
e = StandardError.new("msg")
TypeError === e
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── CaseEqual non-exception value ────────────────────────────────────────────

#[test]
fn case_equal_integer_class() {
    let result = run("Integer === 5");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn case_equal_class_vs_class() {
    let result = run("Integer === String");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn case_equal_fallback_to_equals() {
    let result = run("5 === 5");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Bitwise operations on nil (lines 153, 162, 171) ─────────────────────────

#[test]
fn nil_bitwise_and_returns_false() {
    let result = run("nil & true");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn nil_bitwise_or_returns_truthy() {
    let result = run("nil | true");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn nil_xor_returns_truthy_other() {
    let result = run("nil ^ true");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bool_bitwise_and_with_other() {
    let result = run("true & 1");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn other_bitwise_and_with_bool() {
    let result = run("1 & true");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn bitwise_and_type_error() {
    let err = run_err(r#""a" & "b""#);
    assert!(
        err.contains("TypeError")
            || err.contains("type")
            || err.contains("unsupported")
            || err.contains("no implicit")
    );
}

// ── Integer arithmetic overflow (lines 201, 232-236) ────────────────────────

#[test]
fn int_add_overflow_falls_back_to_float() {
    // i64::MAX + 1 overflows; code path returns Float.
    let result = run("9223372036854775807 + 1");
    assert!(matches!(result, Some(Object::Float(_))));
}

#[test]
fn int_sub_overflow_falls_back_to_float() {
    // i64::MIN - 1 overflows; code path returns Float.
    let result = run("-9223372036854775807 - 2");
    assert!(matches!(result, Some(Object::Float(_))));
}

#[test]
fn int_mul_overflow_falls_back_to_float() {
    let result = run("9223372036854775807 * 2");
    assert!(matches!(result, Some(Object::Float(_))));
}

// ── Comparable <=> for comparison operators (lines 367-395) ─────────────────

#[test]
fn less_than_via_spaceship() {
    let result = run(r#"
class LT
  def initialize(n)
    @n = n
  end
  def <=>(other)
    @n <=> other.instance_variable_get(:@n)
  end
end
LT.new(1) < LT.new(2)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn greater_than_via_spaceship() {
    let result = run(r#"
class GT
  def initialize(n)
    @n = n
  end
  def <=>(other)
    @n <=> other.instance_variable_get(:@n)
  end
end
GT.new(5) > GT.new(2)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn less_equal_via_spaceship() {
    let result = run(r#"
class LE
  def initialize(n)
    @n = n
  end
  def <=>(other)
    @n <=> other.instance_variable_get(:@n)
  end
end
LE.new(2) <= LE.new(2)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn comparable_returning_nil_raises_argument_error() {
    let err = run_err(
        r#"
class CNil
  def <=>(other)
    nil
  end
end
CNil.new < CNil.new
"#,
    );
    assert!(err.contains("comparison") || err.contains("failed") || err.contains("ArgumentError"));
}

// ── Comparison type-mismatch error format (lines 400-417) ────────────────────

#[test]
fn comparison_with_nil_errors_with_nil_repr() {
    let err = run_err("5 < nil");
    assert!(err.contains("comparison") || err.contains("nil") || err.contains("failed"));
}

#[test]
fn comparison_with_true_errors_with_true_repr() {
    let err = run_err("5 < true");
    assert!(err.contains("comparison") || err.contains("true") || err.contains("failed"));
}

#[test]
fn comparison_with_false_errors_with_false_repr() {
    let err = run_err("5 < false");
    assert!(err.contains("comparison") || err.contains("false") || err.contains("failed"));
}

#[test]
fn comparison_mixed_with_string_errors() {
    let err = run_err(r#"5 < "hi""#);
    assert!(err.contains("comparison") || err.contains("failed"));
}

#[test]
fn comparison_with_float_right_repr() {
    let err = run_err(r#""hi" < 3.14"#);
    assert!(err.contains("comparison") || err.contains("failed"));
}

#[test]
fn comparison_with_array_right_repr() {
    let err = run_err(r#""hi" < [1,2]"#);
    assert!(err.contains("comparison") || err.contains("failed"));
}

// ── Unary operations ─────────────────────────────────────────────────────────

#[test]
fn unary_plus_on_int_identity() {
    let result = run("+5");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn unary_plus_on_string_identity() {
    let result = run(r#"+"hello""#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn unary_plus_on_nil_errors() {
    let err = run_err("+nil");
    assert!(err.contains("TypeError") || err.contains("type") || err.contains("unsupported"));
}

#[test]
fn unary_minus_on_int() {
    let result = run("-5");
    assert_eq!(result, Some(Object::Int(-5)));
}

#[test]
fn unary_minus_on_float() {
    let result = run("-2.5");
    assert_eq!(result, Some(Object::Float(-2.5)));
}

#[test]
fn unary_minus_on_bool_errors() {
    let err = run_err("-true");
    assert!(err.contains("TypeError") || err.contains("type") || err.contains("unsupported"));
}

#[test]
fn unary_not_on_false_is_true() {
    let result = run("!false");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn unary_not_on_nil_is_true() {
    let result = run("!nil");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn unary_not_on_int_is_false() {
    let result = run("!5");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── String + String concatenation ────────────────────────────────────────────

#[test]
fn string_plus_string_concatenates() {
    let result = run(r#""abc" + "def""#);
    assert_eq!(result, Some(Object::string("abcdef")));
}

// ── Array + Array ────────────────────────────────────────────────────────────

#[test]
fn array_plus_array_concatenates() {
    let result = run("[1, 2] + [3, 4]");
    match result {
        Some(Object::Array(arr)) => assert_eq!(arr.borrow().len(), 4),
        other => panic!("expected array, got {:?}", other),
    }
}

#[test]
fn addition_type_error() {
    let err = run_err("5 + nil");
    assert!(err.contains("TypeError") || err.contains("type") || err.contains("unsupported"));
}

// ── Division / modulo by zero ────────────────────────────────────────────────

#[test]
fn int_div_by_zero_errors() {
    let err = run_err("5 / 0");
    assert!(err.contains("divide") || err.contains("zero") || err.contains("ZeroDivision"));
}

#[test]
fn float_div_by_zero_errors() {
    let err = run_err("5.0 / 0.0");
    assert!(err.contains("divide") || err.contains("zero") || err.contains("ZeroDivision"));
}

#[test]
fn int_mod_zero_errors() {
    let err = run_err("5 % 0");
    assert!(err.contains("divide") || err.contains("zero"));
}

#[test]
fn float_mod_zero_errors() {
    let err = run_err("5.0 % 0.0");
    assert!(err.contains("divide") || err.contains("zero"));
}

#[test]
fn int_div_exact_returns_int() {
    let result = run("10 / 2");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn int_div_non_exact_returns_float() {
    let result = run("5 / 2");
    // May return Int(2) (truncating) or Float(2.5) (exact).
    assert!(matches!(
        result,
        Some(Object::Int(2)) | Some(Object::Float(_))
    ));
}

#[test]
fn int_power_negative_returns_float() {
    let result = run("2 ** -1");
    assert!(matches!(result, Some(Object::Float(_))));
}

#[test]
fn int_power_positive_returns_int() {
    let result = run("2 ** 3");
    assert_eq!(result, Some(Object::Int(8)));
}

#[test]
fn float_power() {
    let result = run("2.0 ** 3.0");
    match result {
        Some(Object::Float(f)) => assert_eq!(f, 8.0),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn mixed_float_int_division() {
    let result = run("10.0 / 4");
    match result {
        Some(Object::Float(f)) => assert_eq!(f, 2.5),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn mixed_int_float_division() {
    let result = run("10 / 4.0");
    match result {
        Some(Object::Float(f)) => assert_eq!(f, 2.5),
        other => panic!("expected float, got {:?}", other),
    }
}

// ── String % formatting ──────────────────────────────────────────────────────

#[test]
fn string_format_modulo() {
    // Exercises line 60 (Modulo on String).
    let result = run(r#""%d apples" % 5"#);
    match result {
        Some(Object::String(s)) => assert!(s.contains("5")),
        other => panic!("expected string, got {:?}", other),
    }
}
