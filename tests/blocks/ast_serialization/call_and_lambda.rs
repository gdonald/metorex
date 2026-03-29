// Call expression, Lambda, Grouped, Self, ScopeResolution, and Range serialization tests

use super::eval;
use metorex::object::Object;

// ── Call expression serialization ────────────────────────────────────────────

#[test]
fn call_with_args_serializes_type() {
    let result = eval(
        r#"
def helper(x)
  x
end
class C
  def f(n)
    helper(n)
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Call")));
}

#[test]
fn call_with_args_has_arguments() {
    let result = eval(
        r#"
def helper(x)
  x
end
class C
  def f(n)
    helper(n)
  end
end
C.new.get_source(:f).body[0]["arguments"].length
"#,
    );
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Lambda expression serialization ──────────────────────────────────────────

#[test]
fn lambda_expression_serializes_type() {
    let result = eval(
        r#"
class C
  def f
    lambda do |x|
      x * 2
    end
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Lambda")));
}

#[test]
fn lambda_expression_has_body() {
    let result = eval(
        r#"
class C
  def f
    lambda do |x|
      x * 2
    end
  end
end
C.new.get_source(:f).body[0]["body"].length
"#,
    );
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Grouped expression serialization ─────────────────────────────────────────

#[test]
fn grouped_expression_serializes_type() {
    let result = eval(
        r#"
class C
  def f(x)
    (x + 1)
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("Grouped")));
}

// ── Self in method call chain ────────────────────────────────────────────────

#[test]
fn self_in_method_chain_serializes_as_method_call() {
    let result = eval(
        r#"
class C
  def bar
    42
  end
  def f
    self.bar
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("MethodCall")));
}

// ── ScopeResolution serialization ────────────────────────────────────────────

#[test]
fn scope_resolution_serializes_type() {
    let result = eval(
        r#"
class Outer
  Inner = 42
  def f
    Outer::Inner
  end
end
Outer.new.get_source(:f).body[0]["type"]
"#,
    );
    assert_eq!(result, Some(Object::string("ScopeResolution")));
}

#[test]
fn scope_resolution_has_name() {
    let result = eval(
        r#"
class Outer
  Inner = 42
  def f
    Outer::Inner
  end
end
Outer.new.get_source(:f).body[0]["name"]
"#,
    );
    assert_eq!(result, Some(Object::string("Inner")));
}

// ── Range expression ─────────────────────────────────────────────────────────

#[test]
fn range_expression_in_method_body_serializes_unknown() {
    let result = eval(
        r#"
class C
  def f
    1..5
  end
end
C.new.get_source(:f).body[0]["type"]
"#,
    );
    // Range falls through to the `_` catch-all
    assert_eq!(result, Some(Object::string("Unknown")));
}
