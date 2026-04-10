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
fn unary_plus_error_on_string() {
    let err = run_err(r#"+"hello""#);
    assert!(err.contains("unary") || err.contains("+") || err.contains("operator"));
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
