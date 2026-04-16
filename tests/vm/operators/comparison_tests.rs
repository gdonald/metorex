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

// ── Equality / NotEqual ─────────────────────────────────────────────────────

#[test]
fn not_equal_ints_true() {
    assert_eq!(run("3 != 4"), Some(Object::Bool(true)));
}

#[test]
fn not_equal_ints_false() {
    assert_eq!(run("3 != 3"), Some(Object::Bool(false)));
}

#[test]
fn equal_ints_true() {
    assert_eq!(run("5 == 5"), Some(Object::Bool(true)));
}

// ── Comparison type errors ──────────────────────────────────────────────────

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

// ── Float comparison ────────────────────────────────────────────────────────

#[test]
fn float_comparison_less() {
    assert_eq!(run("1.5 < 2.5"), Some(Object::Bool(true)));
}

#[test]
fn float_int_comparison() {
    assert_eq!(run("1.5 > 1"), Some(Object::Bool(true)));
}

#[test]
fn int_float_comparison() {
    assert_eq!(run("2 >= 1.5"), Some(Object::Bool(true)));
}

// ── String comparisons ──────────────────────────────────────────────────────

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

// ── Instance == with custom method ──────────────────────────────────────────

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

// ── Instance <=> via Comparable protocol ────────────────────────────────────

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

// ── Case equality (===) ─────────────────────────────────────────────────────

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

// ── Triple equals fallback ──────────────────────────────────────────────────

#[test]
fn triple_equals() {
    assert_eq!(run("1 === 1"), Some(Object::Bool(true)));
}

#[test]
fn triple_equals_false() {
    assert_eq!(run("1 === 2"), Some(Object::Bool(false)));
}
